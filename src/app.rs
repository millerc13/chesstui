use cozy_chess::{Color, Move, Piece, Square};

use crate::game::state::{GameState, GameStatus, GameResult};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Input,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    InGame,
    PostGame,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::MainMenu,
            mode: InputMode::Normal,
            should_quit: false,
            theme: Theme::detect(),
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
        }
    }

    pub fn start_new_game(&mut self) {
        self.game = GameState::new();
        self.screen = Screen::InGame;
        self.mode = InputMode::Normal;
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

    pub fn menu_items(&self) -> &[&str] {
        &["Local Game", "Quit"]
    }

    /// Convert display cursor coordinates to a chess square,
    /// accounting for board flip.
    pub fn cursor_square(&self) -> Square {
        let (file, rank) = if self.board_flipped {
            (7 - self.cursor_file, 7 - self.cursor_rank)
        } else {
            (self.cursor_file, self.cursor_rank)
        };
        // cursor_rank 0 = bottom of display = Rank::First when not flipped
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

    /// Handle selecting a square. If a piece is already selected and the
    /// destination is a legal move, make the move. Otherwise, select the piece.
    pub fn select_square(&mut self, sq: Square) {
        if let Some(_selected) = self.selected_square {
            // Check if sq is a legal destination
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
                // Multiple moves to same destination = promotion
                self.pending_promotion = Some(moves_to_sq);
                self.promotion_choice = 0;
                return;
            }

            // Not a legal dest — try selecting the new square instead
        }

        // Try selecting a piece on this square
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
        let from = mv.from;
        let to = mv.to;
        match self.game.try_make_move(mv) {
            Ok(()) => {
                self.last_move = Some((from, to));
                self.deselect();
                self.pending_promotion = None;
                self.status_message.clear();
                self.input_buffer.clear();

                // Auto-scroll move list
                let total_full_moves = (self.game.move_history().len() + 1) / 2;
                if total_full_moves > 0 {
                    self.move_list_scroll = total_full_moves.saturating_sub(1);
                }

                // Check for game over
                if let GameStatus::Finished(ref result) = self.game.status() {
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

    /// Return the Unicode symbol for a promotion piece option.
    pub fn promotion_piece_symbol(mv: &Move) -> &'static str {
        match mv.promotion {
            Some(Piece::Queen) => "\u{265b}",
            Some(Piece::Rook) => "\u{265c}",
            Some(Piece::Bishop) => "\u{265d}",
            Some(Piece::Knight) => "\u{265e}",
            _ => "?",
        }
    }
}
