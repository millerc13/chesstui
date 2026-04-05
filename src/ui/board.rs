use cozy_chess::{Color as ChessColor, Move, Piece, Square};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;
use super::pieces;

fn put(buf: &mut Buffer, area: Rect, x: u16, y: u16, c: char, style: Style) {
    if x >= area.x && y >= area.y && x < area.x + area.width && y < area.y + area.height {
        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
            cell.set_char(c);
            cell.set_style(style);
        }
    }
}

/// Query the actual character cell aspect ratio (height / width in pixels).
/// Returns something like 2.0–2.5 depending on terminal font.
/// Falls back to 2.0 if pixel dimensions are unavailable.
pub fn cell_aspect_ratio() -> f32 {
    if let Ok(ws) = crossterm::terminal::window_size() {
        if ws.columns > 0 && ws.rows > 0 && ws.width > 0 && ws.height > 0 {
            let cell_w = ws.width as f32 / ws.columns as f32;
            let cell_h = ws.height as f32 / ws.rows as f32;
            if cell_w > 0.0 {
                return cell_h / cell_w;
            }
        }
    }
    2.0 // safe default
}

pub struct ChessBoardWidget<'a> {
    board: &'a cozy_chess::Board,
    theme: &'a Theme,
    flipped: bool,
    cursor: Option<(u8, u8)>,
    selected: Option<Square>,
    legal_moves: &'a [Move],
    last_move: Option<(Square, Square)>,
    in_check: bool,
    king_square: Option<Square>,
    show_move_hints: bool,
}

impl<'a> ChessBoardWidget<'a> {
    pub fn new(board: &'a cozy_chess::Board, theme: &'a Theme) -> Self {
        let in_check = !board.checkers().is_empty();
        let king_square = if in_check {
            let side = board.side_to_move();
            let kings = board.pieces(Piece::King) & board.colors(side);
            let mut sq = None;
            for s in kings { sq = Some(s); }
            sq
        } else {
            None
        };

        Self {
            board, theme, flipped: false, cursor: None, selected: None,
            legal_moves: &[], last_move: None, in_check, king_square,
            show_move_hints: false,
        }
    }

    pub fn flipped(mut self, flipped: bool) -> Self { self.flipped = flipped; self }
    pub fn cursor(mut self, file: u8, rank: u8) -> Self { self.cursor = Some((file, rank)); self }
    pub fn selected(mut self, sq: Option<Square>) -> Self { self.selected = sq; self }
    pub fn legal_moves(mut self, moves: &'a [Move]) -> Self { self.legal_moves = moves; self }
    pub fn last_move(mut self, lm: Option<(Square, Square)>) -> Self { self.last_move = lm; self }
    pub fn show_move_hints(mut self, show: bool) -> Self { self.show_move_hints = show; self }

    fn display_to_chess(&self, dc: u8, dr: u8) -> (u8, u8) {
        if self.flipped { (7 - dc, dr) } else { (dc, 7 - dr) }
    }
}

impl Widget for ChessBoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label_col_w: u16 = 2;
        let label_row_h: u16 = 1;

        let available_w = area.width.saturating_sub(label_col_w);
        let available_h = area.height.saturating_sub(label_row_h);

        // Query the real cell aspect ratio from the terminal.
        // A cell that is 2.3× taller than wide needs cw = round(2.3 * ch) to look square.
        let ratio = cell_aspect_ratio();

        // Find largest ch where cw = round(ratio * ch) fits in available space.
        let max_ch = available_h / 8;
        let mut ch: u16 = max_ch.max(1);
        let mut cw: u16;
        loop {
            cw = (ratio * ch as f32).round() as u16;
            if cw * 8 <= available_w {
                break;
            }
            if ch <= 1 {
                cw = available_w / 8;
                break;
            }
            ch -= 1;
        }
        ch = ch.max(1);
        cw = cw.max(2);

        let board_w = 8 * cw;
        let board_h = 8 * ch;

        // Center board in available area
        let ox = area.x + (area.width.saturating_sub(board_w + label_col_w)) / 2;
        let oy = area.y + (area.height.saturating_sub(board_h + label_row_h)) / 2;
        let bx = ox + label_col_w;
        let by = oy;

