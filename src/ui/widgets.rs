use cozy_chess::{Color as ChessColor, Piece};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;
use ratatui::Frame;

use crate::theme::Theme;

// ─────────────────────────────────────────────────────────────────────────────
// 1. CardButton
// ─────────────────────────────────────────────────────────────────────────────

pub struct CardButton<'a> {
    icon: &'a str,
    title: &'a str,
    subtitle: &'a str,
    tag: Option<&'a str>,
    selected: bool,
    theme: &'a Theme,
}

impl<'a> CardButton<'a> {
    pub fn new(icon: &'a str, title: &'a str, subtitle: &'a str, theme: &'a Theme) -> Self {
        Self {
            icon,
            title,
            subtitle,
            tag: None,
            selected: false,
            theme,
        }
    }

    pub fn tag(mut self, tag: &'a str) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Widget for CardButton<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 4 {
            return;
        }

        let w = area.width as usize;

        // Pick border characters and color based on selection state
        let (tl, tr, bl, br, hz, vt, border_color) = if self.selected {
            ('╔', '╗', '╚', '╝', '═', '║', self.theme.accent)
        } else {
            ('┌', '┐', '└', '┘', '─', '│', self.theme.border_dim)
        };

        let border_style = Style::default().fg(border_color);

        // Top border: ╔═══════════╗
        let inner_w = w.saturating_sub(2);
        let top: String = format!("{}{}{}", tl, hz.to_string().repeat(inner_w), tr);
        buf.set_string(area.x, area.y, &top, border_style);

        // Bottom border: ╚═══════════╝
        let bot: String = format!("{}{}{}", bl, hz.to_string().repeat(inner_w), br);
        buf.set_string(area.x, area.y + 3, &bot, border_style);

        // Middle rows (lines 1 and 2)
        for row in 1..=2u16 {
            let y = area.y + row;
            // Left border
            buf.set_string(area.x, y, &vt.to_string(), border_style);
            // Clear inner area
            let blank = " ".repeat(inner_w);
            buf.set_string(area.x + 1, y, &blank, Style::default());
            // Right border
            buf.set_string(area.x + area.width - 1, y, &vt.to_string(), border_style);
        }

        // Row 1: icon + title (+ optional tag right-aligned)
        let icon_style = Style::default().fg(self.theme.accent);
        let title_style = Style::default()
            .fg(self.theme.text_bright)
            .add_modifier(Modifier::BOLD);

        let content_x = area.x + 2;
        let y1 = area.y + 1;

        buf.set_string(content_x, y1, self.icon, icon_style);
        // icon is typically 1-2 chars wide; leave a space after
        let icon_display_width = unicode_display_width(self.icon);
        let title_x = content_x + icon_display_width as u16 + 1;
        buf.set_string(title_x, y1, self.title, title_style);

        // Optional right-aligned tag
        if let Some(tag) = self.tag {
            let tag_len = tag.len();
            let tag_x = (area.x + area.width).saturating_sub(2 + tag_len as u16);
            let tag_style = Style::default().fg(self.theme.text_dim);
            buf.set_string(tag_x, y1, tag, tag_style);
        }

        // Row 2: subtitle (indented under title)
        let y2 = area.y + 2;
        let subtitle_style = Style::default().fg(self.theme.text_dim);
        buf.set_string(title_x, y2, self.subtitle, subtitle_style);
    }
}

