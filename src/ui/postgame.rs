use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use super::board::ChessBoardWidget;
use crate::app::{App, GameMode};
use crate::game::state::{GameResult, GameStatus};

pub fn draw_postgame(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(4), // result banner
        Constraint::Min(8),    // board + stats side by side
        Constraint::Length(3), // compact move list
        Constraint::Length(4), // action buttons
    ])
    .split(area);

    draw_result_banner(frame, app, rows[0]);
    draw_board_and_stats(frame, app, rows[1]);
    draw_compact_moves(frame, app, rows[2]);
    draw_action_buttons(frame, app, rows[3]);
}

// ── Result Banner ─────────────────────────────────────────────────────────

fn draw_result_banner(frame: &mut Frame, app: &App, area: Rect) {
    let (result_text, result_icon) = match app.game.status() {
        GameStatus::Finished(ref result) => match result {
            GameResult::Checkmate(color) => {
                let icon = if *color == cozy_chess::Color::White {
                    "♔"
                } else {
                    "♚"
                };
                (
                    format!("{:?} WINS BY CHECKMATE", color).to_uppercase(),
                    icon,
                )
            }
            GameResult::Stalemate => ("GAME DRAWN - STALEMATE".to_string(), "="),
            GameResult::DrawByRepetition => ("GAME DRAWN - REPETITION".to_string(), "="),
            GameResult::DrawByFiftyMove => ("GAME DRAWN - FIFTY MOVES".to_string(), "="),
            GameResult::DrawByInsufficientMaterial => {
                ("GAME DRAWN - INSUFFICIENT MATERIAL".to_string(), "=")
            }
            GameResult::DrawByAgreement => ("GAME DRAWN - AGREEMENT".to_string(), "="),
            GameResult::Resignation(color) => {
                let winner = if *color == cozy_chess::Color::White {
                    cozy_chess::Color::Black
                } else {
                    cozy_chess::Color::White
                };
                let icon = if winner == cozy_chess::Color::White {
                    "♔"
                } else {
                    "♚"
                };
                (
                    format!("{:?} WINS BY RESIGNATION", winner).to_uppercase(),
                    icon,
                )
            }
        },
        GameStatus::InProgress => (app.status_message.clone().to_uppercase(), "?"),
    };

    let opponent_text = match &app.game_mode {
        GameMode::VsAi(_) => match app.game.status() {
            GameStatus::Finished(ref result) => match result {
                GameResult::Checkmate(color) => {
                    if is_player_winner(app, *color) {
                        "You defeated Computer".to_string()
                    } else {
                        "You lost to Computer".to_string()
                    }
                }
                GameResult::Resignation(color) => {
                    if is_player_color(app, *color) {
                        "You resigned against Computer".to_string()
                    } else {
                        "Computer resigned".to_string()
                    }
                }
                _ => "Draw against Computer".to_string(),
            },
            _ => String::new(),
        },
        GameMode::Online { opponent_name, .. } => match app.game.status() {
            GameStatus::Finished(ref result) => match result {
                GameResult::Checkmate(color) => {
                    if is_player_winner(app, *color) {
                        format!("You defeated {}", opponent_name)
                    } else {
                        format!("You lost to {}", opponent_name)
                    }
                }
                _ => format!("Draw against {}", opponent_name),
            },
            _ => String::new(),
        },
        GameMode::Local => "Local game".to_string(),
    };

    // Center the banner in a ~50 char wide box
    let banner_w = 50u16.min(area.width);
    let banner_x = area.x + (area.width.saturating_sub(banner_w)) / 2;
    let banner_area = Rect::new(banner_x, area.y, banner_w, area.height.min(4));

    let buf = frame.buffer_mut();
    let border_style = Style::default().fg(app.theme.accent);
    let text_style = Style::default()
        .fg(app.theme.text_bright)
        .add_modifier(Modifier::BOLD);
    let sub_style = Style::default().fg(app.theme.text_dim);

    let w = banner_area.width as usize;
    let inner_w = w.saturating_sub(2);

    // Top border
    let top = format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(inner_w));
    buf.set_string(banner_area.x, banner_area.y, &top, border_style);

    // Line 1: icon + result text
    let line1_content = format!("{}  {}", result_icon, result_text);
    let pad1 = inner_w.saturating_sub(line1_content.chars().count());
    let left_pad1 = pad1 / 2;
    if banner_area.y + 1 < banner_area.y + banner_area.height {
        buf.set_string(banner_area.x, banner_area.y + 1, &"\u{2551}", border_style);
        let content_x = banner_area.x + 1 + left_pad1 as u16;
        buf.set_string(content_x, banner_area.y + 1, &line1_content, text_style);
        let right_border_x = banner_area.x + banner_area.width - 1;
        // Fill with spaces
        let fill = " ".repeat(inner_w);
        buf.set_string(
            banner_area.x + 1,
            banner_area.y + 1,
            &fill,
            Style::default(),
        );
        buf.set_string(content_x, banner_area.y + 1, &line1_content, text_style);
        buf.set_string(banner_area.x, banner_area.y + 1, "\u{2551}", border_style);
        buf.set_string(right_border_x, banner_area.y + 1, "\u{2551}", border_style);
    }

    // Line 2: opponent info
    if banner_area.y + 2 < banner_area.y + banner_area.height {
        let pad2 = inner_w.saturating_sub(opponent_text.chars().count());
        let left_pad2 = pad2 / 2;
        let fill = " ".repeat(inner_w);
        let right_border_x = banner_area.x + banner_area.width - 1;
        buf.set_string(
            banner_area.x + 1,
            banner_area.y + 2,
            &fill,
            Style::default(),
        );
        buf.set_string(
            banner_area.x + 1 + left_pad2 as u16,
            banner_area.y + 2,
            &opponent_text,
            sub_style,
        );
        buf.set_string(banner_area.x, banner_area.y + 2, "\u{2551}", border_style);
        buf.set_string(right_border_x, banner_area.y + 2, "\u{2551}", border_style);
    }

    // Bottom border
    if banner_area.y + 3 < banner_area.y + banner_area.height {
        let bot = format!("\u{255a}{}\u{255d}", "\u{2550}".repeat(inner_w));
        buf.set_string(banner_area.x, banner_area.y + 3, &bot, border_style);
    }
}

