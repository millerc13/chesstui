use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::board::ChessBoardWidget;
use crate::app::App;

pub fn draw_replay_viewer(frame: &mut Frame, app: &App) {
    let viewer = match &app.replay_viewer {
        Some(v) => v,
        None => return,
    };

    let area = frame.area();

    let cols = Layout::horizontal([
        Constraint::Min(20),    // board
        Constraint::Length(30), // move list sidebar
    ])
    .split(area);

    // Left: board + status
    let left_rows = Layout::vertical([
        Constraint::Length(2), // header
        Constraint::Min(10),   // board
        Constraint::Length(1), // move counter
        Constraint::Length(1), // hints
    ])
    .split(cols[0]);

    // Header: result
    let header = Paragraph::new(Line::from(vec![Span::styled(
        format!(
            "  {} \u{2014} {}  ",
            viewer.game.result, viewer.game.result_detail
        ),
        Style::default()
            .fg(app.theme.text_bright)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(header, left_rows[0]);

    // Board
    let board = viewer.current_board();
    let widget = ChessBoardWidget::new(board, &app.theme).flipped(app.board_flipped);

    // Derive last move highlight from current position
    let widget = if viewer.current_move > 0 {
        if let Some(san) = viewer.game.moves.get(viewer.current_move - 1) {
            let prev_board = &viewer.boards[viewer.current_move - 1];
            if let Some(mv) = crate::game::notation::parse_san(prev_board, san) {
                widget.last_move(Some((mv.from, mv.to)))
            } else {
                widget
            }
        } else {
            widget
        }
    } else {
        widget
    };

    frame.render_widget(widget, left_rows[1]);

    // Move counter
    let counter = Paragraph::new(Line::from(Span::styled(
        format!("Move {} of {}", viewer.current_move, viewer.total_moves()),
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(counter, left_rows[2]);

    // Hints
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("\u{2190}\u{2192}", Style::default().fg(app.theme.accent)),
        Span::styled(" step  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("Home/End", Style::default().fg(app.theme.accent)),
        Span::styled(" jump  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("f", Style::default().fg(app.theme.accent)),
        Span::styled(" flip  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("q", Style::default().fg(app.theme.accent)),
        Span::styled(" back", Style::default().fg(app.theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hints, left_rows[3]);

    // Right: move list
    draw_move_list(frame, app, cols[1]);
}

fn draw_move_list(frame: &mut Frame, app: &App, area: Rect) {
    let viewer = match &app.replay_viewer {
        Some(v) => v,
        None => return,
    };

    let rows = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // separator
        Constraint::Min(1),    // moves
    ])
    .split(area);

    let header = Paragraph::new(Line::from(Span::styled(
        format!(
            "  {} \u{2014} {}",
            viewer.game.mode,
            viewer.game.date.get(..10).unwrap_or(&viewer.game.date)
        ),
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(header, rows[0]);

    let sep = "\u{2500}".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            sep,
            Style::default().fg(app.theme.text_dim),
        ))),
        rows[1],
    );

    // Move pairs
    let visible = rows[2].height as usize;
    let mut lines = Vec::new();
    let moves = &viewer.game.moves;

    let total_pairs = moves.len().div_ceil(2);
    // Scroll so current move is visible
    let current_pair = if viewer.current_move > 0 {
        (viewer.current_move - 1) / 2
    } else {
        0
    };
    let scroll = if current_pair >= visible {
        current_pair - visible + 1
    } else {
        0
    };

    for pair_idx in scroll..total_pairs.min(scroll + visible) {
        let move_num = pair_idx + 1;
        let white_idx = pair_idx * 2;
        let black_idx = pair_idx * 2 + 1;

        let white_san = moves.get(white_idx).map(|s| s.as_str()).unwrap_or("");
        let black_san = moves.get(black_idx).map(|s| s.as_str()).unwrap_or("");

        let num_style = Style::default().fg(app.theme.text_dim);

        let white_active = viewer.current_move == white_idx + 1;
        let black_active = viewer.current_move == black_idx + 1;

        let cursor_style = Style::default()
            .fg(app.theme.table_cursor_fg)
            .bg(app.theme.table_cursor_bg)
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(app.theme.text_primary);

        let white_style = if white_active {
            cursor_style
        } else {
            normal_style
        };
        let black_style = if black_active {
            cursor_style
        } else {
            normal_style
        };

        // Pad black_san to fill remaining width for full-row highlight
        let row_width = area.width as usize;
        let used = 6 + 10; // "  N.  " + white_san column
        let black_width = row_width.saturating_sub(used);

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<4}", format!("{}.", move_num)), num_style),
            Span::styled(format!("{:<10}", white_san), white_style),
            Span::styled(
                format!("{:<width$}", black_san, width = black_width),
                black_style,
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), rows[2]);
}
