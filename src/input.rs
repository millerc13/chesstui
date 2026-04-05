use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::Write;

use crate::app::{App, InputMode, MenuTab, MultiplayerState, Screen};
use crate::config::PieceStyle;
use crate::game::move_input::{InputResult, MoveInputParser};

fn debug_log(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/chesstui-debug.log")
    {
        let _ = writeln!(f, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg);
    }
}

/// Returns true if the mouse event changed app state and a redraw is needed.
pub fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::{MouseEventKind, MouseButton};

    if !matches!(event.kind, MouseEventKind::Down(MouseButton::Left)) {
        return false;
    }
    if !matches!(app.screen, Screen::InGame) || !matches!(app.mode, InputMode::Play) {
        return false;
    }
    if app.pending_promotion.is_some() {
        return false;
    }

    let layout = &app.board_layout;
    if layout.sq_w < 0.1 || layout.sq_h < 0.1 { return false; }

    let col = event.column;
    let row = event.row;
    if col < layout.board_x || row < layout.board_y { return false; }

    let dc = ((col - layout.board_x) as f32 / layout.sq_w) as u8;
    let dr = ((row - layout.board_y) as f32 / layout.sq_h) as u8;
    if dc >= 8 || dr >= 8 { return false; }

    let (file, rank) = if app.board_flipped {
        (7 - dc, dr)
    } else {
        (dc, 7 - dr)
    };

    let sq = cozy_chess::Square::new(
        cozy_chess::File::index(file as usize),
        cozy_chess::Rank::index(rank as usize),
    );

    app.set_cursor_to_square(sq);
    app.select_square(sq);
    app.try_ai_move();
    true
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    debug_log(&format!(
        "handle_key: kind={:?} code={:?} modifiers={:?} screen={:?} show_help={}",
        key.kind, key.code, key.modifiers, app.screen, app.show_help
    ));

    if key.kind != crossterm::event::KeyEventKind::Press {
        debug_log("  -> skipped: not a Press event");
        return;
    }

    // When help modal is open, route all input there first
    if app.show_help {
        debug_log("  -> routing to handle_help_input (show_help=true)");
        handle_help_input(app, key);
        return;
    }

    match app.screen {
        Screen::Launch => {
            debug_log("  -> routing to handle_launch");
            handle_launch(app, key);
        }
        Screen::ColorPicker => {
            debug_log("  -> routing to handle_color_picker");
            handle_color_picker(app, key);
        }
        Screen::MainMenu => {
            debug_log("  -> routing to handle_menu");
            handle_menu(app, key);
        }
        Screen::InGame => {
            debug_log("  -> routing to handle_in_game");
            handle_in_game(app, key);
        }
        Screen::PostGame => {
            debug_log("  -> routing to handle_postgame");
            handle_postgame(app, key);
        }
        Screen::ReplayViewer => {
            debug_log("  -> routing to handle_replay_viewer");
            handle_replay_viewer(app, key);
        }
    }
}

// ── Launch Screen ─────────────────────────────────────────────────────────

fn handle_launch(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.launch_selection = (app.launch_selection + 1).min(2);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.launch_selection = app.launch_selection.saturating_sub(1);
        }
        KeyCode::Enter => {
            match app.launch_selection {
                0 => {
                    // Sign Up -> multiplayer signup flow
                    app.screen = Screen::MainMenu;
                    app.active_tab = MenuTab::Multiplayer;
                }
                1 => {
                    // Log In -> multiplayer login flow
                    app.has_account = true;
                    app.screen = Screen::MainMenu;
                    app.active_tab = MenuTab::Multiplayer;
                }
                2 => {
                    // Guest -> main menu
                    app.screen = Screen::MainMenu;
                }
                _ => {}
            }
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }
        _ => {}
    }
}

// ── Color Picker ───────────────────────────────────────────────────────────