fn is_player_winner(app: &App, winner: cozy_chess::Color) -> bool {
    match &app.game_mode {
        GameMode::VsAi(ai_color) => winner != *ai_color,
        GameMode::Online { my_color, .. } => winner == *my_color,
        GameMode::Local => true,
    }
}

fn is_player_color(app: &App, color: cozy_chess::Color) -> bool {
    match &app.game_mode {
        GameMode::VsAi(ai_color) => color != *ai_color,
        GameMode::Online { my_color, .. } => color == *my_color,
        GameMode::Local => true,
    }
}

// ── Board + Stats ─────────────────────────────────────────────────────────

fn draw_board_and_stats(frame: &mut Frame, app: &App, area: Rect) {
    let cols =
        Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)]).split(area);

    // Board
    let board_widget = ChessBoardWidget::new(app.game.board(), &app.theme)
        .flipped(app.board_flipped)
        .last_move(app.last_move);
    frame.render_widget(board_widget, cols[0]);

    // Stats panel
    draw_stats_panel(frame, app, cols[1]);
}

fn draw_stats_panel(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 6 || area.height < 4 {
        return;
    }

    let buf = frame.buffer_mut();
    let border_style = Style::default().fg(app.theme.border_dim);
    let title_style = Style::default()
        .fg(app.theme.text_bright)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(app.theme.text_dim);
    let value_style = Style::default().fg(app.theme.text_bright);

    let w = area.width as usize;
    let inner_w = w.saturating_sub(2);

    // Top border with title
    let title = " STATS ";
    let title_len = title.len();
    let dashes_before = 1usize.min(inner_w);
    let dashes_after = inner_w.saturating_sub(dashes_before + title_len);
    let top = format!(
        "\u{250c}{}{}{}\u{2510}",
        "\u{2500}".repeat(dashes_before),
        title,
        "\u{2500}".repeat(dashes_after)
    );
    buf.set_string(area.x, area.y, &top, border_style);
    // Re-draw title portion in bright style
    buf.set_string(
        area.x + 1 + dashes_before as u16,
        area.y,
        title,
        title_style,
    );

    // Side borders and content rows
    let total_half_moves = app.game.move_history().len();
    let full_moves = (total_half_moves + 1) / 2;

    let duration = match app.game_start_time {
        Some(start) => {
            let elapsed = start.elapsed();
            let secs = elapsed.as_secs();
            let mins = secs / 60;
            let remaining_secs = secs % 60;
            format!("{:02}:{:02}", mins, remaining_secs)
        }
        None => "--:--".to_string(),
    };

    let opening = "--".to_string();
    let accuracy = "--".to_string();
    let moves_str = format!("{}", full_moves);
    let stats: [(&str, &str); 4] = [
        ("Moves:", &moves_str),
        ("Duration:", &duration),
        ("Opening:", &opening),
        ("Accuracy:", &accuracy),
    ];

    for (i, (label, value)) in stats.iter().enumerate() {
        let y = area.y + 1 + i as u16;
        if y >= area.y + area.height.saturating_sub(1) {
            break;
        }
        // Left border
        buf.set_string(area.x, y, "\u{2502}", border_style);
        // Clear inner
        let fill = " ".repeat(inner_w);
        buf.set_string(area.x + 1, y, &fill, Style::default());
        // Label
        buf.set_string(area.x + 2, y, label, label_style);
        // Value right-aligned within the box
        let value_x = (area.x + area.width).saturating_sub(2 + value.len() as u16);
        buf.set_string(value_x, y, value, value_style);
        // Right border
        buf.set_string(area.x + area.width - 1, y, "\u{2502}", border_style);
    }

    // Fill remaining rows with empty bordered lines
    let content_rows = stats.len() as u16;
    for row in content_rows..area.height.saturating_sub(2) {
        let y = area.y + 1 + row;
        if y >= area.y + area.height.saturating_sub(1) {
            break;
        }
        buf.set_string(area.x, y, "\u{2502}", border_style);
        let fill = " ".repeat(inner_w);
        buf.set_string(area.x + 1, y, &fill, Style::default());
        buf.set_string(area.x + area.width - 1, y, "\u{2502}", border_style);
    }

    // Bottom border
    let bot_y = area.y + area.height.saturating_sub(1);
    let bot = format!("\u{2514}{}\u{2518}", "\u{2500}".repeat(inner_w));
    buf.set_string(area.x, bot_y, &bot, border_style);
}

