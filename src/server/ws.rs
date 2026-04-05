use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::protocol::{ClientMessage, FriendInfo, ServerMessage, UserInfo};

use super::game_room::GameRoom;
use super::matchmaking::{MatchResult, WaitingPlayer};
use super::{auth, db, AppState};
use crate::game::state::GameState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_connection(socket, state))
}

async fn send_msg(
    tx: &mpsc::Sender<ServerMessage>,
    msg: ServerMessage,
) {
    let _ = tx.send(msg).await;
}

async fn handle_connection(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Channel for sending messages back to this client
    let (out_tx, mut out_rx) = mpsc::channel::<ServerMessage>(64);

    // Task to forward outgoing messages to the WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_sink.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Authentication phase
    let mut authenticated_user: Option<db::DbUser> = None;

    while let Some(Ok(msg)) = ws_stream.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => {
                send_msg(&out_tx, ServerMessage::Error {
                    message: "Invalid message format".to_string(),
                }).await;
                continue;
            }
        };

        match client_msg {
            ClientMessage::Ping => {
                send_msg(&out_tx, ServerMessage::Pong).await;
            }
            ClientMessage::Authenticate { email } => {
                // Rate limiting
                {
                    let mut limits = state.otp_rate_limits.lock().await;
                    if let Some(last) = limits.get(&email) {
                        if last.elapsed() < Duration::from_secs(60) {
                            send_msg(&out_tx, ServerMessage::AuthError {
                                reason: "Please wait before requesting another code".to_string(),
                            }).await;
                            continue;
                        }
                    }
                    limits.insert(email.clone(), Instant::now());
                }

                let code = auth::generate_otp();
                let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

                if let Err(e) = db::store_otp(&state.pool, &email, &code, expires_at).await {
                    tracing::error!("Failed to store OTP: {}", e);
                    send_msg(&out_tx, ServerMessage::AuthError {
                        reason: "Internal error".to_string(),
                    }).await;
                    continue;
                }

                if let Err(e) = auth::send_otp_email(&state.resend_api_key, &email, &code).await {
                    tracing::error!("Failed to send OTP email: {}", e);
                    send_msg(&out_tx, ServerMessage::AuthError {
                        reason: "Failed to send verification email".to_string(),
                    }).await;
                    continue;
                }

                send_msg(&out_tx, ServerMessage::AuthCodeSent).await;
            }
            ClientMessage::VerifyOtp { email, code } => {
                match db::verify_otp(&state.pool, &email, &code).await {
                    Ok(true) => {
                        match db::find_or_create_user(&state.pool, &email).await {
                            Ok(user) => {
                                let token = auth::generate_session_token();
                                let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
                                if let Err(e) = db::create_session(
                                    &state.pool, user.id, &token, expires_at,
                                ).await {
                                    tracing::error!("Failed to create session: {}", e);
                                    send_msg(&out_tx, ServerMessage::AuthError {
                                        reason: "Internal error".to_string(),
                                    }).await;
                                    continue;
                                }

                                let user_info = user_to_info(&user);
                                let has_password = user.password_hash.is_some();
                                let preferences = user.preferences.clone();
                                authenticated_user = Some(user);

                                send_msg(&out_tx, ServerMessage::Authenticated {
                                    token,
                                    user: user_info,
                                }).await;

                                if !has_password {
                                    send_msg(&out_tx, ServerMessage::NeedPassword).await;
                                }

                                send_msg(&out_tx, ServerMessage::PreferencesLoaded {
                                    preferences,
                                }).await;

                                if authenticated_user.as_ref().unwrap().display_name.is_none() {
                                    send_msg(&out_tx, ServerMessage::NeedDisplayName).await;
                                }

                                break; // Exit auth loop
                            }
                            Err(e) => {
                                tracing::error!("Failed to find/create user: {}", e);
                                send_msg(&out_tx, ServerMessage::AuthError {
                                    reason: "Internal error".to_string(),
                                }).await;
                            }
                        }
                    }
                    Ok(false) => {
                        send_msg(&out_tx, ServerMessage::AuthError {
                            reason: "Invalid or expired code".to_string(),
                        }).await;
                    }
                    Err(e) => {
                        tracing::error!("OTP verification error: {}", e);
                        send_msg(&out_tx, ServerMessage::AuthError {
                            reason: "Internal error".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::LoginWithPassword { email, password } => {
                match db::get_user_for_login(&state.pool, &email).await {
                    Ok(Some(user)) => {
                        if let Some(ref hash) = user.password_hash {
                            if auth::verify_password(hash, &password) {
                                let token = auth::generate_session_token();
                                let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
                                if let Err(e) = db::create_session(
                                    &state.pool, user.id, &token, expires_at,
                                ).await {
                                    tracing::error!("Failed to create session: {}", e);
                                    send_msg(&out_tx, ServerMessage::AuthError {
                                        reason: "Internal error".to_string(),
                                    }).await;
                                    continue;
                                }

                                let user_info = user_to_info(&user);
                                let preferences = user.preferences.clone();
                                authenticated_user = Some(user);

                                send_msg(&out_tx, ServerMessage::Authenticated {
                                    token,
                                    user: user_info,
                                }).await;

                                send_msg(&out_tx, ServerMessage::PreferencesLoaded {
                                    preferences,
                                }).await;

                                if authenticated_user.as_ref().unwrap().display_name.is_none() {
                                    send_msg(&out_tx, ServerMessage::NeedDisplayName).await;
                                }

                                break; // Exit auth loop
                            } else {
                                send_msg(&out_tx, ServerMessage::AuthError {
                                    reason: "Invalid password".to_string(),
                                }).await;
                            }
                        } else {
                            send_msg(&out_tx, ServerMessage::AuthError {
                                reason: "No password set for this account".to_string(),
                            }).await;
                        }
                    }
                    Ok(None) => {
                        send_msg(&out_tx, ServerMessage::AuthError {
                            reason: "User not found".to_string(),
                        }).await;
                    }
                    Err(e) => {
                        tracing::error!("Login error: {}", e);
                        send_msg(&out_tx, ServerMessage::AuthError {
                            reason: "Internal error".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::ResumeSession { token } => {
                match db::validate_session(&state.pool, &token).await {
                    Ok(Some(user)) => {
                        let user_info = user_to_info(&user);
                        let preferences = user.preferences.clone();
                        authenticated_user = Some(user);

                        send_msg(&out_tx, ServerMessage::Authenticated {
                            token,
                            user: user_info,
                        }).await;

                        send_msg(&out_tx, ServerMessage::PreferencesLoaded {
                            preferences,
                        }).await;

                        if authenticated_user.as_ref().unwrap().display_name.is_none() {
                            send_msg(&out_tx, ServerMessage::NeedDisplayName).await;
                        }

                        break; // Exit auth loop
                    }
                    Ok(None) => {
                        send_msg(&out_tx, ServerMessage::AuthError {
                            reason: "Invalid or expired session".to_string(),
                        }).await;
                    }
                    Err(e) => {
                        tracing::error!("Session validation error: {}", e);
                        send_msg(&out_tx, ServerMessage::AuthError {
                            reason: "Internal error".to_string(),
                        }).await;
                    }
                }
            }
            _ => {
                send_msg(&out_tx, ServerMessage::Error {
                    message: "Not authenticated".to_string(),
                }).await;
            }
        }
    }

    // If we never authenticated, clean up and return
    let user = match authenticated_user {
        Some(u) => u,
        None => {
            send_task.abort();
            return;
        }
    };

    let user_id = user.id;
    state.connected_users.insert(user_id, ());
    let current_game_id: Option<String> = None;

    // Main message loop (authenticated)
    while let Some(Ok(msg)) = ws_stream.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => {
                send_msg(&out_tx, ServerMessage::Error {
                    message: "Invalid message format".to_string(),
                }).await;
                continue;
            }
        };

        match client_msg {
            ClientMessage::Ping => {
                send_msg(&out_tx, ServerMessage::Pong).await;
            }
            ClientMessage::SetDisplayName { name } => {
                if let Err(e) = db::update_display_name(&state.pool, user_id, &name).await {
                    tracing::error!("Failed to update display name: {}", e);
                }
            }
            ClientMessage::FindGame => {
                let (match_tx, mut match_rx) = mpsc::channel::<MatchResult>(1);

                let display_name = match db::find_user_by_email(&state.pool, &user.email).await {
                    Ok(Some(u)) => u.display_name.unwrap_or_else(|| "Anonymous".to_string()),
                    _ => "Anonymous".to_string(),
                };

                let elo = user.elo;

                let waiting = WaitingPlayer {
                    user_id,
                    display_name: display_name.clone(),
                    elo,
                    tx: match_tx,
                };

                let matched = {
                    let mut mm = state.matchmaker.lock().await;
                    mm.enqueue(waiting)
                };

                if let Some((player_a, player_b)) = matched {
                    // Create a game room
                    let game_id = Uuid::new_v4().to_string();

                    let (white_tx, _) = mpsc::channel::<ServerMessage>(64);
                    let (black_tx, _) = mpsc::channel::<ServerMessage>(64);

                    let room = GameRoom {
                        id: game_id.clone(),
                        state: GameState::new(),
                        white_tx,
                        black_tx,
                        white_id: player_a.user_id,
                        black_id: player_b.user_id,
                        white_name: player_a.display_name.clone(),
                        black_name: player_b.display_name.clone(),
                        moves_san: Vec::new(),
                        external_result: None,
                        draw_offered_by: None,
                    };

                    state.games.insert(game_id.clone(), Arc::new(Mutex::new(room)));

                    // Notify both players
                    let _ = player_a.tx.send(MatchResult {
                        game_id: game_id.clone(),
                        opponent_name: player_b.display_name.clone(),
                        my_color: "white".to_string(),
                    }).await;

                    let _ = player_b.tx.send(MatchResult {
                        game_id,
                        opponent_name: player_a.display_name.clone(),
                        my_color: "black".to_string(),
                    }).await;
                } else {
                    send_msg(&out_tx, ServerMessage::Searching).await;

                    // Wait for match result in background
                    let out_tx_clone = out_tx.clone();
                    tokio::spawn(async move {
                        if let Some(result) = match_rx.recv().await {
                            let _ = out_tx_clone.send(ServerMessage::GameStart {
                                game_id: result.game_id,
                                opponent: result.opponent_name,
                                my_color: result.my_color,
                            }).await;
                        }
                    });
                }
            }
            ClientMessage::CancelSearch => {
                let mut mm = state.matchmaker.lock().await;
                mm.remove(&user_id);
            }
            ClientMessage::MakeMove { game_id, mv } => {
                if let Some(room_arc) = state.games.get(&game_id) {
                    let room_arc = room_arc.clone();
                    let mut room = room_arc.lock().await;

                    match room.try_move(&user_id, &mv) {
                        Ok(canonical_san) => {
                            let move_msg = ServerMessage::MoveMade {
                                game_id: game_id.clone(),
                                mv: canonical_san,
                            };

                            // Broadcast to both players via game room channels
                            let _ = room.white_tx.send(move_msg.clone()).await;
                            let _ = room.black_tx.send(move_msg.clone()).await;

                            // Also send to the current connection
                            send_msg(&out_tx, move_msg).await;

                            // Check if game is finished
                            if room.is_finished() {
                                if let Some((result, detail)) = room.result_strings() {
                                    let game_over = ServerMessage::GameOver {
                                        game_id: game_id.clone(),
                                        result: result.clone(),
                                        detail: detail.clone(),
                                    };
                                    let _ = room.white_tx.send(game_over.clone()).await;
                                    let _ = room.black_tx.send(game_over.clone()).await;
                                    send_msg(&out_tx, game_over).await;

                                    // Save to database
                                    let moves_json = serde_json::to_value(&room.moves_san)
                                        .unwrap_or(serde_json::Value::Array(vec![]));
                                    let white_id = room.white_id;
                                    let black_id = room.black_id;
                                    let pool = state.pool.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) = db::save_finished_game(
                                            &pool, white_id, black_id,
                                            &result, &detail, &moves_json,
                                        ).await {
                                            tracing::error!("Failed to save game: {}", e);
                                        }
                                    });
                                }
                            }
                        }
                        Err(reason) => {
                            send_msg(&out_tx, ServerMessage::MoveRejected { reason }).await;
                        }
                    }
                } else {
                    send_msg(&out_tx, ServerMessage::Error {
                        message: "Game not found".to_string(),
                    }).await;
                }
            }
            ClientMessage::Resign { game_id } => {
                if let Some(room_arc) = state.games.get(&game_id) {
                    let room_arc = room_arc.clone();
                    let mut room = room_arc.lock().await;

                    if let Some(color) = room.color_of(&user_id) {
                        room.set_resignation(color);

                        if let Some((result, detail)) = room.result_strings() {
                            let game_over = ServerMessage::GameOver {
                                game_id: game_id.clone(),
                                result: result.clone(),
                                detail: detail.clone(),
                            };
                            let _ = room.white_tx.send(game_over.clone()).await;
                            let _ = room.black_tx.send(game_over.clone()).await;
                            send_msg(&out_tx, game_over).await;

                            // Save to database
                            let moves_json = serde_json::to_value(&room.moves_san)
                                .unwrap_or(serde_json::Value::Array(vec![]));
                            let white_id = room.white_id;
                            let black_id = room.black_id;
                            let pool = state.pool.clone();
                            tokio::spawn(async move {
                                if let Err(e) = db::save_finished_game(
                                    &pool, white_id, black_id,
                                    &result, &detail, &moves_json,
                                ).await {
                                    tracing::error!("Failed to save game: {}", e);
                                }
                            });
                        }
                    }
                }
            }
            ClientMessage::OfferDraw { game_id } => {
                if let Some(room_arc) = state.games.get(&game_id) {
                    let room_arc = room_arc.clone();
                    let mut room = room_arc.lock().await;

                    if let Some(color) = room.color_of(&user_id) {
                        room.draw_offered_by = Some(color);
                        let draw_msg = ServerMessage::DrawOffered {
                            game_id: game_id.clone(),
                        };
                        // Send to the opponent
                        match color {
                            cozy_chess::Color::White => {
                                let _ = room.black_tx.send(draw_msg).await;
                            }
                            cozy_chess::Color::Black => {
                                let _ = room.white_tx.send(draw_msg).await;
                            }
                        }
                    }
                }
            }
            ClientMessage::AcceptDraw { game_id } => {
                if let Some(room_arc) = state.games.get(&game_id) {
                    let room_arc = room_arc.clone();
                    let mut room = room_arc.lock().await;

                    if let Some(color) = room.color_of(&user_id) {
                        // Can only accept if the opponent offered
                        let opponent_offered = room.draw_offered_by.map_or(false, |c| c != color);
                        if opponent_offered {
                            room.set_draw_by_agreement();

                            if let Some((result, detail)) = room.result_strings() {
                                let game_over = ServerMessage::GameOver {
                                    game_id: game_id.clone(),
                                    result: result.clone(),
                                    detail: detail.clone(),
                                };
                                let _ = room.white_tx.send(game_over.clone()).await;
                                let _ = room.black_tx.send(game_over.clone()).await;
                                send_msg(&out_tx, game_over).await;

                                let moves_json = serde_json::to_value(&room.moves_san)
                                    .unwrap_or(serde_json::Value::Array(vec![]));
                                let white_id = room.white_id;
                                let black_id = room.black_id;
                                let pool = state.pool.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = db::save_finished_game(
                                        &pool, white_id, black_id,
                                        &result, &detail, &moves_json,
                                    ).await {
                                        tracing::error!("Failed to save game: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            }
            ClientMessage::DeclineDraw { game_id } => {
                if let Some(room_arc) = state.games.get(&game_id) {
                    let room_arc = room_arc.clone();
                    let mut room = room_arc.lock().await;
                    room.draw_offered_by = None;
                }
            }
            ClientMessage::SetPassword { password } => {
                match auth::hash_password(&password) {
                    Ok(hash) => {
                        if let Err(e) = db::set_password(&state.pool, user_id, &hash).await {
                            tracing::error!("Failed to set password: {}", e);
                            send_msg(&out_tx, ServerMessage::Error {
                                message: "Failed to set password".to_string(),
                            }).await;
                        } else {
                            send_msg(&out_tx, ServerMessage::PasswordSet).await;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to hash password: {}", e);
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "Failed to set password".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::GetPreferences => {
                match db::get_preferences(&state.pool, user_id).await {
                    Ok(preferences) => {
                        send_msg(&out_tx, ServerMessage::PreferencesLoaded { preferences }).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to get preferences: {}", e);
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "Failed to load preferences".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::UpdatePreferences { preferences } => {
                match db::update_preferences(&state.pool, user_id, &preferences).await {
                    Ok(()) => {
                        send_msg(&out_tx, ServerMessage::PreferencesUpdated).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to update preferences: {}", e);
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "Failed to update preferences".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::AddFriend { email } => {
                match db::find_user_by_email(&state.pool, &email).await {
                    Ok(Some(friend_user)) => {
                        if friend_user.id == user_id {
                            send_msg(&out_tx, ServerMessage::Error {
                                message: "Cannot add yourself as a friend".to_string(),
                            }).await;
                            continue;
                        }
                        if let Err(e) = db::add_friend(&state.pool, user_id, friend_user.id).await {
                            tracing::error!("Failed to add friend: {}", e);
                            send_msg(&out_tx, ServerMessage::Error {
                                message: "Failed to add friend".to_string(),
                            }).await;
                        } else {
                            let online = state.connected_users.contains_key(&friend_user.id);
                            send_msg(&out_tx, ServerMessage::FriendAdded {
                                friend: FriendInfo {
                                    id: friend_user.id.to_string(),
                                    display_name: friend_user.display_name.unwrap_or_else(|| "Anonymous".to_string()),
                                    elo: friend_user.elo,
                                    online,
                                },
                            }).await;
                        }
                    }
                    Ok(None) => {
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "User not found".to_string(),
                        }).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to look up friend: {}", e);
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "Failed to add friend".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::RemoveFriend { friend_id } => {
                match Uuid::parse_str(&friend_id) {
                    Ok(fid) => {
                        if let Err(e) = db::remove_friend(&state.pool, user_id, fid).await {
                            tracing::error!("Failed to remove friend: {}", e);
                            send_msg(&out_tx, ServerMessage::Error {
                                message: "Failed to remove friend".to_string(),
                            }).await;
                        } else {
                            send_msg(&out_tx, ServerMessage::FriendRemoved { friend_id }).await;
                        }
                    }
                    Err(_) => {
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "Invalid friend ID".to_string(),
                        }).await;
                    }
                }
            }
            ClientMessage::ListFriends => {
                match db::list_friends(&state.pool, user_id).await {
                    Ok(friends) => {
                        let friend_infos: Vec<FriendInfo> = friends.into_iter().map(|f| {
                            let online = state.connected_users.contains_key(&f.id);
                            FriendInfo {
                                id: f.id.to_string(),
                                display_name: f.display_name.unwrap_or_else(|| "Anonymous".to_string()),
                                elo: f.elo,
                                online,
                            }
                        }).collect();
                        send_msg(&out_tx, ServerMessage::FriendsList { friends: friend_infos }).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to list friends: {}", e);
                        send_msg(&out_tx, ServerMessage::Error {
                            message: "Failed to list friends".to_string(),
                        }).await;
                    }
                }
            }
            _ => {
                // Ignore auth messages in game loop
            }
        }
    }

    // Clean up on disconnect
    state.connected_users.remove(&user_id);
    {
        let mut mm = state.matchmaker.lock().await;
        mm.remove(&user_id);
    }

    // Handle game abandonment — resign any active games
    if let Some(game_id) = &current_game_id {
        if let Some(room_arc) = state.games.get(game_id) {
            let room_arc = room_arc.clone();
            let mut room = room_arc.lock().await;
            if !room.is_finished() {
                if let Some(color) = room.color_of(&user_id) {
                    room.set_resignation(color);
                    if let Some((result, detail)) = room.result_strings() {
                        let game_over = ServerMessage::GameOver {
                            game_id: game_id.clone(),
                            result,
                            detail,
                        };
                        let _ = room.white_tx.send(game_over.clone()).await;
                        let _ = room.black_tx.send(game_over.clone()).await;
                    }
                }
            }
        }
    }

    send_task.abort();
}

fn user_to_info(user: &db::DbUser) -> UserInfo {
    UserInfo {
        id: user.id.to_string(),
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        elo: user.elo,
    }
}
