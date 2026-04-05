use crate::game::notation::to_algebraic;
use cozy_chess::{Board, Move, Piece, Square};

// ──────────────────────────────────────────────────────────────────────────────
// Public types
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum InputResult {
    /// Exactly one legal move matches — execute it.
    Exact(Move),
    /// Multiple moves still match — need more input. Contains count of matches.
    NeedMore(usize),
    /// No legal move matches the current input.
    NoMatch,
}

// ──────────────────────────────────────────────────────────────────────────────
// MoveInputParser
// ──────────────────────────────────────────────────────────────────────────────

/// Incrementally matches user keystrokes against the set of legal moves
/// in a given position, using SAN strings for matching.
pub struct MoveInputParser {
    /// Pre-computed (move, SAN) pairs for the position.
    legal_moves: Vec<(Move, String)>,
    /// Characters typed so far.
    buffer: String,
    /// Indices into `legal_moves` that still match the current buffer.
    matching: Vec<usize>,
}

impl MoveInputParser {
    /// Build a parser from the current board position.
    /// Generates all legal moves and their SAN strings up-front.
    pub fn new(board: &Board) -> Self {
        let mut legal_moves: Vec<(Move, String)> = Vec::new();
        board.generate_moves(|list| {
            for mv in list {
                let san = to_algebraic(board, &mv);
                legal_moves.push((mv, san));
            }
            false
        });

        let matching: Vec<usize> = (0..legal_moves.len()).collect();

        Self {
            legal_moves,
            buffer: String::new(),
            matching,
        }
    }

