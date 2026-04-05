use cozy_chess::{Board, Color, Move, Piece, Square};

use crate::config::PieceStyle;
use crate::game::replay::{self, SavedGame};
use crate::game::state::{GameState, GameStatus, GameResult};
use crate::theme::{ColorScheme, Theme};

#[derive(Default, Clone)]
pub struct BoardLayout {
    pub board_x: u16,
    pub board_y: u16,
    pub sq_w: f32,
    pub sq_h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Play,    // Unified: typing + visual nav both active
    Command, // :command mode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Launch,
    ColorPicker,
    MainMenu,
    InGame,
    PostGame,
    ReplayViewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTab {
    Play,
    Replays,
    Multiplayer,
    Settings,
}

impl MenuTab {
    pub const ALL: &'static [MenuTab] = &[
        MenuTab::Play,
        MenuTab::Replays,
        MenuTab::Multiplayer,
        MenuTab::Settings,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            MenuTab::Play => "PLAY",
            MenuTab::Replays => "REPLAYS",
            MenuTab::Multiplayer => "ONLINE",
            MenuTab::Settings => "⚙",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FriendInfo {
    pub name: String,
    pub elo: i32,
    pub online: bool,
    pub activity: String, // "In Game", "In Menu", "Idle", "Offline"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMenuItem {
    VsComputer,
    LocalGame,
    Puzzles,
    AiCoach,
    AnalysisBoard,
    Chess960,
}

impl PlayMenuItem {
    pub const ALL: &'static [PlayMenuItem] = &[
        PlayMenuItem::VsComputer,
        PlayMenuItem::LocalGame,
        PlayMenuItem::Puzzles,
        PlayMenuItem::AiCoach,
        PlayMenuItem::AnalysisBoard,
        PlayMenuItem::Chess960,
    ];

    pub fn is_available(&self) -> bool {
        matches!(self, PlayMenuItem::VsComputer | PlayMenuItem::LocalGame)
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PlayMenuItem::VsComputer => "♚",
            PlayMenuItem::LocalGame => "♟",
            PlayMenuItem::Puzzles => "✦",
            PlayMenuItem::AiCoach => "⚡",
            PlayMenuItem::AnalysisBoard => "⊞",
            PlayMenuItem::Chess960 => "♔",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            PlayMenuItem::VsComputer => "VS COMPUTER",
            PlayMenuItem::LocalGame => "LOCAL GAME",
            PlayMenuItem::Puzzles => "PUZZLES",
            PlayMenuItem::AiCoach => "AI COACH",
            PlayMenuItem::AnalysisBoard => "ANALYSIS BOARD",
            PlayMenuItem::Chess960 => "CHESS960",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            PlayMenuItem::VsComputer => "Battle the AI",
            PlayMenuItem::LocalGame => "2 players, 1 terminal",
            PlayMenuItem::Puzzles => "Sharpen your tactics",
            PlayMenuItem::AiCoach => "Learn from your mistakes",
            PlayMenuItem::AnalysisBoard => "Explore lines & eval",
            PlayMenuItem::Chess960 => "Fischer Random chess",
        }
    }

    pub fn tag(&self) -> Option<&'static str> {
        if self.is_available() { None } else { Some("Soon") }
    }
}

pub struct ReplayViewerState {
    pub game: SavedGame,
    pub current_move: usize,
    pub board: Board,
    pub boards: Vec<Board>,
}

impl ReplayViewerState {
    pub fn from_saved(game: SavedGame) -> Self {
        let mut board = Board::default();
        let mut boards = vec![board.clone()];
        for san in &game.moves {
            if let Some(mv) = crate::game::notation::parse_san(&board, san) {
                board.play_unchecked(mv);
                boards.push(board.clone());
            } else {
                break;
            }
        }
        Self {
            game,
            current_move: 0,
            board: Board::default(),
            boards,
        }
    }

    pub fn current_board(&self) -> &Board {
        &self.boards[self.current_move]
    }

    pub fn total_moves(&self) -> usize {
        self.boards.len() - 1
    }

