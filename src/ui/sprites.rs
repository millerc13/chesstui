use cozy_chess::{Color as ChessColor, Piece};
use crate::config::PieceStyle;

// ── Pixel Color (16x16, full color) ─────────────────────────────────────

mod pixel_color {
    pub const WHITE_KING:   &[u8] = include_bytes!("../../assets/pieces/color/white_king.png");
    pub const BLACK_KING:   &[u8] = include_bytes!("../../assets/pieces/color/black_king.png");
    pub const WHITE_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/color/white_queen.png");
    pub const BLACK_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/color/black_queen.png");
    pub const WHITE_ROOK:   &[u8] = include_bytes!("../../assets/pieces/color/white_rook.png");
    pub const BLACK_ROOK:   &[u8] = include_bytes!("../../assets/pieces/color/black_rook.png");
    pub const WHITE_BISHOP: &[u8] = include_bytes!("../../assets/pieces/color/white_bishop.png");
    pub const BLACK_BISHOP: &[u8] = include_bytes!("../../assets/pieces/color/black_bishop.png");
    pub const WHITE_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/color/white_knight.png");
    pub const BLACK_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/color/black_knight.png");
    pub const WHITE_PAWN:   &[u8] = include_bytes!("../../assets/pieces/color/white_pawn.png");
    pub const BLACK_PAWN:   &[u8] = include_bytes!("../../assets/pieces/color/black_pawn.png");
}

// ── Pixel Duo (16x16, two-tone) ─────────────────────────────────────────

mod pixel_duo {
    pub const WHITE_KING:   &[u8] = include_bytes!("../../assets/pieces/duo/white_king.png");
    pub const BLACK_KING:   &[u8] = include_bytes!("../../assets/pieces/duo/black_king.png");
    pub const WHITE_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/duo/white_queen.png");
    pub const BLACK_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/duo/black_queen.png");
    pub const WHITE_ROOK:   &[u8] = include_bytes!("../../assets/pieces/duo/white_rook.png");
    pub const BLACK_ROOK:   &[u8] = include_bytes!("../../assets/pieces/duo/black_rook.png");
    pub const WHITE_BISHOP: &[u8] = include_bytes!("../../assets/pieces/duo/white_bishop.png");
    pub const BLACK_BISHOP: &[u8] = include_bytes!("../../assets/pieces/duo/black_bishop.png");
    pub const WHITE_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/duo/white_knight.png");
    pub const BLACK_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/duo/black_knight.png");
    pub const WHITE_PAWN:   &[u8] = include_bytes!("../../assets/pieces/duo/white_pawn.png");
    pub const BLACK_PAWN:   &[u8] = include_bytes!("../../assets/pieces/duo/black_pawn.png");
}

// ── Pixel Mono (16x16, monochrome) ──────────────────────────────────────

mod pixel_mono {
    pub const WHITE_KING:   &[u8] = include_bytes!("../../assets/pieces/mono/white_king.png");
    pub const BLACK_KING:   &[u8] = include_bytes!("../../assets/pieces/mono/black_king.png");
    pub const WHITE_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/mono/white_queen.png");
    pub const BLACK_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/mono/black_queen.png");
    pub const WHITE_ROOK:   &[u8] = include_bytes!("../../assets/pieces/mono/white_rook.png");
    pub const BLACK_ROOK:   &[u8] = include_bytes!("../../assets/pieces/mono/black_rook.png");
    pub const WHITE_BISHOP: &[u8] = include_bytes!("../../assets/pieces/mono/white_bishop.png");
    pub const BLACK_BISHOP: &[u8] = include_bytes!("../../assets/pieces/mono/black_bishop.png");
    pub const WHITE_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/mono/white_knight.png");
    pub const BLACK_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/mono/black_knight.png");
    pub const WHITE_PAWN:   &[u8] = include_bytes!("../../assets/pieces/mono/white_pawn.png");
    pub const BLACK_PAWN:   &[u8] = include_bytes!("../../assets/pieces/mono/black_pawn.png");
}

// ── Classic (512x512, high-res) ─────────────────────────────────────────

mod classic {
    pub const WHITE_KING:   &[u8] = include_bytes!("../../assets/pieces/classic/chess_king_white.png");
    pub const BLACK_KING:   &[u8] = include_bytes!("../../assets/pieces/classic/chess_king_black.png");
    pub const WHITE_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/classic/chess_queen_white.png");
    pub const BLACK_QUEEN:  &[u8] = include_bytes!("../../assets/pieces/classic/chess_queen_black.png");
    pub const WHITE_ROOK:   &[u8] = include_bytes!("../../assets/pieces/classic/chess_rook_white.png");
    pub const BLACK_ROOK:   &[u8] = include_bytes!("../../assets/pieces/classic/chess_rook_black.png");
    pub const WHITE_BISHOP: &[u8] = include_bytes!("../../assets/pieces/classic/chess_bishop_white.png");
    pub const BLACK_BISHOP: &[u8] = include_bytes!("../../assets/pieces/classic/chess_bishop_black.png");
    pub const WHITE_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/classic/chess_knight_white.png");
    pub const BLACK_KNIGHT: &[u8] = include_bytes!("../../assets/pieces/classic/chess_knight_black.png");
    pub const WHITE_PAWN:   &[u8] = include_bytes!("../../assets/pieces/classic/chess_pawn_white.png");
    pub const BLACK_PAWN:   &[u8] = include_bytes!("../../assets/pieces/classic/chess_pawn_black.png");
}

