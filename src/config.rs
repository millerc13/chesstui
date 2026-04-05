use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

// ── Piece style ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceStyle {
    Generated3D,
    PixelColor,
    PixelDuo,
    PixelMono,
    Classic,
}

impl Default for PieceStyle {
    fn default() -> Self { Self::Generated3D }
}

impl PieceStyle {
    pub const ALL: &'static [PieceStyle] = &[
        PieceStyle::Generated3D,
        PieceStyle::PixelColor,
        PieceStyle::PixelDuo,
        PieceStyle::PixelMono,
        PieceStyle::Classic,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Generated3D => "3D",
            Self::PixelColor => "Pixel Color",
            Self::PixelDuo => "Pixel Duo",
            Self::PixelMono => "Pixel Mono",
            Self::Classic => "Classic",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Generated3D => "3D rendered pieces with Phong shading",
            Self::PixelColor => "16\u{00d7}16 pixel art, full color",
            Self::PixelDuo => "16\u{00d7}16 pixel art, two-tone",
            Self::PixelMono => "16\u{00d7}16 pixel art, monochrome",
            Self::Classic => "512\u{00d7}512 high-res classic pieces",
        }
    }

    pub fn from_name(s: &str) -> Option<PieceStyle> {
        Self::ALL.iter().find(|p| p.name() == s).copied()
    }
}

impl fmt::Display for PieceStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

// ── Config ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server_url: String,
    pub color_scheme: Option<String>,
    pub piece_style: Option<String>,
    #[serde(default = "default_true")]
    pub show_move_hints: bool,
}

fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "ws://152.117.85.110:7600/ws".to_string(),
            color_scheme: None,
            piece_style: None,
            show_move_hints: true,
        }
    }
}

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    home.join(".chesstui")
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(toml_str) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, toml_str);
        }
    }
}
