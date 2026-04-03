//! Search algorithm: Iterative Deepening + Alpha-Beta + Quiescence.
//!
//! The search is the heart of the engine. It explores the game tree to find
//! the best move, using pruning to avoid searching hopeless branches.

use cozy_chess::{Board, Color, Move, Piece};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::difficulty::DifficultyLevel;
use super::evaluation::{eval_terminal, evaluate};
use super::tables::*;
use super::AIResult;

/// Transposition table entry.
#[derive(Clone, Copy)]
struct TTEntry {
    /// Zobrist hash of the position (for collision detection).
    hash: u64,
    /// Depth this position was searched to.
    depth: u8,
    /// Score found.
    score: i32,
    /// Type of bound: exact, lower, or upper.
    flag: TTFlag,
    /// Best move found (for move ordering).
    best_move: Option<Move>,
}

#[derive(Clone, Copy, PartialEq)]
enum TTFlag {
    Exact,
    LowerBound, // Beta cutoff -- score is at least this good.
    UpperBound, // Failed low -- score is at most this good.
}

pub struct SearchEngine {
    config: DifficultyLevel,
    cancel: Arc<AtomicBool>,
    nodes: u64,
    tt: Vec<Option<TTEntry>>,
    tt_mask: usize,
    /// Principal variation from the last completed iteration.
    best_pv: Vec<Move>,
    best_score: i32,
}

impl SearchEngine {
    pub fn new(config: DifficultyLevel, cancel: Arc<AtomicBool>) -> Self {
        let tt_size = config.tt_size.max(1).next_power_of_two();
        Self {
            config,
            cancel,
            nodes: 0,
            tt: vec![None; tt_size],
            tt_mask: tt_size - 1,
            best_pv: Vec::new(),
            best_score: 0,
        }
    }

    /// Iterative deepening: search depth 1, then 2, then 3, etc.
    /// Returns the result from the last fully completed depth.
    pub fn iterative_deepening(
        &mut self,
        board: &Board,
        start: Instant,
    ) -> Option<AIResult> {
        let max_depth = self.config.max_depth;
        let max_time = self.config.max_think_time;

        let mut best_move: Option<Move> = None;
        let mut best_score = 0i32;
        let mut best_pv = Vec::new();

        for depth in 1..=max_depth {
            // Check time before starting a new iteration.
            if start.elapsed() >= max_time {
                break;
            }
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            let mut pv = Vec::new();
            let score = self.alpha_beta(
                board,
                depth,
                -i32::MAX,
                i32::MAX,
                &mut pv,
                start,
                0, // ply
            );

            // If search was cancelled mid-iteration, discard partial results.
            if self.cancel.load(Ordering::Relaxed) || start.elapsed() >= max_time {
                // Only discard if we have at least one completed iteration.
                if best_move.is_some() {
                    break;
                }
            }

            if let Some(&mv) = pv.first() {
                best_move = Some(mv);
                best_score = score;
                best_pv = pv;
            }

            self.best_pv = best_pv.clone();
            self.best_score = best_score;
        }

        best_move.map(|mv| AIResult {
            chosen_move: mv,
            eval_cp: best_score,
            nodes_searched: self.nodes,
            think_time: start.elapsed(),
            pv: best_pv,
            depth_reached: max_depth,
        })
    }

    /// Alpha-beta search with fail-soft.
    fn alpha_beta(
        &mut self,
        board: &Board,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        pv: &mut Vec<Move>,
        start: Instant,
        ply: u8,
    ) -> i32 {
        self.nodes += 1;

        // Time check every 4096 nodes to avoid syscall overhead.
        if self.nodes % 4096 == 0 {
            if start.elapsed() >= self.config.max_think_time
                || self.cancel.load(Ordering::Relaxed)
            {
                return 0;
            }
        }

        // Generate all legal moves.
        let mut moves = Vec::new();
        board.generate_moves(|mvs| {
            moves.extend(mvs);
            false
        });

        // Terminal node: checkmate or stalemate.
        if moves.is_empty() {
            return eval_terminal(board);
        }

        // Leaf node: run quiescence search or static eval.
        if depth == 0 {
            return self.quiescence(board, alpha, beta, self.config.quiescence_depth, start);
        }

        // Probe transposition table.
        let hash = board.hash();
        let tt_idx = (hash as usize) & self.tt_mask;
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = &self.tt[tt_idx] {
            if entry.hash == hash && entry.depth >= depth {
                match entry.flag {
                    TTFlag::Exact => {
                        if let Some(mv) = entry.best_move {
                            pv.push(mv);
                        }
                        return entry.score;
                    }
                    TTFlag::LowerBound => {
                        if entry.score >= beta {
                            return entry.score;
                        }
                    }
                    TTFlag::UpperBound => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                }
            }
            // Even if we can't use the score, use the best move for ordering.
            if entry.hash == hash {
                tt_move = entry.best_move;
            }
        }

        // Move ordering: sort moves to maximize pruning.
        self.order_moves(&mut moves, board, tt_move);

        let mut best_score = -i32::MAX;
        let mut best_move = moves[0]; // There's at least one move.
        let mut node_pv = Vec::new();

        for (i, &mv) in moves.iter().enumerate() {
            let mut child = board.clone();
            child.play_unchecked(mv);

            let mut child_pv = Vec::new();
            let score;

            // Principal Variation Search: search the first move with full window,
            // then try null windows for the rest.
            if i == 0 {
                score = -self.alpha_beta(
                    &child,
                    depth - 1,
                    -beta,
                    -alpha,
                    &mut child_pv,
                    start,
                    ply + 1,
                );
            } else {
                // Null-window search.
                let mut null_score = -self.alpha_beta(
                    &child,
                    depth - 1,
                    -alpha - 1,
                    -alpha,
                    &mut child_pv,
                    start,
                    ply + 1,
                );

                // If null-window search found a better move, re-search with full window.
                if null_score > alpha && null_score < beta {
                    child_pv.clear();
                    null_score = -self.alpha_beta(
                        &child,
                        depth - 1,
                        -beta,
                        -alpha,
                        &mut child_pv,
                        start,
                        ply + 1,
                    );
                }
                score = null_score;
            }

            if score > best_score {
                best_score = score;
                best_move = mv;
                node_pv.clear();
                node_pv.push(mv);
                node_pv.extend_from_slice(&child_pv);
            }

            if score > alpha {
                alpha = score;
            }

            // Beta cutoff: opponent won't allow this line.
            if alpha >= beta {
                break;
            }
        }

        // Store in transposition table.
        let flag = if best_score >= beta {
            TTFlag::LowerBound
        } else if best_score <= alpha {
            TTFlag::UpperBound
        } else {
            TTFlag::Exact
        };

        self.tt[tt_idx] = Some(TTEntry {
            hash,
            depth,
            score: best_score,
            flag,
            best_move: Some(best_move),
        });

        *pv = node_pv;
        best_score
    }

