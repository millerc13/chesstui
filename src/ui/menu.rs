use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph};
use ratatui::Frame;

use crate::app::{App, MenuTab, PlayMenuItem};
use crate::config::PieceStyle;
use super::board_image;
use super::widgets::CardButton;

const TITLE_ART: &[&str] = &[
    r" ██████╗██╗  ██╗███████╗███████╗███████╗████████╗██╗   ██╗██╗",
    r"██╔════╝██║  ██║██╔════╝██╔════╝██╔════╝╚══██╔══╝██║   ██║██║",
    r"██║     ███████║█████╗  ███████╗███████╗   ██║   ██║   ██║██║",
    r"██║     ██╔══██║██╔══╝  ╚════██║╚════██║   ██║   ██║   ██║██║",
    r"╚██████╗██║  ██║███████╗███████║███████║   ██║   ╚██████╔╝██║",
    r" ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝   ╚═╝    ╚═════╝ ╚═╝",
];

pub fn draw_menu(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // First split: main area vs friends sidebar (right column spans full height)
    let sidebar_width: u16 = 28.min(area.width / 4);
    let main_cols = Layout::horizontal([
        Constraint::Min(40),
        Constraint::Length(sidebar_width),
    ])
    .split(area);

    // Friends sidebar takes the full right column
    super::friends::draw_friends_sidebar(frame, main_cols[1], app);

    // Left column: header + tabs + content + footer
    let left = main_cols[0];
    let rows = Layout::vertical([
        Constraint::Length(1),                           // top padding
        Constraint::Length(TITLE_ART.len() as u16),      // logo
        Constraint::Length(1),                           // subtitle
        Constraint::Length(1),                           // spacer
        Constraint::Length(1),                           // tab bar
        Constraint::Length(1),                           // spacer
        Constraint::Min(6),                              // tab content
        Constraint::Length(1),                           // stats bar
        Constraint::Length(1),                           // hints
        Constraint::Length(1),                           // footer
    ])
    .split(left);

    draw_title(frame, app, rows[1]);
    draw_subtitle(frame, app, rows[2]);
    draw_tab_bar(frame, app, rows[4]);
    draw_tab_content(frame, app, rows[6]);
    draw_stats_bar(frame, app, rows[7]);
    draw_hints(frame, app, rows[8]);
    draw_footer(frame, app, rows[9]);
}

