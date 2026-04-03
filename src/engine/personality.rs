//! Personality system: makes weak AI play like a human, not a random number generator.
//!
//! The key insight: a depth-1 engine that picks a random non-best move feels alien.
//! A human beginner makes *specific kinds* of mistakes:
//! - They see obvious captures but miss quiet defensive moves.
//! - They forget about pieces they haven't looked at recently.
//! - They don't count attackers/defenders correctly.
//! - They get "tunnel vision" on one part of the board.
//!
//! This module simulates those patterns.

use cozy_chess::{Board, Move, Piece, Square};
use rand::Rng;

use super::difficulty::DifficultyLevel;
use super::search::SearchEngine;

/// Given the engine's best move and the full position, decide whether to
/// play a different (worse) move to simulate human weakness.
///
/// Returns `None` if the AI should play the engine's best move.
/// Returns `Some(move)` if a personality-adjusted move was chosen.
pub fn apply_personality(
    board: &Board,
    config: &DifficultyLevel,
    pv: &[Move],
    best_eval: i32,
    engine: &SearchEngine,
) -> Option<Move> {
    let personality = &config.personality;

    // At the highest levels, always play the best move.
    if personality.blunder_rate == 0.0 && personality.eval_noise_cp == 0 {
        return None;
    }

    let mut rng = rand::thread_rng();

    // Get all legal moves with their evaluations.
    let all_moves = engine.evaluate_all_moves(board);
    if all_moves.len() <= 1 {
        return None; // Forced move.
    }

    let best_move = pv.first().copied()?;
    let best_score = all_moves
        .iter()
        .find(|(m, _)| *m == best_move)
        .map(|(_, s)| *s)
        .unwrap_or(best_eval);

    // Step 1: Add evaluation noise.
    // This simulates the human inability to precisely judge a position.
    let noisy_moves: Vec<(Move, i32)> = all_moves
        .iter()
        .map(|(mv, score)| {
            let noise = if personality.eval_noise_cp > 0 {
                rng.gen_range(
                    -(personality.eval_noise_cp)..=(personality.eval_noise_cp),
                )
            } else {
                0
            };
            (*mv, score + noise)
        })
        .collect();

    // Step 2: Decide whether to blunder this move.
    let should_blunder = rng.gen::<f32>() < personality.blunder_rate;

    if should_blunder {
        // Pick a "human-like" blunder, not a random move.
        if let Some(blunder) = pick_human_blunder(board, &noisy_moves, best_score, config, &mut rng)
        {
            return Some(blunder);
        }
    }

    // Step 3: Even without a blunder, the eval noise might cause a different
    // move to appear "best" to this level of player.
    let mut sorted_noisy = noisy_moves;
    sorted_noisy.sort_by(|a, b| b.1.cmp(&a.1));

    let noisy_best = sorted_noisy[0].0;
    if noisy_best != best_move {
        // The noise caused a different move to look best.
        // Only allow this if the real evaluation difference is within tolerance.
        let real_score_of_noisy = all_moves
            .iter()
            .find(|(m, _)| *m == noisy_best)
            .map(|(_, s)| *s)
            .unwrap_or(0);
        let loss = best_score - real_score_of_noisy;
        if loss <= personality.max_blunder_cp {
            return Some(noisy_best);
        }
    }

    None // Play the best move.
}

/// Pick a "human-like" blunder. Humans don't make random mistakes --
/// they make *motivated* mistakes. This function categorizes common
/// human error patterns and selects one.
fn pick_human_blunder(
    board: &Board,
    moves: &[(Move, i32)],
    best_score: i32,
    config: &DifficultyLevel,
    rng: &mut impl Rng,
) -> Option<Move> {
    let personality = &config.personality;
    let max_loss = personality.max_blunder_cp;

    // Filter to moves that aren't too terrible (within the max blunder range).
    let acceptable: Vec<&(Move, i32)> = moves
        .iter()
        .filter(|(_, score)| best_score - score <= max_loss)
        .collect();

    if acceptable.len() <= 1 {
        return None;
    }

    // Weight different types of "human" blunders:
    let blunder_type = rng.gen_range(0..100);

    match blunder_type {
        // 40%: "Tempting" move -- a capture or check that looks good but isn't best.
        0..=39 => {
            let tempting: Vec<_> = acceptable
                .iter()
                .filter(|(mv, _)| {
                    let is_capture = board.colors(!board.side_to_move()).has(mv.to);
                    let mut child = board.clone();
                    child.play_unchecked(**mv);
                    let is_check = !child.checkers().is_empty();
                    (is_capture || is_check) && **mv != moves[0].0
                })
                .collect();

            if let Some(&&(mv, _)) = tempting.first() {
                return Some(mv);
            }
        }
        // 25%: "Lazy" move -- a developing move or quiet move, missing a tactic.
        40..=64 => {
            let quiet: Vec<_> = acceptable
                .iter()
                .filter(|(mv, _)| {
                    let is_capture = board.colors(!board.side_to_move()).has(mv.to);
                    !is_capture
                })
                .collect();

            if quiet.len() > 1 {
                let idx = rng.gen_range(0..quiet.len().min(3)); // Pick from top 3 quiet moves.
                return Some(quiet[idx].0);
            }
        }
        // 20%: "Greedy" move -- takes material even if it loses positionally.
        65..=84 => {
            let greedy: Vec<_> = acceptable
                .iter()
                .filter(|(mv, _)| board.colors(!board.side_to_move()).has(mv.to))
                .collect();

            if let Some(&&(mv, _)) = greedy.first() {
                return Some(mv);
            }
        }
        // 15%: "Second-best" -- just plays the second best move.
        85..=99 => {
            if acceptable.len() >= 2 {
                return Some(acceptable[1].0);
            }
        }
        _ => {}
    }

    None
}
