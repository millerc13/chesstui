use crate::game::notation;
use crate::game::state::{GameResult, GameState, GameStatus};
use crate::protocol::ServerMessage;
use cozy_chess::Color;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct GameRoom {
    pub id: String,
    pub state: GameState,
    pub white_tx: mpsc::Sender<ServerMessage>,
    pub black_tx: mpsc::Sender<ServerMessage>,
    pub white_id: Uuid,
    pub black_id: Uuid,
    pub white_name: String,
    pub black_name: String,
    pub moves_san: Vec<String>,
    /// Set when the game ends outside of normal play (resignation, draw agreement)
    pub external_result: Option<(String, String)>,
    pub draw_offered_by: Option<Color>,
}

impl GameRoom {
    /// Try to apply a move from the given player. Returns Ok with the canonical SAN if valid.
    pub fn try_move(&mut self, user_id: &Uuid, san: &str) -> Result<String, String> {
        if self.is_finished() {
            return Err("Game is already finished".to_string());
        }

        // Check it's this player's turn
        let expected_id = match self.state.side_to_move() {
            Color::White => &self.white_id,
            Color::Black => &self.black_id,
        };
        if user_id != expected_id {
            return Err("Not your turn".to_string());
        }

        // Parse SAN to a Move
        let mv = notation::parse_san(self.state.board(), san)
            .ok_or_else(|| format!("Invalid move: {}", san))?;

        // Generate canonical SAN before applying (needs pre-move board)
        let canonical_san = notation::to_algebraic(self.state.board(), &mv);

        // Apply the move
        self.state.try_make_move(mv)?;

        // Clear any pending draw offer when a move is made
        self.draw_offered_by = None;

        // Store the move
        self.moves_san.push(canonical_san.clone());

        Ok(canonical_san)
    }

    pub fn is_finished(&self) -> bool {
        self.external_result.is_some()
            || matches!(self.state.status(), GameStatus::Finished(_))
    }

    pub fn result_strings(&self) -> Option<(String, String)> {
        if let Some(ref ext) = self.external_result {
            return Some(ext.clone());
        }
        match self.state.status() {
            GameStatus::Finished(ref result) => {
                let (res, detail) = match result {
                    GameResult::Checkmate(color) => {
                        let winner = match color {
                            Color::White => "White wins",
                            Color::Black => "Black wins",
                        };
                        (winner.to_string(), "Checkmate".to_string())
                    }
                    GameResult::Stalemate => ("Draw".to_string(), "Stalemate".to_string()),
                    GameResult::DrawByRepetition => {
                        ("Draw".to_string(), "Threefold repetition".to_string())
                    }
                    GameResult::DrawByFiftyMove => {
                        ("Draw".to_string(), "Fifty-move rule".to_string())
                    }
                    GameResult::DrawByInsufficientMaterial => {
                        ("Draw".to_string(), "Insufficient material".to_string())
                    }
                    GameResult::DrawByAgreement => {
                        ("Draw".to_string(), "By agreement".to_string())
                    }
                    GameResult::Resignation(color) => {
                        let winner = match color {
                            Color::White => "Black wins",
                            Color::Black => "White wins",
                        };
                        (winner.to_string(), "Resignation".to_string())
                    }
                };
                Some((res, detail))
            }
            GameStatus::InProgress => None,
        }
    }

    /// Mark the game as a resignation by the given player.
    pub fn set_resignation(&mut self, resigning_color: Color) {
        let winner = match resigning_color {
            Color::White => "Black wins",
            Color::Black => "White wins",
        };
        self.external_result = Some((winner.to_string(), "Resignation".to_string()));
    }

    /// Mark the game as drawn by agreement.
    pub fn set_draw_by_agreement(&mut self) {
        self.external_result = Some(("Draw".to_string(), "By agreement".to_string()));
    }

    /// Get the color for a given user, if they are in this game.
    pub fn color_of(&self, user_id: &Uuid) -> Option<Color> {
        if *user_id == self.white_id {
            Some(Color::White)
        } else if *user_id == self.black_id {
            Some(Color::Black)
        } else {
            None
        }
    }
}