/// Simple ASCII width estimation (treats all chars as width 1).
fn unicode_display_width(s: &str) -> usize {
    s.chars().count()
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. RoundedModal
// ─────────────────────────────────────────────────────────────────────────────

/// Renders a centered rounded-border modal and returns the inner content Rect.
/// Clears the background behind the modal, draws rounded borders with a title
/// in the top border and a footer hint in the bottom border.
pub fn render_rounded_modal(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    footer: &str,
    theme: &Theme,
) -> Rect {
    let buf = frame.buffer_mut();

    // Clear the interior with a solid background (inside the border only)
    let bg_color = ratatui::style::Color::Indexed(235); // dark gray
    let clear_style = Style::default().fg(theme.text_primary).bg(bg_color);
    // Top and bottom border rows: only fill between the corner characters (x+1 .. x+w-1)
    for x in (area.x + 1)..(area.x + area.width.saturating_sub(1)) {
        if let Some(cell) = buf.cell_mut((x, area.y)) {
            cell.set_char(' ');
            cell.set_style(clear_style);
        }
        let bot = area.y + area.height.saturating_sub(1);
        if let Some(cell) = buf.cell_mut((x, bot)) {
            cell.set_char(' ');
            cell.set_style(clear_style);
        }
    }
    // Interior rows: fill full width including border columns (border chars get overwritten below)
    for y in (area.y + 1)..(area.y + area.height.saturating_sub(1)) {
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_style(clear_style);
            }
        }
    }

    let w = area.width as usize;
    let border_style = Style::default().fg(theme.accent).bg(bg_color);
    let title_style = Style::default()
        .fg(theme.text_bright)
        .bg(bg_color)
        .add_modifier(Modifier::BOLD);
    let footer_style = Style::default().fg(theme.text_dim).bg(bg_color);

    // Top border: ╭─ TITLE ──────────╮
    let inner_w = w.saturating_sub(2);
    let title_segment = if title.is_empty() {
        String::new()
    } else {
        format!(" {} ", title)
    };
    let title_seg_len = title_segment.chars().count();
    let remaining = inner_w.saturating_sub(title_seg_len);
    let top_left_dashes = 1.min(remaining);
    let top_right_dashes = remaining.saturating_sub(top_left_dashes);

    // Draw top-left corner
    buf.set_string(area.x, area.y, "╭", border_style);
    // Dashes before title
    let x = area.x + 1;
    buf.set_string(x, area.y, &"─".repeat(top_left_dashes), border_style);
    // Title text
    let x = x + top_left_dashes as u16;
    buf.set_string(x, area.y, &title_segment, title_style);
    // Dashes after title
    let x = x + title_seg_len as u16;
    buf.set_string(x, area.y, &"─".repeat(top_right_dashes), border_style);
    // Top-right corner
    buf.set_string(area.x + area.width - 1, area.y, "╮", border_style);

    // Side borders
    for row in 1..area.height.saturating_sub(1) {
        let y = area.y + row;
        buf.set_string(area.x, y, "│", border_style);
        buf.set_string(area.x + area.width - 1, y, "│", border_style);
    }

    // Bottom border: ╰─ footer ──────────╯
    let bot_y = area.y + area.height - 1;
    let footer_segment = if footer.is_empty() {
        String::new()
    } else {
        format!(" {} ", footer)
    };
    let footer_seg_len = footer_segment.chars().count();
    let remaining = inner_w.saturating_sub(footer_seg_len);
    let bot_left_dashes = 1.min(remaining);
    let bot_right_dashes = remaining.saturating_sub(bot_left_dashes);

    buf.set_string(area.x, bot_y, "╰", border_style);
    let x = area.x + 1;
    buf.set_string(x, bot_y, &"─".repeat(bot_left_dashes), border_style);
    let x = x + bot_left_dashes as u16;
    buf.set_string(x, bot_y, &footer_segment, footer_style);
    let x = x + footer_seg_len as u16;
    buf.set_string(x, bot_y, &"─".repeat(bot_right_dashes), border_style);
    buf.set_string(area.x + area.width - 1, bot_y, "╯", border_style);

    // Return inner rect (inset by 1 on each side, plus 1 line top/bottom for border)
    Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. SectionHeader
// ─────────────────────────────────────────────────────────────────────────────

/// Renders a dim section header like: `── LABEL ──────────────`
pub fn render_section_header(
    buf: &mut Buffer,
    area: Rect,
    y: u16,
    label: &str,
    theme: &Theme,
) {
    if y < area.y || y >= area.y + area.height {
        return;
    }

    // Detect if the cell already has a bg set (e.g. from a modal), and preserve it
    let existing_bg = buf.cell((area.x, y))
        .map(|c| c.bg)
        .unwrap_or(ratatui::style::Color::Reset);
    let style = Style::default().fg(theme.text_dim).bg(existing_bg);
    let prefix = "── ";
    let label_part = format!("{} ", label);
    let used = prefix.chars().count() + label_part.chars().count();
    let trail_len = (area.width as usize).saturating_sub(used);
    let trail = "─".repeat(trail_len);

    let full = format!("{}{}{}", prefix, label_part, trail);
    buf.set_string(area.x, y, &full, style);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. PlayerBar
// ─────────────────────────────────────────────────────────────────────────────

pub struct PlayerBar<'a> {
    icon: char,
    name: &'a str,
    rating: u32,
    captured: &'a [(Piece, ChessColor)],
    timer: &'a str,
    theme: &'a Theme,
}

