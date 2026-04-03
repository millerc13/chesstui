//! Static position evaluation.
//!
//! Evaluates a position from the perspective of the side to move.
//! Returns a score in centipawns: positive = good for side to move.

use cozy_chess::{Board, Color, File, Piece, Square, BitBoard};

use super::difficulty::EvalConfig;
use super::tables::*;

/// Full evaluation entry point. Dispatches to sub-evaluators based on
/// which components are active in the difficulty config.
pub fn evaluate(board: &Board, config: &EvalConfig) -> i32 {
    let white_score = evaluate_for_color(board, Color::White, config);
    let black_score = evaluate_for_color(board, Color::Black, config);
    let raw = white_score - black_score;

    // Return from the perspective of the side to move.
    match board.side_to_move() {
        Color::White => raw,
        Color::Black => -raw,
    }
}

/// Evaluate all components for a single color.
fn evaluate_for_color(board: &Board, color: Color, config: &EvalConfig) -> i32 {
    let mut score = 0;

    if config.material {
        score += eval_material(board, color);
    }
    if config.piece_square_tables {
        score += eval_pst(board, color);
    }
    if config.pawn_structure {
        score += eval_pawn_structure(board, color);
    }
    if config.king_safety {
        score += eval_king_safety(board, color);
    }
    if config.mobility {
        score += eval_mobility(board, color);
    }
    if config.center_control {
        score += eval_center_control(board, color);
    }
    if config.bishop_pair {
        score += eval_bishop_pair(board, color);
    }
    if config.rook_on_open_file {
        score += eval_rook_files(board, color);
    }
    if config.passed_pawns {
        score += eval_passed_pawns(board, color);
    }

    score
}

// ---------------------------------------------------------------
// Individual evaluation components
// ---------------------------------------------------------------

/// Material: count pieces and multiply by their base values.
fn eval_material(board: &Board, color: Color) -> i32 {
    let pieces = |piece: Piece| -> i32 {
        (board.pieces(piece) & board.colors(color)).len() as i32
    };

    pieces(Piece::Pawn) * PAWN_VALUE
        + pieces(Piece::Knight) * KNIGHT_VALUE
        + pieces(Piece::Bishop) * BISHOP_VALUE
        + pieces(Piece::Rook) * ROOK_VALUE
        + pieces(Piece::Queen) * QUEEN_VALUE
}

/// Piece-square table bonuses.
fn eval_pst(board: &Board, color: Color) -> i32 {
    let is_white = color == Color::White;
    let is_endgame = is_endgame_phase(board);
    let mut score = 0;

    let color_bb = board.colors(color);

    for sq in color_bb & board.pieces(Piece::Pawn) {
        score += PAWN_PST[pst_index(sq as usize, is_white)];
    }
    for sq in color_bb & board.pieces(Piece::Knight) {
        score += KNIGHT_PST[pst_index(sq as usize, is_white)];
    }
    for sq in color_bb & board.pieces(Piece::Bishop) {
        score += BISHOP_PST[pst_index(sq as usize, is_white)];
    }
    for sq in color_bb & board.pieces(Piece::Rook) {
        score += ROOK_PST[pst_index(sq as usize, is_white)];
    }
    for sq in color_bb & board.pieces(Piece::Queen) {
        score += QUEEN_PST[pst_index(sq as usize, is_white)];
    }
    for sq in color_bb & board.pieces(Piece::King) {
        let idx = pst_index(sq as usize, is_white);
        if is_endgame {
            score += KING_ENDGAME_PST[idx];
        } else {
            score += KING_MIDDLEGAME_PST[idx];
        }
    }

    score
}

/// Pawn structure: doubled pawns, isolated pawns.
fn eval_pawn_structure(board: &Board, color: Color) -> i32 {
    let pawns = board.pieces(Piece::Pawn) & board.colors(color);
    let mut score = 0;

    for file_idx in 0..8u8 {
        let file = File::index(file_idx as usize);
        let file_bb = BitBoard::from(file);
        let pawns_on_file = (pawns & file_bb).len() as i32;

        // Doubled pawns: penalty for each extra pawn on the same file.
        if pawns_on_file > 1 {
            score += DOUBLED_PAWN_PENALTY * (pawns_on_file - 1);
        }

        // Isolated pawns: no friendly pawns on adjacent files.
        if pawns_on_file > 0 {
            let has_neighbor = {
                let left = if file_idx > 0 {
                    let f = File::index((file_idx - 1) as usize);
                    (pawns & BitBoard::from(f)).len() > 0
                } else {
                    false
                };
                let right = if file_idx < 7 {
                    let f = File::index((file_idx + 1) as usize);
                    (pawns & BitBoard::from(f)).len() > 0
                } else {
                    false
                };
                left || right
            };
            if !has_neighbor {
                score += ISOLATED_PAWN_PENALTY * pawns_on_file;
            }
        }
    }

    score
}

