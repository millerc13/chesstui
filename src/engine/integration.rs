//! Integration with the TUI game loop.
//!
//! The AI runs on a background thread via `tokio::spawn_blocking` or
//! `std::thread::spawn`. The main thread continues rendering the UI
//! (showing a "thinking..." animation) while the AI computes.
//!
//! Communication is via channels: the game loop sends a position,
//! the AI thread sends back a move.

use cozy_chess::{Board, Move};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

use super::{AIResult, ChessAI};

/// Message sent from the game loop to the AI thread.
pub struct ThinkRequest {
    pub board: Board,
    pub move_history: Vec<Move>,
    pub response: oneshot::Sender<Option<AIResult>>,
}

/// Manages the AI background thread and communication channels.
pub struct AIController {
    /// Send think requests to the AI thread.
    tx: mpsc::Sender<ThinkRequest>,
    /// Shared cancellation flag.
    cancel: Arc<AtomicBool>,
    /// Whether the AI is currently thinking.
    pub is_thinking: Arc<AtomicBool>,
}

impl AIController {
    /// Spawn the AI on a background thread.
    /// `level` is the difficulty level (1-10).
    pub fn new(level: u8) -> Self {
        let (tx, mut rx) = mpsc::channel::<ThinkRequest>(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let is_thinking = Arc::new(AtomicBool::new(false));

        let cancel_clone = cancel.clone();
        let thinking_clone = is_thinking.clone();

        // Spawn a dedicated OS thread for the CPU-intensive search.
        // We do NOT use tokio::spawn because the search is blocking
        // and would starve the async runtime.
        std::thread::Builder::new()
            .name("chess-ai".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async move {
                    let mut ai = ChessAI::new(level);
                    // Override the AI's cancel flag with our shared one.
                    // (In practice, wire this up through the constructor.)

                    while let Some(request) = rx.recv().await {
                        thinking_clone.store(true, Ordering::Relaxed);
                        cancel_clone.store(false, Ordering::Relaxed);

                        let result = ai.think(&request.board, &request.move_history);

                        // Apply human-like delay for low levels.
                        if let Some(ref res) = result {
                            let min_time = ai.difficulty().personality.min_think_time;
                            if res.think_time < min_time {
                                let delay = min_time - res.think_time;
                                tokio::time::sleep(delay).await;
                            }
                        }

                        thinking_clone.store(false, Ordering::Relaxed);
                        let _ = request.response.send(result);
                    }
                });
            })
            .expect("Failed to spawn AI thread");

        Self {
            tx,
            cancel,
            is_thinking,
        }
    }

    /// Request the AI to think about a position.
    /// Returns a oneshot receiver that will contain the AI's move.
    pub async fn request_move(
        &self,
        board: Board,
        move_history: Vec<Move>,
    ) -> oneshot::Receiver<Option<AIResult>> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let request = ThinkRequest {
            board,
            move_history,
            response: resp_tx,
        };

        self.tx.send(request).await.expect("AI thread died");
        resp_rx
    }

    /// Cancel the current search (e.g., if the user wants to undo or quit).
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Check if the AI is currently thinking.
    pub fn is_thinking(&self) -> bool {
        self.is_thinking.load(Ordering::Relaxed)
    }
}

// ---------------------------------------------------------------
// Example: how the game loop would use the AI controller.
// ---------------------------------------------------------------
//
// ```rust
// // In your game loop (pseudocode):
// async fn game_loop(mut terminal: Terminal, ai: AIController) {
//     let mut board = Board::default();
//     let mut move_history = Vec::new();
//     let mut ai_receiver: Option<oneshot::Receiver<Option<AIResult>>> = None;
//
//     loop {
//         // Render the board.
//         terminal.draw(|f| {
//             render_board(f, &board);
//             if ai.is_thinking() {
//                 render_thinking_indicator(f); // "AI is thinking..."
//             }
//         })?;
//
//         // Handle input.
//         if let Some(event) = poll_event(Duration::from_millis(50))? {
//             match event {
//                 Event::PlayerMove(mv) => {
//                     board.play(mv);
//                     move_history.push(mv);
//
//                     // Request AI move.
//                     ai_receiver = Some(
//                         ai.request_move(board.clone(), move_history.clone()).await
//                     );
//                 }
//                 Event::Quit => break,
//                 _ => {}
//             }
//         }
//
//         // Check if AI has finished thinking.
//         if let Some(ref mut rx) = ai_receiver {
//             if let Ok(result) = rx.try_recv() {
//                 if let Some(ai_result) = result {
//                     board.play(ai_result.chosen_move);
//                     move_history.push(ai_result.chosen_move);
//                     // Display eval, depth, etc.
//                 }
//                 ai_receiver = None;
//             }
//         }
//     }
// }
// ```
