use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::widgets::render_section_header;
use crate::game::notation::to_algebraic;
use crate::game::state::GameState;
use crate::theme::Theme;

pub struct MoveListWidget<'a> {
    game: &'a GameState,
    theme: &'a Theme,
    scroll: usize,
}

impl<'a> MoveListWidget<'a> {
    pub fn new(game: &'a GameState, theme: &'a Theme, scroll: usize) -> Self {
        Self {
            game,
            theme,
            scroll,
        }
    }
}

impl Widget for MoveListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 3 {
            return;
        }

        // Section header: ── MOVES ──────────
        render_section_header(buf, area, area.y, "MOVES", self.theme);

        // Content area starts after header + 1 spacer
        let content_y = area.y + 2;
        let content_h = area.height.saturating_sub(2) as usize;
        if content_h == 0 {
            return;
        }

        let history = self.game.move_history();
        if history.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                " No moves yet",
                Style::default()
                    .fg(self.theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            )));
            empty.render(Rect::new(area.x, content_y, area.width, 1), buf);
            return;
        }

        // Build move pairs
        let total_moves = history.len();
        let mut lines: Vec<Line> = Vec::new();
        let mut i = 0;
        let mut move_num = 1;
        let last_pair_idx = (total_moves.saturating_sub(1)) / 2; // index of the last pair

        while i < total_moves {
            let is_last_pair = (i / 2) == last_pair_idx;

            let num_style = if is_last_pair {
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.text_dim)
            };

            let mut spans = vec![Span::styled(format!(" {:>2}. ", move_num), num_style)];

            // White's move
            let white_san = to_algebraic(&history[i].previous_board, &history[i].mv);
            let white_style = if is_last_pair {
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.text_primary)
            };
            spans.push(Span::styled(format!("{:<6}", white_san), white_style));

            // Black's move
            if i + 1 < total_moves {
                let black_san = to_algebraic(&history[i + 1].previous_board, &history[i + 1].mv);
                let black_style = if is_last_pair {
                    Style::default()
                        .fg(self.theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.text_primary)
                };
                spans.push(Span::styled(format!("{:<6}", black_san), black_style));
            } else {
                // No black move yet — show cursor marker
                spans.push(Span::styled(
                    "\u{25b8}",
                    Style::default().fg(self.theme.accent),
                ));
            }

            lines.push(Line::from(spans));
            i += 2;
            move_num += 1;
        }

        // Auto-scroll to keep current move visible
        let max_scroll = lines.len().saturating_sub(content_h);
        let scroll = self.scroll.min(max_scroll);

        let visible_lines: Vec<Line> = lines.into_iter().skip(scroll).take(content_h).collect();

        let content_area = Rect::new(area.x, content_y, area.width, content_h as u16);
        Paragraph::new(visible_lines).render(content_area, buf);
    }
}