fn handle_color_picker(app: &mut App, key: KeyEvent) {
    use crate::theme::ColorScheme;
    let count = ColorScheme::ALL.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.color_scheme_index = (app.color_scheme_index + 1) % count;
            app.apply_color_scheme();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.color_scheme_index = (app.color_scheme_index + count - 1) % count;
            app.apply_color_scheme();
        }
        KeyCode::Enter => {
            app.apply_color_scheme();
            // Save the color scheme to config so it persists
            let scheme_name = crate::theme::ColorScheme::ALL[app.color_scheme_index].name();
            let mut config = crate::config::Config::load();
            config.color_scheme = Some(scheme_name.to_string());
            config.save();
            app.screen = Screen::MainMenu;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

// ── Menu ────────────────────────────────────────────────────────────────────

fn handle_menu(app: &mut App, key: KeyEvent) {
    debug_log(&format!(
        "  handle_menu: active_tab={:?} multiplayer_state={:?} code={:?}",
        app.active_tab, app.multiplayer_state, key.code
    ));

    // When multiplayer tab is in a text input state, send all keys there first
    if app.active_tab == MenuTab::Multiplayer && multiplayer_is_text_input(&app.multiplayer_state) {
        debug_log("  -> early return: multiplayer text input mode");
        handle_multiplayer_tab(app, key);
        return;
    }

    let tab_count = MenuTab::ALL.len();

    // On Play tab with VS Computer selected, left/right adjust AI difficulty
    // instead of switching tabs
    let play_tab_intercept_lr = app.active_tab == MenuTab::Play
        && app.play_selection < crate::app::PlayMenuItem::ALL.len()
        && matches!(
            crate::app::PlayMenuItem::ALL[app.play_selection],
            crate::app::PlayMenuItem::VsComputer
        );

    // Tab switching with left/right or h/l
    match key.code {
        KeyCode::Left | KeyCode::Char('h') if !play_tab_intercept_lr => {
            let idx = MenuTab::ALL.iter().position(|t| *t == app.active_tab).unwrap_or(0);
            let new_idx = (idx + tab_count - 1) % tab_count;
            app.active_tab = MenuTab::ALL[new_idx];
            if app.active_tab == MenuTab::Replays {
                app.load_replays();
            }
            return;
        }
        KeyCode::Right | KeyCode::Char('l') if !play_tab_intercept_lr => {
            let idx = MenuTab::ALL.iter().position(|t| *t == app.active_tab).unwrap_or(0);
            let new_idx = (idx + 1) % tab_count;
            app.active_tab = MenuTab::ALL[new_idx];
            if app.active_tab == MenuTab::Replays {
                app.load_replays();
            }
            return;
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
            app.help_search.clear();
            app.help_scroll = 0;
            return;
        }
        _ => {}
    }

    // Tab-specific input
    match app.active_tab {
        MenuTab::Play => handle_play_tab(app, key),
        MenuTab::Replays => handle_replays_tab(app, key),
        MenuTab::Multiplayer => handle_multiplayer_tab(app, key),
        MenuTab::Settings => handle_settings_tab(app, key),
    }
}

fn multiplayer_is_text_input(state: &MultiplayerState) -> bool {
    matches!(
        state,
        MultiplayerState::EnteringEmail
            | MultiplayerState::EnteringOtp
            | MultiplayerState::EnteringDisplayName
            | MultiplayerState::EnteringPassword
            | MultiplayerState::EnteringLoginPassword
    )
}

fn handle_play_tab(app: &mut App, key: KeyEvent) {
    use crate::app::PlayMenuItem;
    let item_count = PlayMenuItem::ALL.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.play_selection = (app.play_selection + 1) % item_count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.play_selection = (app.play_selection + item_count - 1) % item_count;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Decrease AI difficulty on VS Computer card
            if app.play_selection < item_count {
                if let PlayMenuItem::VsComputer = PlayMenuItem::ALL[app.play_selection] {
                    if app.ai_difficulty > 1 {
                        app.ai_difficulty -= 1;
                    }
                }
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            // Increase AI difficulty on VS Computer card
            if app.play_selection < item_count {
                if let PlayMenuItem::VsComputer = PlayMenuItem::ALL[app.play_selection] {
                    if app.ai_difficulty < 10 {
                        app.ai_difficulty += 1;
                    }
                }
            }
        }
        KeyCode::Enter => {
            if app.play_selection < item_count {
                let item = PlayMenuItem::ALL[app.play_selection];
                if !item.is_available() {
                    app.status_message = "Coming soon!".to_string();
                } else {
                    match item {
                        PlayMenuItem::VsComputer => app.start_ai_game(),
                        PlayMenuItem::LocalGame => app.start_new_game(),
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_replays_tab(app: &mut App, key: KeyEvent) {
    if app.replay_list.is_empty() {
        return;
    }
    let count = app.replay_list.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.replay_selection = (app.replay_selection + 1) % count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.replay_selection = (app.replay_selection + count - 1) % count;
        }
        KeyCode::Enter => {
            app.open_replay(app.replay_selection);
        }
        KeyCode::Char('d') => {
            app.delete_selected_replay();
        }
        _ => {}
    }
}

fn handle_settings_tab(app: &mut App, key: KeyEvent) {
    let count = PieceStyle::ALL.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.settings_style_index = (app.settings_style_index + 1) % count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.settings_style_index = (app.settings_style_index + count - 1) % count;
        }
        KeyCode::Enter => {
            app.apply_piece_style();
        }
        _ => {}
    }
}

fn handle_multiplayer_tab(app: &mut App, key: KeyEvent) {
    match &app.multiplayer_state.clone() {
        MultiplayerState::LoggedOut => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if app.multiplayer_selection < 1 {
                    app.multiplayer_selection += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.multiplayer_selection = app.multiplayer_selection.saturating_sub(1);
            }
            KeyCode::Enter => {
                let url = app.server_url.clone();
                app.network = Some(crate::network::NetworkClient::connect(&url));
                if app.multiplayer_selection == 0 {
                    // Sign Up
                    app.has_account = false;
                    app.multiplayer_state = MultiplayerState::EnteringEmail;
                } else {
                    // Log In
                    app.has_account = true;
                    app.multiplayer_state = MultiplayerState::EnteringEmail;
                }
            }
            _ => {}
        },
        MultiplayerState::EnteringEmail => match key.code {
            KeyCode::Char(c) => app.login_input.push(c),
            KeyCode::Backspace => {
                app.login_input.pop();
            }
            KeyCode::Enter => {
                if !app.login_input.is_empty() {
                    if app.has_account {
                        // Go to password entry for login
                        app.multiplayer_state = MultiplayerState::EnteringLoginPassword;
                    } else {
                        // Send OTP for signup
                        if let Some(ref net) = app.network {
                            net.send(crate::protocol::ClientMessage::Authenticate {
                                email: app.login_input.clone(),
                            });
                        }
                        app.multiplayer_state = MultiplayerState::WaitingForOtp;
                    }
                }
            }
            KeyCode::Esc => {
                app.login_input.clear();
                app.multiplayer_state = MultiplayerState::LoggedOut;
            }
            _ => {}
        },
        MultiplayerState::EnteringOtp => match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if app.otp_input.len() < 6 {
                    app.otp_input.push(c);
                }
            }
            KeyCode::Backspace => {
                app.otp_input.pop();
            }
            KeyCode::Enter => {
                if app.otp_input.len() == 6 {
                    if let Some(ref net) = app.network {
                        net.send(crate::protocol::ClientMessage::VerifyOtp {
                            email: app.login_input.clone(),
                            code: app.otp_input.clone(),
                        });
                    }
                }
            }
            KeyCode::Esc => {
                app.otp_input.clear();
                app.multiplayer_state = MultiplayerState::EnteringEmail;
            }
            _ => {}
        },
        MultiplayerState::EnteringPassword => match key.code {
            KeyCode::Char(c) => app.password_input.push(c),
            KeyCode::Backspace => {
                app.password_input.pop();
            }
            KeyCode::Enter => {
                if app.password_input.len() >= 6 {
                    if let Some(ref net) = app.network {
                        net.send(crate::protocol::ClientMessage::SetPassword {
                            password: app.password_input.clone(),
                        });
                    }
                }
            }
            KeyCode::Esc => {
                app.password_input.clear();
                app.multiplayer_state = MultiplayerState::LoggedOut;
            }
            _ => {}
        },
        MultiplayerState::EnteringLoginPassword => match key.code {
            KeyCode::Char(c) => app.password_input.push(c),
            KeyCode::Backspace => {
                app.password_input.pop();
            }
            KeyCode::Enter => {
                if !app.password_input.is_empty() {
                    if let Some(ref net) = app.network {
                        net.send(crate::protocol::ClientMessage::LoginWithPassword {
                            email: app.login_input.clone(),
                            password: app.password_input.clone(),
                        });
                    }
                }
            }
            KeyCode::Esc => {
                app.password_input.clear();
                app.multiplayer_state = MultiplayerState::EnteringEmail;
            }
            _ => {}
        },
        MultiplayerState::EnteringDisplayName => match key.code {
            KeyCode::Char(c) => app.display_name_input.push(c),
            KeyCode::Backspace => {
                app.display_name_input.pop();
            }
            KeyCode::Enter => {
                if !app.display_name_input.is_empty() {
                    if let Some(ref net) = app.network {
                        net.send(crate::protocol::ClientMessage::SetDisplayName {
                            name: app.display_name_input.clone(),
                        });
                    }
                    // Optimistically transition to LoggedIn
                    let name = app.display_name_input.clone();
                    app.multiplayer_state = MultiplayerState::LoggedIn {
                        display_name: name,
                        elo: 1200,
                    };
                }
            }
            _ => {}
        },
        MultiplayerState::LoggedIn { .. } => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if app.multiplayer_selection < 1 {
                    app.multiplayer_selection += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.multiplayer_selection = app.multiplayer_selection.saturating_sub(1);
            }
            KeyCode::Enter => match app.multiplayer_selection {
                0 => {
                    // Find Game
                    if let Some(ref net) = app.network {
                        net.send(crate::protocol::ClientMessage::FindGame);
                    }
                    app.multiplayer_state = MultiplayerState::Searching;
                }
                1 => {
                    // Log Out
                    app.network = None;
                    app.multiplayer_state = MultiplayerState::LoggedOut;
                    crate::network::session::clear_session();
                }
                _ => {}
            },
            _ => {}
        },
        MultiplayerState::Searching => match key.code {
            KeyCode::Esc => {
                if let Some(ref net) = app.network {
                    net.send(crate::protocol::ClientMessage::CancelSearch);
                }
                app.multiplayer_state = MultiplayerState::LoggedOut;
            }
            _ => {}
        },
        _ => {} // WaitingForOtp, Connecting, InGame — no input handling needed
    }
}

