//! Difficulty level definitions.
//!
//! Each level controls: max search depth, evaluation complexity,
//! time budget, mistake probability, and personality traits.

use std::time::Duration;

/// Which evaluation components are active at a given difficulty level.
#[derive(Clone, Debug)]
pub struct EvalConfig {
    pub material: bool,
    pub piece_square_tables: bool,
    pub pawn_structure: bool,
    pub king_safety: bool,
    pub mobility: bool,
    pub center_control: bool,
    pub bishop_pair: bool,
    pub rook_on_open_file: bool,
    pub passed_pawns: bool,
    pub endgame_knowledge: bool,
}

/// Personality quirks that make weak AI feel human-like.
#[derive(Clone, Debug)]
pub struct PersonalityConfig {
    /// Probability [0.0, 1.0] of playing a suboptimal move instead of the best.
    pub blunder_rate: f32,
    /// When blundering, maximum centipawn loss the AI will accept.
    /// A value of 200 means the AI might play moves up to 2 pawns worse.
    pub max_blunder_cp: i32,
    /// Whether the AI "sees" back-rank mate threats.
    pub sees_back_rank: bool,
    /// Whether the AI understands basic forks/pins/skewers.
    pub sees_tactics_depth: u8,
    /// Noise added to evaluation (in centipawns). At low levels, this makes
    /// the AI misjudge positions slightly, like a human would.
    pub eval_noise_cp: i32,
    /// Whether the AI tries to trade when ahead (basic endgame principle).
    pub trades_when_ahead: bool,
    /// Whether to add a human-like delay before responding (even if search
    /// finishes instantly). Makes low-level AI feel less robotic.
    pub min_think_time: Duration,
}

#[derive(Clone, Debug)]
pub struct DifficultyLevel {
    pub level: u8,
    pub name: &'static str,
    pub description: &'static str,
    pub approx_elo: u16,
    pub max_depth: u8,
    /// Maximum search time before the engine must return a move.
    pub max_think_time: Duration,
    /// Quiescence search depth limit (0 = no quiescence search).
    pub quiescence_depth: u8,
    pub eval: EvalConfig,
    pub personality: PersonalityConfig,
    pub use_opening_book: bool,
    /// Transposition table size in entries. 0 = disabled.
    pub tt_size: usize,
}

impl DifficultyLevel {
    pub fn from_level(level: u8) -> Self {
        match level.clamp(1, 10) {
            1 => Self::level_1(),
            2 => Self::level_2(),
            3 => Self::level_3(),
            4 => Self::level_4(),
            5 => Self::level_5(),
            6 => Self::level_6(),
            7 => Self::level_7(),
            8 => Self::level_8(),
            9 => Self::level_9(),
            10 => Self::level_10(),
            _ => unreachable!(),
        }
    }

    pub fn all_levels() -> Vec<DifficultyLevel> {
        (1..=10).map(DifficultyLevel::from_level).collect()
    }

    // ---------------------------------------------------------------
    // Level 1: "Pawn Pusher" (~400 ELO)
    // A complete beginner. Sees only material, searches 1 ply,
    // blunders frequently. Like someone who just learned the rules.
    // ---------------------------------------------------------------
    fn level_1() -> Self {
        Self {
            level: 1,
            name: "Pawn Pusher",
            description: "Just learned how the pieces move",
            approx_elo: 400,
            max_depth: 1,
            max_think_time: Duration::from_millis(200),
            quiescence_depth: 0,
            eval: EvalConfig {
                material: true,
                piece_square_tables: false,
                pawn_structure: false,
                king_safety: false,
                mobility: false,
                center_control: false,
                bishop_pair: false,
                rook_on_open_file: false,
                passed_pawns: false,
                endgame_knowledge: false,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.35,
                max_blunder_cp: 400,
                sees_back_rank: false,
                sees_tactics_depth: 0,
                eval_noise_cp: 150,
                trades_when_ahead: false,
                min_think_time: Duration::from_millis(500),
            },
            use_opening_book: false,
            tt_size: 0,
        }
    }