    /// Returns the current input buffer.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Clears the input buffer and resets matching to all legal moves.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.matching = (0..self.legal_moves.len()).collect();
    }

    /// Feed one character and return the match state.
    pub fn feed(&mut self, ch: char) -> InputResult {
        self.buffer.push(ch);

        // 1. Castling shorthand: "OO"/"oo" → O-O,  "OOO"/"ooo" → O-O-O
        if let Some(result) = self.try_castling() {
            return result;
        }

        // 2. Square-to-square format: e.g. "e2e4" or "e7e8q"
        if let Some(result) = self.try_square_to_square() {
            return result;
        }

        // 3. SAN prefix matching (with lenient 'x'-strip variant)
        self.matching = self
            .legal_moves
            .iter()
            .enumerate()
            .filter(|(_, (_, san))| san_matches(san, &self.buffer))
            .map(|(i, _)| i)
            .collect();

        self.result_from_matching_with_completeness()
    }

    /// Returns references to the (Move, SAN) pairs that currently match.
    pub fn matching_moves(&self) -> Vec<(&Move, &str)> {
        self.matching
            .iter()
            .map(|&i| (&self.legal_moves[i].0, self.legal_moves[i].1.as_str()))
            .collect()
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Only returns `Exact` when the buffer
    /// matches the *complete* SAN of one move (ignoring check/checkmate suffixes).
    /// If there's one prefix match but the input is still a partial token, return
    /// `NeedMore(1)` to keep prompting for more input.
    fn result_from_matching_with_completeness(&self) -> InputResult {
        match self.matching.len() {
            0 => InputResult::NoMatch,
            1 => {
                let (mv, san) = &self.legal_moves[self.matching[0]];
                let stripped = san.trim_end_matches(|c| c == '+' || c == '#');
                // Only auto-execute when the buffer equals the complete (stripped) SAN.
                if self.buffer == stripped {
                    InputResult::Exact(*mv)
                } else {
                    InputResult::NeedMore(1)
                }
            }
            n => InputResult::NeedMore(n),
        }
    }

    /// Check if the buffer looks like a castling shorthand.
    /// Returns Some(result) if handled, None otherwise.
    fn try_castling(&self) -> Option<InputResult> {
        let buf = self.buffer.to_ascii_uppercase();

        // Need exactly "OO" or "OOO" (we only handle on exact length match
        // so that typing a single 'O' still falls through to SAN matching).
        let target_san = match buf.as_str() {
            "OO" => "O-O",
            "OOO" => "O-O-O",
            _ => return None,
        };

        // Find the matching castling move.
        let matches: Vec<usize> = self
            .legal_moves
            .iter()
            .enumerate()
            .filter(|(_, (_, san))| san == target_san)
            .map(|(i, _)| i)
            .collect();

        Some(match matches.len() {
            0 => InputResult::NoMatch,
            1 => InputResult::Exact(self.legal_moves[matches[0]].0),
            n => InputResult::NeedMore(n),
        })
    }

    /// Check if the buffer looks like a square-to-square coordinate string.
    /// Handles partial input (2-3 chars) as NeedMore, and complete input (4-5 chars)
    /// as Exact/NoMatch.
    fn try_square_to_square(&self) -> Option<InputResult> {
        let buf = &self.buffer;

        // Check if the first two chars form a valid source square
        if buf.len() < 2 {
            return None;
        }
        let from = buf[0..2].parse::<Square>().ok()?;

        // 2-3 chars: we have a source square, check if any legal move starts there
        if buf.len() == 2 || buf.len() == 3 {
            let has_moves = self.legal_moves.iter().any(|(mv, _)| mv.from == from);
            if has_moves {
                return Some(InputResult::NeedMore(1));
            } else {
                return Some(InputResult::NoMatch);
            }
        }

        // 4-5 chars: full from+to coordinate
        let to = buf[2..4].parse::<Square>().ok()?;

        // Map standard castling coordinates to cozy-chess king→rook encoding
        let effective_to = match (from, to) {
            (Square::E1, Square::G1) => Square::H1, // White kingside
            (Square::E1, Square::C1) => Square::A1, // White queenside
            (Square::E8, Square::G8) => Square::H8, // Black kingside
            (Square::E8, Square::C8) => Square::A8, // Black queenside
            _ => to,
        };

        let promo_piece: Option<Piece> = if buf.len() == 5 {
            match buf.chars().nth(4).map(|c| c.to_ascii_lowercase()) {
                Some('q') => Some(Piece::Queen),
                Some('r') => Some(Piece::Rook),
                Some('b') => Some(Piece::Bishop),
                Some('n') => Some(Piece::Knight),
                _ => return None,
            }
        } else {
            None
        };

        // Try with effective_to (handles castling remapping), fall back to original to
        let matches: Vec<usize> = self
            .legal_moves
            .iter()
            .enumerate()
            .filter(|(_, (mv, _))| {
                mv.from == from
                    && (mv.to == effective_to || mv.to == to)
                    && mv.promotion == promo_piece
            })
            .map(|(i, _)| i)
            .collect();

        // If multiple matches are all promotions and no suffix given, default to queen
        if matches.len() > 1 && promo_piece.is_none() {
            let all_promos = matches
                .iter()
                .all(|&i| self.legal_moves[i].0.promotion.is_some());
            if all_promos {
                if let Some(&qi) = matches
                    .iter()
                    .find(|&&i| self.legal_moves[i].0.promotion == Some(Piece::Queen))
                {
                    return Some(InputResult::Exact(self.legal_moves[qi].0));
                }
            }
        }

        Some(match matches.len() {
            0 => InputResult::NoMatch,
            1 => InputResult::Exact(self.legal_moves[matches[0]].0),
            n => InputResult::NeedMore(n),
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// SAN matching helper
// ──────────────────────────────────────────────────────────────────────────────

/// Returns true if `san` matches the `input` prefix.
///
/// Two strategies are tried:
///   1. Direct prefix: strip check/checkmate suffix from `san`, then check
///      whether it starts with `input`.
///   2. Lenient 'x'-strip: remove 'x' from both the stripped SAN and `input`,
///      then retry. This lets "ed5" match "exd5".
fn san_matches(san: &str, input: &str) -> bool {
    // Strip trailing '+' / '#' for comparison.
    let stripped = san.trim_end_matches(|c| c == '+' || c == '#');

    // 1. Direct prefix match.
    if stripped.starts_with(input) {
        return true;
    }

    // 2. Lenient: remove 'x' from both sides.
    let san_no_x: String = stripped.chars().filter(|&c| c != 'x').collect();
    let input_no_x: String = input.chars().filter(|&c| c != 'x').collect();
    if san_no_x.starts_with(&input_no_x) {
        return true;
    }

    false
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn san_matches_direct() {
        assert!(san_matches("e4", "e"));
        assert!(san_matches("e4", "e4"));
        assert!(!san_matches("e4", "e5"));
    }

    #[test]
    fn san_matches_lenient_x() {
        // "exd5" should match input "ed5"
        assert!(san_matches("exd5", "ed5"));
        // "exd5" should also match input "exd5"
        assert!(san_matches("exd5", "exd5"));
    }

    #[test]
    fn san_matches_strips_check_suffix() {
        assert!(san_matches("Nf3+", "Nf3"));
        assert!(san_matches("Qh5#", "Qh5"));
    }
}
