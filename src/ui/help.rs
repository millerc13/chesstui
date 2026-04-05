use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::app::App;
use crate::ui::widgets::{render_rounded_modal, render_section_header};

/// (category, key, description)
const KEYBINDINGS: &[(&str, &str, &str)] = &[
    // Movement
    ("MOVEMENT", "h j k l", "Move cursor"),
    ("MOVEMENT", "Enter", "Select / move piece"),
    ("MOVEMENT", "Tab", "Next movable piece"),
    ("MOVEMENT", "Shift+Tab", "Previous movable piece"),
    ("MOVEMENT", "← → ↑ ↓", "Arrow navigation"),
    // Input
    ("INPUT", "e2e4", "Algebraic move"),
    ("INPUT", "Nf3", "Piece notation"),
    ("INPUT", "O-O", "Castle kingside"),
    ("INPUT", "O-O-O", "Castle queenside"),
    // Commands
    ("COMMANDS", ":resign", "Resign game"),
    ("COMMANDS", ":flip", "Flip board"),
    ("COMMANDS", ":new", "New game"),
    ("COMMANDS", ":kitty", "Toggle image mode"),
    ("COMMANDS", ":debug", "Toggle debug panel"),
    ("COMMANDS", ":quit", "Exit"),
    // General
    ("GENERAL", "?", "Toggle help"),
    ("GENERAL", "Esc", "Cancel / close"),
    ("GENERAL", "Click", "Select square"),
    ("GENERAL", ":", "Enter command mode"),
];

pub fn draw_help_modal(frame: &mut Frame, app: &App) {
    // Debug log
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/chesstui-debug.log")
    {
        use std::io::Write;
        let _ = writeln!(
            f,
            "[draw] draw_help_modal called, show_help={}",
            app.show_help
        );
    }
    let area = frame.area();
    // Size the modal to fully cover the board image (board is ~66% width, full height minus bars)
    let modal_width = (area.width * 75 / 100)
        .max(55)
        .min(area.width.saturating_sub(2));
    let modal_height = area.height.saturating_sub(4).max(10);

    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    let inner = render_rounded_modal(frame, modal_area, "HELP", "Esc to close", &app.theme);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    let buf = frame.buffer_mut();
    let theme = &app.theme;
    let search = &app.help_search;
    let search_lower = search.to_lowercase();

    let bg = ratatui::style::Color::Indexed(235);

    // Line 0: Search input
    let search_style = Style::default().fg(theme.text_bright).bg(bg);
    let prompt_style = Style::default().fg(theme.accent).bg(bg);
    let cursor_style = Style::default().fg(theme.text_dim).bg(bg);

    let search_y = inner.y;
    buf.set_string(inner.x, search_y, "> ", prompt_style);
    buf.set_string(inner.x + 2, search_y, search, search_style);
    let cursor_x = inner.x + 2 + search.len() as u16;
    if cursor_x < inner.x + inner.width {
        buf.set_string(cursor_x, search_y, "▏", cursor_style);
    }

    // Line 1: Horizontal divider
    let divider_y = inner.y + 1;
    if divider_y < inner.y + inner.height {
        let divider = "─".repeat(inner.width as usize);
        let divider_style = Style::default().fg(theme.border_dim).bg(bg);
        buf.set_string(inner.x, divider_y, &divider, divider_style);
    }

    // Build the content lines: Vec of enum { SectionHeader(label), Binding(key, desc), Blank }
    // Filter by search query
    let filtered: Vec<(&str, &str, &str)> = if search.is_empty() {
        KEYBINDINGS.to_vec()
    } else {
        KEYBINDINGS
            .iter()
            .filter(|(_cat, key, desc)| {
                key.to_lowercase().contains(&search_lower)
                    || desc.to_lowercase().contains(&search_lower)
            })
            .copied()
            .collect()
    };

    let result_count = filtered.len();

    // Build display lines as a flat list
    // Each entry: either a section header or a keybinding line
    enum DisplayLine<'a> {
        Blank,
        Section(&'a str),
        Binding(&'a str, &'a str),
    }

    let mut lines: Vec<DisplayLine> = Vec::new();
    let mut current_category: Option<&str> = None;

    for (cat, key, desc) in &filtered {
        if current_category != Some(cat) {
            if current_category.is_some() {
                lines.push(DisplayLine::Blank);
            }
            lines.push(DisplayLine::Section(cat));
            current_category = Some(cat);
        }
        lines.push(DisplayLine::Binding(key, desc));
    }

    // Add blank + result count at bottom
    lines.push(DisplayLine::Blank);

    // Available content area starts at line 2 (after search + divider)
    let content_start_y = inner.y + 2;
    let content_height = inner.height.saturating_sub(2) as usize;

    if content_height == 0 {
        return;
    }

    // Clamp scroll
    let max_scroll = lines.len().saturating_sub(content_height);
    let scroll = app.help_scroll.min(max_scroll);

    // Render visible lines
    let key_style = Style::default()
        .fg(theme.accent)
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.text_primary).bg(bg);

    // Reserve last visible line for the result count
    let visible_end = (scroll + content_height).min(lines.len());

    for (i, line) in lines[scroll..visible_end].iter().enumerate() {
        let line_y = content_start_y + i as u16;
        if line_y >= inner.y + inner.height {
            break;
        }

        match line {
            DisplayLine::Blank => {}
            DisplayLine::Section(label) => {
                render_section_header(buf, inner, line_y, label, theme);
            }
            DisplayLine::Binding(key, desc) => {
                let key_col = format!("  {:>14}  ", key);
                buf.set_string(inner.x, line_y, &key_col, key_style);
                let desc_x = inner.x + key_col.len() as u16;
                if desc_x < inner.x + inner.width {
                    buf.set_string(desc_x, line_y, desc, desc_style);
                }
            }
        }
    }

    // Draw result count at the very bottom of the inner area
    let count_y = inner.y + inner.height - 1;
    let count_text = if search.is_empty() {
        format!("{} keybindings", result_count)
    } else {
        format!("{} results", result_count)
    };
    let count_style = Style::default().fg(theme.text_dim).bg(bg);
    // Center the count text
    let count_x = inner.x + (inner.width.saturating_sub(count_text.len() as u16)) / 2;
    if count_y >= content_start_y {
        buf.set_string(count_x, count_y, &count_text, count_style);
    }
}
