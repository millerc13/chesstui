use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::app::{App, InputMode};

pub struct CommandBarWidget<'a> {
    app: &'a App,
}

impl<'a> CommandBarWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for CommandBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        let theme = &self.app.theme;
        let sep = Style::default().fg(Color::Indexed(238));

        // ── Top line: MODE │ input_text▏ ──
        let (mode_label, mode_color) = match self.app.mode {
            InputMode::Play => ("PLAY", theme.mode_play),
            InputMode::Command => ("CMD", theme.mode_command),
        };

        let input_text = match self.app.mode {
            InputMode::Command => {
                format!(":{}", self.app.command_buffer)
            }
            InputMode::Play => {
                if !self.app.status_message.is_empty() {
                    self.app.status_message.clone()
                } else {
                    self.app.input_buffer.clone()
                }
            }
        };

        let is_status = self.app.mode == InputMode::Play
            && self.app.input_buffer.is_empty()
            && !self.app.status_message.is_empty();

        // Build left side
        let mut left_spans = vec![
            Span::styled(
                format!(" {} ", mode_label),
                Style::default().fg(theme.text_bright).bg(mode_color),
            ),
            Span::styled(" \u{2502} ", sep),
        ];

        if is_status {
            left_spans.push(Span::styled(
                &input_text,
                Style::default()
                    .fg(theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            ));
        } else {
            left_spans.push(Span::styled(
                &input_text,
                Style::default().fg(theme.text_bright),
            ));
            left_spans.push(Span::styled("\u{258f}", Style::default().fg(theme.accent)));
        }

        // Build right side: Turn N · White to move
        let side = self.app.game.side_to_move();
        let side_label = match side {
            cozy_chess::Color::White => "White",
            cozy_chess::Color::Black => "Black",
        };
        let move_num = self.app.game.fullmove_number();
        let right_text = format!("Turn {} \u{00b7} {} to move ", move_num, side_label);

        let left_len: usize = left_spans.iter().map(|s| s.width()).sum();
        let right_len = right_text.len();
        let padding = (area.width as usize).saturating_sub(left_len + right_len);

        left_spans.push(Span::raw(" ".repeat(padding)));
        left_spans.push(Span::styled(
            right_text,
            Style::default().fg(theme.text_dim),
        ));

        let top_line = Line::from(left_spans);
        let top_area = Rect::new(area.x, area.y, area.width, 1);
        Paragraph::new(top_line).render(top_area, buf);

        // ── Bottom line: key hints ──
        let hints_str = build_hints(theme);
        let bottom_line = Line::from(hints_str);
        let bottom_area = Rect::new(area.x, area.y + 1, area.width, 1);
        Paragraph::new(bottom_line).render(bottom_area, buf);
    }
}

fn build_hints(theme: &crate::theme::Theme) -> Vec<Span<'static>> {
    let key = Style::default().fg(theme.shortcut_color);
    let desc = Style::default().fg(theme.text_dim);

    vec![
        Span::styled(" Tab", key),
        Span::styled(":cycle  ", desc),
        Span::styled("Arrows", key),
        Span::styled(":jump  ", desc),
        Span::styled("Enter", key),
        Span::styled(":select  ", desc),
        Span::styled("?", key),
        Span::styled(":help ", desc),
    ]
}