    pub fn go_next(&mut self) {
        if self.current_move < self.total_moves() {
            self.current_move += 1;
        }
    }

    pub fn go_prev(&mut self) {
        if self.current_move > 0 {
            self.current_move -= 1;
        }
    }

    pub fn go_start(&mut self) {
        self.current_move = 0;
    }

    pub fn go_end(&mut self) {
        self.current_move = self.total_moves();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameMode {
    Local,
    VsAi(Color), // The color the AI plays
    Online {
        game_id: String,
        my_color: Color,
        opponent_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiplayerState {
    LoggedOut,
    Connecting,
    EnteringEmail,
    WaitingForOtp,
    EnteringOtp,
    EnteringDisplayName,
    EnteringPassword,
    EnteringLoginPassword,
    LoggedIn { display_name: String, elo: i32 },
    Searching,
    InGame,
}

pub struct App {
    pub screen: Screen,
    pub mode: InputMode,
    pub should_quit: bool,
    pub theme: Theme,
    pub menu_selection: usize,
    pub game: GameState,
    pub cursor_file: u8,
    pub cursor_rank: u8,
    pub selected_square: Option<Square>,
    pub legal_moves_for_selected: Vec<Move>,
    pub last_move: Option<(Square, Square)>,
    pub board_flipped: bool,
    pub input_buffer: String,
    pub command_buffer: String,
    pub status_message: String,
    pub move_list_scroll: usize,
    pub show_help: bool,
    pub show_move_hints: bool,
    pub pending_promotion: Option<Vec<Move>>,
    pub promotion_choice: usize,
    pub color_scheme_index: usize,
    pub game_mode: GameMode,
    // Tabbed menu
    pub active_tab: MenuTab,
    pub play_selection: usize,
    pub replay_list: Vec<SavedGame>,
    pub replay_selection: usize,
    // Replay viewer
    pub replay_viewer: Option<ReplayViewerState>,
    // Multiplayer
    pub network: Option<crate::network::NetworkClient>,
    pub multiplayer_state: MultiplayerState,
    pub multiplayer_selection: usize,
    pub login_input: String,
    pub otp_input: String,
    pub display_name_input: String,
    pub server_url: String,
    pub password_input: String,
    pub has_account: bool,
    pub tick: u64,
    // Image-based board rendering
    pub use_kitty: bool,
    pub board_picker: Option<ratatui_image::picker::Picker>,
    pub board_image_dirty: bool,
    pub cached_board_sq_px: u32,
    pub kitty_cache: Option<crate::ui::kitty_transmit::KittyBoardCache>,
    pub kitty_image_hash: u64,
    pub kitty_image_id: u32,
    pub cached_piece_cache: Option<crate::ui::board_image::PieceCache>,
    // Piece style
    pub piece_style: PieceStyle,
    pub settings_style_index: usize,
    // Cached preview for settings
    pub cached_preview_protocol: Option<ratatui_image::protocol::StatefulProtocol>,
    pub cached_preview_style: Option<PieceStyle>,
    pub cached_preview_sq_px: u32,
    // Debug panel
    pub show_debug: bool,
    // Board layout for mouse hit-testing
    pub board_layout: BoardLayout,
    // AI move delay (None = no pending AI move, Some(instant) = AI will move after this time)
    pub ai_move_at: Option<std::time::Instant>,
    // AI difficulty
    pub ai_difficulty: usize,
    // Friends
    pub friends_list: Vec<FriendInfo>,
    // Help modal
    pub help_search: String,
    pub help_scroll: usize,
    // Game timing
    pub game_start_time: Option<std::time::Instant>,
    // Launch screen
    pub launch_selection: usize, // 0=SignUp, 1=LogIn, 2=Guest
    // Post-game screen
    pub postgame_selection: usize, // 0=Rematch, 1=Review, 2=Copy PGN, 3=Menu
}

impl App {
    pub fn new() -> Self {
        let config = crate::config::Config::load();
        let piece_style = config.piece_style
            .as_deref()
            .and_then(PieceStyle::from_name)
            .unwrap_or_default();
        let settings_style_index = PieceStyle::ALL.iter().position(|&s| s == piece_style).unwrap_or(0);

        let initial_screen = if config.color_scheme.is_some() {
            Screen::Launch
        } else {
            Screen::ColorPicker
        };

        // Restore saved color scheme
        let mut theme = Theme::default();
        let mut color_scheme_index = 0usize;
        if let Some(ref scheme_name) = config.color_scheme {
            for (i, cs) in ColorScheme::ALL.iter().enumerate() {
                if cs.name() == scheme_name {
                    color_scheme_index = i;
                    theme = Theme::from_scheme(*cs);
                    break;
                }
            }
        }

        Self {
            screen: initial_screen,
            mode: InputMode::Play,
            should_quit: false,
            theme,
            menu_selection: 0,
            game: GameState::new(),
            cursor_file: 4,
            cursor_rank: 4,
            selected_square: None,
            legal_moves_for_selected: Vec::new(),
            last_move: None,
            board_flipped: false,
            input_buffer: String::new(),
            command_buffer: String::new(),
            status_message: String::new(),
            move_list_scroll: 0,
            show_help: false,
            show_move_hints: config.show_move_hints,
            pending_promotion: None,
            promotion_choice: 0,
            color_scheme_index,
            game_mode: GameMode::Local,
            active_tab: MenuTab::Play,
            play_selection: 0,
            replay_list: Vec::new(),
            replay_selection: 0,
            replay_viewer: None,
            network: None,
            multiplayer_state: MultiplayerState::LoggedOut,
            multiplayer_selection: 0,
            login_input: String::new(),
            otp_input: String::new(),
            display_name_input: String::new(),
            server_url: config.server_url,
            password_input: String::new(),
            has_account: false,
            tick: 0,
            use_kitty: true,
            board_picker: ratatui_image::picker::Picker::from_query_stdio().ok(),
            board_image_dirty: true,
            cached_board_sq_px: 0,
            kitty_cache: None,
            kitty_image_hash: 0,
            kitty_image_id: 42,
            cached_piece_cache: None,
            piece_style,
            settings_style_index,
            cached_preview_protocol: None,
            cached_preview_style: None,
            cached_preview_sq_px: 0,
            show_debug: false,
            board_layout: BoardLayout::default(),
            ai_move_at: None,
            ai_difficulty: 5,
            friends_list: Vec::new(),
            help_search: String::new(),
            help_scroll: 0,
            game_start_time: None,
            launch_selection: 0,
            postgame_selection: 0,
        }
    }

    pub fn load_replays(&mut self) {
        self.replay_list = replay::load_replays();
        if self.replay_selection >= self.replay_list.len() {
            self.replay_selection = self.replay_list.len().saturating_sub(1);
        }
    }

    pub fn open_replay(&mut self, index: usize) {
        if index < self.replay_list.len() {
            let game = self.replay_list[index].clone();
            self.replay_viewer = Some(ReplayViewerState::from_saved(game));
            self.screen = Screen::ReplayViewer;
        }
    }

    pub fn delete_selected_replay(&mut self) {
        if self.replay_selection < self.replay_list.len() {
            let id = self.replay_list[self.replay_selection].id.clone();
            replay::delete_replay(&id);
            self.load_replays();
        }
    }

    pub fn apply_color_scheme(&mut self) {
        let scheme = ColorScheme::ALL[self.color_scheme_index];
        self.theme = Theme::from_scheme(scheme);
    }

    pub fn apply_piece_style(&mut self) {
        self.piece_style = PieceStyle::ALL[self.settings_style_index];
        self.board_image_dirty = true;
        self.kitty_cache = None;
        self.kitty_image_hash = 0;
        self.cached_piece_cache = None;
        let mut config = crate::config::Config::load();
        config.piece_style = Some(self.piece_style.name().to_string());
        config.save();
    }

    pub fn start_new_game(&mut self) {
        self.start_game_with_mode(GameMode::Local);
    }

    pub fn start_ai_game(&mut self) {
        self.start_game_with_mode(GameMode::VsAi(Color::Black));
    }

    fn start_game_with_mode(&mut self, mode: GameMode) {
        self.game = GameState::new();
        self.game_mode = mode;
        self.screen = Screen::InGame;
        self.game_start_time = Some(std::time::Instant::now());
        self.mode = InputMode::Play;
        self.cursor_file = 4;
        self.cursor_rank = 4;
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.last_move = None;
        self.board_flipped = false;
        self.input_buffer.clear();
        self.command_buffer.clear();
        self.status_message.clear();
        self.move_list_scroll = 0;
        self.show_help = false;
        self.pending_promotion = None;
        self.promotion_choice = 0;
        self.board_image_dirty = true;
    }

    /// Schedule an AI response after a short delay so the player sees their move land first.
    pub fn try_ai_move(&mut self) {
        if let GameMode::VsAi(ai_color) = self.game_mode {
            if self.game.side_to_move() == ai_color {
                if let GameStatus::InProgress = self.game.status() {
                    if self.ai_move_at.is_none() {
                        self.ai_move_at = Some(std::time::Instant::now() + std::time::Duration::from_millis(300));
                    }
                }
            }
        }
    }

    /// Called from the main loop to execute the AI move when the delay has elapsed.
    pub fn poll_ai_move(&mut self) -> bool {
        if let Some(deadline) = self.ai_move_at {
            if std::time::Instant::now() >= deadline {
                self.ai_move_at = None;
                if let GameMode::VsAi(ai_color) = self.game_mode {
                    if self.game.side_to_move() == ai_color {
                        if let GameStatus::InProgress = self.game.status() {
                            if let Some(mv) = crate::ai::choose_move(self.game.board()) {
                                self.make_move(mv);
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub fn cursor_square(&self) -> Square {
        let (file, rank) = if self.board_flipped {
            (7 - self.cursor_file, 7 - self.cursor_rank)
        } else {
            (self.cursor_file, self.cursor_rank)
        };
        Square::new(
            cozy_chess::File::index(file as usize),
            cozy_chess::Rank::index(rank as usize),
        )
    }

    pub fn move_cursor(&mut self, df: i8, dr: i8) {
        let new_f = self.cursor_file as i8 + df;
        let new_r = self.cursor_rank as i8 + dr;
        self.cursor_file = new_f.clamp(0, 7) as u8;
        self.cursor_rank = new_r.clamp(0, 7) as u8;
    }

    pub fn select_square(&mut self, sq: Square) {
        if let Some(_selected) = self.selected_square {
            let moves_to_sq: Vec<Move> = self
                .legal_moves_for_selected
                .iter()
                .filter(|m| m.to == sq)
                .copied()
                .collect();

            if moves_to_sq.len() == 1 {
                self.make_move(moves_to_sq[0]);
                return;
            } else if moves_to_sq.len() > 1 {
                self.pending_promotion = Some(moves_to_sq);
                self.promotion_choice = 0;
                return;
            }
        }

        let side = self.game.side_to_move();
        if self.game.board().colors(side).has(sq) {
            self.selected_square = Some(sq);
            self.legal_moves_for_selected = self.game.legal_moves_from(sq);
            self.status_message.clear();
        } else {
            self.deselect();
        }
    }

    pub fn deselect(&mut self) {
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
    }

    pub fn make_move(&mut self, mv: Move) {
        // For online games, send the move to the server instead of applying locally
        if let GameMode::Online { ref game_id, my_color, .. } = self.game_mode {
            if self.game.side_to_move() != my_color {
                self.status_message = "Not your turn".to_string();
                return;
            }
            let san = crate::game::notation::to_algebraic(self.game.board(), &mv);
            if let Some(ref net) = self.network {
                net.send(crate::protocol::ClientMessage::MakeMove {
                    game_id: game_id.clone(),
                    mv: san,
                });
            }
            return; // Don't apply locally — wait for server MoveMade
        }

        let from = mv.from;
        let to = mv.to;
        match self.game.try_make_move(mv) {
            Ok(()) => {
                self.last_move = Some((from, to));
                self.deselect();
                self.pending_promotion = None;
                self.status_message.clear();
                self.input_buffer.clear();
                self.board_image_dirty = true;

                let total_full_moves = (self.game.move_history().len() + 1) / 2;
                if total_full_moves > 0 {
                    self.move_list_scroll = total_full_moves.saturating_sub(1);
                }

                if let GameStatus::Finished(ref result) = self.game.status() {
                    replay::save_game(&self.game, &self.game_mode, result);
                    self.screen = Screen::PostGame;
                    self.postgame_selection = 0;
                    self.status_message = match result {
                        GameResult::Checkmate(c) => format!("Checkmate! {:?} wins!", c),
                        GameResult::Stalemate => "Draw by stalemate".to_string(),
                        GameResult::DrawByRepetition => "Draw by repetition".to_string(),
                        GameResult::DrawByFiftyMove => "Draw by fifty-move rule".to_string(),
                        GameResult::DrawByInsufficientMaterial => {
                            "Draw by insufficient material".to_string()
                        }
                        GameResult::DrawByAgreement => "Draw by agreement".to_string(),
                        GameResult::Resignation(c) => format!("{:?} resigned", c),
                    };
                }
            }
            Err(e) => {
                self.status_message = e;
            }
        }
    }

    pub fn captured_by_white(&self) -> &[crate::game::state::CapturedPiece] {
        self.game.captured_by(Color::White)
    }

    pub fn captured_by_black(&self) -> &[crate::game::state::CapturedPiece] {
        self.game.captured_by(Color::Black)
    }

    pub fn promotion_piece_symbol(mv: &Move) -> &'static str {
        match mv.promotion {
            Some(Piece::Queen) => "\u{265b}",
            Some(Piece::Rook) => "\u{265c}",
            Some(Piece::Bishop) => "\u{265d}",
            Some(Piece::Knight) => "\u{265e}",
            _ => "?",
        }
    }

    // ── Navigation helpers ─────────────────────────────────────────────────

    pub fn set_cursor_to_square(&mut self, sq: Square) {
        let file = sq.file() as u8;
        let rank = sq.rank() as u8;
        if self.board_flipped {
            self.cursor_file = 7 - file;
            self.cursor_rank = 7 - rank;
        } else {
            self.cursor_file = file;
            self.cursor_rank = rank;
        }
    }

    pub fn movable_pieces(&self) -> Vec<Square> {
        let side = self.game.side_to_move();
        let mut squares = std::collections::BTreeSet::new();
        self.game.board().generate_moves(|mvs| {
            let from = mvs.from;
            if self.game.board().colors(side).has(from) {
                squares.insert(from);
            }
            false
        });
        // Sort: rank descending (top first), file ascending (left first)
        let mut result: Vec<Square> = squares.into_iter().collect();
        result.sort_by(|a, b| {
            let rank_cmp = (b.rank() as u8).cmp(&(a.rank() as u8));
            rank_cmp.then((a.file() as u8).cmp(&(b.file() as u8)))
        });
        result
    }

    pub fn jump_to_next_piece(&mut self, forward: bool) {
        let pieces = self.movable_pieces();
        if pieces.is_empty() { return; }
        let cur = self.cursor_square();
        let idx = pieces.iter().position(|&s| s == cur);
        let next = match idx {
            Some(i) => {
                if forward {
                    (i + 1) % pieces.len()
                } else {
                    (i + pieces.len() - 1) % pieces.len()
                }
            }
            None => 0,
        };
        self.set_cursor_to_square(pieces[next]);
    }

    pub fn jump_between_pieces(&mut self, df: i8, dr: i8) {
        let pieces = self.movable_pieces();
        if pieces.is_empty() { return; }
        let cur = self.cursor_square();
        let cf = cur.file() as i8;
        let cr = cur.rank() as i8;

        let best = Self::find_nearest_in_direction(&pieces, cf, cr, df, dr);
        if let Some(sq) = best {
            self.set_cursor_to_square(sq);
        }
    }

    pub fn jump_between_destinations(&mut self, df: i8, dr: i8) {
        let destinations: Vec<Square> = self.legal_moves_for_selected
            .iter()
            .map(|m| m.to)
            .collect();
        if destinations.is_empty() { return; }
        let cur = self.cursor_square();
        let cf = cur.file() as i8;
        let cr = cur.rank() as i8;

        let best = Self::find_nearest_in_direction(&destinations, cf, cr, df, dr);
        if let Some(sq) = best {
            self.set_cursor_to_square(sq);
        }
    }

    fn find_nearest_in_direction(candidates: &[Square], cf: i8, cr: i8, df: i8, dr: i8) -> Option<Square> {
        let mut best: Option<(Square, i8)> = None;
        for &sq in candidates {
            let sf = sq.file() as i8;
            let sr = sq.rank() as i8;
            let delta_f = sf - cf;
            let delta_r = sr - cr;
            // Skip current position
            if delta_f == 0 && delta_r == 0 { continue; }

            // Check direction match
            let matches = if df != 0 && dr == 0 {
                // Horizontal: must move in correct file direction, prefer same rank
                delta_f.signum() == df
            } else if df == 0 && dr != 0 {
                // Vertical: must move in correct rank direction, prefer same file
                delta_r.signum() == dr
            } else {
                // Diagonal or general: match both
                (df == 0 || delta_f.signum() == df) && (dr == 0 || delta_r.signum() == dr)
            };

            if !matches { continue; }

            let dist = delta_f.abs() + delta_r.abs();
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((sq, dist));
            }
        }
        best.map(|(sq, _)| sq)
    }

    pub fn apply_server_move(&mut self, san: &str) {
        if let Some(mv) = crate::game::notation::parse_san(self.game.board(), san) {
            let from = mv.from;
            let to = mv.to;
            if self.game.try_make_move(mv).is_ok() {
                self.last_move = Some((from, to));
                self.deselect();
                self.pending_promotion = None;
                self.input_buffer.clear();
                self.board_image_dirty = true;

                let total_full_moves = (self.game.move_history().len() + 1) / 2;
                if total_full_moves > 0 {
                    self.move_list_scroll = total_full_moves.saturating_sub(1);
                }

                if let GameStatus::Finished(ref result) = self.game.status() {
                    self.screen = Screen::PostGame;
                    self.postgame_selection = 0;
                    self.status_message = match result {
                        GameResult::Checkmate(c) => format!("Checkmate! {:?} wins!", c),
                        GameResult::Stalemate => "Draw by stalemate".to_string(),
                        _ => format!("{:?}", result),
                    };
                }
            }
        }
    }

    pub fn apply_server_preferences(&mut self, prefs: &serde_json::Value) {
        if let Some(scheme) = prefs.get("color_scheme").and_then(|v| v.as_str()) {
            use crate::theme::ColorScheme;
            for (i, cs) in ColorScheme::ALL.iter().enumerate() {
                if cs.name() == scheme {
                    self.color_scheme_index = i;
                    self.apply_color_scheme();
                    break;
                }
            }
        }
    }

    pub fn get_preferences_json(&self) -> serde_json::Value {
        use crate::theme::ColorScheme;
        let scheme_name = ColorScheme::ALL[self.color_scheme_index].name();
        serde_json::json!({
            "color_scheme": scheme_name,
        })
    }

    pub fn start_online_game(&mut self, game_id: String, my_color: Color, opponent_name: String) {
        self.game = GameState::new();
        self.game_start_time = Some(std::time::Instant::now());
        self.board_flipped = my_color == Color::Black;
        self.game_mode = GameMode::Online { game_id, my_color, opponent_name };
        self.screen = Screen::InGame;
        self.mode = InputMode::Play;
        self.cursor_file = 4;
        self.cursor_rank = 4;
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.last_move = None;
        self.input_buffer.clear();
        self.command_buffer.clear();
        self.status_message.clear();
        self.move_list_scroll = 0;
        self.show_help = false;
        self.board_image_dirty = true;
        self.pending_promotion = None;
        self.promotion_choice = 0;
    }
}