/// King safety: penalize open files near king, reward pawn shield.
fn eval_king_safety(board: &Board, color: Color) -> i32 {
    let king_bb = board.pieces(Piece::King) & board.colors(color);
    let king_sq = king_bb.into_iter().next();
    let king_sq = match king_sq {
        Some(sq) => sq,
        None => return 0,
    };

    // In the endgame, king safety is less important.
    if is_endgame_phase(board) {
        return 0;
    }

    let king_file = king_sq.file() as u8;
    let our_pawns = board.pieces(Piece::Pawn) & board.colors(color);
    let mut score = 0;

    // Check pawn shield on the 2-3 files nearest the king.
    let start_file = king_file.saturating_sub(1);
    let end_file = (king_file + 1).min(7);

    for file_idx in start_file..=end_file {
        let file = File::index(file_idx as usize);
        let file_bb = BitBoard::from(file);
        if (our_pawns & file_bb).len() == 0 {
            // No pawn shield on this file near the king.
            score -= 15;
        }
    }

    score
}

/// Mobility: count legal moves as a rough proxy.
/// We approximate by counting attacked squares for each piece type.
fn eval_mobility(board: &Board, color: Color) -> i32 {
    // Count moves for the side. This is expensive, so we use a simplified
    // approach: count the number of squares attacked by each piece type.
    // A proper implementation would use the move generator, but for
    // performance we approximate.
    let our_pieces = board.colors(color);
    let occupied = board.occupied();

    let mut mobility = 0i32;

    // Knight mobility: each legal destination = +2 cp
    for sq in our_pieces & board.pieces(Piece::Knight) {
        let attacks = cozy_chess::get_knight_moves(sq) & !our_pieces;
        mobility += attacks.len() as i32 * 2;
    }

    // Bishop mobility: each legal destination = +3 cp (bishops love open diagonals)
    for sq in our_pieces & board.pieces(Piece::Bishop) {
        let attacks = cozy_chess::get_bishop_moves(sq, occupied) & !our_pieces;
        mobility += attacks.len() as i32 * 3;
    }

    // Rook mobility: each legal destination = +2 cp
    for sq in our_pieces & board.pieces(Piece::Rook) {
        let attacks = cozy_chess::get_rook_moves(sq, occupied) & !our_pieces;
        mobility += attacks.len() as i32 * 2;
    }

    // Queen mobility: each legal destination = +1 cp (queen mobility is less
    // valuable per square because the queen is already powerful)
    for sq in our_pieces & board.pieces(Piece::Queen) {
        let attacks = (cozy_chess::get_bishop_moves(sq, occupied)
            | cozy_chess::get_rook_moves(sq, occupied))
            & !our_pieces;
        mobility += attacks.len() as i32;
    }

    mobility
}

/// Center control: bonus for controlling/occupying e4, d4, e5, d5.
fn eval_center_control(board: &Board, color: Color) -> i32 {
    let center = BitBoard::from(Square::D4)
        | BitBoard::from(Square::E4)
        | BitBoard::from(Square::D5)
        | BitBoard::from(Square::E5);

    let our_pieces = board.colors(color);
    let occupied_center = (our_pieces & center).len() as i32;

    // Bonus for occupying center squares.
    occupied_center * 10
}

/// Bishop pair bonus: having two bishops is worth extra.
fn eval_bishop_pair(board: &Board, color: Color) -> i32 {
    let bishops = board.pieces(Piece::Bishop) & board.colors(color);
    if bishops.len() >= 2 {
        BISHOP_PAIR_BONUS
    } else {
        0
    }
}

