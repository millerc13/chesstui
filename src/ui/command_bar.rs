use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
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

        // Top line: input/status
        let top_line = match self.app.mode {
            InputMode::Input => {
                Line::from(vec![
                    Span::styled(" > ", Style::default().fg(theme.mode_input)),
                    Span::styled(&self.app.input_buffer, Style::default().fg(theme.text_bright)),
                    Span::styled("\u{2588}", Style::default().fg(theme.mode_input)), // cursor block
                ])
            }
            InputMode::Command => {
                Line::from(vec![
                    Span::styled(" :", Style::default().fg(theme.mode_command)),
                    Span::styled(&self.app.command_buffer, Style::default().fg(theme.text_bright)),
                    Span::styled("\u{2588}", Style::default().fg(theme.mode_command)),
                ])
            }
            InputMode::Normal => {
                if self.app.status_message.is_empty() {
                    Line::from(Span::styled("", Style::default()))
                } else {
                    Line::from(Span::styled(
                        format!(" {}", self.app.status_message),
                        Style::default().fg(theme.text_dim),
                    ))
                }
            }
        };

        let top_area = Rect::new(area.x, area.y, area.width, 1);
        Paragraph::new(top_line).render(top_area, buf);

        // Bottom line: mode indicator + version
        let mode_label = match self.app.mode {
            InputMode::Normal => ("NORMAL", theme.mode_normal),
            InputMode::Input => ("INPUT", theme.mode_input),
            InputMode::Command => ("COMMAND", theme.mode_command),
        };

        let bottom_line = Line::from(vec![
            Span::styled(
                format!(" {} ", mode_label.0),
                Style::default().fg(theme.text_bright).bg(mode_label.1),
            ),
            Span::styled(
                "  hjkl:move  Enter:select  ?:help  ::cmd",
                Style::default().fg(theme.text_dim),
            ),
        ]);

        let bottom_area = Rect::new(area.x, area.y + 1, area.width, 1);
        Paragraph::new(bottom_line).render(bottom_area, buf);
    }
}
