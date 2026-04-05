use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph};
use ratatui::Frame;

use crate::app::{App, MultiplayerState};
use super::widgets::CardButton;

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub fn draw_multiplayer_tab(frame: &mut Frame, app: &App, area: Rect) {
    match &app.multiplayer_state {
        MultiplayerState::LoggedOut => draw_logged_out(frame, app, area),
        MultiplayerState::Connecting => draw_connecting(frame, app, area),
        MultiplayerState::EnteringEmail => draw_email_input(frame, app, area),
        MultiplayerState::WaitingForOtp => draw_waiting_otp(frame, app, area),
        MultiplayerState::EnteringOtp => draw_otp_input(frame, app, area),
        MultiplayerState::EnteringDisplayName => draw_display_name_input(frame, app, area),
        MultiplayerState::EnteringPassword => draw_password_input(frame, app, area, "Set a Password", "Min 6 characters · Enter to confirm"),
        MultiplayerState::EnteringLoginPassword => draw_password_input(frame, app, area, "Enter Password", "Enter to log in · Esc to go back"),
        MultiplayerState::LoggedIn { display_name, elo } => {
            draw_logged_in(frame, app, area, display_name, *elo)
        }
        MultiplayerState::Searching => draw_searching(frame, app, area),
        MultiplayerState::InGame => {} // Handled by Screen::InGame
    }
}

fn draw_logged_out(frame: &mut Frame, app: &App, area: Rect) {
    let card_width: u16 = 36;
    let card_height: u16 = 4;
    let gap: u16 = 1;
    let header_height: u16 = 3;
    let total_height = header_height + 2 * card_height + gap;
    let content = center_block(area, card_width, total_height);

    // Server URL line centered
    let url_rect = Rect::new(content.x, content.y, card_width, 1);
    let url_line = Paragraph::new(Line::from(vec![
        Span::styled("Server: ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            &app.server_url,
            Style::default().fg(app.theme.accent_secondary),
        ),
    ]));
    frame.render_widget(url_line, url_rect);

    // CardButton items
    let items: &[(&str, &str, &str)] = &[
        ("◆", "Sign Up", "Create a new account"),
        ("→", "Log In", "Welcome back"),
    ];
    for (i, (icon, title, sub)) in items.iter().enumerate() {
        let y = content.y + header_height + i as u16 * (card_height + gap);
        let card_area = Rect::new(content.x, y, card_width, card_height);
        let card = CardButton::new(icon, title, sub, &app.theme)
            .selected(i == app.multiplayer_selection);
        frame.render_widget(card, card_area);
    }
}

fn draw_connecting(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 36, 5);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.text_dim))
        .title(Span::styled(
            " Connecting ",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let spinner = SPINNER[(app.tick as usize) % SPINNER.len()];
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {} ", spinner),
                Style::default().fg(app.theme.accent_secondary),
            ),
            Span::styled(
                "Reaching server...",
                Style::default().fg(app.theme.text_dim),
            ),
        ]),
        Line::from(""),
    ])
    .block(block);
    frame.render_widget(text, content);
}

fn draw_email_input(frame: &mut Frame, app: &App, area: Rect) {
    draw_text_input(
        frame,
        app,
        area,
        "Enter your email",
        &app.login_input.clone(),
        "Enter to submit · Esc to cancel",
    );
}

fn draw_waiting_otp(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 44, 7);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.text_dim))
        .title(Span::styled(
            " Check Your Email ",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let spinner = SPINNER[(app.tick as usize) % SPINNER.len()];
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  A 6-digit code was sent to your email",
            Style::default().fg(app.theme.text_bright),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {} ", spinner),
                Style::default().fg(app.theme.accent_secondary),
            ),
            Span::styled(
                "Waiting...",
                Style::default()
                    .fg(app.theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
        Line::from(""),
    ])
    .block(block);
    frame.render_widget(text, content);
}

fn draw_otp_input(frame: &mut Frame, app: &App, area: Rect) {
    draw_text_input(
        frame,
        app,
        area,
        "Enter verification code",
        &app.otp_input.clone(),
        "Enter to verify · Esc to cancel",
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
    let card_width: u16 = 36;
    let card_height: u16 = 4;
    let gap: u16 = 1;
    let profile_height: u16 = 3; // profile line + spacer
    let total_height = profile_height + 2 * card_height + gap;
    let content = center_block(area, card_width, total_height);

    // Profile line — centered, name bright, ELO in accent
    let profile_rect = Rect::new(content.x, content.y, card_width, 1);
    let profile = Paragraph::new(Line::from(vec![
        Span::styled("♚ ", Style::default().fg(app.theme.accent)),
        Span::styled(
            display_name,
            Style::default()
                .fg(app.theme.text_bright)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(ratatui::style::Color::Indexed(238))),
        Span::styled("ELO: ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            format!("{}", elo),
            Style::default().fg(app.theme.accent_secondary),
        ),
    ]));
    frame.render_widget(profile, profile_rect);

    // CardButton items
    let items: &[(&str, &str, &str)] = &[
        ("⚔", "Find Game", "Match with an opponent"),
        ("×", "Log Out", "Return to guest mode"),
    ];
    for (i, (icon, title, sub)) in items.iter().enumerate() {
        let y = content.y + profile_height + i as u16 * (card_height + gap);
        let card_area = Rect::new(content.x, y, card_width, card_height);
        let card = CardButton::new(icon, title, sub, &app.theme)
            .selected(i == app.multiplayer_selection);
        frame.render_widget(card, card_area);
    }
}

fn draw_searching(frame: &mut Frame, app: &App, area: Rect) {
    let content = center_block(area, 36, 7);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.text_dim))
        .title(Span::styled(
            " Matchmaking ",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let spinner = SPINNER[(app.tick as usize) % SPINNER.len()];
    let dots = ".".repeat((app.tick as usize / 5) % 4);
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {} ", spinner),
                Style::default().fg(app.theme.accent_secondary),
            ),
            Span::styled(
                format!("Looking for opponent{}", dots),
                Style::default().fg(app.theme.text_bright),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Esc to cancel",
            Style::default()
                .fg(app.theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
    ])
    .block(block);
    frame.render_widget(text, content);
}

fn draw_text_input(frame: &mut Frame, app: &App, area: Rect, label: &str, value: &str, hint: &str) {
    let has_error = !app.status_message.is_empty();
    let height = if has_error { 11 } else { 9 };
    let content = center_block(area, 44, height);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.text_dim))
        .title(Span::styled(
            format!(" {} ", label),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    // Input field with cursor
    let display = format!("  {}▏", value);
    let input_padded = format!("{:<40}", display);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", input_padded),
            Style::default()
                .fg(app.theme.text_bright),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default()
                .fg(app.theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    if has_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", app.status_message),
            Style::default().fg(ratatui::style::Color::Rgb(224, 108, 117)),
        )));
    }
    lines.push(Line::from(""));

    let text = Paragraph::new(lines).block(block);
    frame.render_widget(text, content);
}

fn draw_password_input(frame: &mut Frame, app: &App, area: Rect, label: &str, hint: &str) {
    let content = center_block(area, 44, 9);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.text_dim))
        .title(Span::styled(
            format!(" {} ", label),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let masked: String = "•".repeat(app.password_input.len());
    let display = format!("  {}▏", masked);
    let input_padded = format!("{:<40}", display);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", input_padded),
            Style::default().fg(app.theme.text_bright),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default()
                .fg(app.theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
    ])
    .block(block);
    frame.render_widget(text, content);
}

fn center_block(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