/// Rook on open/semi-open files.
fn eval_rook_files(board: &Board, color: Color) -> i32 {
    let rooks = board.pieces(Piece::Rook) & board.colors(color);
    let our_pawns = board.pieces(Piece::Pawn) & board.colors(color);
    let their_pawns = board.pieces(Piece::Pawn) & board.colors(!color);
    let mut score = 0;

    for sq in rooks {
        let file = BitBoard::from(sq.file());
        let our_pawns_on_file = (our_pawns & file).len();
        let their_pawns_on_file = (their_pawns & file).len();

        if our_pawns_on_file == 0 && their_pawns_on_file == 0 {
            score += ROOK_OPEN_FILE;
        } else if our_pawns_on_file == 0 {
            score += ROOK_SEMI_OPEN_FILE;
        }
    }

    score
}

/// Passed pawn evaluation: bonus for pawns with no opposing pawns ahead.
fn eval_passed_pawns(board: &Board, color: Color) -> i32 {
    let our_pawns = board.pieces(Piece::Pawn) & board.colors(color);
    let their_pawns = board.pieces(Piece::Pawn) & board.colors(!color);
    let mut score = 0;

    for sq in our_pawns {
        let file_idx = sq.file() as u8;
        let rank = sq.rank() as u8;

        // Check if there are enemy pawns on this file or adjacent files
        // that are ahead of this pawn (can block or capture it).
        let mut is_passed = true;

        let check_start = file_idx.saturating_sub(1);
        let check_end = (file_idx + 1).min(7);

        for check_file in check_start..=check_end {
            let file = File::index(check_file as usize);
            let file_bb = BitBoard::from(file);
            for enemy_sq in their_pawns & file_bb {
                let enemy_rank = enemy_sq.rank() as u8;
                // For White, enemy must be on a higher rank to block.
                // For Black, enemy must be on a lower rank.
                let blocking = match color {
                    Color::White => enemy_rank > rank,
                    Color::Black => enemy_rank < rank,
                };
                if blocking {
                    is_passed = false;
                    break;
                }
            }
            if !is_passed {
                break;
            }
        }

        if is_passed {
            // Bonus increases as the pawn advances.
            let advancement = match color {
                Color::White => rank.saturating_sub(1) as usize, // rank 2 -> index 0
                Color::Black => (6u8.saturating_sub(rank)) as usize,
            };
            if advancement < PASSED_PAWN_BONUS.len() {
                score += PASSED_PAWN_BONUS[advancement];
            }
        }
    }

    score
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Heuristic: is this an endgame position?
/// We consider it endgame when both sides have no queens,
/// or each side has at most queen + 1 minor piece.
pub fn is_endgame_phase(board: &Board) -> bool {
    let white_queens = (board.pieces(Piece::Queen) & board.colors(Color::White)).len();
    let black_queens = (board.pieces(Piece::Queen) & board.colors(Color::Black)).len();

    if white_queens == 0 && black_queens == 0 {
        return true;
    }

    // Count non-pawn, non-king material for each side.
    let white_minors = (board.pieces(Piece::Knight) & board.colors(Color::White)).len()
        + (board.pieces(Piece::Bishop) & board.colors(Color::White)).len();
    let white_rooks = (board.pieces(Piece::Rook) & board.colors(Color::White)).len();

    let black_minors = (board.pieces(Piece::Knight) & board.colors(Color::Black)).len()
        + (board.pieces(Piece::Bishop) & board.colors(Color::Black)).len();
    let black_rooks = (board.pieces(Piece::Rook) & board.colors(Color::Black)).len();

    let white_total = white_queens as i32 * 9 + white_rooks as i32 * 5 + white_minors as i32 * 3;
    let black_total = black_queens as i32 * 9 + black_rooks as i32 * 5 + black_minors as i32 * 3;

    // Endgame if total material (excluding pawns/kings) is low.
    white_total + black_total <= 14
}

/// Checkmate/stalemate evaluation. Call this when there are no legal moves.
pub fn eval_terminal(board: &Board) -> i32 {
    // If the side to move is in check and has no legal moves, it's checkmate.
    // The side to move lost.
    if !board.checkers().is_empty() {
        // Checkmate: return a very negative score (we lost).
        -KING_VALUE
    } else {
        // Stalemate: draw.
        0
    }
}