    // ---------------------------------------------------------------
    // Level 2: "Coffee Shop" (~600 ELO)
    // Casual player. Knows piece values, has vague sense of center.
    // Still blunders pieces regularly.
    // ---------------------------------------------------------------
    fn level_2() -> Self {
        Self {
            level: 2,
            name: "Coffee Shop",
            description: "Plays for fun, sometimes forgets to guard pieces",
            approx_elo: 600,
            max_depth: 2,
            max_think_time: Duration::from_millis(300),
            quiescence_depth: 1,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: false,
                king_safety: false,
                mobility: false,
                center_control: true,
                bishop_pair: false,
                rook_on_open_file: false,
                passed_pawns: false,
                endgame_knowledge: false,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.25,
                max_blunder_cp: 300,
                sees_back_rank: false,
                sees_tactics_depth: 1,
                eval_noise_cp: 120,
                trades_when_ahead: false,
                min_think_time: Duration::from_millis(600),
            },
            use_opening_book: false,
            tt_size: 0,
        }
    }

    // ---------------------------------------------------------------
    // Level 3: "Park Bench" (~800 ELO)
    // Knows opening principles. Starting to think one move ahead.
    // ---------------------------------------------------------------
    fn level_3() -> Self {
        Self {
            level: 3,
            name: "Park Bench",
            description: "Knows the basics, developing pieces toward the center",
            approx_elo: 800,
            max_depth: 3,
            max_think_time: Duration::from_millis(500),
            quiescence_depth: 2,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: false,
                king_safety: true,
                mobility: false,
                center_control: true,
                bishop_pair: false,
                rook_on_open_file: false,
                passed_pawns: false,
                endgame_knowledge: false,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.18,
                max_blunder_cp: 250,
                sees_back_rank: false,
                sees_tactics_depth: 1,
                eval_noise_cp: 90,
                trades_when_ahead: false,
                min_think_time: Duration::from_millis(700),
            },
            use_opening_book: true,
            tt_size: 0,
        }
    }

    // ---------------------------------------------------------------
    // Level 4: "Casual Player" (~1000 ELO)
    // Solid understanding of fundamentals. Rarely hangs pieces but
    // misses multi-move tactics.
    // ---------------------------------------------------------------
    fn level_4() -> Self {
        Self {
            level: 4,
            name: "Casual Player",
            description: "Plays online regularly, knows basic tactics",
            approx_elo: 1000,
            max_depth: 3,
            max_think_time: Duration::from_millis(800),
            quiescence_depth: 3,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: false,
                rook_on_open_file: false,
                passed_pawns: false,
                endgame_knowledge: false,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.12,
                max_blunder_cp: 200,
                sees_back_rank: true,
                sees_tactics_depth: 2,
                eval_noise_cp: 60,
                trades_when_ahead: false,
                min_think_time: Duration::from_millis(800),
            },
            use_opening_book: true,
            tt_size: 1 << 16, // 64K entries
        }
    }

    // ---------------------------------------------------------------
    // Level 5: "Tournament Hopeful" (~1200 ELO)
    // Has played in a few tournaments. Understands pawn structure,
    // sees 2-move tactics.
    // ---------------------------------------------------------------
    fn level_5() -> Self {
        Self {
            level: 5,
            name: "Tournament Hopeful",
            description: "Joined a chess club, studying tactics daily",
            approx_elo: 1200,
            max_depth: 4,
            max_think_time: Duration::from_secs(1),
            quiescence_depth: 4,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: true,
                rook_on_open_file: true,
                passed_pawns: false,
                endgame_knowledge: false,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.07,
                max_blunder_cp: 150,
                sees_back_rank: true,
                sees_tactics_depth: 3,
                eval_noise_cp: 40,
                trades_when_ahead: true,
                min_think_time: Duration::from_secs(1),
            },
            use_opening_book: true,
            tt_size: 1 << 17, // 128K entries
        }
    }

    // ---------------------------------------------------------------
    // Level 6: "Rated Player" (~1400 ELO)
    // Understands strategy, has a real rating. Rarely blunders
    // but makes positional errors.
    // ---------------------------------------------------------------
    fn level_6() -> Self {
        Self {
            level: 6,
            name: "Rated Player",
            description: "Has a real rating, studies openings and endgames",
            approx_elo: 1400,
            max_depth: 5,
            max_think_time: Duration::from_millis(1500),
            quiescence_depth: 6,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: true,
                rook_on_open_file: true,
                passed_pawns: true,
                endgame_knowledge: false,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.04,
                max_blunder_cp: 100,
                sees_back_rank: true,
                sees_tactics_depth: 4,
                eval_noise_cp: 25,
                trades_when_ahead: true,
                min_think_time: Duration::from_secs(1),
            },
            use_opening_book: true,
            tt_size: 1 << 18, // 256K entries
        }
    }

    // ---------------------------------------------------------------
    // Level 7: "Club Veteran" (~1600 ELO)
    // Strong amateur. Deep search, nearly full evaluation.
    // ---------------------------------------------------------------
    fn level_7() -> Self {
        Self {
            level: 7,
            name: "Club Veteran",
            description: "Decades of experience, sharp tactical eye",
            approx_elo: 1600,
            max_depth: 6,
            max_think_time: Duration::from_secs(2),
            quiescence_depth: 8,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: true,
                rook_on_open_file: true,
                passed_pawns: true,
                endgame_knowledge: true,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.02,
                max_blunder_cp: 60,
                sees_back_rank: true,
                sees_tactics_depth: 5,
                eval_noise_cp: 15,
                trades_when_ahead: true,
                min_think_time: Duration::from_millis(1200),
            },
            use_opening_book: true,
            tt_size: 1 << 19, // 512K entries
        }
    }

    // ---------------------------------------------------------------
    // Level 8: "Club Champion" (~1800 ELO)
    // Full evaluation, deep search, no deliberate mistakes.
    // ---------------------------------------------------------------
    fn level_8() -> Self {
        Self {
            level: 8,
            name: "Club Champion",
            description: "Wins the club tournament, feared in rapid games",
            approx_elo: 1800,
            max_depth: 7,
            max_think_time: Duration::from_secs(2),
            quiescence_depth: 10,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: true,
                rook_on_open_file: true,
                passed_pawns: true,
                endgame_knowledge: true,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.0,
                max_blunder_cp: 0,
                sees_back_rank: true,
                sees_tactics_depth: 6,
                eval_noise_cp: 8,
                trades_when_ahead: true,
                min_think_time: Duration::from_millis(800),
            },
            use_opening_book: true,
            tt_size: 1 << 20, // 1M entries (~24 MB)
        }
    }

    // ---------------------------------------------------------------
    // Level 9: "Expert" (~1950 ELO)
    // Near-master strength. Full engine, deeper search.
    // ---------------------------------------------------------------
    fn level_9() -> Self {
        Self {
            level: 9,
            name: "Expert",
            description: "Tournament expert, plays like a machine",
            approx_elo: 1950,
            max_depth: 8,
            max_think_time: Duration::from_secs(2),
            quiescence_depth: 12,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: true,
                rook_on_open_file: true,
                passed_pawns: true,
                endgame_knowledge: true,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.0,
                max_blunder_cp: 0,
                sees_back_rank: true,
                sees_tactics_depth: 8,
                eval_noise_cp: 0,
                trades_when_ahead: true,
                min_think_time: Duration::from_millis(500),
            },
            use_opening_book: true,
            tt_size: 1 << 20,
        }
    }

    // ---------------------------------------------------------------
    // Level 10: "Grandmaster Wannabe" (~2100 ELO)
    // Maximum engine strength. Deepest search within time budget.
    // ---------------------------------------------------------------
    fn level_10() -> Self {
        Self {
            level: 10,
            name: "Grandmaster Wannabe",
            description: "The engine unleashed -- good luck",
            approx_elo: 2100,
            max_depth: 10,
            max_think_time: Duration::from_secs(2),
            quiescence_depth: 16,
            eval: EvalConfig {
                material: true,
                piece_square_tables: true,
                pawn_structure: true,
                king_safety: true,
                mobility: true,
                center_control: true,
                bishop_pair: true,
                rook_on_open_file: true,
                passed_pawns: true,
                endgame_knowledge: true,
            },
            personality: PersonalityConfig {
                blunder_rate: 0.0,
                max_blunder_cp: 0,
                sees_back_rank: true,
                sees_tactics_depth: 10,
                eval_noise_cp: 0,
                trades_when_ahead: true,
                min_think_time: Duration::from_millis(300),
            },
            use_opening_book: true,
            tt_size: 1 << 21, // 2M entries (~48 MB)
        }
    }
}
