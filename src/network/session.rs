use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub token: String,
    pub email: String,
    pub display_name: Option<String>,
    pub server_url: String,
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn session_path() -> PathBuf {
    let mut path = home_dir();
    path.push(".chesstui");
    path.push("session.json");
    path
}

pub fn load_session() -> Option<StoredSession> {
    let path = session_path();
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_session(session: &StoredSession) {
    let path = session_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(path, json);
    }
}

pub fn clear_session() {
    let path = session_path();
    let _ = std::fs::remove_file(path);
}