// ── Compact Move List ─────────────────────────────────────────────────────

fn draw_compact_moves(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 10 || area.height < 3 {
        return;
    }

    let buf = frame.buffer_mut();
    let border_style = Style::default().fg(app.theme.border_dim);
    let title_style = Style::default()
        .fg(app.theme.text_bright)
        .add_modifier(Modifier::BOLD);
    let move_style = Style::default().fg(app.theme.text_primary);
    let hint_style = Style::default().fg(app.theme.text_dim);

    let w = area.width as usize;
    let inner_w = w.saturating_sub(2);

    // Top border with title
    let title = " MOVES ";
    let title_len = title.len();
    let dashes_before = 1usize.min(inner_w);
    let dashes_after = inner_w.saturating_sub(dashes_before + title_len);
    let top = format!(
        "\u{250c}{}{}{}\u{2510}",
        "\u{2500}".repeat(dashes_before),
        title,
        "\u{2500}".repeat(dashes_after)
    );
    buf.set_string(area.x, area.y, &top, border_style);
    buf.set_string(
        area.x + 1 + dashes_before as u16,
        area.y,
        title,
        title_style,
    );

    // Content row: build compact move string
    let y = area.y + 1;
    buf.set_string(area.x, y, "\u{2502}", border_style);
    let fill = " ".repeat(inner_w);
    buf.set_string(area.x + 1, y, &fill, Style::default());
    buf.set_string(area.x + area.width - 1, y, "\u{2502}", border_style);

    // Build move text
    let mut move_text = String::new();
    let mut board = cozy_chess::Board::default();
    for (i, record) in app.game.move_history().iter().enumerate() {
        if i % 2 == 0 {
            move_text.push_str(&format!("{}.", i / 2 + 1));
        }
        let san = crate::game::notation::to_algebraic(&board, &record.mv);
        move_text.push_str(&san);
        move_text.push(' ');
        board.play_unchecked(record.mv);
    }

    let hint = "h/l step";
    let hint_len = hint.len();
    let max_move_len = inner_w.saturating_sub(hint_len + 2);

    // Truncate if needed
    let display_moves = if move_text.chars().count() > max_move_len {
        let truncated: String = move_text
            .chars()
            .take(max_move_len.saturating_sub(3))
            .collect();
        format!("{}...", truncated)
    } else {
        move_text.trim().to_string()
    };

    buf.set_string(area.x + 2, y, &display_moves, move_style);

    // Right-aligned hint
    let hint_x = (area.x + area.width).saturating_sub(2 + hint_len as u16);
    buf.set_string(hint_x, y, hint, hint_style);

    // Bottom border
    let bot_y = area.y + 2;
    if bot_y < area.y + area.height {
        let bot = format!("\u{2514}{}\u{2518}", "\u{2500}".repeat(inner_w));
        buf.set_string(area.x, bot_y, &bot, border_style);
    }
}