/// Returns raw PNG bytes for a given style/piece/color combo.
/// Returns `None` for `Generated3D` (those are rendered programmatically).
pub fn get_sprite_data(style: PieceStyle, piece: Piece, color: ChessColor) -> Option<&'static [u8]> {
    let lookup = match style {
        PieceStyle::Generated3D => return None,
        PieceStyle::PixelColor => lookup_pixel_color,
        PieceStyle::PixelDuo => lookup_pixel_duo,
        PieceStyle::PixelMono => lookup_pixel_mono,
        PieceStyle::Classic => lookup_classic,
    };
    Some(lookup(piece, color))
}

fn lookup_pixel_color(piece: Piece, color: ChessColor) -> &'static [u8] {
    match (piece, color) {
        (Piece::King,   ChessColor::White) => pixel_color::WHITE_KING,
        (Piece::King,   ChessColor::Black) => pixel_color::BLACK_KING,
        (Piece::Queen,  ChessColor::White) => pixel_color::WHITE_QUEEN,
        (Piece::Queen,  ChessColor::Black) => pixel_color::BLACK_QUEEN,
        (Piece::Rook,   ChessColor::White) => pixel_color::WHITE_ROOK,
        (Piece::Rook,   ChessColor::Black) => pixel_color::BLACK_ROOK,
        (Piece::Bishop, ChessColor::White) => pixel_color::WHITE_BISHOP,
        (Piece::Bishop, ChessColor::Black) => pixel_color::BLACK_BISHOP,
        (Piece::Knight, ChessColor::White) => pixel_color::WHITE_KNIGHT,
        (Piece::Knight, ChessColor::Black) => pixel_color::BLACK_KNIGHT,
        (Piece::Pawn,   ChessColor::White) => pixel_color::WHITE_PAWN,
        (Piece::Pawn,   ChessColor::Black) => pixel_color::BLACK_PAWN,
    }
}

fn lookup_pixel_duo(piece: Piece, color: ChessColor) -> &'static [u8] {
    match (piece, color) {
        (Piece::King,   ChessColor::White) => pixel_duo::WHITE_KING,
        (Piece::King,   ChessColor::Black) => pixel_duo::BLACK_KING,
        (Piece::Queen,  ChessColor::White) => pixel_duo::WHITE_QUEEN,
        (Piece::Queen,  ChessColor::Black) => pixel_duo::BLACK_QUEEN,
        (Piece::Rook,   ChessColor::White) => pixel_duo::WHITE_ROOK,
        (Piece::Rook,   ChessColor::Black) => pixel_duo::BLACK_ROOK,
        (Piece::Bishop, ChessColor::White) => pixel_duo::WHITE_BISHOP,
        (Piece::Bishop, ChessColor::Black) => pixel_duo::BLACK_BISHOP,
        (Piece::Knight, ChessColor::White) => pixel_duo::WHITE_KNIGHT,
        (Piece::Knight, ChessColor::Black) => pixel_duo::BLACK_KNIGHT,
        (Piece::Pawn,   ChessColor::White) => pixel_duo::WHITE_PAWN,
        (Piece::Pawn,   ChessColor::Black) => pixel_duo::BLACK_PAWN,
    }
}

fn lookup_pixel_mono(piece: Piece, color: ChessColor) -> &'static [u8] {
    match (piece, color) {
        (Piece::King,   ChessColor::White) => pixel_mono::WHITE_KING,
        (Piece::King,   ChessColor::Black) => pixel_mono::BLACK_KING,
        (Piece::Queen,  ChessColor::White) => pixel_mono::WHITE_QUEEN,
        (Piece::Queen,  ChessColor::Black) => pixel_mono::BLACK_QUEEN,
        (Piece::Rook,   ChessColor::White) => pixel_mono::WHITE_ROOK,
        (Piece::Rook,   ChessColor::Black) => pixel_mono::BLACK_ROOK,
        (Piece::Bishop, ChessColor::White) => pixel_mono::WHITE_BISHOP,
        (Piece::Bishop, ChessColor::Black) => pixel_mono::BLACK_BISHOP,
        (Piece::Knight, ChessColor::White) => pixel_mono::WHITE_KNIGHT,
        (Piece::Knight, ChessColor::Black) => pixel_mono::BLACK_KNIGHT,
        (Piece::Pawn,   ChessColor::White) => pixel_mono::WHITE_PAWN,
        (Piece::Pawn,   ChessColor::Black) => pixel_mono::BLACK_PAWN,
    }
}

fn lookup_classic(piece: Piece, color: ChessColor) -> &'static [u8] {
    match (piece, color) {
        (Piece::King,   ChessColor::White) => classic::WHITE_KING,
        (Piece::King,   ChessColor::Black) => classic::BLACK_KING,
        (Piece::Queen,  ChessColor::White) => classic::WHITE_QUEEN,
        (Piece::Queen,  ChessColor::Black) => classic::BLACK_QUEEN,
        (Piece::Rook,   ChessColor::White) => classic::WHITE_ROOK,
        (Piece::Rook,   ChessColor::Black) => classic::BLACK_ROOK,
        (Piece::Bishop, ChessColor::White) => classic::WHITE_BISHOP,
        (Piece::Bishop, ChessColor::Black) => classic::BLACK_BISHOP,
        (Piece::Knight, ChessColor::White) => classic::WHITE_KNIGHT,
        (Piece::Knight, ChessColor::Black) => classic::BLACK_KNIGHT,
        (Piece::Pawn,   ChessColor::White) => classic::WHITE_PAWN,
        (Piece::Pawn,   ChessColor::Black) => classic::BLACK_PAWN,
    }
}
