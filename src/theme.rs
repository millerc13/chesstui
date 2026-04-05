use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Green,
    Amber,
    Blue,
    Magenta,
    Cyan,
    Mono,
}

impl ColorScheme {
    pub const ALL: &'static [ColorScheme] = &[
        ColorScheme::Amber,
        ColorScheme::Green,
        ColorScheme::Blue,
        ColorScheme::Magenta,
        ColorScheme::Cyan,
        ColorScheme::Mono,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ColorScheme::Green => "Phosphor Green",
            ColorScheme::Amber => "Orange",
            ColorScheme::Blue => "Ice Blue",
            ColorScheme::Magenta => "Neon Magenta",
            ColorScheme::Cyan => "Cyan Terminal",
            ColorScheme::Mono => "Monochrome",
        }
    }
}

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
    pub legal_dot: Color,
    // UI chrome
    pub accent: Color,
    pub text_primary: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub border_focused: Color,
    pub border_dim: Color,
    // Mode indicators
    pub mode_play: Color,
    pub mode_command: Color,
    // Table cursor (k9s-style selection)
    pub table_cursor_fg: Color,
    pub table_cursor_bg: Color,
    // Multi-color palette (LunarVim/AstroNvim style)
    pub accent_secondary: Color,
    pub accent_tertiary: Color,
    pub logo_color: Color,
    pub icon_color: Color,
    pub shortcut_color: Color,
}