fn draw_title(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = TITLE_ART
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(app.theme.logo_color))))
        .collect();
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn draw_subtitle(frame: &mut Frame, app: &App, area: Rect) {
    let sub = Paragraph::new(Line::from(Span::styled(
        "Terminal Chess",
        Style::default()
            .fg(app.theme.text_dim)
            .add_modifier(Modifier::ITALIC),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(sub, area);
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();
    for (i, tab) in MenuTab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                "  \u{2502}  ",
                Style::default().fg(Color::Indexed(238)),
            ));
        }
        let is_active = *tab == app.active_tab;
        if is_active {
            spans.push(Span::styled(
                tab.name(),
                Style::default()
                    .fg(app.theme.text_bright)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
        } else {
            spans.push(Span::styled(
                tab.name(),
                Style::default().fg(app.theme.text_dim),
            ));
        }
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn draw_tab_content(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.active_tab {
        MenuTab::Play => draw_play_tab(frame, app, area),
        MenuTab::Replays => draw_replays_tab(frame, app, area),
        MenuTab::Multiplayer => super::multiplayer::draw_multiplayer_tab(frame, app, area),
        MenuTab::Settings => draw_settings_tab(frame, app, area),
    }
}

// ── Play tab: CardButton-based menu ────────────────────────────────

fn draw_play_tab(frame: &mut Frame, app: &App, area: Rect) {
    let items = PlayMenuItem::ALL;
    let card_height: u16 = 4;
    let total_height = items.len() as u16 * card_height;
    let card_width: u16 = 40.min(area.width.saturating_sub(4));

    let content = center_block(area, card_width, total_height.min(area.height));

    for (i, item) in items.iter().enumerate() {
        let y = content.y + i as u16 * card_height;
        if y + card_height > content.y + content.height {
            break;
        }

        let card_area = Rect::new(content.x, y, card_width, card_height);

        // Build subtitle — for VsComputer, include difficulty level
        let subtitle_owned: String;
        let subtitle = if matches!(item, PlayMenuItem::VsComputer) {
            subtitle_owned = format!("Battle the AI  [Lvl {} \u{25b8}]", app.ai_difficulty);
            &subtitle_owned
        } else {
            item.subtitle()
        };

        let mut card = CardButton::new(item.icon(), item.title(), subtitle, &app.theme)
            .selected(i == app.play_selection);

        if let Some(tag) = item.tag() {
            card = card.tag(tag);
        }

        frame.render_widget(card, card_area);
    }
}

// ── Replays tab ──────────────────────────────────────────────────────

fn draw_replays_tab(frame: &mut Frame, app: &App, area: Rect) {
    if app.replay_list.is_empty() {
        let content = center_block(area, 44, 3);
        let lines = vec![
            Line::from(vec![
                Span::styled("\u{25cb}  ", Style::default().fg(app.theme.icon_color)),
                Span::styled(
                    "No saved games yet",
                    Style::default().fg(app.theme.text_dim),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    "Play a game to see replays here!",
                    Style::default().fg(app.theme.text_dim).add_modifier(Modifier::ITALIC),
                ),
            ]),
        ];
        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center),
            content,
        );
        return;
    }

    let table_width: usize = 70;

    let rows = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // separator
        Constraint::Min(1),   // list
        Constraint::Length(1), // scrollbar
        Constraint::Length(1), // footer
    ])
    .split(area);

    // Header
    let header_text = format!(
        "  {:<4}  {:<12}  {:<26}{:>7}  {:<8}",
        "#", "Date", "Result", "Moves", "Mode"
    );
    let header_padded = format!("{:<width$}", header_text, width = table_width);
    let header = Line::from(Span::styled(
        header_padded,
        Style::default()
            .fg(app.theme.accent_secondary)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(
        Paragraph::new(header).alignment(Alignment::Center),
        rows[0],
    );

    // Separator
    let sep = "\u{2500}".repeat(table_width);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            sep,
            Style::default().fg(Color::Indexed(238)),
        )))
        .alignment(Alignment::Center),
        rows[1],
    );

    // Replay list
    let visible = rows[2].height as usize;
    let scroll = if app.replay_selection >= visible {
        app.replay_selection - visible + 1
    } else {
        0
    };

    let mut lines = Vec::new();
    for (i, game) in app.replay_list.iter().enumerate().skip(scroll).take(visible) {
        let date_short = if game.date.len() >= 10 {
            &game.date[..10]
        } else {
            &game.date
        };
        let result_text = format!("{} ({})", game.result, game.result_detail);
        let line_text = format!(
            "  {:<4}  {:<12}  {:<26}{:>7}  {:<8}",
            i + 1,
            date_short,
            result_text,
            game.move_count,
            game.mode,
        );
        let padded = format!("{:<width$}", line_text, width = table_width);

        let style = if i == app.replay_selection {
            Style::default()
                .fg(app.theme.table_cursor_fg)
                .bg(app.theme.table_cursor_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_primary)
        };
        lines.push(Line::from(Span::styled(padded, style)));
    }
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        rows[2],
    );

    // Scrollbar indicator
    if app.replay_list.len() > visible && visible > 0 {
        let progress = app.replay_selection as f64 / (app.replay_list.len() - 1).max(1) as f64;
        let bar = scrollbar_string(progress, 20);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                bar,
                Style::default().fg(app.theme.text_dim),
            )))
            .alignment(Alignment::Center),
            rows[3],
        );
    }

    // Footer hints
    let footer = Line::from(vec![
        Span::styled("Enter", Style::default().fg(app.theme.accent)),
        Span::styled(" replay  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("d", Style::default().fg(app.theme.shortcut_color)),
        Span::styled(" delete  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("j/k", Style::default().fg(app.theme.accent)),
        Span::styled(" navigate", Style::default().fg(app.theme.text_dim)),
    ]);
    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), rows[4]);
}

// ── Settings tab ────────────────────────────────────────────────────

