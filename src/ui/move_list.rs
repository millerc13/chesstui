use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

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
        Self { game, theme, scroll }
    }
}

impl Widget for MoveListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Moves ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_dim));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let history = self.game.move_history();
        if history.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No moves yet",
                Style::default().fg(self.theme.text_dim),
            )));
            empty.render(inner, buf);
            return;
        }

        // Build move pairs: "1. e4 e5", "2. Nf3 Nc6", etc.
        let mut lines: Vec<Line> = Vec::new();
        let mut i = 0;
        let mut move_num = 1;
        while i < history.len() {
            let mut spans = vec![
                Span::styled(
                    format!("{:>3}. ", move_num),
                    Style::default().fg(self.theme.text_dim),
                ),
            ];

            // White's move
            let white_san = to_algebraic(&history[i].previous_board, &history[i].mv);
            spans.push(Span::styled(
                format!("{:<8}", white_san),
                Style::default().fg(self.theme.text_primary),
            ));

            // Black's move (if exists)
            if i + 1 < history.len() {
                let black_san = to_algebraic(&history[i + 1].previous_board, &history[i + 1].mv);
                spans.push(Span::styled(
                    black_san,
                    Style::default().fg(self.theme.text_primary),
                ));
            }

            lines.push(Line::from(spans));
            i += 2;
            move_num += 1;
        }

        // Apply scrolling
        let visible_height = inner.height as usize;
        let max_scroll = lines.len().saturating_sub(visible_height);
        let scroll = self.scroll.min(max_scroll);

        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(scroll)
            .take(visible_height)
            .collect();

        let paragraph = Paragraph::new(visible_lines);
        paragraph.render(inner, buf);
    }
}