impl Theme {
    /// Build theme from a color scheme. Each scheme defines 4 tones:
    /// bright (pieces/highlights), primary (text/accents), dim (grid/labels), dark (backgrounds).
    pub fn from_scheme(scheme: ColorScheme) -> Self {
        let (bright, primary, dim, _dark) = match scheme {
            ColorScheme::Green   => (Color::Indexed(82),  Color::Indexed(34),  Color::Indexed(28),  Color::Indexed(22)),
            ColorScheme::Amber   => (Color::Indexed(220), Color::Indexed(214), Color::Indexed(172), Color::Indexed(130)),
            ColorScheme::Blue    => (Color::Indexed(75),  Color::Indexed(33),  Color::Indexed(26),  Color::Indexed(17)),
            ColorScheme::Magenta => (Color::Indexed(213), Color::Indexed(170), Color::Indexed(133), Color::Indexed(53)),
            ColorScheme::Cyan    => (Color::Indexed(87),  Color::Indexed(37),  Color::Indexed(30),  Color::Indexed(23)),
            ColorScheme::Mono    => (Color::White,        Color::Indexed(250), Color::Indexed(243), Color::Indexed(237)),
        };

        // Square colors: enough contrast between them AND with piece colors
        // Light squares are muted (not near-white) so white pieces stand out
        // Dark squares are mid-tone so black pieces are visible
        let dark_sq = match scheme {
            ColorScheme::Green   => Color::Rgb(100, 133, 68),
            ColorScheme::Amber   => Color::Rgb(140, 100, 45),
            ColorScheme::Blue    => Color::Rgb(60, 85, 130),
            ColorScheme::Magenta => Color::Rgb(120, 55, 100),
            ColorScheme::Cyan    => Color::Rgb(45, 110, 110),
            ColorScheme::Mono    => Color::Indexed(240),
        };

        let light_sq = match scheme {
            ColorScheme::Green   => Color::Rgb(190, 200, 160),
            ColorScheme::Amber   => Color::Rgb(200, 180, 140),
            ColorScheme::Blue    => Color::Rgb(165, 180, 200),
            ColorScheme::Magenta => Color::Rgb(195, 165, 185),
            ColorScheme::Cyan    => Color::Rgb(165, 200, 195),
            ColorScheme::Mono    => Color::Indexed(248),
        };

        // Piece colors: bright for "white" side, dark for "black" side
        // Tinted to match the scheme so pieces feel cohesive
        let (white_pc, black_pc) = match scheme {
            ColorScheme::Green   => (Color::Rgb(255, 255, 240), Color::Rgb(50, 50, 40)),
            ColorScheme::Amber   => (Color::Rgb(255, 240, 200), Color::Rgb(60, 40, 20)),
            ColorScheme::Blue    => (Color::Rgb(230, 240, 255), Color::Rgb(30, 40, 60)),
            ColorScheme::Magenta => (Color::Rgb(255, 230, 250), Color::Rgb(55, 30, 50)),
            ColorScheme::Cyan    => (Color::Rgb(230, 255, 255), Color::Rgb(30, 50, 50)),
            ColorScheme::Mono    => (Color::Rgb(240, 240, 240), Color::Rgb(50, 50, 50)),
        };

        // Highlights tinted to match the scheme
        let (sel_bg, legal_bg, lm_light, lm_dark, cursor) = match scheme {
            ColorScheme::Green   => (
                Color::Rgb(130, 170, 100), Color::Rgb(170, 180, 140),
                Color::Rgb(245, 246, 130), Color::Rgb(186, 202, 68),
                Color::Rgb(200, 200, 100),
            ),
            ColorScheme::Amber   => (
                Color::Rgb(180, 140, 60), Color::Rgb(200, 180, 120),
                Color::Rgb(240, 220, 100), Color::Rgb(200, 170, 50),
                Color::Rgb(220, 200, 80),
            ),
            ColorScheme::Blue    => (
                Color::Rgb(100, 130, 180), Color::Rgb(150, 170, 200),
                Color::Rgb(140, 180, 240), Color::Rgb(90, 130, 200),
                Color::Rgb(130, 170, 220),
            ),
            ColorScheme::Magenta => (
                Color::Rgb(170, 100, 150), Color::Rgb(190, 150, 180),
                Color::Rgb(230, 150, 200), Color::Rgb(180, 100, 160),
                Color::Rgb(210, 150, 180),
            ),
            ColorScheme::Cyan    => (
                Color::Rgb(80, 170, 160), Color::Rgb(140, 190, 190),
                Color::Rgb(130, 230, 220), Color::Rgb(70, 180, 170),
                Color::Rgb(120, 200, 180),
            ),
            ColorScheme::Mono    => (
                Color::Indexed(247), Color::Indexed(249),
                Color::Indexed(250), Color::Indexed(246),
                Color::Indexed(248),
            ),
        };

        Self {
            light_square: light_sq,
            dark_square: dark_sq,
            white_piece: white_pc,
            black_piece: black_pc,
            selected_bg: sel_bg,
            legal_move_bg: legal_bg,
            last_move_light: lm_light,
            last_move_dark: lm_dark,
            check_bg: Color::Rgb(200, 80, 80),
            cursor_bg: cursor,
            legal_dot: Color::Indexed(243),
            accent: primary,
            text_primary: primary,
            text_dim: dim,
            text_bright: bright,
            border_focused: primary,
            border_dim: dim,
            mode_play: primary,
            mode_command: Color::Yellow,
            table_cursor_fg: match scheme {
                ColorScheme::Green   => Color::Rgb(20, 20, 15),
                ColorScheme::Amber   => Color::Rgb(30, 20, 10),
                ColorScheme::Blue    => Color::Rgb(15, 20, 30),
                ColorScheme::Magenta => Color::Rgb(30, 15, 25),
                ColorScheme::Cyan    => Color::Rgb(15, 30, 30),
                ColorScheme::Mono    => Color::Rgb(20, 20, 20),
            },
            table_cursor_bg: match scheme {
                ColorScheme::Green   => Color::Rgb(100, 160, 70),
                ColorScheme::Amber   => Color::Rgb(180, 140, 50),
                ColorScheme::Blue    => Color::Rgb(80, 130, 200),
                ColorScheme::Magenta => Color::Rgb(170, 90, 150),
                ColorScheme::Cyan    => Color::Rgb(60, 170, 160),
                ColorScheme::Mono    => Color::Indexed(252),
            },
            // Multi-color palette — complementary colors from AstroNvim palette
            accent_secondary: match scheme {
                ColorScheme::Green   => Color::Rgb(97, 175, 239),   // blue
                ColorScheme::Amber   => Color::Rgb(97, 175, 239),   // blue
                ColorScheme::Blue    => Color::Rgb(152, 190, 101),  // green
                ColorScheme::Magenta => Color::Rgb(97, 175, 239),   // blue
                ColorScheme::Cyan    => Color::Rgb(152, 190, 101),  // green
                ColorScheme::Mono    => Color::Rgb(171, 178, 191),  // light gray
            },
            accent_tertiary: match scheme {
                ColorScheme::Green   => Color::Rgb(224, 108, 117),  // coral
                ColorScheme::Amber   => Color::Rgb(198, 120, 221),  // purple
                ColorScheme::Blue    => Color::Rgb(229, 192, 123),  // yellow
                ColorScheme::Magenta => Color::Rgb(229, 192, 123),  // yellow
                ColorScheme::Cyan    => Color::Rgb(224, 108, 117),  // coral
                ColorScheme::Mono    => Color::Rgb(92, 99, 112),    // dark gray
            },
            logo_color: match scheme {
                ColorScheme::Green   => Color::Rgb(152, 190, 101),  // bright green
                ColorScheme::Amber   => Color::Rgb(235, 174, 52),   // gold
                ColorScheme::Blue    => Color::Rgb(97, 175, 239),   // bright blue
                ColorScheme::Magenta => Color::Rgb(198, 120, 221),  // purple
                ColorScheme::Cyan    => Color::Rgb(86, 182, 194),   // teal
                ColorScheme::Mono    => Color::Rgb(201, 201, 201),  // white
            },
            icon_color: match scheme {
                ColorScheme::Green   => Color::Rgb(86, 182, 194),   // teal
                ColorScheme::Amber   => Color::Rgb(86, 182, 194),   // teal
                ColorScheme::Blue    => Color::Rgb(152, 190, 101),  // green
                ColorScheme::Magenta => Color::Rgb(86, 182, 194),   // teal
                ColorScheme::Cyan    => Color::Rgb(152, 190, 101),  // green
                ColorScheme::Mono    => Color::Rgb(171, 178, 191),  // light gray
            },
            shortcut_color: match scheme {
                ColorScheme::Green   => Color::Rgb(224, 108, 117),  // coral
                ColorScheme::Amber   => Color::Rgb(224, 108, 117),  // coral
                ColorScheme::Blue    => Color::Rgb(224, 108, 117),  // coral
                ColorScheme::Magenta => Color::Rgb(229, 192, 123),  // yellow
                ColorScheme::Cyan    => Color::Rgb(224, 108, 117),  // coral
                ColorScheme::Mono    => Color::Rgb(201, 201, 201),  // white
            },
        }
    }

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
        Self::from_scheme(ColorScheme::Amber)
    }
}