// ── Action Buttons ────────────────────────────────────────────────────────

fn draw_action_buttons(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 20 || area.height < 3 {
        return;
    }

    let buttons = [
        ("\u{21bb}", "REMATCH"),
        ("\u{25c4}\u{25ba}", "REVIEW"),
        ("\u{2398}", "COPY PGN"),
        ("\u{2190}", "MENU"),
    ];

    let button_count = buttons.len() as u16;
    let total_gap = (button_count - 1) * 1; // 1 char gap between buttons
    let button_w = (area.width.saturating_sub(total_gap)) / button_count;
    let button_w = button_w.min(15).max(8);

    // Center the button row
    let total_w = button_w * button_count + total_gap;
    let start_x = area.x + (area.width.saturating_sub(total_w)) / 2;

    let buf = frame.buffer_mut();

    for (i, (icon, title)) in buttons.iter().enumerate() {
        let selected = i == app.postgame_selection;
        let x = start_x + (i as u16) * (button_w + 1);
        let button_area = Rect::new(x, area.y, button_w, 3.min(area.height));

        draw_simple_button(buf, button_area, icon, title, selected, &app.theme);
    }
}

fn draw_simple_button(
    buf: &mut Buffer,
    area: Rect,
    icon: &str,
    title: &str,
    selected: bool,
    theme: &crate::theme::Theme,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }

    let (tl, tr, bl, br, hz, vt, border_color) = if selected {
        (
            '\u{2554}',
            '\u{2557}',
            '\u{255a}',
            '\u{255d}',
            '\u{2550}',
            '\u{2551}',
            theme.accent,
        )
    } else {
        (
            '\u{250c}',
            '\u{2510}',
            '\u{2514}',
            '\u{2518}',
            '\u{2500}',
            '\u{2502}',
            theme.border_dim,
        )
    };

    let border_style = Style::default().fg(border_color);
    let w = area.width as usize;
    let inner_w = w.saturating_sub(2);

    // Top border
    let top = format!("{}{}{}", tl, hz.to_string().repeat(inner_w), tr);
    buf.set_string(area.x, area.y, &top, border_style);

    // Middle row: centered title with icon
    let content = format!("{} {}", icon, title);
    let content_len = content.chars().count();
    let pad = inner_w.saturating_sub(content_len);
    let left_pad = pad / 2;

    let y = area.y + 1;
    buf.set_string(area.x, y, &vt.to_string(), border_style);
    let fill = " ".repeat(inner_w);
    buf.set_string(area.x + 1, y, &fill, Style::default());

    let text_style = if selected {
        Style::default()
            .fg(theme.text_bright)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_primary)
    };
    buf.set_string(area.x + 1 + left_pad as u16, y, &content, text_style);
    buf.set_string(area.x + area.width - 1, y, &vt.to_string(), border_style);

    // Bottom border
    if area.height >= 3 {
        let bot = format!("{}{}{}", bl, hz.to_string().repeat(inner_w), br);
        buf.set_string(area.x, area.y + 2, &bot, border_style);
    }
}
