use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::theme::Theme;

pub struct HelpOverlay<'a> {
    theme: &'a Theme,
}

impl<'a> HelpOverlay<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    pub fn render_overlay(self, frame: &mut Frame) {
        let area = frame.area();
        let popup_width = 50u16.min(area.width.saturating_sub(4));
        let popup_height = 20u16.min(area.height.saturating_sub(4));

        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Key Bindings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_focused))
            .style(Style::default().bg(self.theme.border_dim));

        let inner = block.inner(popup_area);

        let help_lines = vec![
            self.help_line("h/j/k/l", "Move cursor left/down/up/right"),
            self.help_line("Arrows", "Move cursor"),
            Line::from(""),
            self.help_line("Enter/Space", "Select piece or confirm move"),
            self.help_line("Esc", "Deselect / cancel"),
            Line::from(""),
            self.help_line("a-h", "Start SAN input (pawn file)"),
            self.help_line("N/B/R/Q/K", "Start SAN input (piece)"),
            self.help_line("O", "Start castling input"),
            Line::from(""),
            self.help_line("f", "Flip board"),
            self.help_line(":", "Command mode"),
            self.help_line("?", "Toggle this help"),
            self.help_line("q", "Quit"),
            Line::from(""),
            self.help_line(":quit", "Quit game"),
            self.help_line(":resign", "Resign"),
            self.help_line(":flip", "Flip board"),
            self.help_line(":new", "New game"),
        ];

        let paragraph = Paragraph::new(help_lines);

        frame.render_widget(block, popup_area);
        frame.render_widget(paragraph, inner);
    }

    fn help_line(&self, key: &str, desc: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                format!(" {:>14}  ", key),
                Style::default().fg(self.theme.accent),
            ),
            Span::styled(
                desc.to_string(),
                Style::default().fg(self.theme.text_primary),
            ),
        ])
    }
}