    /// Quiescence search: only search captures to avoid the "horizon effect"
    /// where the engine stops searching just before a big capture.
    fn quiescence(
        &mut self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        depth_remaining: u8,
        start: Instant,
    ) -> i32 {
        self.nodes += 1;

        // Stand-pat: the side to move can choose not to capture.
        let stand_pat = evaluate(board, &self.config.eval);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        if depth_remaining == 0 {
            return stand_pat;
        }

        // Generate only capture moves.
        let mut captures = Vec::new();
        let opponent = board.colors(!board.side_to_move());
        board.generate_moves(|mvs| {
            for mv in mvs {
                // A move is a capture if the destination square is occupied by opponent.
                if opponent.has(mv.to) {
                    captures.push(mv);
                }
            }
            false
        });

        // Order captures by MVV-LVA (Most Valuable Victim - Least Valuable Attacker).
        captures.sort_by(|a, b| {
            let a_score = self.mvv_lva_score(board, *a);
            let b_score = self.mvv_lva_score(board, *b);
            b_score.cmp(&a_score)
        });

        for &mv in &captures {
            let mut child = board.clone();
            child.play_unchecked(mv);

            let score = -self.quiescence(&child, -beta, -alpha, depth_remaining - 1, start);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Move ordering heuristic. Good move ordering is critical for alpha-beta
    /// pruning efficiency. Order:
    /// 1. TT move (from previous iteration or transposition)
    /// 2. Captures ordered by MVV-LVA
    /// 3. Non-captures
    fn order_moves(&self, moves: &mut Vec<Move>, board: &Board, tt_move: Option<Move>) {
        let opponent = board.colors(!board.side_to_move());

        moves.sort_by(|a, b| {
            let a_score = self.move_order_score(board, *a, &tt_move, opponent);
            let b_score = self.move_order_score(board, *b, &tt_move, opponent);
            b_score.cmp(&a_score)
        });
    }

    fn move_order_score(
        &self,
        board: &Board,
        mv: Move,
        tt_move: &Option<Move>,
        opponent: cozy_chess::BitBoard,
    ) -> i32 {
        // TT move gets highest priority.
        if Some(mv) == *tt_move {
            return 100_000;
        }

        let mut score = 0;

        // Captures get priority, ordered by MVV-LVA.
        if opponent.has(mv.to) {
            score += 10_000 + self.mvv_lva_score(board, mv);
        }

        // Promotion bonus.
        if mv.promotion.is_some() {
            score += 9_000;
        }

        score
    }

    /// MVV-LVA: Most Valuable Victim - Least Valuable Attacker.
    /// Prefer capturing high-value pieces with low-value pieces.
    fn mvv_lva_score(&self, board: &Board, mv: Move) -> i32 {
        let victim_value = self.piece_on_square(board, mv.to);
        let attacker_value = self.piece_on_square(board, mv.from);
        victim_value * 10 - attacker_value
    }

    fn piece_on_square(&self, board: &Board, sq: cozy_chess::Square) -> i32 {
        if board.pieces(Piece::Pawn).has(sq) {
            PAWN_VALUE
        } else if board.pieces(Piece::Knight).has(sq) {
            KNIGHT_VALUE
        } else if board.pieces(Piece::Bishop).has(sq) {
            BISHOP_VALUE
        } else if board.pieces(Piece::Rook).has(sq) {
            ROOK_VALUE
        } else if board.pieces(Piece::Queen).has(sq) {
            QUEEN_VALUE
        } else if board.pieces(Piece::King).has(sq) {
            KING_VALUE
        } else {
            0
        }
    }

    /// Returns all legal moves with their engine evaluations.
    /// Used by the personality module to pick "human-like" suboptimal moves.
    pub fn evaluate_all_moves(&self, board: &Board) -> Vec<(Move, i32)> {
        let mut moves = Vec::new();
        board.generate_moves(|mvs| {
            moves.extend(mvs);
            false
        });

        let mut scored: Vec<(Move, i32)> = moves
            .into_iter()
            .map(|mv| {
                let mut child = board.clone();
                child.play_unchecked(mv);
                // Evaluate from the opponent's perspective, then negate.
                let score = -evaluate(&child, &self.config.eval);
                (mv, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored
    }
}
