use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::perf;
use crate::theme::Theme;

pub struct DebugPanel<'a> {
    theme: &'a Theme,
}

impl<'a> DebugPanel<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }
}

impl Widget for DebugPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 3 {
            return;
        }

        // Header
        let deco_len = area.width.saturating_sub(10) as usize;
        let deco = "\u{2500}".repeat(deco_len);
        let header = Line::from(vec![
            Span::styled(" \u{1f41b} ", Style::default().fg(Color::Red)),
            Span::styled("Debug ", Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD)),
            Span::styled(deco, Style::default().fg(Color::Indexed(238))),
        ]);
        Paragraph::new(header).render(Rect::new(area.x, area.y, area.width, 1), buf);

        // Stats line
        let frame_us = perf::frame_time_us();
        let input_lag = perf::input_lag_us();
        let fps = if frame_us > 0 { 1_000_000 / frame_us.max(1) } else { 0 };

        let stats = Line::from(vec![
            Span::styled(
                format!(" {}fps ", fps),
                Style::default().fg(if fps >= 8 { Color::Green } else { Color::Red }),
            ),
            Span::styled(
                format!("{}ms ", frame_us / 1000),
                Style::default().fg(self.theme.text_dim),
            ),
            Span::styled(
                format!("lag:{}ms", input_lag / 1000),
                Style::default().fg(if input_lag < 50_000 { Color::Green } else { Color::Yellow }),
            ),
        ]);
        if area.height >= 3 {
            Paragraph::new(stats).render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
        }

        // Log entries
        let log_y = area.y + 3;
        let log_h = area.height.saturating_sub(3) as usize;
        if log_h == 0 { return; }

        let entries = perf::drain_ring(log_h);
        let lines: Vec<Line> = entries.iter().map(|entry| {
            let color = if entry.contains("frame") {
                Color::Indexed(238)
            } else if entry.contains("render_board") || entry.contains("protocol") {
                Color::Yellow
            } else if entry.contains("terminal.draw") {
                Color::Cyan
            } else {
                self.theme.text_dim
            };
            Line::from(Span::styled(
                truncate(entry, area.width as usize - 1),
                Style::default().fg(color),
            ))
        }).collect();

        let log_area = Rect::new(area.x, log_y, area.width, log_h as u16);
        Paragraph::new(lines).render(log_area, buf);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!(" {}", s)
    } else {
        format!(" {}\u{2026}", &s[..max.saturating_sub(2)])
    }
}
