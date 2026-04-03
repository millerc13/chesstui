//! Chess AI Engine
//!
//! Self-contained chess engine with multi-level difficulty.
//! Uses `cozy-chess` for legal move generation (bitboard-based),
//! builds search + evaluation + personality on top.

pub mod evaluation;
pub mod search;
pub mod difficulty;
pub mod opening_book;
pub mod personality;
pub mod tables;
pub mod integration;

use cozy_chess::{Board, Move};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use self::difficulty::DifficultyLevel;
use self::opening_book::OpeningBook;
use self::search::SearchEngine;

/// Top-level AI controller. Owns the search engine, opening book,
/// and difficulty configuration.
pub struct ChessAI {
    difficulty: DifficultyLevel,
    opening_book: OpeningBook,
    /// Shared cancellation flag -- the UI thread sets this to stop search early.
    cancel: Arc<AtomicBool>,
}

/// Result returned to the game loop after AI computes a move.
pub struct AIResult {
    pub chosen_move: Move,
    /// Centipawn evaluation from the AI's perspective (positive = good for AI).
    pub eval_cp: i32,
    /// How many nodes the engine searched.
    pub nodes_searched: u64,
    /// How long the computation took.
    pub think_time: Duration,
    /// Principal variation (best line found).
    pub pv: Vec<Move>,
    /// Search depth completed.
    pub depth_reached: u8,
}

impl ChessAI {
    pub fn new(level: u8) -> Self {
        Self {
            difficulty: DifficultyLevel::from_level(level),
            opening_book: OpeningBook::new(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns a clone of the cancellation flag for the UI thread.
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }

    pub fn set_difficulty(&mut self, level: u8) {
        self.difficulty = DifficultyLevel::from_level(level);
    }

    pub fn difficulty(&self) -> &DifficultyLevel {
        &self.difficulty
    }

    /// Compute the best move for the given position.
    /// This is the main entry point -- call from a blocking thread.
    pub fn think(&self, board: &Board, move_history: &[Move]) -> Option<AIResult> {
        self.cancel.store(false, Ordering::Relaxed);
        let start = Instant::now();

        // 1. Try the opening book first (if enabled at this difficulty level).
        if self.difficulty.use_opening_book {
            if let Some(book_move) = self.opening_book.probe(board, move_history) {
                // Verify the book move is legal in the current position.
                let mut legal = Vec::new();
                board.generate_moves(|moves| {
                    legal.extend(moves);
                    false
                });
                if legal.iter().any(|m| *m == book_move) {
                    return Some(AIResult {
                        chosen_move: book_move,
                        eval_cp: 0,
                        nodes_searched: 0,
                        think_time: start.elapsed(),
                        pv: vec![book_move],
                        depth_reached: 0,
                    });
                }
            }
        }

        // 2. Run the search engine.
        let mut engine = SearchEngine::new(
            self.difficulty.clone(),
            self.cancel.clone(),
        );

        let result = engine.iterative_deepening(board, start);

        // 3. Apply personality quirks (deliberate mistakes at lower levels).
        match result {
            Some(mut res) => {
                let chosen = personality::apply_personality(
                    board,
                    &self.difficulty,
                    &res.pv,
                    res.eval_cp,
                    &engine,
                );
                if let Some(personality_move) = chosen {
                    res.chosen_move = personality_move;
                }
                res.think_time = start.elapsed();
                Some(res)
            }
            None => None,
        }
    }
}
