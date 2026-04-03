use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use super::board::ChessBoardWidget;

pub fn draw_postgame(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(3), // result banner
        Constraint::Min(10),  // board
        Constraint::Length(1), // move count
        Constraint::Length(1), // hints
    ])
    .split(area);

    // Result banner
    let banner_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}  ", app.status_message),
            Style::default()
                .fg(app.theme.text_bright)
                .bg(app.theme.accent),
        )),
        Line::from(""),
    ];
    let banner = Paragraph::new(banner_lines).alignment(Alignment::Center);
    frame.render_widget(banner, rows[0]);

    // Final board position
    let board_width = 20u16;
    let board_x = (rows[1].width.saturating_sub(board_width)) / 2;
    let board_area = ratatui::layout::Rect::new(
        rows[1].x + board_x,
        rows[1].y,
        board_width.min(rows[1].width),
        rows[1].height.min(9),
    );
    let board_widget = ChessBoardWidget::new(app.game.board(), &app.theme)
        .flipped(app.board_flipped)
        .last_move(app.last_move);
    frame.render_widget(board_widget, board_area);

    // Move count
    let total_moves = app.game.move_history().len();
    let move_info = Paragraph::new(Line::from(Span::styled(
        format!("Total: {} half-moves ({} full moves)", total_moves, (total_moves + 1) / 2),
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(move_info, rows[2]);

    // Hints
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("n", Style::default().fg(app.theme.accent)),
        Span::styled(" new game  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("m", Style::default().fg(app.theme.accent)),
        Span::styled(" menu  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("q", Style::default().fg(app.theme.accent)),
        Span::styled(" quit", Style::default().fg(app.theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hints, rows[3]);
}
