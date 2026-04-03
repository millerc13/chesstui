//! Compact opening book stored inline in the binary.
//!
//! Instead of a large Polyglot .bin file, we hardcode ~30 common opening lines
//! as move sequences. This adds roughly 2-3 KB to the binary.
//!
//! The book is stored as a trie: each node maps a move to either more moves
//! or a leaf. We traverse the trie using the game's move history.
//!
//! At lower difficulty levels, the opening book is disabled so the AI plays
//! natural-looking (if poor) openings.

use cozy_chess::{Board, Move, Square, Piece};
use rand::Rng;
use std::collections::HashMap;
use std::str::FromStr;

/// Opening book stored as a hashmap from position hash to candidate moves.
/// Built at startup from hardcoded opening lines.
pub struct OpeningBook {
    /// Maps board hash -> list of candidate book moves with weights.
    entries: HashMap<u64, Vec<BookMove>>,
}

struct BookMove {
    mv: Move,
    weight: u16, // Higher = more likely to be chosen.
}

impl OpeningBook {
    pub fn new() -> Self {
        let mut book = Self {
            entries: HashMap::new(),
        };
        book.build();
        book
    }

    /// Look up the current position in the book.
    /// Returns a randomly selected book move (weighted), or None.
    pub fn probe(&self, board: &Board, _move_history: &[Move]) -> Option<Move> {
        let hash = board.hash();
        let candidates = self.entries.get(&hash)?;
        if candidates.is_empty() {
            return None;
        }

        // Weighted random selection.
        let total_weight: u16 = candidates.iter().map(|c| c.weight).sum();
        let mut rng = rand::thread_rng();
        let mut roll = rng.gen_range(0..total_weight);

        for candidate in candidates {
            if roll < candidate.weight {
                return Some(candidate.mv);
            }
            roll -= candidate.weight;
        }

        Some(candidates[0].mv)
    }

    /// Build the book from hardcoded opening lines.
    fn build(&mut self) {
        // Each line is a sequence of UCI moves.
        // We replay each line on a board and record the hash -> move mapping.

        let openings: &[(&str, &[&str], u16)] = &[
            // Italian Game
            ("Italian", &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5"], 10),
            // Sicilian Defense (Open)
            ("Sicilian Open", &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4"], 10),
            // Sicilian Najdorf
            ("Sicilian Najdorf", &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6"], 8),
            // French Defense
            ("French", &["e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6"], 8),
            // Caro-Kann
            ("Caro-Kann", &["e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4"], 8),
            // Queen's Gambit Declined
            ("QGD", &["d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5"], 10),
            // Queen's Gambit Accepted
            ("QGA", &["d2d4", "d7d5", "c2c4", "d5c4", "g1f3", "g8f6", "e2e3"], 7),
            // Slav Defense
            ("Slav", &["d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3"], 7),
            // King's Indian Defense
            ("KID", &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6"], 8),
            // London System
            ("London", &["d2d4", "d7d5", "g1f3", "g8f6", "c1f4", "e7e6"], 9),
            // Ruy Lopez
            ("Ruy Lopez", &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6"], 10),
            // Scotch Game
            ("Scotch", &["e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4"], 7),
            // Pirc Defense
            ("Pirc", &["e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6"], 6),
            // English Opening
            ("English", &["c2c4", "e7e5", "b1c3", "g8f6", "g1f3"], 7),
            // Reti Opening
            ("Reti", &["g1f3", "d7d5", "c2c4", "e7e6", "g2g3"], 6),
            // King's Indian Attack
            ("KIA", &["g1f3", "d7d5", "g2g3", "g8f6", "f1g2", "e7e6"], 6),
            // Dutch Defense
            ("Dutch", &["d2d4", "f7f5", "c2c4", "g8f6", "g1f3"], 5),
            // Nimzo-Indian
            ("Nimzo-Indian", &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"], 8),
            // Grunfeld Defense
            ("Grunfeld", &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5"], 7),
            // Vienna Game
            ("Vienna", &["e2e4", "e7e5", "b1c3", "g8f6", "f1c4"], 5),
            // Scandinavian Defense
            ("Scandinavian", &["e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5a5"], 5),
            // Alekhine Defense
            ("Alekhine", &["e2e4", "g8f6", "e4e5", "f6d5", "d2d4", "d7d6"], 4),
            // Catalan
            ("Catalan", &["d2d4", "g8f6", "c2c4", "e7e6", "g2g3", "d7d5", "f1g2"], 7),
            // Semi-Slav
            ("Semi-Slav", &["d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3", "e7e6"], 7),
        ];

        for (_, moves, weight) in openings {
            let mut board = Board::default();

            // For each position in the line, record the next move as a book move.
            for (i, move_str) in moves.iter().enumerate() {
                let hash = board.hash();
                let mv = parse_uci_move(&board, move_str).expect("Invalid book move");

                let entry = self.entries.entry(hash).or_insert_with(Vec::new);

                // Avoid duplicate entries for the same move.
                if !entry.iter().any(|bm| bm.mv == mv) {
                    entry.push(BookMove {
                        mv,
                        weight: *weight,
                    });
                }

                board.play(mv);
            }
        }
    }
}

/// Parse a UCI move string like "e2e4" into a Move, using the board
/// to determine the piece (needed for promotions).
fn parse_uci_move(board: &Board, uci: &str) -> Option<Move> {
    let bytes = uci.as_bytes();
    if bytes.len() < 4 {
        return None;
    }

    let from_file = (bytes[0] - b'a') as usize;
    let from_rank = (bytes[1] - b'1') as usize;
    let to_file = (bytes[2] - b'a') as usize;
    let to_rank = (bytes[3] - b'1') as usize;

    let from = Square::index(from_rank * 8 + from_file);
    let to = Square::index(to_rank * 8 + to_file);

    let promotion = if bytes.len() == 5 {
        match bytes[4] {
            b'q' => Some(Piece::Queen),
            b'r' => Some(Piece::Rook),
            b'b' => Some(Piece::Bishop),
            b'n' => Some(Piece::Knight),
            _ => None,
        }
    } else {
        None
    };

    let mv = Move { from, to, promotion };

    // Verify it's legal.
    let mut legal = false;
    board.generate_moves(|mvs| {
        for m in mvs {
            if m == mv {
                legal = true;
                return true;
            }
        }
        false
    });

    if legal { Some(mv) } else { None }
}
