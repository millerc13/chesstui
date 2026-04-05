use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::theme::{ColorScheme, Theme};

const LOGO: &[&str] = &[
    r"  ██████╗██╗  ██╗███████╗███████╗███████╗████████╗██╗   ██╗██╗",
    r" ██╔════╝██║  ██║██╔════╝██╔════╝██╔════╝╚══██╔══╝██║   ██║██║",
    r" ██║     ███████║█████╗  ███████╗███████╗   ██║   ██║   ██║██║",
    r" ██║     ██╔══██║██╔══╝  ╚════██║╚════██║   ██║   ██║   ██║██║",
    r" ╚██████╗██║  ██║███████╗███████║███████║   ██║   ╚██████╔╝██║",
    r"  ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝   ╚═╝    ╚═════╝ ╚═╝",
];

pub fn draw_color_picker(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let preview = Theme::from_scheme(ColorScheme::ALL[app.color_scheme_index]);

    // Split into 3 columns: left piece | center content | right piece
    let has_pieces = area.width >= 90;
    let center_area = if has_pieces {
        let cols = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

        // Left piece
        if cols[0].width >= 10 && cols[0].height >= 8 {
            let piece = super::ascii3d::current_display_piece(app.tick);
            let lines = super::ascii3d::render_to_lines(
                piece,
                cols[0].width.min(35),
                cols[0].height.min(20),
                app.tick,
                &preview,
            );
            frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), cols[0]);
        }

        // Right piece (different piece)
        if cols[2].width >= 10 && cols[2].height >= 8 {
            let piece = super::ascii3d::current_display_piece_alt(app.tick);
            let lines = super::ascii3d::render_to_lines(
                piece,
                cols[2].width.min(35),
                cols[2].height.min(20),
                app.tick,
                &preview,
            );
            frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), cols[2]);
        }

        cols[1]
    } else {
        area
    };

    let rows = Layout::vertical([
        Constraint::Length(3),                                 // top padding
        Constraint::Length(LOGO.len() as u16),                 // logo
        Constraint::Length(1),                                 // subtitle
        Constraint::Length(2),                                 // spacer
        Constraint::Length(1),                                 // section title
        Constraint::Length(1),                                 // spacer
        Constraint::Length(ColorScheme::ALL.len() as u16 + 2), // color list + padding
        Constraint::Length(2),                                 // spacer
        Constraint::Length(1),                                 // session status
        Constraint::Length(1),                                 // spacer
        Constraint::Length(1),                                 // continue hint
        Constraint::Min(1),                                    // flex
        Constraint::Length(1),                                 // footer
    ])
    .split(center_area);

    // ── Logo ──
    draw_logo(frame, &preview, rows[1]);

    // ── Subtitle ──
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "Terminal Chess",
        Style::default()
            .fg(preview.text_dim)
            .add_modifier(Modifier::ITALIC),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, rows[2]);

    // ── "Choose Your Theme" section ──
    let section_title = Paragraph::new(Line::from(vec![
        Span::styled("─── ", Style::default().fg(Color::Indexed(238))),
        Span::styled(
            "Choose Your Theme",
            Style::default()
                .fg(preview.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ───", Style::default().fg(Color::Indexed(238))),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(section_title, rows[4]);

    // ── Color scheme list ──
    draw_scheme_list(frame, app, &preview, rows[6]);

    // ── Session status ──
    draw_session_status(frame, app, &preview, rows[8]);

    // ── Continue hint ──
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(preview.shortcut_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to continue  ", Style::default().fg(preview.text_dim)),
        Span::styled("j/k", Style::default().fg(preview.shortcut_color)),
        Span::styled(" to browse themes  ", Style::default().fg(preview.text_dim)),
        Span::styled("q", Style::default().fg(preview.shortcut_color)),
        Span::styled(" to quit", Style::default().fg(preview.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, rows[10]);

    // ── Footer ──
    let version = env!("CARGO_PKG_VERSION");
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("v{}", version),
            Style::default().fg(preview.text_dim),
        ),
        Span::styled("  ·  built by ", Style::default().fg(preview.text_dim)),
        Span::styled(
            "resurgence.cloud",
            Style::default().fg(preview.accent_secondary),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, rows[12]);
}

fn draw_logo(frame: &mut Frame, theme: &Theme, area: Rect) {
    let lines: Vec<Line> = LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(theme.logo_color))))
        .collect();
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn draw_scheme_list(frame: &mut Frame, app: &App, _preview: &Theme, area: Rect) {
    let schemes = ColorScheme::ALL;
    let list_width: u16 = 48;
    let cx = area.x + area.width.saturating_sub(list_width) / 2;

    for (i, scheme) in schemes.iter().enumerate() {
        let y = area.y + 1 + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let scheme_theme = Theme::from_scheme(*scheme);
        let selected = i == app.color_scheme_index;

        // Arrow indicator
        let arrow = if selected { " ▸ " } else { "   " };
        let arrow_style = if selected {
            Style::default().fg(scheme_theme.text_bright)
        } else {
            Style::default().fg(Color::Indexed(240))
        };

        // Name
        let name_style = if selected {
            Style::default()
                .fg(scheme_theme.text_bright)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Indexed(245))
        };

        // Color swatches — show BOTH primary AND secondary scheme colors
        let swatch_primary = "██████";
        let swatch_secondary = "███";

        let buf = frame.buffer_mut();
        buf.set_string(cx, y, arrow, arrow_style);
        buf.set_string(cx + 3, y, scheme.name(), name_style);
        buf.set_string(
            cx + 22,
            y,
            swatch_primary,
            Style::default().fg(scheme_theme.logo_color),
        );
        buf.set_string(
            cx + 28,
            y,
            swatch_secondary,
            Style::default().fg(scheme_theme.accent_secondary),
        );

        // Row highlight for selected
        if selected {
            for x in cx..cx + list_width {
                if x < area.x + area.width {
                    if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, y)) {
                        cell.set_style(cell.style().bg(Color::Indexed(236)));
                    }
                }
            }
        }
    }
}

fn draw_session_status(frame: &mut Frame, _app: &App, theme: &Theme, area: Rect) {
    let session = crate::network::session::load_session();

    let line = if let Some(ref sess) = session {
        let name = sess.display_name.as_deref().unwrap_or(&sess.email);
        Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Rgb(152, 190, 101))),
            Span::styled("Signed in as ", Style::default().fg(theme.text_dim)),
            Span::styled(
                name,
                Style::default()
                    .fg(theme.text_bright)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("○ ", Style::default().fg(theme.text_dim)),
            Span::styled(
                "Not signed in — sign up or log in from the Multiplayer tab",
                Style::default()
                    .fg(theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            ),
        ])
    };

    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}
