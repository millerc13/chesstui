use cozy_chess::{Board, File, Move, Piece, Rank, Square};

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Convert a legal move into Standard Algebraic Notation (SAN).
///
/// The `board` must be the position *before* the move is played.
pub fn to_algebraic(board: &Board, mv: &Move) -> String {
    // 1. Castling
    if let Some(castle) = detect_castling(board, mv) {
        return castle;
    }

    let piece = piece_on(board, mv.from)
        .expect("to_algebraic: no piece on source square");

    let mut s = String::new();

    // 2. Piece letter (non-pawn gets letter; pawn only gets file when capturing)
    match piece {
        Piece::Pawn => {
            if is_capture(board, mv) {
                s.push(file_char(mv.from.file()));
            }
        }
        other => {
            // 3. Disambiguation for non-pawn pieces
            let disambig = disambiguate(board, mv, other);
            s.push(piece_char(other));
            s.push_str(&disambig);
        }
    }

    // 4. Capture marker
    if is_capture(board, mv) {
        s.push('x');
    }

    // 5. Destination square
    s.push(file_char(mv.to.file()));
    s.push(rank_char(mv.to.rank()));

    // 6. Promotion
    if let Some(promo) = mv.promotion {
        s.push('=');
        s.push(piece_char(promo));
    }

    // 7. Check / checkmate suffix
    s.push_str(&check_suffix(board, mv));

    s
}

/// Parse a square name like "e4" into a `Square`. Delegates to cozy-chess.
pub fn parse_square(s: &str) -> Option<Square> {
    s.parse::<Square>().ok()
}

// ──────────────────────────────────────────────────────────────────────────────
// Private helpers
// ──────────────────────────────────────────────────────────────────────────────

fn piece_on(board: &Board, sq: Square) -> Option<Piece> {
    for piece in [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ] {
        if board.pieces(piece).has(sq) {
            return Some(piece);
        }
    }
    None
}

fn piece_char(piece: Piece) -> char {
    match piece {
        Piece::Pawn => 'P',
        Piece::Knight => 'N',
        Piece::Bishop => 'B',
        Piece::Rook => 'R',
        Piece::Queen => 'Q',
        Piece::King => 'K',
    }
}

fn file_char(file: File) -> char {
    match file {
        File::A => 'a',
        File::B => 'b',
        File::C => 'c',
        File::D => 'd',
        File::E => 'e',
        File::F => 'f',
        File::G => 'g',
        File::H => 'h',
    }
}

fn rank_char(rank: Rank) -> char {
    match rank {
        Rank::First => '1',
        Rank::Second => '2',
        Rank::Third => '3',
        Rank::Fourth => '4',
        Rank::Fifth => '5',
        Rank::Sixth => '6',
        Rank::Seventh => '7',
        Rank::Eighth => '8',
    }
}

/// Detect castling.
///
/// cozy-chess represents castling as king -> rook's square:
///   White kingside:  E1 -> H1   →  "O-O"
///   White queenside: E1 -> A1   →  "O-O-O"
///   Black kingside:  E8 -> H8   →  "O-O"
///   Black queenside: E8 -> A8   →  "O-O-O"
fn detect_castling(board: &Board, mv: &Move) -> Option<String> {
    if !board.pieces(Piece::King).has(mv.from) {
        return None;
    }

    let color = board.side_to_move();
    let friendly = board.colors(color);

    // The destination must hold a friendly rook for this to be castling.
    // (A king move to an adjacent square never lands on a friendly rook.)
    if friendly.has(mv.to) && board.pieces(Piece::Rook).has(mv.to) {
        // Kingside: rook is to the right (higher file index)
        if mv.to.file() > mv.from.file() {
            return Some("O-O".to_string());
        } else {
            return Some("O-O-O".to_string());
        }
    }

    None
}

/// Returns true if the move is a capture (normal or en-passant).
fn is_capture(board: &Board, mv: &Move) -> bool {
    let opponent = !board.side_to_move();

    // Normal capture: opponent piece sits on the destination
    if board.colors(opponent).has(mv.to) {
        return true;
    }

    // En-passant: pawn moves diagonally to an empty square
    if let Some(Piece::Pawn) = piece_on(board, mv.from) {
        if mv.from.file() != mv.to.file() {
            return true;
        }
    }

    false
}

/// Build the disambiguation string for non-pawn pieces.
///
/// When two (or more) pieces of the same type can reach the same destination,
/// we add enough info to identify the mover:
///   - File alone if that distinguishes.
///   - Rank alone if that distinguishes.
///   - Both file and rank otherwise.
fn disambiguate(board: &Board, mv: &Move, piece: Piece) -> String {
    // Collect all legal moves by pieces of the same type that also go to mv.to.
    let mut ambiguous: Vec<Move> = Vec::new();

    board.generate_moves(|list| {
        for candidate in list {
            if candidate.from == mv.from {
                continue; // same piece, skip
            }
            if candidate.to != mv.to {
                continue; // different destination
            }
            if !board.pieces(piece).has(candidate.from) {
                continue; // different piece type
            }
            ambiguous.push(candidate);
        }
        false
    });

    if ambiguous.is_empty() {
        return String::new();
    }

    // Check whether file alone is unique among ambiguous pieces
    let same_file = ambiguous
        .iter()
        .any(|c| c.from.file() == mv.from.file());

    if !same_file {
        // File is unique → use only file
        return file_char(mv.from.file()).to_string();
    }

    // Check whether rank alone is unique
    let same_rank = ambiguous
        .iter()
        .any(|c| c.from.rank() == mv.from.rank());

    if !same_rank {
        // Rank is unique → use only rank
        return rank_char(mv.from.rank()).to_string();
    }

    // Both needed
    let mut s = String::new();
    s.push(file_char(mv.from.file()));
    s.push(rank_char(mv.from.rank()));
    s
}

/// Returns "+", "#", or "" based on the position after the move.
fn check_suffix(board: &Board, mv: &Move) -> &'static str {
    let mut after = board.clone();
    after.play_unchecked(*mv);

    if after.checkers().is_empty() {
        return "";
    }

    // In check — is it checkmate (no legal responses)?
    let mut has_legal = false;
    after.generate_moves(|list| {
        has_legal = !list.is_empty();
        has_legal // returning true stops iteration early
    });

    if has_legal {
        "+"
    } else {
        "#"
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests (quick sanity checks; full integration tests live in tests/)
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use cozy_chess::Board;

    #[test]
    fn file_rank_chars() {
        assert_eq!(file_char(File::A), 'a');
        assert_eq!(file_char(File::H), 'h');
        assert_eq!(rank_char(Rank::First), '1');
        assert_eq!(rank_char(Rank::Eighth), '8');
    }

    #[test]
    fn piece_chars() {
        assert_eq!(piece_char(Piece::Knight), 'N');
        assert_eq!(piece_char(Piece::Queen), 'Q');
        assert_eq!(piece_char(Piece::Pawn), 'P');
    }

    #[test]
    fn parse_square_roundtrip() {
        let sq = parse_square("e4").unwrap();
        assert_eq!(sq, Square::E4);
        assert!(parse_square("z9").is_none());
    }
}
