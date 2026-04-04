use cozy_chess::{Board, Color, Move, Piece, Square};

use crate::game::replay::{self, SavedGame};
use crate::game::state::{GameState, GameStatus, GameResult};
use crate::theme::{ColorScheme, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Play,    // Unified: typing + visual nav both active
    Command, // :command mode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
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
    Controls,
}

impl MenuTab {
    pub const ALL: &'static [MenuTab] = &[
        MenuTab::Play,
        MenuTab::Replays,
        MenuTab::Multiplayer,
        MenuTab::Settings,
        MenuTab::Controls,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            MenuTab::Play => "Play",
            MenuTab::Replays => "Replays",
            MenuTab::Multiplayer => "Multiplayer",
            MenuTab::Settings => "Settings",
            MenuTab::Controls => "Controls",
        }
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
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::ColorPicker,
            mode: InputMode::Play,
            should_quit: false,
            theme: Theme::default(),
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
            pending_promotion: None,
            promotion_choice: 0,
            color_scheme_index: 0,
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
            server_url: "ws://127.0.0.1:7600/ws".to_string(),
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
    }

    pub fn try_ai_move(&mut self) {
        if let GameMode::VsAi(ai_color) = self.game_mode.clone() {
            if self.game.side_to_move() == ai_color {
                if let GameStatus::InProgress = self.game.status() {
                    if let Some(mv) = crate::ai::choose_move(self.game.board()) {
                        self.make_move(mv);
                    }
                }
            }
        }
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

                let total_full_moves = (self.game.move_history().len() + 1) / 2;
                if total_full_moves > 0 {
                    self.move_list_scroll = total_full_moves.saturating_sub(1);
                }

                if let GameStatus::Finished(ref result) = self.game.status() {
                    replay::save_game(&self.game, &self.game_mode, result);
                    self.screen = Screen::PostGame;
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

                let total_full_moves = (self.game.move_history().len() + 1) / 2;
                if total_full_moves > 0 {
                    self.move_list_scroll = total_full_moves.saturating_sub(1);
                }

                if let GameStatus::Finished(ref result) = self.game.status() {
                    self.screen = Screen::PostGame;
                    self.status_message = match result {
                        GameResult::Checkmate(c) => format!("Checkmate! {:?} wins!", c),
                        GameResult::Stalemate => "Draw by stalemate".to_string(),
                        _ => format!("{:?}", result),
                    };
                }
            }
        }
    }

    pub fn start_online_game(&mut self, game_id: String, my_color: Color, opponent_name: String) {
        self.game = GameState::new();
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
        self.pending_promotion = None;
        self.promotion_choice = 0;
    }
}
