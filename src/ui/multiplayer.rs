use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, MultiplayerState};

pub fn draw_multiplayer_tab(frame: &mut Frame, app: &App, area: Rect) {
    match &app.multiplayer_state {
        MultiplayerState::LoggedOut => draw_logged_out(frame, app, area),
        MultiplayerState::Connecting => draw_connecting(frame, app, area),
        MultiplayerState::EnteringEmail => draw_email_input(frame, app, area),
        MultiplayerState::WaitingForOtp => draw_waiting_otp(frame, app, area),
        MultiplayerState::EnteringOtp => draw_otp_input(frame, app, area),
        MultiplayerState::EnteringDisplayName => draw_display_name_input(frame, app, area),
        MultiplayerState::LoggedIn { display_name, elo } => {
            draw_logged_in(frame, app, area, display_name, *elo)
        }
        MultiplayerState::Searching => draw_searching(frame, app, area),
        MultiplayerState::InGame => {} // Handled by Screen::InGame
    }
}

fn draw_logged_out(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 40, 7);
    let rows = Layout::vertical([
        Constraint::Length(1), // spacer
        Constraint::Length(1), // title
        Constraint::Length(1), // spacer
        Constraint::Length(1), // server url
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
    ])
    .split(content);

    let title = Paragraph::new(Line::from(Span::styled(
        "Online Play",
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(title, rows[1]);

    let url = Paragraph::new(Line::from(Span::styled(
        app.server_url.as_str(),
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(url, rows[3]);

    let hint = Paragraph::new(Line::from(Span::styled(
        "Press Enter to connect",
        Style::default().fg(app.theme.text_primary),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(hint, rows[5]);
}

fn draw_connecting(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 20, 3);
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(content);

    let msg = Paragraph::new(Line::from(Span::styled(
        "Connecting...",
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(msg, rows[1]);
}

fn draw_email_input(frame: &mut Frame, app: &App, area: Rect) {
    draw_text_input(
        frame,
        app,
        area,
        "Enter your email",
        &app.login_input.clone(),
        "Enter to submit \u{00b7} Esc to cancel",
    );
}

fn draw_waiting_otp(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 44, 7);
    let rows = Layout::vertical([
        Constraint::Length(1), // spacer
        Constraint::Length(1), // title
        Constraint::Length(1), // spacer
        Constraint::Length(1), // detail
        Constraint::Length(1), // spacer
        Constraint::Length(1), // waiting
    ])
    .split(content);

    let title = Paragraph::new(Line::from(Span::styled(
        "Check your email",
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(title, rows[1]);

    let detail = Paragraph::new(Line::from(Span::styled(
        "A 6-digit code was sent to your email",
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(detail, rows[3]);

    let waiting = Paragraph::new(Line::from(Span::styled(
        "Waiting...",
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(waiting, rows[5]);
}

fn draw_otp_input(frame: &mut Frame, app: &App, area: Rect) {
    draw_text_input(
        frame,
        app,
        area,
        "Enter verification code",
        &app.otp_input.clone(),
        "Enter to verify \u{00b7} Esc to cancel",
    );
}

fn draw_display_name_input(frame: &mut Frame, app: &App, area: Rect) {
    draw_text_input(
        frame,
        app,
        area,
        "Choose a display name",
        &app.display_name_input.clone(),
        "Enter to confirm",
    );
}

fn draw_logged_in(frame: &mut Frame, app: &App, area: Rect, display_name: &str, elo: i32) {
    let items = ["Find Game", "Log Out"];
    let row_width: usize = 28;

    let content = center_block(area, row_width as u16 + 4, items.len() as u16 + 5);
    let rows = Layout::vertical([
        Constraint::Length(1), // spacer
        Constraint::Length(1), // name + elo
        Constraint::Length(1), // spacer
        Constraint::Length(1), // Find Game
        Constraint::Length(1), // Log Out
        Constraint::Length(1), // spacer
    ])
    .split(content);

    let profile = Paragraph::new(Line::from(vec![
        Span::styled(
            display_name,
            Style::default()
                .fg(app.theme.text_bright)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  \u{2502}  ELO: {}", elo),
            Style::default().fg(app.theme.text_dim),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(profile, rows[1]);

    for (i, item) in items.iter().enumerate() {
        let selected = i == app.multiplayer_selection;
        let style = if selected {
            Style::default()
                .fg(app.theme.table_cursor_fg)
                .bg(app.theme.table_cursor_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_primary)
        };
        let prefix = if selected { "  \u{25b8} " } else { "    " };
        let text = format!("{}{}", prefix, item);
        let padded = format!("{:<width$}", text, width = row_width);
        let line = Paragraph::new(Line::from(Span::styled(padded, style)))
            .alignment(Alignment::Center);
        frame.render_widget(line, rows[3 + i]);
    }
}

fn draw_searching(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 36, 5);
    let rows = Layout::vertical([
        Constraint::Length(1), // spacer
        Constraint::Length(1), // title
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
        Constraint::Length(1), // spacer
    ])
    .split(content);

    let msg = Paragraph::new(Line::from(Span::styled(
        "Looking for opponent...",
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(msg, rows[1]);

    let hint = Paragraph::new(Line::from(Span::styled(
        "Esc to cancel",
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(hint, rows[3]);
}

fn draw_text_input(frame: &mut Frame, app: &App, area: Rect, label: &str, value: &str, hint: &str) {
    let content = center_block(area, 40, 7);
    let rows = Layout::vertical([
        Constraint::Length(1), // spacer
        Constraint::Length(1), // label
        Constraint::Length(1), // spacer
        Constraint::Length(1), // input
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
    ])
    .split(content);

    let label_line = Paragraph::new(Line::from(Span::styled(
        label,
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(label_line, rows[1]);

    // Input field with cursor
    let display = format!("  {}\u{258f}  ", value);
    let input_line = Paragraph::new(Line::from(Span::styled(
        display,
        Style::default()
            .fg(app.theme.text_bright)
            .bg(app.theme.dark_square),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(input_line, rows[3]);

    let hint_line = Paragraph::new(Line::from(Span::styled(
        hint,
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(hint_line, rows[5]);
}

fn center_block(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