impl<'a> PlayerBar<'a> {
    pub fn new(
        name: &'a str,
        rating: u32,
        captured: &'a [(Piece, ChessColor)],
        timer: &'a str,
        theme: &'a Theme,
    ) -> Self {
        Self {
            icon: '♚',
            name,
            rating,
            captured,
            timer,
            theme,
        }
    }

    pub fn icon(mut self, icon: char) -> Self {
        self.icon = icon;
        self
    }
}

/// Map a (Piece, ChessColor) to its unicode chess symbol.
fn chess_piece_symbol(piece: Piece, color: ChessColor) -> char {
    match (color, piece) {
        (ChessColor::White, Piece::King) => '♔',
        (ChessColor::White, Piece::Queen) => '♕',
        (ChessColor::White, Piece::Rook) => '♖',
        (ChessColor::White, Piece::Bishop) => '♗',
        (ChessColor::White, Piece::Knight) => '♘',
        (ChessColor::White, Piece::Pawn) => '♙',
        (ChessColor::Black, Piece::King) => '♚',
        (ChessColor::Black, Piece::Queen) => '♛',
        (ChessColor::Black, Piece::Rook) => '♜',
        (ChessColor::Black, Piece::Bishop) => '♝',
        (ChessColor::Black, Piece::Knight) => '♞',
        (ChessColor::Black, Piece::Pawn) => '♟',
    }
}

impl Widget for PlayerBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 1 {
            return;
        }

        let icon_style = Style::default().fg(self.theme.accent);
        let name_style = Style::default()
            .fg(self.theme.text_bright)
            .add_modifier(Modifier::BOLD);
        let rating_style = Style::default().fg(self.theme.text_dim);
        let capture_style = Style::default().fg(self.theme.text_primary);
        let timer_style = Style::default()
            .fg(self.theme.text_bright)
            .add_modifier(Modifier::BOLD);

        let y = area.y;
        let mut x = area.x;

        // Icon
        let icon_str = self.icon.to_string();
        buf.set_string(x, y, &icon_str, icon_style);
        x += unicode_display_width(&icon_str) as u16 + 1;

        // Name
        buf.set_string(x, y, self.name, name_style);
        x += self.name.len() as u16;

        // Rating in parens
        let rating_text = format!(" ({})", self.rating);
        buf.set_string(x, y, &rating_text, rating_style);
        x += rating_text.len() as u16 + 1;

        // Captured pieces
        for &(piece, color) in self.captured {
            let sym = chess_piece_symbol(piece, color);
            buf.set_string(x, y, &sym.to_string(), capture_style);
            x += 1;
        }

        // Timer on far right
        let timer_x = (area.x + area.width).saturating_sub(self.timer.len() as u16);
        buf.set_string(timer_x, y, self.timer, timer_style);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. TabBar
// ─────────────────────────────────────────────────────────────────────────────

pub struct TabBar<'a> {
    tabs: &'a [&'a str],
    active: usize,
    theme: &'a Theme,
}

impl<'a> TabBar<'a> {
    pub fn new(tabs: &'a [&'a str], active: usize, theme: &'a Theme) -> Self {
        Self {
            tabs,
            active,
            theme,
        }
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        let y_label = area.y;
        let y_underline = area.y + 1;
        let mut x = area.x;

        let active_style = Style::default()
            .fg(self.theme.text_bright)
            .add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(self.theme.text_dim);
        let underline_active = Style::default().fg(self.theme.accent);
        let underline_dim = Style::default().fg(self.theme.border_dim);

        for (i, &tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active;
            let style = if is_active { active_style } else { inactive_style };

            // Padding before tab
            if i > 0 {
                buf.set_string(x, y_label, "  ", inactive_style);
                if y_underline < area.y + area.height {
                    buf.set_string(x, y_underline, "  ", underline_dim);
                }
                x += 2;
            }

            // Tab label
            buf.set_string(x, y_label, tab, style);

            // Underline for this tab
            if y_underline < area.y + area.height {
                let tab_width = tab.chars().count();
                let ul_style = if is_active {
                    underline_active
                } else {
                    underline_dim
                };
                let ul_char = if is_active { "━" } else { "─" };
                let underline = ul_char.repeat(tab_width);
                buf.set_string(x, y_underline, &underline, ul_style);
            }

            x += tab.chars().count() as u16;

            if x >= area.x + area.width {
                break;
            }
        }

        // Fill remaining underline width with dim dashes
        if y_underline < area.y + area.height {
            let remaining = (area.x + area.width).saturating_sub(x) as usize;
            if remaining > 0 {
                buf.set_string(x, y_underline, &"─".repeat(remaining), underline_dim);
            }
        }
    }
}