// ── Help Modal ─────────────────────────────────────────────────────────────

fn handle_help_input(app: &mut App, key: KeyEvent) {
    debug_log(&format!("  handle_help_input: code={:?} show_help={}", key.code, app.show_help));
    // Handle Ctrl+j / Ctrl+k for scrolling before the generic Char(c) arm
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('j') => {
                app.help_scroll = app.help_scroll.saturating_add(1);
                return;
            }
            KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
                return;
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.show_help = false;
            app.help_search.clear();
            app.help_scroll = 0;
            app.board_image_dirty = true;
            app.kitty_image_hash = 0; // force Kitty retransmit after modal closes
        }
        KeyCode::Backspace => {
            app.help_search.pop();
        }
        KeyCode::Down => {
            app.help_scroll = app.help_scroll.saturating_add(1);
        }
        KeyCode::Up => {
            app.help_scroll = app.help_scroll.saturating_sub(1);
        }
        KeyCode::Char(c) => {
            app.help_search.push(c);
            app.help_scroll = 0;
        }
        _ => {}
    }
}

// ── In-Game ─────────────────────────────────────────────────────────────────

fn handle_in_game(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    match app.mode {
        InputMode::Play => handle_play(app, key),
        InputMode::Command => handle_command(app, key),
    }

    // After any human input, let the AI respond if it's the AI's turn
    app.try_ai_move();
}