        for dr in 0..8u8 {
            for dc in 0..8u8 {
                let (file, rank) = self.display_to_chess(dc, dr);
                let sq = Square::new(
                    cozy_chess::File::index(file as usize),
                    cozy_chess::Rank::index(rank as usize),
                );

                let is_light = (file + rank) % 2 != 0;
                let highlight = self.highlight_bg(file, rank, sq);
                let is_legal = self.selected.is_some()
                    && self.legal_moves.iter().any(|m| m.to == sq);

                let sq_bg = match highlight {
                    Some(bg) => bg,
                    None if is_light => self.theme.light_square,
                    None => self.theme.dark_square,
                };

                let cx = bx + (dc as u16) * cw;
                let cy = by + (dr as u16) * ch;

                // Fill square with background
                let fill = Style::default().bg(sq_bg);
                for row in 0..ch {
                    for col in 0..cw {
                        put(buf, area, cx + col, cy + row, ' ', fill);
                    }
                }

                // Draw piece
                if let Some((piece, color)) = self.get_piece(sq) {
                    let fg = if color == ChessColor::White {
                        self.theme.white_piece
                    } else {
                        self.theme.black_piece
                    };
                    pieces::draw_piece(buf, area, cx, cy, cw, ch, piece, fg, sq_bg);
                    // Show hint on occupied squares that are capture targets
                    if is_legal && self.show_move_hints {
                        let hint = format!("{}", sq);
                        let hint_style = Style::default()
                            .fg(self.theme.accent)
                            .bg(sq_bg)
                            .add_modifier(Modifier::BOLD);
                        let hx = cx + cw.saturating_sub(hint.len() as u16);
                        let hy = cy + ch.saturating_sub(1);
                        for (i, c) in hint.chars().enumerate() {
                            put(buf, area, hx + i as u16, hy, c, hint_style);
                        }
                    }
                } else if is_legal {
                    if self.show_move_hints {
                        // Show square name centered in the cell
                        let hint = format!("{}", sq);
                        let hint_style = Style::default()
                            .fg(self.theme.accent)
                            .bg(sq_bg)
                            .add_modifier(Modifier::BOLD);
                        let hx = cx + (cw.saturating_sub(hint.len() as u16)) / 2;
                        let hy = cy + ch / 2;
                        for (i, c) in hint.chars().enumerate() {
                            put(buf, area, hx + i as u16, hy, c, hint_style);
                        }
                    } else {
                        let dot_style = Style::default().fg(self.theme.accent).bg(sq_bg);
                        put(buf, area, cx + cw / 2, cy + ch / 2, '\u{2022}', dot_style);
                    }
                }
            }
        }

        // Rank labels
        let label_style = Style::default()
            .fg(self.theme.accent_secondary)
            .add_modifier(Modifier::DIM);
        for dr in 0..8u8 {
            let (_, rank) = self.display_to_chess(0, dr);
            let y = by + (dr as u16) * ch + ch / 2;
            put(buf, area, ox, y, (b'1' + rank) as char, label_style);
        }

        // File labels
        let fy = by + board_h;
        for dc in 0..8u8 {
            let (file, _) = self.display_to_chess(dc, 0);
            let x = bx + (dc as u16) * cw + cw / 2;
            put(buf, area, x, fy, (b'a' + file) as char, label_style);
        }
    }
}

impl ChessBoardWidget<'_> {
    fn highlight_bg(&self, file: u8, rank: u8, sq: Square) -> Option<Color> {
        if self.in_check {
            if let Some(king_sq) = self.king_square {
                if sq == king_sq { return Some(self.theme.check_bg); }
            }
        }

        if let Some((cf, cr)) = self.cursor {
            let (chess_f, chess_r) = if self.flipped {
                (7 - cf, 7 - cr)
            } else {
                (cf, cr)
            };
            let cursor_sq = Square::new(
                cozy_chess::File::index(chess_f as usize),
                cozy_chess::Rank::index(chess_r as usize),
            );
            if sq == cursor_sq { return Some(self.theme.cursor_bg); }
        }

        if let Some(sel) = self.selected {
            if sq == sel { return Some(self.theme.selected_bg); }
        }

        if self.selected.is_some() && self.legal_moves.iter().any(|m| m.to == sq) {
            return Some(self.theme.legal_move_bg);
        }

        if let Some((from, to)) = self.last_move {
            if sq == from || sq == to {
                let is_light = (file + rank) % 2 != 0;
                return Some(if is_light {
                    self.theme.last_move_light
                } else {
                    self.theme.last_move_dark
                });
            }
        }

        None
    }

    fn get_piece(&self, sq: Square) -> Option<(Piece, ChessColor)> {
        for piece in [Piece::King, Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn] {
            if self.board.pieces(piece).has(sq) {
                let color = if self.board.colors(ChessColor::White).has(sq) {
                    ChessColor::White
                } else {
                    ChessColor::Black
                };
                return Some((piece, color));
            }
        }
        None
    }
}
