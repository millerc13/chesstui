use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, InputMode, Screen};
use crate::game::move_input::{InputResult, MoveInputParser};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Ignore key release events (crossterm 0.28 sends both press and release)
    if key.kind != crossterm::event::KeyEventKind::Press {
        return;
    }

    match app.screen {
        Screen::MainMenu => handle_menu(app, key),
        Screen::InGame => handle_in_game(app, key),
        Screen::PostGame => handle_postgame(app, key),
    }
}

// ── Menu ────────────────────────────────────────────────────────────────────

fn handle_menu(app: &mut App, key: KeyEvent) {
    let item_count = app.menu_items().len();
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => {
            app.menu_selection = (app.menu_selection + 1) % item_count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.menu_selection = (app.menu_selection + item_count - 1) % item_count;
        }
        KeyCode::Enter => {
            match app.menu_selection {
                0 => app.start_new_game(),
                1 => app.should_quit = true,
                _ => {}
            }
        }
        _ => {}
    }
}

// ── In-Game ─────────────────────────────────────────────────────────────────

fn handle_in_game(app: &mut App, key: KeyEvent) {
    // Handle Ctrl+C everywhere
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    match app.mode {
        InputMode::Normal => handle_normal(app, key),
        InputMode::Input => handle_input(app, key),
        InputMode::Command => handle_command(app, key),
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) {
    // If promotion is pending, handle that first
    if app.pending_promotion.is_some() {
        handle_promotion(app, key);
        return;
    }

    match key.code {
        // Cursor movement
        KeyCode::Char('h') | KeyCode::Left => app.move_cursor(-1, 0),
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor(0, -1),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor(0, 1),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(1, 0),

        // Select / confirm
        KeyCode::Enter | KeyCode::Char(' ') => {
            let sq = app.cursor_square();
            app.select_square(sq);
        }

        // Deselect
        KeyCode::Esc => {
            app.deselect();
            app.status_message.clear();
        }

        // Flip board
        KeyCode::Char('f') => {
            app.board_flipped = !app.board_flipped;
        }

        // Command mode
        KeyCode::Char(':') => {
            app.mode = InputMode::Command;
            app.command_buffer.clear();
        }

        // Help
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }

        // Quit
        KeyCode::Char('q') => {
            app.should_quit = true;
        }

        // SAN input mode — triggered by piece letters or pawn file letters
        KeyCode::Char(c @ ('a'..='h' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'O')) => {
            app.mode = InputMode::Input;
            app.input_buffer.clear();
            // Feed the character through input parsing
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
        KeyCode::Char('h') | KeyCode::Left => {
            if app.promotion_choice > 0 {
                app.promotion_choice -= 1;
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
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

fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
            app.input_buffer.clear();
            app.status_message.clear();
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
            if app.input_buffer.is_empty() {
                app.mode = InputMode::Normal;
            }
        }
        KeyCode::Char(c) => {
            feed_input_char(app, c);
        }
        _ => {}
    }
}

fn feed_input_char(app: &mut App, c: char) {
    app.input_buffer.push(c);

    // Create a fresh parser and feed the entire buffer
    let mut parser = MoveInputParser::new(app.game.board());
    let mut result = InputResult::NeedMore(0);
    for ch in app.input_buffer.chars() {
        result = parser.feed(ch);
    }

    match result {
        InputResult::Exact(mv) => {
            app.make_move(mv);
            app.mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        InputResult::NoMatch => {
            app.status_message = format!("No match for '{}'", app.input_buffer);
            app.input_buffer.clear();
            app.mode = InputMode::Normal;
        }
        InputResult::NeedMore(_) => {
            // Stay in input mode
        }
    }
}

fn handle_command(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
            app.command_buffer.clear();
        }
        KeyCode::Enter => {
            let cmd = app.command_buffer.trim().to_string();
            execute_command(app, &cmd);
            app.command_buffer.clear();
            app.mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.command_buffer.pop();
            if app.command_buffer.is_empty() {
                app.mode = InputMode::Normal;
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