fn handle_play(app: &mut App, key: KeyEvent) {
    debug_log(&format!(
        "  handle_play: code={:?} modifiers={:?} pending_promo={}",
        key.code, key.modifiers, app.pending_promotion.is_some()
    ));

    // Promotion takes priority
    if app.pending_promotion.is_some() {
        handle_promotion(app, key);
        return;
    }

    match key.code {
        // ── Escape: clear buffer → deselect → nothing ──
        KeyCode::Esc => {
            if !app.input_buffer.is_empty() {
                app.input_buffer.clear();
                app.status_message.clear();
            } else {
                app.deselect();
                app.status_message.clear();
            }
        }

        // ── Backspace: delete last input char ──
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }

        // ── Tab / Shift+Tab: cycle between movable pieces ──
        KeyCode::Tab => {
            app.input_buffer.clear();
            app.deselect();
            app.jump_to_next_piece(!key.modifiers.contains(KeyModifiers::SHIFT));
        }
        KeyCode::BackTab => {
            app.input_buffer.clear();
            app.deselect();
            app.jump_to_next_piece(false);
        }

        // ── Arrow keys: smart jump ──
        KeyCode::Left => {
            app.input_buffer.clear();
            if app.selected_square.is_some() {
                app.jump_between_destinations(-1, 0);
            } else {
                app.jump_between_pieces(-1, 0);
            }
        }
        KeyCode::Right => {
            app.input_buffer.clear();
            if app.selected_square.is_some() {
                app.jump_between_destinations(1, 0);
            } else {
                app.jump_between_pieces(1, 0);
            }
        }
        KeyCode::Up => {
            app.input_buffer.clear();
            if app.selected_square.is_some() {
                app.jump_between_destinations(0, 1);
            } else {
                app.jump_between_pieces(0, 1);
            }
        }
        KeyCode::Down => {
            app.input_buffer.clear();
            if app.selected_square.is_some() {
                app.jump_between_destinations(0, -1);
            } else {
                app.jump_between_pieces(0, -1);
            }
        }

        // ── Enter: select piece or confirm move ──
        KeyCode::Enter => {
            let sq = app.cursor_square();
            app.select_square(sq);
        }

        // ── Command mode ──
        KeyCode::Char(':') => {
            app.input_buffer.clear();
            app.mode = InputMode::Command;
            app.command_buffer.clear();
        }

        // ── Help toggle ──
        KeyCode::Char('?') => {
            debug_log(&format!("  '?' pressed! show_help was {}, toggling", app.show_help));
            app.show_help = !app.show_help;
            app.help_search.clear();
            app.help_scroll = 0;
            debug_log(&format!("  show_help is now {}", app.show_help));
        }

        // ── Quit (only when buffer empty) ──
        KeyCode::Char('q') => {
            if app.input_buffer.is_empty() {
                app.should_quit = true;
            } else {
                feed_input_char(app, 'q');
            }
        }

        // ── Chess input: file letters ──
        KeyCode::Char(c @ ('a'..='h')) => {
            feed_input_char(app, c);
        }

        // ── Chess input: rank numbers (only when buffer non-empty) ──
        KeyCode::Char(c @ ('1'..='8')) => {
            if !app.input_buffer.is_empty() {
                feed_input_char(app, c);
            }
        }

        // ── Chess input: capture 'x' and promotion suffixes 'n'/'r' (only when buffer non-empty) ──
        KeyCode::Char(c @ ('x' | 'n' | 'r')) => {
            if !app.input_buffer.is_empty() {
                feed_input_char(app, c);
            }
        }

        // ── Chess input: piece letters and castling ──
        KeyCode::Char(c @ ('N' | 'B' | 'R' | 'Q' | 'K' | 'O')) => {
            feed_input_char(app, c);
        }

        other => {
            debug_log(&format!("  handle_play UNMATCHED: code={:?} modifiers={:?}", other, key.modifiers));
        }
    }
}

