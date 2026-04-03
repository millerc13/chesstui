use ratatui::style::Color;

/// Color theme for the entire TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    // Board colors
    pub light_square: Color,
    pub dark_square: Color,
    pub white_piece: Color,
    pub black_piece: Color,
    // Highlights
    pub selected_bg: Color,
    pub legal_move_bg: Color,
    pub last_move_light: Color,
    pub last_move_dark: Color,
    pub check_bg: Color,
    pub cursor_bg: Color,
    // UI chrome
    pub accent: Color,
    pub text_primary: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub border_focused: Color,
    pub border_dim: Color,
    // Mode indicators
    pub mode_normal: Color,
    pub mode_input: Color,
    pub mode_command: Color,
}

impl Theme {
    /// True-color (24-bit) theme — rich colors for modern terminals.
    pub fn default_truecolor() -> Self {
        Self {
            light_square: Color::Rgb(240, 217, 181),
            dark_square: Color::Rgb(181, 136, 99),
            white_piece: Color::Rgb(255, 255, 255),
            black_piece: Color::Rgb(30, 30, 30),
            selected_bg: Color::Rgb(130, 170, 100),
            legal_move_bg: Color::Rgb(100, 140, 80),
            last_move_light: Color::Rgb(205, 210, 106),
            last_move_dark: Color::Rgb(170, 162, 58),
            check_bg: Color::Rgb(200, 50, 50),
            cursor_bg: Color::Rgb(80, 130, 200),
            accent: Color::Rgb(100, 180, 220),
            text_primary: Color::Rgb(220, 220, 220),
            text_dim: Color::Rgb(120, 120, 120),
            text_bright: Color::Rgb(255, 255, 255),
            border_focused: Color::Rgb(100, 180, 220),
            border_dim: Color::Rgb(80, 80, 80),
            mode_normal: Color::Rgb(100, 180, 220),
            mode_input: Color::Rgb(180, 220, 100),
            mode_command: Color::Rgb(220, 160, 100),
        }
    }

    /// 256-color fallback for terminals without truecolor support.
    pub fn default_256() -> Self {
        Self {
            light_square: Color::Indexed(223),   // light tan
            dark_square: Color::Indexed(137),     // brown
            white_piece: Color::Indexed(231),     // white
            black_piece: Color::Indexed(232),     // near-black
            selected_bg: Color::Indexed(107),     // olive green
            legal_move_bg: Color::Indexed(65),    // dim green
            last_move_light: Color::Indexed(186), // yellow-green
            last_move_dark: Color::Indexed(142),  // darker yellow-green
            check_bg: Color::Indexed(160),        // red
            cursor_bg: Color::Indexed(67),        // steel blue
            accent: Color::Indexed(74),           // teal
            text_primary: Color::Indexed(252),    // light grey
            text_dim: Color::Indexed(243),        // mid grey
            text_bright: Color::Indexed(231),     // white
            border_focused: Color::Indexed(74),
            border_dim: Color::Indexed(240),
            mode_normal: Color::Indexed(74),
            mode_input: Color::Indexed(149),
            mode_command: Color::Indexed(179),
        }
    }

    /// Auto-detect: use truecolor if `$COLORTERM` is set to `truecolor` or `24bit`.
    pub fn detect() -> Self {
        match std::env::var("COLORTERM").as_deref() {
            Ok("truecolor") | Ok("24bit") => Self::default_truecolor(),
            _ => Self::default_256(),
        }
    }

    /// Return the base background color for a square given its file (0-7) and rank (0-7).
    pub fn square_bg(&self, file: u8, rank: u8) -> Color {
        if (file + rank) % 2 == 0 {
            self.dark_square
        } else {
            self.light_square
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::detect()
    }
}
