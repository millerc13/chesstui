use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, InputMode, MenuTab, Screen};
use crate::game::move_input::{InputResult, MoveInputParser};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return;
    }

    match app.screen {
        Screen::ColorPicker => handle_color_picker(app, key),
        Screen::MainMenu => handle_menu(app, key),
        Screen::InGame => handle_in_game(app, key),
        Screen::PostGame => handle_postgame(app, key),
        Screen::ReplayViewer => handle_replay_viewer(app, key),
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
            app.screen = Screen::MainMenu;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

// ── Menu ────────────────────────────────────────────────────────────────────

fn handle_menu(app: &mut App, key: KeyEvent) {
    let tab_count = MenuTab::ALL.len();

    // Tab switching with left/right or h/l
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            let idx = MenuTab::ALL.iter().position(|t| *t == app.active_tab).unwrap_or(0);
            let new_idx = (idx + tab_count - 1) % tab_count;
            app.active_tab = MenuTab::ALL[new_idx];
            if app.active_tab == MenuTab::Replays {
                app.load_replays();
            }
            return;
        }
        KeyCode::Right | KeyCode::Char('l') => {
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
        _ => {}
    }

    // Tab-specific input
    match app.active_tab {
        MenuTab::Play => handle_play_tab(app, key),
        MenuTab::Replays => handle_replays_tab(app, key),
        _ => {} // placeholder tabs have no interaction
    }
}

fn handle_play_tab(app: &mut App, key: KeyEvent) {
    let item_count = 3; // Play vs AI, Local Game, Quit
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.play_selection = (app.play_selection + 1) % item_count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.play_selection = (app.play_selection + item_count - 1) % item_count;
        }
        KeyCode::Enter => {
            match app.play_selection {
                0 => app.start_ai_game(),
                1 => app.start_new_game(),
                2 => app.should_quit = true,
                _ => {}
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
            app.show_help = !app.show_help;
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

        _ => {}
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
        }
        InputResult::NeedMore(_) => {
            // Stay, buffer is displayed in command bar
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
        }
        "flip" | "f" => {
            app.board_flipped = !app.board_flipped;
        }
        "new" | "n" => {
            app.start_new_game();
        }
        _ => {
            app.status_message = format!("Unknown command: {}", cmd);
        }
    }
}

// ── Post-Game ───────────────────────────────────────────────────────────────

fn handle_postgame(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') => app.start_new_game(),
        KeyCode::Char('m') => {
            app.screen = Screen::MainMenu;
            app.menu_selection = 0;
        }
        KeyCode::Char('q') => app.should_quit = true,
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
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.replay_viewer = None;
            app.screen = Screen::MainMenu;
            app.active_tab = MenuTab::Replays;
        }
        _ => {}
    }
}
