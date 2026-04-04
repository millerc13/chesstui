use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, MenuTab};

const TITLE_ART: &[&str] = &[
    r"       _                   _         _ ",
    r"   ___| |__   ___  ___ ___| |_ _   _(_)",
    r"  / __| '_ \ / _ \/ __/ __| __| | | | |",
    r" | (__| | | |  __/\__ \__ \ |_| |_| | |",
    r"  \___|_| |_|\___||___/___/\__|\__,_|_|",
];

pub fn draw_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(TITLE_ART.len() as u16 + 1), // title
        Constraint::Length(1),                           // subtitle
        Constraint::Length(1),                           // spacer
        Constraint::Length(1),                           // tab bar
        Constraint::Length(1),                           // tab underline
        Constraint::Min(8),                              // tab content
        Constraint::Length(1),                           // hints
    ])
    .split(area);

    draw_title(frame, app, rows[0]);
    draw_subtitle(frame, app, rows[1]);
    draw_tab_bar(frame, app, rows[3]);
    draw_tab_underline(frame, app, rows[4]);
    draw_tab_content(frame, app, rows[5]);
    draw_hints(frame, app, rows[6]);
}

fn draw_title(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = TITLE_ART
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(app.theme.accent))))
        .collect();
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn draw_subtitle(frame: &mut Frame, app: &App, area: Rect) {
    let sub = Paragraph::new(Line::from(Span::styled(
        "Terminal Chess",
        Style::default().fg(app.theme.text_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(sub, area);
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();
    for (i, tab) in MenuTab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                " \u{2502} ",
                Style::default().fg(app.theme.text_dim),
            ));
        }
        let style = if *tab == app.active_tab {
            Style::default()
                .fg(app.theme.text_bright)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_dim)
        };
        spans.push(Span::styled(format!(" {} ", tab.name()), style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn draw_tab_underline(frame: &mut Frame, app: &App, area: Rect) {
    let line = "\u{2500}".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            line,
            Style::default().fg(app.theme.text_dim),
        ))),
        area,
    );
}

fn draw_tab_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        MenuTab::Play => draw_play_tab(frame, app, area),
        MenuTab::Replays => draw_replays_tab(frame, app, area),
        MenuTab::Multiplayer => super::multiplayer::draw_multiplayer_tab(frame, app, area),
        MenuTab::Settings => draw_placeholder(frame, app, area, "Settings", "Board theme, piece style, sound \u{2014} coming soon."),
        MenuTab::Controls => draw_controls_tab(frame, app, area),
    }
}

fn draw_play_tab(frame: &mut Frame, app: &App, area: Rect) {
    let items = ["Play vs AI", "Local Game", "Quit"];
    let row_width: usize = 24;
    let content_area = center_block(area, row_width as u16, items.len() as u16 + 2);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for (i, item) in items.iter().enumerate() {
        let selected = i == app.play_selection;
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
        // Pad to full row width
        let padded = format!("{:<width$}", text, width = row_width);
        lines.push(Line::from(Span::styled(padded, style)));
    }
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        content_area,
    );
}

fn draw_replays_tab(frame: &mut Frame, app: &App, area: Rect) {
    if app.replay_list.is_empty() {
        let msg = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "No saved games yet",
                Style::default().fg(app.theme.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Play your first game to see replays here!",
                Style::default().fg(app.theme.text_dim),
            )),
        ];
        frame.render_widget(Paragraph::new(msg).alignment(Alignment::Center), area);
        return;
    }

    // Column widths: #(4) Date(12) Result(26) Moves(7) Mode(8) + gaps(2 each) + margins(4)
    let table_width: usize = 70;

    let rows = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // separator
        Constraint::Min(1),   // list
        Constraint::Length(1), // spacer
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
            .fg(app.theme.text_dim)
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
            Style::default().fg(app.theme.text_dim),
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
        // Pad to full table width for consistent highlight
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

    // Footer hints
    let footer = Line::from(vec![
        Span::styled("Enter", Style::default().fg(app.theme.accent)),
        Span::styled(" replay  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("d", Style::default().fg(app.theme.accent)),
        Span::styled(" delete  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("j/k", Style::default().fg(app.theme.accent)),
        Span::styled(" navigate", Style::default().fg(app.theme.text_dim)),
    ]);
    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), rows[4]);
}

fn draw_controls_tab(frame: &mut Frame, app: &App, area: Rect) {
    let content_area = center_block(area, 44, 18);
    let dim = Style::default().fg(app.theme.text_dim);
    let bright = Style::default().fg(app.theme.text_primary);
    let accent = Style::default().fg(app.theme.accent);

    let sep = "\u{2500}".repeat(30);
    let lines = vec![
        Line::from(Span::styled("Game Controls", accent)),
        Line::from(Span::styled(sep.clone(), dim)),
        Line::from(vec![Span::styled("  Move input    ", bright), Span::styled("e2e4, Nf3, O-O", dim)]),
        Line::from(vec![Span::styled("  Navigation    ", bright), Span::styled("\u{2190}\u{2191}\u{2193}\u{2192} / hjkl", dim)]),
        Line::from(vec![Span::styled("  Cycle pieces  ", bright), Span::styled("Tab / Shift+Tab", dim)]),
        Line::from(vec![Span::styled("  Select        ", bright), Span::styled("Enter", dim)]),
        Line::from(vec![Span::styled("  Deselect      ", bright), Span::styled("Esc", dim)]),
        Line::from(vec![Span::styled("  Command mode  ", bright), Span::styled(":", dim)]),
        Line::from(vec![Span::styled("  Help          ", bright), Span::styled("?", dim)]),
        Line::from(""),
        Line::from(Span::styled("Commands", accent)),
        Line::from(Span::styled(sep, dim)),
        Line::from(vec![Span::styled("  :quit         ", bright), Span::styled("Exit game", dim)]),
        Line::from(vec![Span::styled("  :resign       ", bright), Span::styled("Resign", dim)]),
        Line::from(vec![Span::styled("  :flip         ", bright), Span::styled("Flip board", dim)]),
        Line::from(vec![Span::styled("  :new          ", bright), Span::styled("New game", dim)]),
    ];
    frame.render_widget(Paragraph::new(lines), content_area);
}

fn draw_placeholder(frame: &mut Frame, app: &App, area: Rect, title: &str, desc: &str) {
    let content = center_block(area, 36, 7);
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.text_dim))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", desc),
            Style::default().fg(app.theme.text_primary),
        )),
        Line::from(""),
    ])
    .block(inner_block);

    frame.render_widget(text, content);
}

fn draw_hints(frame: &mut Frame, app: &App, area: Rect) {
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("\u{2190}\u{2192}", Style::default().fg(app.theme.accent)),
        Span::styled(" tabs  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("j/k", Style::default().fg(app.theme.accent)),
        Span::styled(" navigate  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("Enter", Style::default().fg(app.theme.accent)),
        Span::styled(" select  ", Style::default().fg(app.theme.text_dim)),
        Span::styled("q", Style::default().fg(app.theme.accent)),
        Span::styled(" quit", Style::default().fg(app.theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hints, area);
}

fn center_block(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
