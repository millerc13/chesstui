use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use super::board::ChessBoardWidget;
use super::captured::CapturedWidget;
use super::command_bar::CommandBarWidget;
use super::move_list::MoveListWidget;

pub fn draw_game(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Vertical: turn indicator | main area | command bar
    let rows = Layout::vertical([
        Constraint::Length(1),  // turn indicator
        Constraint::Min(10),   // main content
        Constraint::Length(2), // command bar
    ])
    .split(area);

    // Turn indicator
    draw_turn_indicator(frame, app, rows[0]);

    // Main area: board (left) | side panel (right)
    let cols = Layout::horizontal([
        Constraint::Length(20), // board (1 rank label + 16 squares + padding)
        Constraint::Min(20),   // side panel
    ])
    .split(rows[1]);

    // Board
    let board_widget = ChessBoardWidget::new(app.game.board(), &app.theme)
        .flipped(app.board_flipped)
        .cursor(app.cursor_file, app.cursor_rank)
        .selected(app.selected_square)
        .legal_moves(&app.legal_moves_for_selected)
        .last_move(app.last_move);
    frame.render_widget(board_widget, cols[0]);

    // Side panel: move list (top) | captured pieces (bottom)
    let side = Layout::vertical([
        Constraint::Min(6),    // move list
        Constraint::Length(4), // captured pieces
    ])
    .split(cols[1]);

    let move_list = MoveListWidget::new(&app.game, &app.theme, app.move_list_scroll);
    frame.render_widget(move_list, side[0]);

    let captured = CapturedWidget::new(app, &app.theme);
    frame.render_widget(captured, side[1]);

    // Command bar
    let cmd_bar = CommandBarWidget::new(app);
    frame.render_widget(cmd_bar, rows[2]);

    // Promotion overlay
    if let Some(promo_moves) = &app.pending_promotion {
        draw_promotion_popup(frame, app, promo_moves.as_slice(), area);
    }
}

fn draw_turn_indicator(frame: &mut Frame, app: &App, area: Rect) {
    let side = app.game.side_to_move();
    let (dot, label) = match side {
        cozy_chess::Color::White => ("\u{25cf}", "White to move"),
        cozy_chess::Color::Black => ("\u{25cf}", "Black to move"),
    };
    let dot_color = match side {
        cozy_chess::Color::White => app.theme.white_piece,
        cozy_chess::Color::Black => app.theme.text_dim,
    };

    let line = Line::from(vec![
        Span::styled(format!(" {} ", dot), Style::default().fg(dot_color)),
        Span::styled(label, Style::default().fg(app.theme.text_primary)),
        Span::styled(
            format!("  Move {}", app.game.fullmove_number()),
            Style::default().fg(app.theme.text_dim),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_promotion_popup(frame: &mut Frame, app: &App, promo_moves: &[cozy_chess::Move], area: Rect) {
    // Center a small popup
    let popup_width = (promo_moves.len() as u16) * 4 + 2;
    let popup_height = 3;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear background
    for py in popup_area.y..popup_area.y + popup_area.height {
        for px in popup_area.x..popup_area.x + popup_area.width {
            if let Some(cell) = frame.buffer_mut().cell_mut(ratatui::layout::Position::new(px, py)) {
                cell.set_char(' ');
                cell.set_style(Style::default().bg(app.theme.border_dim));
            }
        }
    }

    // Title
    let title = "Promote to:";
    frame.buffer_mut().set_string(
        popup_area.x + 1,
        popup_area.y,
        title,
        Style::default().fg(app.theme.text_bright).bg(app.theme.border_dim),
    );

    // Piece options
    for (i, mv) in promo_moves.iter().enumerate() {
        let sym = App::promotion_piece_symbol(mv);
        let style = if i == app.promotion_choice {
            Style::default()
                .fg(app.theme.text_bright)
                .bg(app.theme.accent)
        } else {
            Style::default()
                .fg(app.theme.text_primary)
                .bg(app.theme.border_dim)
        };
        frame.buffer_mut().set_string(
            popup_area.x + 1 + (i as u16) * 4,
            popup_area.y + 1,
            format!(" {} ", sym),
            style,
        );
    }

    // Hint
    let hint = "h/l choose  Enter confirm  Esc cancel";
    let hint_x = popup_area.x + 1;
    if popup_area.y + 2 < area.y + area.height {
        frame.buffer_mut().set_string(
            hint_x,
            popup_area.y + 2,
            hint,
            Style::default().fg(app.theme.text_dim).bg(app.theme.border_dim),
        );
    }
}
