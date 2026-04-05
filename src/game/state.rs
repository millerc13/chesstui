use cozy_chess::{Board, Color, Move, Piece, Square};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    Checkmate(Color),
    Stalemate,
    DrawByRepetition,
    DrawByFiftyMove,
    DrawByInsufficientMaterial,
    DrawByAgreement,
    Resignation(Color),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Finished(GameResult),
}

#[derive(Debug, Clone)]
pub struct CapturedPiece {
    pub piece: Piece,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub mv: Move,
    pub captured: Option<CapturedPiece>,
    pub previous_board: Board,
}

pub struct GameState {
    board: Board,
    move_history: Vec<MoveRecord>,
    /// Pieces captured by White (i.e., Black pieces taken)
    captured_by_white: Vec<CapturedPiece>,
    /// Pieces captured by Black (i.e., White pieces taken)
    captured_by_black: Vec<CapturedPiece>,
    status: GameStatus,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
            move_history: Vec::new(),
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            status: GameStatus::InProgress,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn side_to_move(&self) -> Color {
        self.board.side_to_move()
    }

    pub fn fullmove_number(&self) -> u32 {
        // Starts at 1, increments after Black moves.
        // Each full move = one White move + one Black move.
        let half_moves = self.move_history.len();
        1 + (half_moves / 2) as u32
    }

    pub fn move_history(&self) -> &[MoveRecord] {
        &self.move_history
    }

    pub fn status(&self) -> GameStatus {
        self.status.clone()
    }

    pub fn captured_by(&self, color: Color) -> &[CapturedPiece] {
        match color {
            Color::White => &self.captured_by_white,
            Color::Black => &self.captured_by_black,
        }
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.board.generate_moves(|list| {
            moves.extend(list);
            false
        });
        moves
    }

    pub fn legal_moves_from(&self, square: Square) -> Vec<Move> {
        self.legal_moves()
            .into_iter()
            .filter(|mv| mv.from == square)
            .collect()
    }

    pub fn is_in_check(&self) -> bool {
        !self.board.checkers().is_empty()
    }

    pub fn try_make_move(&mut self, mv: Move) -> Result<(), String> {
        if self.status != GameStatus::InProgress {
            return Err("Game is already finished".to_string());
        }

        // Check legality
        let legal = self.legal_moves();
        if !legal.contains(&mv) {
            return Err(format!("Illegal move: {:?}", mv));
        }

        let previous_board = self.board.clone();
        let mover_color = self.board.side_to_move();
        let opponent_color = !mover_color;

        // Detect capture
        let captured = self.detect_capture(mv, opponent_color);

        // Apply the move
        self.board.play_unchecked(mv);

        // Record captured piece
        if let Some(ref cap) = captured {
            match mover_color {
                Color::White => self.captured_by_white.push(cap.clone()),
                Color::Black => self.captured_by_black.push(cap.clone()),
            }
        }

        self.move_history.push(MoveRecord {
            mv,
            captured: captured.clone(),
            previous_board,
        });

        self.update_status();

        Ok(())
    }

    fn detect_capture(&self, mv: Move, opponent_color: Color) -> Option<CapturedPiece> {
        let dest = mv.to;
        let mover_color = self.board.side_to_move();

        // Normal capture: opponent piece on destination square
        if self.board.colors(opponent_color).has(dest) {
            // Find which piece type is on the destination
            let piece = self.piece_on(dest)?;
            return Some(CapturedPiece {
                piece,
                color: opponent_color,
            });
        }

        // En passant: pawn moves diagonally to empty square
        let moving_piece = self.piece_on(mv.from)?;
        if moving_piece == Piece::Pawn && mv.from.file() != dest.file() {
            // The captured pawn is on the same rank as the moving pawn's origin,
            // but on the destination file.
            let ep_rank = mv.from.rank();
            let ep_file = dest.file();
            let ep_square = Square::new(ep_file, ep_rank);
            if self.board.colors(opponent_color).has(ep_square) {
                let _ = mover_color; // suppress warning
                return Some(CapturedPiece {
                    piece: Piece::Pawn,
                    color: opponent_color,
                });
            }
        }

        None
    }

    fn piece_on(&self, square: Square) -> Option<Piece> {
        [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ]
        .into_iter()
        .find(|&piece| self.board.pieces(piece).has(square))
    }

    fn update_status(&mut self) {
        let legal = self.legal_moves();

        if legal.is_empty() {
            if self.is_in_check() {
                // The side that just moved delivered checkmate
                let winner = !self.board.side_to_move();
                self.status = GameStatus::Finished(GameResult::Checkmate(winner));
            } else {
                self.status = GameStatus::Finished(GameResult::Stalemate);
            }
            return;
        }

        if self.is_insufficient_material() {
            self.status = GameStatus::Finished(GameResult::DrawByInsufficientMaterial);
            return;
        }

        // Fifty-move rule: halfmove clock >= 100 (50 full moves)
        // cozy-chess Board doesn't expose halfmove clock directly, so we skip for now.

        self.status = GameStatus::InProgress;
    }

    fn is_insufficient_material(&self) -> bool {
        // King vs King
        let white = self.board.colors(Color::White);
        let black = self.board.colors(Color::Black);

        let white_count = white.len();
        let black_count = black.len();

        if white_count == 1 && black_count == 1 {
            return true;
        }

        // King + minor piece vs King
        let minor = self.board.pieces(Piece::Knight) | self.board.pieces(Piece::Bishop);
        if white_count == 2 && black_count == 1 && (white & minor).len() == 1 {
            return true;
        }
        if black_count == 2 && white_count == 1 && (black & minor).len() == 1 {
            return true;
        }

        false
    }

    pub fn undo(&mut self) {
        if let Some(record) = self.move_history.pop() {
            self.board = record.previous_board;

            // Remove the captured piece from the appropriate list
            if let Some(ref cap) = record.captured {
                // The side that moved was the opposite of the current side_to_move after undo.
                // After restoring the board, side_to_move() is the mover again.
                let mover = self.board.side_to_move();
                match mover {
                    Color::White => {
                        self.captured_by_white.pop();
                    }
                    Color::Black => {
                        self.captured_by_black.pop();
                    }
                }
                let _ = cap; // used for documentation clarity
            }

            // Restore status to InProgress if we undid back out of a finished state
            self.status = GameStatus::InProgress;
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