fn draw_settings_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let list_len = PieceStyle::ALL.len() as u16;
    // Total content height: section header + list + blank + description + blank + hints
    let content_h = 1 + list_len + 1 + 1 + 1 + 1;
    // Content width: left list panel + gap + right preview panel
    let list_w: u16 = 24;
    let preview_w: u16 = 34;
    let gap: u16 = 3;
    let content_w = list_w + gap + preview_w;

    // Center the whole block
    let outer = center_block(area, content_w, content_h);

    // Split into left (list) and right (preview) columns
    let cols = Layout::horizontal([
        Constraint::Length(list_w),
        Constraint::Length(gap),
        Constraint::Length(preview_w),
    ]).split(outer);

    let left = cols[0];
    let right = cols[2];

    // ── Left column: style list ──────────────────────────────────────

    let left_rows = Layout::vertical([
        Constraint::Length(1),          // section header
        Constraint::Length(1),          // blank
        Constraint::Length(list_len),   // style list
        Constraint::Length(1),          // blank
        Constraint::Length(1),          // description
        Constraint::Min(0),            // spacer
        Constraint::Length(1),          // hints
    ]).split(left);

    // Section header
    let header = Line::from(Span::styled(
        "Piece Style",
        Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(header), left_rows[0]);

    // Style list
    let current_idx = PieceStyle::ALL.iter().position(|&s| s == app.piece_style).unwrap_or(0);
    let items: Vec<Line> = PieceStyle::ALL.iter().enumerate().map(|(i, style)| {
        let cursor_str = if i == app.settings_style_index { " \u{25b8} " } else { "   " };
        let check = if i == current_idx { " \u{2713}" } else { "" };
        let base_style = if i == app.settings_style_index {
            Style::default().fg(app.theme.text_bright).bg(app.theme.accent)
        } else {
            Style::default().fg(app.theme.text_primary)
        };
        Line::from(vec![
            Span::styled(format!("{}{}", cursor_str, style.name()), base_style),
            Span::styled(check.to_string(), Style::default().fg(app.theme.accent_secondary)),
        ])
    }).collect();
    frame.render_widget(Paragraph::new(items), left_rows[2]);

    // Description
    let desc = PieceStyle::ALL[app.settings_style_index].description();
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            desc,
            Style::default().fg(app.theme.text_dim).add_modifier(Modifier::ITALIC),
        ))),
        left_rows[4],
    );

    // Hints
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "j/k nav  Enter select",
            Style::default().fg(app.theme.text_dim),
        ))),
        left_rows[6],
    );

    // ── Right column: piece preview ──────────────────────────────────

    // Draw a bordered frame for the preview
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.border_dim))
        .title(Span::styled(
            " Preview ",
            Style::default().fg(app.theme.text_dim),
        ));
    let preview_inner = preview_block.inner(right);
    frame.render_widget(preview_block, right);

    // Render piece preview image if terminal supports graphics protocol
    let cell_size = app.board_picker.as_ref().map(|p| p.font_size());
    if let Some(cell_size) = cell_size {
        if cell_size.0 > 0 && cell_size.1 > 0 && preview_inner.width > 2 && preview_inner.height > 2 {
            // Compute pixel dimensions for the preview
            let avail_px_w = preview_inner.width as u32 * cell_size.0 as u32;
            let avail_px_h = preview_inner.height as u32 * cell_size.1 as u32;

            // Preview is 6 wide × 2 tall squares. Find sq_px that fits.
            let sq_px_from_w = avail_px_w / 6;
            let sq_px_from_h = avail_px_h / 2;
            let sq_px = sq_px_from_w.min(sq_px_from_h).max(8);

            let selected_style = PieceStyle::ALL[app.settings_style_index];

            // Only regenerate preview protocol when style or size changed
            let need_regen = app.cached_preview_protocol.is_none()
                || app.cached_preview_style != Some(selected_style)
                || app.cached_preview_sq_px != sq_px;
            if need_regen {
                let img = board_image::render_piece_preview(&app.theme, sq_px, selected_style);
                let dyn_img: image::DynamicImage = img.into();
                let picker = app.board_picker.as_ref().unwrap();
                let protocol = picker.new_resize_protocol(dyn_img);
                app.cached_preview_protocol = Some(protocol);
                app.cached_preview_style = Some(selected_style);
                app.cached_preview_sq_px = sq_px;
            }

            // Compute char cell dimensions the image will occupy
            let img_px_w = sq_px * 6;
            let img_px_h = sq_px * 2;
            let img_char_w = ((img_px_w + cell_size.0 as u32 - 1) / cell_size.0 as u32) as u16;
            let img_char_h = ((img_px_h + cell_size.1 as u32 - 1) / cell_size.1 as u32) as u16;

            // Center image within preview_inner
            let ix = preview_inner.x + preview_inner.width.saturating_sub(img_char_w) / 2;
            let iy = preview_inner.y + preview_inner.height.saturating_sub(img_char_h) / 2;
            let img_area = Rect::new(
                ix, iy,
                img_char_w.min(preview_inner.width),
                img_char_h.min(preview_inner.height),
            );

            let protocol = app.cached_preview_protocol.as_mut().unwrap();
            let image_widget = ratatui_image::StatefulImage::default();
            frame.render_stateful_widget(image_widget, img_area, protocol);
        }
    } else {
        // Fallback: text-based preview
        let fallback = vec![
            Line::from(Span::styled(
                "\u{2654} \u{2655} \u{2656} \u{2657} \u{2658} \u{2659}",
                Style::default().fg(app.theme.text_bright),
            )),
            Line::from(Span::styled(
                "\u{265a} \u{265b} \u{265c} \u{265d} \u{265e} \u{265f}",
                Style::default().fg(app.theme.text_dim),
            )),
        ];
        let fy = preview_inner.y + preview_inner.height.saturating_sub(2) / 2;
        let fallback_area = Rect::new(
            preview_inner.x, fy,
            preview_inner.width, 2.min(preview_inner.height),
        );
        frame.render_widget(
            Paragraph::new(fallback).alignment(Alignment::Center),
            fallback_area,
        );
    }
}

