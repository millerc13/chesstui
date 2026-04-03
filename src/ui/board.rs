use cozy_chess::{Color as ChessColor, Move, Piece, Square};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Unicode chess piece symbols.
fn piece_symbol(piece: Piece, color: ChessColor) -> &'static str {
    match (color, piece) {
        (ChessColor::White, Piece::King)   => "\u{2654}",
        (ChessColor::White, Piece::Queen)  => "\u{2655}",
        (ChessColor::White, Piece::Rook)   => "\u{2656}",
        (ChessColor::White, Piece::Bishop) => "\u{2657}",
        (ChessColor::White, Piece::Knight) => "\u{2658}",
        (ChessColor::White, Piece::Pawn)   => "\u{2659}",
        (ChessColor::Black, Piece::King)   => "\u{265a}",
        (ChessColor::Black, Piece::Queen)  => "\u{265b}",
        (ChessColor::Black, Piece::Rook)   => "\u{265c}",
        (ChessColor::Black, Piece::Bishop) => "\u{265d}",
        (ChessColor::Black, Piece::Knight) => "\u{265e}",
        (ChessColor::Black, Piece::Pawn)   => "\u{265f}",
    }
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
}

impl<'a> ChessBoardWidget<'a> {
    pub fn new(board: &'a cozy_chess::Board, theme: &'a Theme) -> Self {
        // Find if the side to move is in check and where the king is
        let in_check = !board.checkers().is_empty();
        let king_square = if in_check {
            let side = board.side_to_move();
            let kings = board.pieces(Piece::King) & board.colors(side);
            // There should be exactly one king
            let mut sq = None;
            for s in kings {
                sq = Some(s);
            }
            sq
        } else {
            None
        };

        Self {
            board,
            theme,
            flipped: false,
            cursor: None,
            selected: None,
            legal_moves: &[],
            last_move: None,
            in_check,
            king_square,
        }
    }

    pub fn flipped(mut self, flipped: bool) -> Self {
        self.flipped = flipped;
        self
    }

    pub fn cursor(mut self, file: u8, rank: u8) -> Self {
        self.cursor = Some((file, rank));
        self
    }

    pub fn selected(mut self, sq: Option<Square>) -> Self {
        self.selected = sq;
        self
    }

    pub fn legal_moves(mut self, moves: &'a [Move]) -> Self {
        self.legal_moves = moves;
        self
    }

    pub fn last_move(mut self, lm: Option<(Square, Square)>) -> Self {
        self.last_move = lm;
        self
    }

    /// Convert display row/col (0-7) to chess file/rank indices,
    /// accounting for flip.
    fn display_to_chess(&self, display_col: u8, display_row: u8) -> (u8, u8) {
        if self.flipped {
            (7 - display_col, display_row)
        } else {
            (display_col, 7 - display_row)
        }
    }
}

impl Widget for ChessBoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Board needs: 1 col for rank labels + 16 cols (8 squares * 2 chars) + 1 col padding
        // and 8 rows + 1 row for file labels
        let board_x = area.x + 1; // 1 col for rank labels
        let board_y = area.y;

        // Render 8x8 grid
        for display_row in 0..8u8 {
            for display_col in 0..8u8 {
                let (file, rank) = self.display_to_chess(display_col, display_row);
                let sq = Square::new(
                    cozy_chess::File::index(file as usize),
                    cozy_chess::Rank::index(rank as usize),
                );

                // Determine background color with highlight priority
                let bg = self.determine_bg(file, rank, sq, display_col, display_row);

                // Determine piece on this square
                let (symbol, fg) = self.piece_at(sq);

                let style = Style::default().fg(fg).bg(bg);
                let x = board_x + (display_col as u16) * 2;
                let y = board_y + display_row as u16;

                if x + 1 < area.x + area.width && y < area.y + area.height {
                    buf.set_string(x, y, symbol, style);
                }
            }
        }

        // Rank labels
        for display_row in 0..8u8 {
            let (_, rank) = self.display_to_chess(0, display_row);
            let label = format!("{}", rank + 1);
            let y = board_y + display_row as u16;
            if y < area.y + area.height {
                buf.set_string(
                    area.x,
                    y,
                    &label,
                    Style::default().fg(self.theme.text_dim),
                );
            }
        }

        // File labels
        let file_y = board_y + 8;
        if file_y < area.y + area.height {
            for display_col in 0..8u8 {
                let (file, _) = self.display_to_chess(display_col, 0);
                let label = (b'a' + file) as char;
                let x = board_x + (display_col as u16) * 2;
                if x < area.x + area.width {
                    buf.set_string(
                        x,
                        file_y,
                        &format!("{} ", label),
                        Style::default().fg(self.theme.text_dim),
                    );
                }
            }
        }
    }
}

impl ChessBoardWidget<'_> {
    fn determine_bg(&self, file: u8, rank: u8, sq: Square, _display_col: u8, _display_row: u8) -> Color {
        let is_light = (file + rank) % 2 != 0;

        // Check highlight (highest priority)
        if self.in_check {
            if let Some(king_sq) = self.king_square {
                if sq == king_sq {
                    return self.theme.check_bg;
                }
            }
        }

        // Cursor — cursor coords are display-space (file 0-7, rank 0-7 where 0=bottom).
        // Convert to chess square the same way App::cursor_square() does.
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
            if sq == cursor_sq {
                return self.theme.cursor_bg;
            }
        }

        // Selected square
        if let Some(sel) = self.selected {
            if sq == sel {
                return self.theme.selected_bg;
            }
        }

        // Legal move destinations
        if self.legal_moves.iter().any(|m| m.to == sq) {
            return self.theme.legal_move_bg;
        }

        // Last move
        if let Some((from, to)) = self.last_move {
            if sq == from || sq == to {
                return if is_light {
                    self.theme.last_move_light
                } else {
                    self.theme.last_move_dark
                };
            }
        }

        // Base square color
        self.theme.square_bg(file, rank)
    }

    fn piece_at(&self, sq: Square) -> (&str, Color) {
        for piece in [Piece::King, Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn] {
            if self.board.pieces(piece).has(sq) {
                if self.board.colors(ChessColor::White).has(sq) {
                    return (piece_symbol(piece, ChessColor::White), self.theme.white_piece);
                } else {
                    return (piece_symbol(piece, ChessColor::Black), self.theme.black_piece);
                }
            }
        }

        // Empty square — show dot for legal moves
        if self.legal_moves.iter().any(|m| m.to == sq) {
            return ("\u{00b7} ", self.theme.text_dim);
        }

        ("  ", Color::Reset)
    }
}
