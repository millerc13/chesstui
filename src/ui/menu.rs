use ratatui::layout::{Constraint, Layout, Alignment};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

const TITLE_ART: &[&str] = &[
    r"       _                   _         _ ",
    r"   ___| |__   ___  ___ ___| |_ _   _(_)",
    r"  / __| '_ \ / _ \/ __/ __| __| | | | |",
    r" | (__| | | |  __/\__ \__ \ |_| |_| | |",
    r"  \___|_| |_|\___||___/___/\__|\__,_|_|",
];

pub fn draw_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Percentage(30),
        Constraint::Length(TITLE_ART.len() as u16 + 2),
        Constraint::Length(2),
        Constraint::Length(app.menu_items().len() as u16 + 2),
        Constraint::Percentage(30),
        Constraint::Length(1),
    ])
    .split(area);

    // Title
    let title_lines: Vec<Line> = TITLE_ART
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(app.theme.accent))))
        .collect();
    let title = Paragraph::new(title_lines).alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "Terminal Chess",
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[2]);

    // Menu items
    let menu_lines: Vec<Line> = app
        .menu_items()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if i == app.menu_selection {
                Line::from(Span::styled(
                    format!("  > {} ", item),
                    Style::default()
                        .fg(app.theme.text_bright)
                        .bg(app.theme.accent),
                ))
            } else {
                Line::from(Span::styled(
                    format!("    {} ", item),
                    Style::default().fg(app.theme.text_primary),
                ))
            }
        })
        .collect();
    let menu = Paragraph::new(menu_lines).alignment(Alignment::Center);
    frame.render_widget(menu, chunks[3]);

    // Hints bar
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("j/k", Style::default().fg(app.theme.accent)),
        Span::styled(" navigate  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("Enter", Style::default().fg(app.theme.accent)),
        Span::styled(" select  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("q", Style::default().fg(app.theme.accent)),
        Span::styled(" quit", Style::default().fg(app.theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hints, chunks[5]);
}