fn handle_promotion(app: &mut App, key: KeyEvent) {
    let promo_moves: Vec<cozy_chess::Move> = match &app.pending_promotion {
        Some(moves) => moves.clone(),
        None => return,
    };

    match key.code {
        KeyCode::Left => {
            if app.promotion_choice > 0 {
                app.promotion_choice -= 1;
            }
        }
        KeyCode::Right => {
            if app.promotion_choice + 1 < promo_moves.len() {
                app.promotion_choice += 1;
            }
        }
        KeyCode::Enter => {
            let mv = promo_moves[app.promotion_choice];
            app.make_move(mv);
        }
        KeyCode::Esc => {
            app.pending_promotion = None;
            app.promotion_choice = 0;
            app.deselect();
        }
        _ => {}
    }
}

fn feed_input_char(app: &mut App, c: char) {
    app.input_buffer.push(c);

    let mut parser = MoveInputParser::new(app.game.board());
    let mut result = InputResult::NeedMore(0);
    for ch in app.input_buffer.chars() {
        result = parser.feed(ch);
    }

    match result {
        InputResult::Exact(mv) => {
            app.make_move(mv);
            app.input_buffer.clear();
        }
        InputResult::NoMatch => {
            app.status_message = format!("No match for '{}'", app.input_buffer);
            app.input_buffer.clear();
            app.deselect();
        }
        InputResult::NeedMore(_) => {
            // When we have a valid source square (2 chars), move cursor there
            // and select it to show available moves
            if app.input_buffer.len() == 2 {
                if let Ok(sq) = app.input_buffer.parse::<cozy_chess::Square>() {
                    app.set_cursor_to_square(sq);
                    app.select_square(sq);
                }
            }
        }
    }
}

