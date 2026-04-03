//! Piece-Square Tables (PSTs)
//!
//! Each table is from White's perspective. For Black, mirror vertically
//! (index with `square ^ 56`).
//!
//! Values are in centipawns. Positive = good for the piece to be on that square.
//! Tables are indexed [0..64] where 0=A1, 1=B1, ..., 63=H8.

/// Piece base values in centipawns.
pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 330;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20000;

/// Bishop pair bonus in centipawns.
pub const BISHOP_PAIR_BONUS: i32 = 30;

/// Rook on open file bonus.
pub const ROOK_OPEN_FILE: i32 = 25;
/// Rook on semi-open file bonus.
pub const ROOK_SEMI_OPEN_FILE: i32 = 15;

/// Passed pawn bonuses by rank (from the pawn's perspective).
/// Index 0 = rank 2 (just advanced one), index 5 = rank 7 (about to promote).
pub const PASSED_PAWN_BONUS: [i32; 6] = [10, 15, 25, 45, 75, 120];

/// Doubled pawn penalty.
pub const DOUBLED_PAWN_PENALTY: i32 = -15;
/// Isolated pawn penalty.
pub const ISOLATED_PAWN_PENALTY: i32 = -20;

// ---------------------------------------------------------------
// Piece-Square Tables
//
// Layout: rank 1 (row 0) at indices 0..8, rank 8 (row 7) at 56..64.
// So for White, index 0 = A1 (back rank queenside).
// ---------------------------------------------------------------

#[rustfmt::skip]
pub const PAWN_PST: [i32; 64] = [
    //  A    B    C    D    E    F    G    H
      0,   0,   0,   0,   0,   0,   0,   0,  // rank 1 (pawns never here)
      5,  10,  10, -20, -20,  10,  10,   5,  // rank 2
      5,  -5, -10,   0,   0, -10,  -5,   5,  // rank 3
      0,   0,   0,  20,  20,   0,   0,   0,  // rank 4
      5,   5,  10,  25,  25,  10,   5,   5,  // rank 5
     10,  10,  20,  30,  30,  20,  10,  10,  // rank 6
     50,  50,  50,  50,  50,  50,  50,  50,  // rank 7
      0,   0,   0,   0,   0,   0,   0,   0,  // rank 8 (pawns promote)
];

#[rustfmt::skip]
pub const KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
pub const BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
pub const ROOK_PST: [i32; 64] = [
      0,   0,   0,   5,   5,   0,   0,   0,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      5,  10,  10,  10,  10,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub const QUEEN_PST: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -10,   5,   5,   5,   5,   5,   0, -10,
      0,   0,   5,   5,   5,   5,   0,  -5,
     -5,   0,   5,   5,   5,   5,   0,  -5,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

/// King PST for the middlegame -- wants to stay castled and safe.
#[rustfmt::skip]
pub const KING_MIDDLEGAME_PST: [i32; 64] = [
     20,  30,  10,   0,   0,  10,  30,  20,
     20,  20,   0,   0,   0,   0,  20,  20,
    -10, -20, -20, -20, -20, -20, -20, -10,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
];

/// King PST for the endgame -- king becomes active, wants center.
#[rustfmt::skip]
pub const KING_ENDGAME_PST: [i32; 64] = [
    -50, -30, -30, -30, -30, -30, -30, -50,
    -30, -30,   0,   0,   0,   0, -30, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -50, -40, -30, -20, -20, -30, -40, -50,
];

/// Returns the PST index for a square, mirrored for Black.
/// cozy-chess squares: A1=0, B1=1, ..., H8=63
/// Our PST layout matches this directly for White.
/// For Black, we mirror vertically: flip the rank.
#[inline]
pub fn pst_index(square_index: usize, is_white: bool) -> usize {
    if is_white {
        square_index
    } else {
        // Mirror vertically: rank 0 <-> rank 7, rank 1 <-> rank 6, etc.
        let rank = square_index / 8;
        let file = square_index % 8;
        (7 - rank) * 8 + file
    }
}
