use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::ascii3d::{render_to_lines, Piece3D};
use super::widgets::CardButton;
use crate::app::App;

const LOGO: &[&str] = &[
    r"  ██████╗██╗  ██╗███████╗███████╗███████╗████████╗██╗   ██╗██╗",
    r" ██╔════╝██║  ██║██╔════╝██╔════╝██╔════╝╚══██╔══╝██║   ██║██║",
    r" ██║     ███████║█████╗  ███████╗███████╗   ██║   ██║   ██║██║",
    r" ██║     ██╔══██║██╔══╝  ╚════██║╚════██║   ██║   ██║   ██║██║",
    r" ╚██████╗██║  ██║███████╗███████║███████║   ██║   ╚██████╔╝██║",
    r"  ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝   ╚═╝    ╚═════╝ ╚═╝",
];

pub fn draw_launch(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let theme = &app.theme;
    let tick = app.tick;

    // 3-column layout: left piece | center content | right piece
    let has_pieces = area.width >= 80;
    let center = if has_pieces {
        let cols = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

        // Left piece — King
        if cols[0].width >= 8 && cols[0].height >= 6 {
            let lines = render_to_lines(
                Piece3D::King,
                cols[0].width.min(30),
                cols[0].height.min(20),
                tick,
                theme,
            );
            frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), cols[0]);
        }

        // Right piece — Queen
        if cols[2].width >= 8 && cols[2].height >= 6 {
            let lines = render_to_lines(
                Piece3D::Queen,
                cols[2].width.min(30),
                cols[2].height.min(20),
                tick,
                theme,
            );
            frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), cols[2]);
        }

        cols[1]
    } else {
        area
    };

    // Center content layout
    let rows = Layout::vertical([
        Constraint::Length(2),                 // top padding
        Constraint::Length(LOGO.len() as u16), // big CHESSTUI logo
        Constraint::Length(1),                 // subtitle
        Constraint::Length(2),                 // spacer
        Constraint::Length(4),                 // button 1: Sign Up
        Constraint::Length(1),                 // gap
        Constraint::Length(4),                 // button 2: Log In
        Constraint::Length(1),                 // gap
        Constraint::Length(4),                 // button 3: Play as Guest
        Constraint::Length(2),                 // spacer
        Constraint::Length(1),                 // hint line
        Constraint::Min(1),                    // flex
        Constraint::Length(1),                 // branding footer
    ])
    .split(center);

    // Big CHESSTUI logo
    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(theme.logo_color))))
        .collect();
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(Alignment::Center),
        rows[1],
    );

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "Terminal Chess",
        Style::default()
            .fg(theme.text_dim)
            .add_modifier(Modifier::ITALIC),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, rows[2]);

    // Buttons
    let button_width: u16 = 30;
    let buttons_area = |row: Rect| -> Rect {
        let x = row.x + row.width.saturating_sub(button_width) / 2;
        Rect::new(x, row.y, button_width.min(row.width), row.height)
    };

    let btn1 = CardButton::new("◆", "Sign Up", "Create an account", theme)
        .selected(app.launch_selection == 0);
    frame.render_widget(btn1, buttons_area(rows[4]));

    let btn2 =
        CardButton::new("→", "Log In", "Welcome back", theme).selected(app.launch_selection == 1);
    frame.render_widget(btn2, buttons_area(rows[6]));

    let btn3 = CardButton::new("♟", "Play as Guest", "Jump right in", theme)
        .selected(app.launch_selection == 2);
    frame.render_widget(btn3, buttons_area(rows[8]));

    // Hint line
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(
            "j/k",
            Style::default()
                .fg(theme.shortcut_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" navigate  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(theme.shortcut_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" select  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "q",
            Style::default()
                .fg(theme.shortcut_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", Style::default().fg(theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, rows[10]);

    // Branding footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("built by ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "resurgence.cloud",
            Style::default().fg(theme.accent_secondary),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, rows[12]);
}