fn handle_command(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Play;
            app.command_buffer.clear();
        }
        KeyCode::Enter => {
            let cmd = app.command_buffer.trim().to_string();
            execute_command(app, &cmd);
            app.command_buffer.clear();
            app.mode = InputMode::Play;
        }
        KeyCode::Backspace => {
            app.command_buffer.pop();
            if app.command_buffer.is_empty() {
                app.mode = InputMode::Play;
            }
        }
        KeyCode::Char(c) => {
            app.command_buffer.push(c);
        }
        _ => {}
    }
}

fn execute_command(app: &mut App, cmd: &str) {
    match cmd {
        "q" | "quit" => app.should_quit = true,
        "resign" | "res" => {
            let loser = app.game.side_to_move();
            let result = crate::game::state::GameResult::Resignation(loser);
            crate::game::replay::save_game(&app.game, &app.game_mode, &result);
            app.status_message = format!("{:?} resigned", loser);
            app.screen = Screen::PostGame;
            app.postgame_selection = 0;
        }
        "flip" | "f" => {
            app.board_flipped = !app.board_flipped;
            app.board_image_dirty = true;
        }
        "new" | "n" => {
            app.start_new_game();
        }
        "debug" | "dbg" => {
            app.show_debug = !app.show_debug;
        }
        "kitty" => {
            app.use_kitty = !app.use_kitty;
            app.board_image_dirty = true;
            app.status_message = if app.use_kitty {
                "Kitty image rendering ON".to_string()
            } else {
                "Kitty image rendering OFF (character mode)".to_string()
            };
        }
        _ => {
            app.status_message = format!("Unknown command: {}", cmd);
        }
    }
}

// ── Post-Game ───────────────────────────────────────────────────────────────

