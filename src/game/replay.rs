use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::app::GameMode;
use crate::game::notation::to_algebraic;
use crate::game::state::{GameResult, GameState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGame {
    pub id: String,
    pub date: String,
    pub result: String,
    pub result_detail: String,
    pub mode: String,
    pub moves: Vec<String>,
    pub move_count: usize,
}

/// Returns the path to ~/.chesstui/replays/, creating it if necessary.
pub fn replays_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let dir = PathBuf::from(home).join(".chesstui").join("replays");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create replays directory");
    }
    dir
}

/// Save the completed game to a JSON file. Returns the generated ID on success.
pub fn save_game(game: &GameState, mode: &GameMode, result: &GameResult) -> Option<String> {
    let now = chrono::Local::now();
    let id = now.format("%Y%m%d_%H%M%S").to_string();
    let date = now.format("%Y-%m-%dT%H:%M:%S%:z").to_string();

    let (result_str, result_detail) = match result {
        GameResult::Checkmate(color) => {
            let winner = match color {
                cozy_chess::Color::White => "Black wins",
                cozy_chess::Color::Black => "White wins",
            };
            (winner.to_string(), "Checkmate".to_string())
        }
        GameResult::Stalemate => ("Draw".to_string(), "Stalemate".to_string()),
        GameResult::DrawByRepetition => ("Draw".to_string(), "Repetition".to_string()),
        GameResult::DrawByFiftyMove => ("Draw".to_string(), "Fifty-move rule".to_string()),
        GameResult::DrawByInsufficientMaterial => {
            ("Draw".to_string(), "Insufficient material".to_string())
        }
        GameResult::DrawByAgreement => ("Draw".to_string(), "Agreement".to_string()),
        GameResult::Resignation(color) => {
            let winner = match color {
                cozy_chess::Color::White => "Black wins",
                cozy_chess::Color::Black => "White wins",
            };
            (winner.to_string(), "Resignation".to_string())
        }
    };

    let mode_str = match mode {
        GameMode::VsAi(_) => "vs AI".to_string(),
        GameMode::Local => "Local".to_string(),
        GameMode::Online { opponent_name, .. } => format!("Online vs {}", opponent_name),
    };

    let moves: Vec<String> = game
        .move_history()
        .iter()
        .map(|record| to_algebraic(&record.previous_board, &record.mv))
        .collect();

    let saved = SavedGame {
        id: id.clone(),
        date,
        result: result_str,
        result_detail,
        mode: mode_str,
        move_count: moves.len(),
        moves,
    };

    let json = serde_json::to_string_pretty(&saved).ok()?;
    let path = replays_dir().join(format!("{}.json", id));
    fs::write(path, json).ok()?;

    Some(id)
}

/// Load all saved replays, sorted by date descending (most recent first).
pub fn load_replays() -> Vec<SavedGame> {
    let dir = replays_dir();
    let mut replays: Vec<SavedGame> = fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "json")
        })
        .filter_map(|entry| {
            let contents = fs::read_to_string(entry.path()).ok()?;
            serde_json::from_str::<SavedGame>(&contents).ok()
        })
        .collect();

    replays.sort_by(|a, b| b.date.cmp(&a.date));
    replays
}

/// Delete a saved replay by its ID. Returns true if the file was removed.
pub fn delete_replay(id: &str) -> bool {
    let path = replays_dir().join(format!("{}.json", id));
    fs::remove_file(path).is_ok()
}