// ── Stats bar ─────────────────────────────────────────────────────

fn draw_stats_bar(frame: &mut Frame, app: &App, area: Rect) {
    let stats = Paragraph::new(Line::from(vec![
        Span::styled("Games: ", Style::default().fg(app.theme.text_dim)),
        Span::styled("0", Style::default().fg(app.theme.text_primary)),
        Span::styled(" │ ", Style::default().fg(Color::Indexed(238))),
        Span::styled("W:", Style::default().fg(app.theme.text_dim)),
        Span::styled("0", Style::default().fg(app.theme.text_primary)),
        Span::styled(" D:", Style::default().fg(app.theme.text_dim)),
        Span::styled("0", Style::default().fg(app.theme.text_primary)),
        Span::styled(" L:", Style::default().fg(app.theme.text_dim)),
        Span::styled("0", Style::default().fg(app.theme.text_primary)),
        Span::styled(" │ ", Style::default().fg(Color::Indexed(238))),
        Span::styled("ELO: ", Style::default().fg(app.theme.text_dim)),
        Span::styled("--", Style::default().fg(app.theme.text_primary)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(stats, area);
}

// ── Hints bar ──────────────────────────────────────────────────────

fn draw_hints(frame: &mut Frame, app: &App, area: Rect) {
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("\u{2190}\u{2192}", Style::default().fg(app.theme.accent)),
        Span::styled(" tabs  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("\u{2502}", Style::default().fg(Color::Indexed(238))),
        Span::styled("  j/k", Style::default().fg(app.theme.accent)),
        Span::styled(" navigate  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("\u{2502}", Style::default().fg(Color::Indexed(238))),
        Span::styled("  Enter", Style::default().fg(app.theme.accent)),
        Span::styled(" select  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("\u{2502}", Style::default().fg(Color::Indexed(238))),
        Span::styled("  q", Style::default().fg(app.theme.shortcut_color)),
        Span::styled(" quit", Style::default().fg(app.theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hints, area);
}

// ── Footer ─────────────────────────────────────────────────────────

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("chesstui v{}", version),
            Style::default().fg(app.theme.text_dim),
        ),
        Span::styled("  \u{00b7}  built by ", Style::default().fg(app.theme.text_dim)),
        Span::styled("resurgence.cloud", Style::default().fg(app.theme.accent_secondary)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

// ── Scrollbar ──────────────────────────────────────────────────────

fn scrollbar_string(progress: f64, width: usize) -> String {
    let pos = (progress * (width - 1) as f64).round() as usize;
    let mut s = String::with_capacity(width);
    for i in 0..width {
        if i == pos {
            s.push('\u{2588}');
        } else if i < pos {
            s.push('\u{2584}');
        } else {
            s.push('\u{2581}');
        }
    }
    s
}

// ── Helpers ────────────────────────────────────────────────────────

fn center_block(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