fn handle_postgame(app: &mut App, key: KeyEvent) {
    const POSTGAME_BUTTON_COUNT: usize = 4;
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            app.postgame_selection = app.postgame_selection.saturating_sub(1);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.postgame_selection + 1 < POSTGAME_BUTTON_COUNT {
                app.postgame_selection += 1;
            }
        }
        KeyCode::Enter => {
            match app.postgame_selection {
                0 => {
                    // Rematch — start a new game with the same mode
                    let mode = app.game_mode.clone();
                    match mode {
                        crate::app::GameMode::VsAi(_) => app.start_ai_game(),
                        _ => app.start_new_game(),
                    }
                }
                1 => {
                    // Review — set up replay viewer from current game
                    let moves: Vec<String> = {
                        let mut board = cozy_chess::Board::default();
                        let mut sans = Vec::new();
                        for record in app.game.move_history() {
                            let san = crate::game::notation::to_algebraic(&board, &record.mv);
                            board.play_unchecked(record.mv);
                            sans.push(san);
                        }
                        sans
                    };
                    let saved = crate::game::replay::SavedGame {
                        id: String::new(),
                        date: String::new(),
                        result: app.status_message.clone(),
                        result_detail: String::new(),
                        mode: format!("{:?}", app.game_mode),
                        move_count: moves.len(),
                        moves,
                        white_player: None,
                        black_player: None,
                        server_game_id: None,
                    };
                    app.replay_viewer = Some(crate::app::ReplayViewerState::from_saved(saved));
                    app.screen = Screen::ReplayViewer;
                }
                2 => {
                    // Copy PGN — build PGN string and copy to clipboard
                    let mut pgn = String::new();
                    let mut board = cozy_chess::Board::default();
                    for (i, record) in app.game.move_history().iter().enumerate() {
                        if i % 2 == 0 {
                            pgn.push_str(&format!("{}.", i / 2 + 1));
                        }
                        let san = crate::game::notation::to_algebraic(&board, &record.mv);
                        pgn.push_str(&san);
                        pgn.push(' ');
                        board.play_unchecked(record.mv);
                    }
                    app.status_message = "PGN copied to clipboard!".to_string();
                    #[cfg(target_os = "macos")]
                    {
                        use std::process::{Command, Stdio};
                        if let Ok(mut child) = Command::new("pbcopy")
                            .stdin(Stdio::piped())
                            .spawn()
                        {
                            if let Some(ref mut stdin) = child.stdin {
                                use std::io::Write;
                                let _ = stdin.write_all(pgn.as_bytes());
                            }
                            let _ = child.wait();
                        }
                    }
                    #[cfg(target_os = "linux")]
                    {
                        use std::process::{Command, Stdio};
                        if let Ok(mut child) = Command::new("xclip")
                            .args(["-selection", "clipboard"])
                            .stdin(Stdio::piped())
                            .spawn()
                        {
                            if let Some(ref mut stdin) = child.stdin {
                                use std::io::Write;
                                let _ = stdin.write_all(pgn.as_bytes());
                            }
                            let _ = child.wait();
                        }
                    }
                }
                3 => {
                    // Menu
                    app.screen = Screen::MainMenu;
                    app.menu_selection = 0;
                }
                _ => {}
            }
        }
        KeyCode::Char('q') => {
            app.screen = Screen::MainMenu;
            app.menu_selection = 0;
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
            app.help_search.clear();
            app.help_scroll = 0;
        }
        _ => {}
    }
}

// ── Replay Viewer ──────────────────────────────────────────────────────────

fn handle_replay_viewer(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(ref mut viewer) = app.replay_viewer {
                viewer.go_next();
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(ref mut viewer) = app.replay_viewer {
                viewer.go_prev();
            }
        }
        KeyCode::Home => {
            if let Some(ref mut viewer) = app.replay_viewer {
                viewer.go_start();
            }
        }
        KeyCode::End => {
            if let Some(ref mut viewer) = app.replay_viewer {
                viewer.go_end();
            }
        }
        KeyCode::Char('f') => {
            app.board_flipped = !app.board_flipped;
            app.board_image_dirty = true;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.replay_viewer = None;
            app.screen = Screen::MainMenu;
            app.active_tab = MenuTab::Replays;
        }
        _ => {}
    }
}
