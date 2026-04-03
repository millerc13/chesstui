# Phase 1: Local Hotseat Chess Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A playable local chess game in the terminal — two humans taking turns on one keyboard, with VIM-style controls, full rules enforcement, and a polished TUI.

**Architecture:** Single-binary Rust app using `cozy-chess` for move generation/validation, `ratatui` + `crossterm` for terminal rendering, and a modal input system (NORMAL/INPUT/COMMAND) inspired by VIM. The app state machine flows: MainMenu → InGame → PostGame. No networking, no AI wiring in this phase.

**Tech Stack:** Rust 2021 edition, cozy-chess 0.3, ratatui 0.29, crossterm 0.28, clap 4 (CLI args), serde + toml (config)

---

## File Map

Every file that will be created or modified, with its responsibility:

```
chesstui/
  Cargo.toml                  # MODIFY: add clap, serde, toml deps
  src/
    main.rs                   # CREATE: entry point, terminal setup, event loop
    app.rs                    # CREATE: App state machine (screen, mode, tick)
    theme.rs                  # CREATE: color constants, Theme struct, 256-color fallback
    input.rs                  # CREATE: key event dispatch, mode transitions
    game/
      mod.rs                  # CREATE: re-exports
      state.rs                # CREATE: GameState wrapping cozy-chess Board + metadata
      notation.rs             # CREATE: move-to-algebraic, algebraic-to-move formatting
      move_input.rs           # CREATE: legal-move trie, algebraic parsing, disambiguation
    ui/
      mod.rs                  # CREATE: re-exports
      menu.rs                 # CREATE: main menu rendering
      board.rs                # CREATE: ChessBoard widget (the core rendering)
      game.rs                 # CREATE: in-game layout orchestration
      move_list.rs            # CREATE: move history panel widget
      captured.rs             # CREATE: captured pieces widget
      command_bar.rs          # CREATE: input/command/status bar widget
      postgame.rs             # CREATE: post-game result screen
      help.rs                 # CREATE: help overlay widget
    engine/                   # EXISTS: AI engine code (not wired up this phase)
      mod.rs                  # EXISTS
      ...                     # EXISTS (6 more files)
  tests/
    chess_rules.rs            # CREATE: integration tests for game state
    notation.rs               # CREATE: tests for algebraic notation
    move_input.rs             # CREATE: tests for move input parsing/trie
```

---

### Task 1: Initialize Git and Project Scaffold

**Files:**
- Modify: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize git repo and set remote**

```bash
cd /Users/cjmiller/development/chesstui
git init
git remote add origin git@github.com:millerc13/chesstui.git
```

- [ ] **Step 2: Create .gitignore**

```gitignore
/target
*.swp
*.swo
.DS_Store
```

- [ ] **Step 3: Update Cargo.toml with all Phase 1 dependencies**

```toml
[package]
name = "chesstui"
version = "0.1.0"
edition = "2021"
description = "Multiplayer chess in the terminal with built-in AI"
license = "MIT"
repository = "https://github.com/millerc13/chesstui"

[dependencies]
cozy-chess = "0.3"
ratatui = "0.29"
crossterm = "0.28"
clap = { version = "4", features = ["derive"] }
rand = "0.8"
tokio = { version = "1", features = ["full"] }

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

- [ ] **Step 4: Create minimal main.rs that compiles and exits**

```rust
fn main() {
    println!("chesstui v0.1.0");
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build 2>&1`
Expected: Compiles successfully (downloading deps on first run)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/engine/ .gitignore
git commit -m "feat: initialize project with deps and engine scaffold"
```

---

### Task 2: Game State Module

**Files:**
- Create: `src/game/mod.rs`
- Create: `src/game/state.rs`
- Create: `tests/chess_rules.rs`

This wraps `cozy-chess::Board` with game-level metadata: move history, captured pieces, game status, whose turn it is.

- [ ] **Step 1: Write tests for GameState**

Create `tests/chess_rules.rs`:

```rust
use chesstui::game::state::{GameState, GameStatus, GameResult};

#[test]
fn new_game_starts_with_white_to_move() {
    let game = GameState::new();
    assert_eq!(game.side_to_move(), cozy_chess::Color::White);
    assert_eq!(game.fullmove_number(), 1);
    assert!(game.move_history().is_empty());
    assert_eq!(game.status(), GameStatus::InProgress);
}

#[test]
fn making_a_move_switches_turn() {
    let mut game = GameState::new();
    // e2e4
    let mv = find_move(&game, "e2", "e4", None);
    assert!(game.try_make_move(mv).is_ok());
    assert_eq!(game.side_to_move(), cozy_chess::Color::Black);
}

#[test]
fn illegal_move_is_rejected() {
    let mut game = GameState::new();
    // Try to move a black piece on White's turn — e7e5
    let mv = cozy_chess::Move {
        from: cozy_chess::Square::E7,
        to: cozy_chess::Square::E5,
        promotion: None,
    };
    assert!(game.try_make_move(mv).is_err());
}

#[test]
fn captured_pieces_are_tracked() {
    let mut game = GameState::new();
    // Scholars mate sequence to test captures
    // 1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7#
    make_uci(&mut game, "e2e4");
    make_uci(&mut game, "e7e5");
    make_uci(&mut game, "f1c4");
    make_uci(&mut game, "b8c6");
    make_uci(&mut game, "d1h5");
    make_uci(&mut game, "g8f6");
    make_uci(&mut game, "h5f7"); // Qxf7#
    // White captured a pawn on f7
    assert_eq!(game.captured_by(cozy_chess::Color::White).len(), 1);
    assert_eq!(game.status(), GameStatus::Finished(GameResult::Checkmate(cozy_chess::Color::White)));
}

#[test]
fn undo_restores_previous_position() {
    let mut game = GameState::new();
    make_uci(&mut game, "e2e4");
    make_uci(&mut game, "e7e5");
    assert_eq!(game.move_history().len(), 2);
    game.undo();
    assert_eq!(game.move_history().len(), 1);
    assert_eq!(game.side_to_move(), cozy_chess::Color::Black);
    game.undo();
    assert_eq!(game.move_history().len(), 0);
    assert_eq!(game.side_to_move(), cozy_chess::Color::White);
}

// Helpers
fn find_move(game: &GameState, from: &str, to: &str, promo: Option<cozy_chess::Piece>) -> cozy_chess::Move {
    let from_sq = from.parse::<cozy_chess::Square>().unwrap();
    let to_sq = to.parse::<cozy_chess::Square>().unwrap();
    cozy_chess::Move { from: from_sq, to: to_sq, promotion: promo }
}

fn make_uci(game: &mut GameState, uci: &str) {
    let bytes = uci.as_bytes();
    let from = format!("{}{}", bytes[0] as char, bytes[1] as char);
    let to = format!("{}{}", bytes[2] as char, bytes[3] as char);
    let promo = if bytes.len() == 5 {
        match bytes[4] {
            b'q' => Some(cozy_chess::Piece::Queen),
            b'r' => Some(cozy_chess::Piece::Rook),
            b'b' => Some(cozy_chess::Piece::Bishop),
            b'n' => Some(cozy_chess::Piece::Knight),
            _ => None,
        }
    } else {
        None
    };
    let mv = find_move(game, &from, &to, promo);
    game.try_make_move(mv).expect(&format!("Move {} should be legal", uci));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test chess_rules 2>&1`
Expected: FAIL — module `game` not found

- [ ] **Step 3: Create `src/game/mod.rs`**

```rust
pub mod state;
```

- [ ] **Step 4: Create `src/game/state.rs`**

```rust
use cozy_chess::{Board, Color, Move, Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Checkmate(Color),  // Color that won
    Stalemate,
    DrawByRepetition,
    DrawByFiftyMove,
    DrawByInsufficientMaterial,
    DrawByAgreement,
    Resignation(Color), // Color that won (opponent resigned)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Finished(GameResult),
}

/// A captured piece, for display purposes.
#[derive(Debug, Clone, Copy)]
pub struct CapturedPiece {
    pub piece: Piece,
    pub color: Color,
}

/// Record of a move made, for undo and move history display.
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub mv: Move,
    pub captured: Option<CapturedPiece>,
    pub previous_board: Board,
}

pub struct GameState {
    board: Board,
    move_history: Vec<MoveRecord>,
    captured_white: Vec<Piece>, // pieces captured BY White (Black's pieces)
    captured_black: Vec<Piece>, // pieces captured BY Black (White's pieces)
    status: GameStatus,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
            move_history: Vec::new(),
            captured_white: Vec::new(),
            captured_black: Vec::new(),
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
        (self.move_history.len() as u32) / 2 + 1
    }

    pub fn move_history(&self) -> &[MoveRecord] {
        &self.move_history
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    /// Pieces captured by the given color.
    pub fn captured_by(&self, color: Color) -> &[Piece] {
        match color {
            Color::White => &self.captured_white,
            Color::Black => &self.captured_black,
        }
    }

    /// Try to make a move. Returns Err if the move is illegal.
    pub fn try_make_move(&mut self, mv: Move) -> Result<(), String> {
        if self.status != GameStatus::InProgress {
            return Err("Game is already over".into());
        }

        // Check if the move is legal.
        let mut legal_moves = Vec::new();
        self.board.generate_moves(|moves| {
            legal_moves.extend(moves);
            false
        });

        if !legal_moves.contains(&mv) {
            return Err(format!("Illegal move: {:?}", mv));
        }

        // Check for capture.
        let opponent_color = !self.board.side_to_move();
        let captured = if self.board.colors(opponent_color).has(mv.to) {
            // Find which piece is on the target square.
            let piece = self.piece_on(mv.to).unwrap();
            Some(CapturedPiece { piece, color: opponent_color })
        } else if self.board.pieces(Piece::Pawn).has(mv.from) && mv.from.file() != mv.to.file()
            && !self.board.occupied().has(mv.to)
        {
            // En passant capture.
            Some(CapturedPiece { piece: Piece::Pawn, color: opponent_color })
        } else {
            None
        };

        let previous_board = self.board.clone();
        let mover = self.board.side_to_move();

        // Apply the move.
        self.board.play(mv);

        // Record capture.
        if let Some(cap) = &captured {
            match mover {
                Color::White => self.captured_white.push(cap.piece),
                Color::Black => self.captured_black.push(cap.piece),
            }
        }

        self.move_history.push(MoveRecord {
            mv,
            captured: captured,
            previous_board,
        });

        // Check game-ending conditions.
        self.update_status();

        Ok(())
    }

    /// Undo the last move.
    pub fn undo(&mut self) {
        if let Some(record) = self.move_history.pop() {
            // Restore board.
            self.board = record.previous_board;

            // Remove captured piece from the list.
            if let Some(cap) = &record.captured {
                let list = match !cap.color {
                    Color::White => &mut self.captured_white,
                    Color::Black => &mut self.captured_black,
                };
                if let Some(pos) = list.iter().rposition(|&p| p == cap.piece) {
                    list.remove(pos);
                }
            }

            self.status = GameStatus::InProgress;
        }
    }

    /// Get all legal moves for the current position.
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.board.generate_moves(|mvs| {
            moves.extend(mvs);
            false
        });
        moves
    }

    /// Get legal moves for a specific square (piece on that square).
    pub fn legal_moves_from(&self, square: Square) -> Vec<Move> {
        self.legal_moves().into_iter().filter(|m| m.from == square).collect()
    }

    fn piece_on(&self, sq: Square) -> Option<Piece> {
        for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
            if self.board.pieces(piece).has(sq) {
                return Some(piece);
            }
        }
        None
    }

    fn update_status(&mut self) {
        let mut has_legal_moves = false;
        self.board.generate_moves(|moves| {
            if moves.len() > 0 {
                has_legal_moves = true;
                true // stop early
            } else {
                false
            }
        });

        if !has_legal_moves {
            if !self.board.checkers().is_empty() {
                // Checkmate — the side that just moved wins.
                let winner = !self.board.side_to_move();
                self.status = GameStatus::Finished(GameResult::Checkmate(winner));
            } else {
                self.status = GameStatus::Finished(GameResult::Stalemate);
            }
            return;
        }

        // Check for insufficient material.
        if self.is_insufficient_material() {
            self.status = GameStatus::Finished(GameResult::DrawByInsufficientMaterial);
        }
    }

    fn is_insufficient_material(&self) -> bool {
        let total_pieces = self.board.occupied().len();
        if total_pieces == 2 {
            return true; // K vs K
        }
        if total_pieces == 3 {
            // K+B vs K or K+N vs K
            let minors = self.board.pieces(Piece::Knight).len() + self.board.pieces(Piece::Bishop).len();
            if minors == 1 {
                return true;
            }
        }
        false
    }

    /// Check if the current side to move is in check.
    pub fn is_in_check(&self) -> bool {
        !self.board.checkers().is_empty()
    }
}
```

- [ ] **Step 5: Expose the game module from main.rs (as lib)**

Create `src/lib.rs`:

```rust
pub mod game;
```

Update `src/main.rs`:

```rust
fn main() {
    println!("chesstui v0.1.0");
}
```

- [ ] **Step 6: Run tests**

Run: `cargo test --test chess_rules 2>&1`
Expected: All 5 tests pass

- [ ] **Step 7: Commit**

```bash
git add src/game/ src/lib.rs tests/chess_rules.rs
git commit -m "feat: add GameState with move validation, captures, undo"
```

---

### Task 3: Algebraic Notation Module

**Files:**
- Create: `src/game/notation.rs`
- Create: `tests/notation.rs`

Converts between `cozy_chess::Move` and standard algebraic notation strings (e.g., `Nf3`, `exd5`, `O-O`, `Qxd7#`).

- [ ] **Step 1: Write tests**

Create `tests/notation.rs`:

```rust
use chesstui::game::notation::to_algebraic;
use chesstui::game::state::GameState;
use cozy_chess::{Board, Move, Piece, Square};

#[test]
fn pawn_move_e4() {
    let board = Board::default();
    let mv = Move { from: Square::E2, to: Square::E4, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "e4");
}

#[test]
fn knight_move_nf3() {
    let board = Board::default();
    let mv = Move { from: Square::G1, to: Square::F3, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "Nf3");
}

#[test]
fn pawn_capture_exd5() {
    // Set up a position where e4 pawn can capture on d5
    let board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", false).unwrap();
    let mv = Move { from: Square::E4, to: Square::D5, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "exd5");
}

#[test]
fn kingside_castling() {
    let board = Board::from_fen("r1bqkbnr/pppppppp/2n5/8/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3", false).unwrap();
    // cozy-chess represents castling as king moving to rook square or king moving 2 squares
    // Check which convention cozy-chess uses by generating moves
    let mut castling_move = None;
    board.generate_moves(|moves| {
        for mv in moves {
            if mv.from == Square::E1 && (mv.to == Square::G1 || mv.to == Square::H1) {
                castling_move = Some(mv);
            }
        }
        false
    });
    if let Some(mv) = castling_move {
        assert_eq!(to_algebraic(&board, &mv), "O-O");
    }
}

#[test]
fn promotion_e8_queen() {
    let board = Board::from_fen("8/4P3/8/8/8/8/8/4K2k w - - 0 1", false).unwrap();
    let mv = Move { from: Square::E7, to: Square::E8, promotion: Some(Piece::Queen) };
    assert_eq!(to_algebraic(&board, &mv), "e8=Q");
}

#[test]
fn knight_disambiguation_by_file() {
    // Two knights that can both go to d2
    let board = Board::from_fen("8/8/8/8/8/N4N2/8/4K2k w - - 0 1", false).unwrap();
    let mv = Move { from: Square::A3, to: Square::B1, promotion: None };
    let san = to_algebraic(&board, &mv);
    // Should include file disambiguation if another knight can go there too
    // Actually in this position only one knight can go to b1, so no disambiguation needed
    assert_eq!(san, "Nb1");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test notation 2>&1`
Expected: FAIL — module not found

- [ ] **Step 3: Create `src/game/notation.rs`**

```rust
use cozy_chess::{Board, Color, Move, Piece, Square};

/// Convert a move to standard algebraic notation (SAN).
/// Examples: "e4", "Nf3", "exd5", "O-O", "Qxd7+", "e8=Q"
pub fn to_algebraic(board: &Board, mv: &Move) -> String {
    // Handle castling.
    if board.pieces(Piece::King).has(mv.from) {
        let is_kingside = mv.to.file() as i8 > mv.from.file() as i8;
        let file_diff = (mv.to.file() as i8 - mv.from.file() as i8).abs();
        // cozy-chess encodes castling as king -> rook square
        if file_diff >= 2 || (board.pieces(Piece::Rook).has(mv.to) && board.colors(board.side_to_move()).has(mv.to)) {
            let is_kingside = mv.to.file() as u8 > mv.from.file() as u8;
            let mut child = board.clone();
            child.play(*mv);
            let suffix = check_suffix(&child);
            return if is_kingside {
                format!("O-O{}", suffix)
            } else {
                format!("O-O-O{}", suffix)
            };
        }
    }

    let mut result = String::new();
    let piece = piece_on(board, mv.from).unwrap();
    let is_capture = board.colors(!board.side_to_move()).has(mv.to)
        || (piece == Piece::Pawn && mv.from.file() != mv.to.file());

    // Piece letter (pawns omitted).
    if piece != Piece::Pawn {
        result.push(piece_char(piece));
        // Disambiguation: if another piece of the same type can reach the same square.
        let disambiguation = disambiguate(board, mv, piece);
        result.push_str(&disambiguation);
    } else if is_capture {
        // Pawn captures include the source file.
        result.push(file_char(mv.from.file() as u8));
    }

    // Capture marker.
    if is_capture {
        result.push('x');
    }

    // Destination square.
    result.push(file_char(mv.to.file() as u8));
    result.push(rank_char(mv.to.rank() as u8));

    // Promotion.
    if let Some(promo) = mv.promotion {
        result.push('=');
        result.push(piece_char(promo));
    }

    // Check / checkmate suffix.
    let mut child = board.clone();
    child.play(*mv);
    result.push_str(&check_suffix(&child));

    result
}

fn disambiguate(board: &Board, mv: &Move, piece: Piece) -> String {
    let mut others = Vec::new();
    board.generate_moves(|moves| {
        for m in moves {
            if m.to == mv.to && m.from != mv.from && piece_on(board, m.from) == Some(piece) {
                others.push(m.from);
            }
        }
        false
    });

    if others.is_empty() {
        return String::new();
    }

    let same_file = others.iter().any(|s| s.file() == mv.from.file());
    let same_rank = others.iter().any(|s| s.rank() == mv.from.rank());

    if !same_file {
        format!("{}", file_char(mv.from.file() as u8))
    } else if !same_rank {
        format!("{}", rank_char(mv.from.rank() as u8))
    } else {
        format!("{}{}", file_char(mv.from.file() as u8), rank_char(mv.from.rank() as u8))
    }
}

fn check_suffix(board: &Board) -> &'static str {
    if !board.checkers().is_empty() {
        // Check if it's checkmate.
        let mut has_moves = false;
        board.generate_moves(|moves| {
            if moves.len() > 0 {
                has_moves = true;
                true
            } else {
                false
            }
        });
        if has_moves { "+" } else { "#" }
    } else {
        ""
    }
}

fn piece_on(board: &Board, sq: Square) -> Option<Piece> {
    for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
        if board.pieces(piece).has(sq) {
            return Some(piece);
        }
    }
    None
}

fn piece_char(piece: Piece) -> char {
    match piece {
        Piece::King => 'K',
        Piece::Queen => 'Q',
        Piece::Rook => 'R',
        Piece::Bishop => 'B',
        Piece::Knight => 'N',
        Piece::Pawn => 'P',
    }
}

fn file_char(file: u8) -> char {
    (b'a' + file) as char
}

fn rank_char(rank: u8) -> char {
    (b'1' + rank) as char
}

/// Parse a square string like "e4" into a Square.
pub fn parse_square(s: &str) -> Option<Square> {
    s.parse::<Square>().ok()
}
```

- [ ] **Step 4: Update `src/game/mod.rs`**

```rust
pub mod state;
pub mod notation;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test notation 2>&1`
Expected: All tests pass (the castling test may need adjustment depending on cozy-chess castling representation — see step 6)

- [ ] **Step 6: Fix any castling representation issues**

cozy-chess represents castling as King moving to Rook's square (e1->h1 for kingside). Adjust the `to_algebraic` castling detection if tests fail. Run tests again until green.

- [ ] **Step 7: Commit**

```bash
git add src/game/notation.rs tests/notation.rs
git commit -m "feat: add algebraic notation formatting (SAN)"
```

---

### Task 4: Move Input Parser and Legal-Move Trie

**Files:**
- Create: `src/game/move_input.rs`
- Create: `tests/move_input.rs`

This is the core of the VIM-style input: as the user types characters, the system matches against legal moves in real-time. When exactly one match remains, it auto-executes.

- [ ] **Step 1: Write tests**

Create `tests/move_input.rs`:

```rust
use chesstui::game::move_input::{MoveInputParser, InputResult};
use cozy_chess::Board;

#[test]
fn pawn_e4_from_starting_position() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    assert!(matches!(parser.feed('e'), InputResult::NeedMore(_)));
    // 'e' could be e3 or e4 — need more input
    match parser.feed('4') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.from, cozy_chess::Square::E2);
            assert_eq!(mv.to, cozy_chess::Square::E4);
        }
        other => panic!("Expected Exact, got {:?}", other),
    }
}

#[test]
fn knight_nf3_from_starting_position() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    assert!(matches!(parser.feed('N'), InputResult::NeedMore(_)));
    assert!(matches!(parser.feed('f'), InputResult::NeedMore(_)));
    match parser.feed('3') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.to, cozy_chess::Square::F3);
        }
        other => panic!("Expected Exact, got {:?}", other),
    }
}

#[test]
fn invalid_input_returns_noMatch() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    parser.feed('z'); // not a valid chess character
    // z is not a-h or piece letter, should be NoMatch
}

#[test]
fn square_to_square_e2e4() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    parser.feed('e');
    parser.feed('2');
    parser.feed('e');
    match parser.feed('4') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.from, cozy_chess::Square::E2);
            assert_eq!(mv.to, cozy_chess::Square::E4);
        }
        other => panic!("Expected Exact, got {:?}", other),
    }
}

#[test]
fn castling_oo() {
    let board = Board::from_fen(
        "r1bqk2r/ppppbppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        false,
    ).unwrap();
    let mut parser = MoveInputParser::new(&board);
    assert!(matches!(parser.feed('O'), InputResult::NeedMore(_)));
    match parser.feed('O') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.from, cozy_chess::Square::E1);
            // castling target is rook square in cozy-chess
        }
        other => panic!("Expected Exact for O-O, got {:?}", other),
    }
}

#[test]
fn buffer_and_reset() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    parser.feed('e');
    assert_eq!(parser.buffer(), "e");
    parser.reset();
    assert_eq!(parser.buffer(), "");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test move_input 2>&1`
Expected: FAIL — module not found

- [ ] **Step 3: Create `src/game/move_input.rs`**

```rust
use cozy_chess::{Board, Move, Piece, Square};

#[derive(Debug)]
pub enum InputResult {
    /// Exactly one legal move matches — execute it.
    Exact(Move),
    /// Multiple moves still match — need more input. Contains count of matches.
    NeedMore(usize),
    /// No legal move matches the current input.
    NoMatch,
}

/// Parses incremental chess move input against the legal moves in a position.
/// Supports algebraic notation (e4, Nf3, exd5, O-O) and square-to-square (e2e4).
pub struct MoveInputParser {
    buffer: String,
    legal_moves: Vec<(Move, String)>, // (move, SAN string)
    candidates: Vec<Move>,
}

impl MoveInputParser {
    pub fn new(board: &Board) -> Self {
        let mut legal_moves_raw = Vec::new();
        board.generate_moves(|moves| {
            legal_moves_raw.extend(moves);
            false
        });

        let legal_moves: Vec<(Move, String)> = legal_moves_raw
            .iter()
            .map(|mv| {
                let san = crate::game::notation::to_algebraic(board, mv);
                (*mv, san)
            })
            .collect();

        Self {
            buffer: String::new(),
            legal_moves,
            candidates: legal_moves_raw,
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Feed a character of input. Returns the result after processing.
    pub fn feed(&mut self, ch: char) -> InputResult {
        self.buffer.push(ch);

        // Try matching as castling shorthand.
        if self.buffer == "OO" || self.buffer == "oo" {
            let castling = self.find_castling(true);
            if let Some(mv) = castling {
                return InputResult::Exact(mv);
            }
        }
        if self.buffer == "OOO" || self.buffer == "ooo" {
            let castling = self.find_castling(false);
            if let Some(mv) = castling {
                return InputResult::Exact(mv);
            }
        }

        // Try matching as square-to-square (e.g., "e2e4").
        if self.buffer.len() == 4 || self.buffer.len() == 5 {
            if let Some(mv) = self.try_square_to_square() {
                return InputResult::Exact(mv);
            }
        }

        // Try matching against SAN strings.
        let matches = self.match_san();

        match matches.len() {
            0 => InputResult::NoMatch,
            1 => InputResult::Exact(matches[0]),
            n => InputResult::NeedMore(n),
        }
    }

    /// Match the current buffer against SAN representations of legal moves.
    fn match_san(&self) -> Vec<Move> {
        let buf = &self.buffer;
        self.legal_moves
            .iter()
            .filter(|(_, san)| {
                // Strip check/checkmate suffixes for matching.
                let clean = san.trim_end_matches('+').trim_end_matches('#');
                // Also strip 'x' for lenient matching (e.g., "ed5" matches "exd5").
                let clean_no_x: String = clean.chars().filter(|c| *c != 'x').collect();

                // Match if SAN starts with buffer, or clean version does.
                clean.starts_with(buf)
                    || clean_no_x.starts_with(buf)
                    || clean.eq_ignore_ascii_case(buf)
            })
            .map(|(mv, _)| *mv)
            .collect()
    }

    fn try_square_to_square(&self) -> Option<Move> {
        let bytes = self.buffer.as_bytes();
        if bytes.len() < 4 { return None; }

        let from_file = bytes[0].wrapping_sub(b'a');
        let from_rank = bytes[1].wrapping_sub(b'1');
        let to_file = bytes[2].wrapping_sub(b'a');
        let to_rank = bytes[3].wrapping_sub(b'1');

        if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
            return None;
        }

        let from = Square::index(from_rank as usize * 8 + from_file as usize);
        let to = Square::index(to_rank as usize * 8 + to_file as usize);

        let promo = if bytes.len() == 5 {
            match bytes[4] {
                b'q' | b'Q' => Some(Piece::Queen),
                b'r' | b'R' => Some(Piece::Rook),
                b'b' | b'B' => Some(Piece::Bishop),
                b'n' | b'N' => Some(Piece::Knight),
                _ => None,
            }
        } else {
            None
        };

        let target = Move { from, to, promotion: promo };

        // Check if this exact move is legal.
        if self.candidates.contains(&target) {
            return Some(target);
        }

        // cozy-chess may encode castling differently (king -> rook square).
        // Try matching by from square and approximate to.
        None
    }

    fn find_castling(&self, kingside: bool) -> Option<Move> {
        self.legal_moves.iter()
            .find(|(_, san)| {
                let clean = san.trim_end_matches('+').trim_end_matches('#');
                if kingside {
                    clean == "O-O"
                } else {
                    clean == "O-O-O"
                }
            })
            .map(|(mv, _)| *mv)
    }

    /// Get matching SAN strings for the current buffer (for display).
    pub fn matching_moves(&self) -> Vec<(&Move, &str)> {
        let buf = &self.buffer;
        self.legal_moves
            .iter()
            .filter(|(_, san)| {
                let clean = san.trim_end_matches('+').trim_end_matches('#');
                let clean_no_x: String = clean.chars().filter(|c| *c != 'x').collect();
                clean.starts_with(buf) || clean_no_x.starts_with(buf)
            })
            .map(|(mv, san)| (mv, san.as_str()))
            .collect()
    }
}
```

- [ ] **Step 4: Update `src/game/mod.rs`**

```rust
pub mod state;
pub mod notation;
pub mod move_input;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test move_input 2>&1`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/game/move_input.rs tests/move_input.rs
git commit -m "feat: add move input parser with SAN matching and auto-execute"
```

---

### Task 5: Theme and Color System

**Files:**
- Create: `src/theme.rs`

- [ ] **Step 1: Create `src/theme.rs`**

```rust
use ratatui::style::Color;

pub struct Theme {
    // Board
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
    pub fn default_truecolor() -> Self {
        Self {
            light_square: Color::Rgb(181, 136, 99),
            dark_square: Color::Rgb(109, 76, 47),
            white_piece: Color::Rgb(240, 235, 220),
            black_piece: Color::Rgb(40, 35, 30),

            selected_bg: Color::Rgb(130, 170, 100),
            legal_move_bg: Color::Rgb(100, 140, 80),
            last_move_light: Color::Rgb(205, 210, 106),
            last_move_dark: Color::Rgb(170, 162, 58),
            check_bg: Color::Rgb(200, 50, 50),
            cursor_bg: Color::Rgb(80, 120, 200),

            accent: Color::Rgb(100, 160, 220),
            text_primary: Color::Rgb(200, 200, 200),
            text_dim: Color::Rgb(120, 120, 120),
            text_bright: Color::Rgb(240, 240, 240),
            border_focused: Color::Rgb(100, 160, 220),
            border_dim: Color::Rgb(60, 60, 60),

            mode_normal: Color::Rgb(120, 120, 120),
            mode_input: Color::Rgb(220, 180, 50),
            mode_command: Color::Rgb(80, 180, 220),
        }
    }

    pub fn default_256() -> Self {
        Self {
            light_square: Color::Indexed(180),
            dark_square: Color::Indexed(137),
            white_piece: Color::Indexed(230),
            black_piece: Color::Indexed(235),

            selected_bg: Color::Indexed(107),
            legal_move_bg: Color::Indexed(65),
            last_move_light: Color::Indexed(185),
            last_move_dark: Color::Indexed(142),
            check_bg: Color::Indexed(160),
            cursor_bg: Color::Indexed(68),

            accent: Color::Indexed(74),
            text_primary: Color::Indexed(251),
            text_dim: Color::Indexed(243),
            text_bright: Color::Indexed(255),
            border_focused: Color::Indexed(74),
            border_dim: Color::Indexed(238),

            mode_normal: Color::Indexed(243),
            mode_input: Color::Indexed(220),
            mode_command: Color::Indexed(80),
        }
    }

    /// Detect terminal color support and return the best theme.
    pub fn detect() -> Self {
        match std::env::var("COLORTERM").as_deref() {
            Ok("truecolor") | Ok("24bit") => Self::default_truecolor(),
            _ => Self::default_256(),
        }
    }

    /// Get the square background color for a given file/rank.
    pub fn square_bg(&self, file: u8, rank: u8) -> Color {
        if (file + rank) % 2 == 0 {
            self.dark_square
        } else {
            self.light_square
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build 2>&1`
Expected: Compiles (theme.rs is not yet used, just needs to compile)

- [ ] **Step 3: Commit**

```bash
git add src/theme.rs
git commit -m "feat: add theme system with truecolor and 256-color support"
```

---

### Task 6: App State Machine and Terminal Setup

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`

This sets up the ratatui terminal, event loop, and the screen state machine (MainMenu → InGame → PostGame).

- [ ] **Step 1: Create `src/app.rs`**

```rust
use cozy_chess::Color as ChessColor;
use crate::game::state::{GameState, GameStatus};
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

    // Main menu
    pub menu_selection: usize,

    // In-game state
    pub game: GameState,
    pub cursor_file: u8,  // 0-7
    pub cursor_rank: u8,  // 0-7
    pub selected_square: Option<cozy_chess::Square>,
    pub legal_moves_for_selected: Vec<cozy_chess::Move>,
    pub last_move: Option<(cozy_chess::Square, cozy_chess::Square)>,
    pub board_flipped: bool,
    pub input_buffer: String,
    pub command_buffer: String,
    pub status_message: String,
    pub move_list_scroll: usize,
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
            cursor_file: 4, // start on e-file
            cursor_rank: 0, // rank 1
            selected_square: None,
            legal_moves_for_selected: Vec::new(),
            last_move: None,
            board_flipped: false,
            input_buffer: String::new(),
            command_buffer: String::new(),
            status_message: String::new(),
            move_list_scroll: 0,
        }
    }

    pub fn start_new_game(&mut self) {
        self.game = GameState::new();
        self.screen = Screen::InGame;
        self.mode = InputMode::Normal;
        self.cursor_file = 4;
        self.cursor_rank = 0;
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.last_move = None;
        self.input_buffer.clear();
        self.command_buffer.clear();
        self.status_message.clear();
        self.move_list_scroll = 0;
        self.board_flipped = false;
    }

    pub fn select_square(&mut self, sq: cozy_chess::Square) {
        // If we have a piece selected and this is a legal destination, make the move.
        if self.selected_square.is_some() {
            let legal_dest: Vec<cozy_chess::Move> = self.legal_moves_for_selected
                .iter()
                .filter(|m| m.to == sq)
                .copied()
                .collect();

            if legal_dest.len() == 1 {
                self.make_move(legal_dest[0]);
                return;
            } else if legal_dest.len() > 1 {
                // Promotion — need to pick. For now, default to queen.
                let queen_move = legal_dest.iter().find(|m| m.promotion == Some(cozy_chess::Piece::Queen));
                if let Some(&mv) = queen_move {
                    self.make_move(mv);
                    return;
                }
            }
        }

        // Try to select a piece on this square.
        let side = self.game.side_to_move();
        if self.game.board().colors(side).has(sq) {
            self.selected_square = Some(sq);
            self.legal_moves_for_selected = self.game.legal_moves_from(sq);
            if self.legal_moves_for_selected.is_empty() {
                self.selected_square = None;
            }
        } else {
            self.deselect();
        }
    }

    pub fn deselect(&mut self) {
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
    }

    pub fn make_move(&mut self, mv: cozy_chess::Move) {
        let from = mv.from;
        let to = mv.to;
        if self.game.try_make_move(mv).is_ok() {
            self.last_move = Some((from, to));
            self.deselect();
            self.input_buffer.clear();

            // Auto-scroll move list.
            let total_moves = self.game.move_history().len();
            self.move_list_scroll = total_moves.saturating_sub(10);

            // Check game over.
            if let GameStatus::Finished(_) = self.game.status() {
                self.screen = Screen::PostGame;
            }
        }
    }

    /// Get the square under the cursor, accounting for board flip.
    pub fn cursor_square(&self) -> cozy_chess::Square {
        let (file, rank) = if self.board_flipped {
            (7 - self.cursor_file, 7 - self.cursor_rank)
        } else {
            (self.cursor_file, self.cursor_rank)
        };
        cozy_chess::Square::index(rank as usize * 8 + file as usize)
    }

    pub fn move_cursor(&mut self, df: i8, dr: i8) {
        let new_file = (self.cursor_file as i8 + df).clamp(0, 7) as u8;
        let new_rank = (self.cursor_rank as i8 + dr).clamp(0, 7) as u8;
        self.cursor_file = new_file;
        self.cursor_rank = new_rank;
    }

    pub fn menu_items() -> &'static [&'static str] {
        &["Local Game", "Quit"]
    }
}
```

- [ ] **Step 2: Rewrite `src/main.rs` with terminal setup and event loop**

```rust
mod app;
mod theme;

use app::{App, InputMode, Screen};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
    // Setup terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| chesstui::ui::draw(frame, app))?;

        if app.should_quit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                chesstui::input::handle_key(app, key);
            }
        }
    }
}
```

- [ ] **Step 3: Update `src/lib.rs`**

```rust
pub mod game;
pub mod theme;
pub mod ui;
pub mod input;
```

Note: `ui` and `input` modules will be created in the next tasks. For now, create stubs so it compiles.

- [ ] **Step 4: Create stub `src/ui/mod.rs`**

```rust
use ratatui::Frame;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    // Placeholder — will be implemented in subsequent tasks.
    let area = frame.area();
    let block = ratatui::widgets::Block::default()
        .title("ChessTUI")
        .borders(ratatui::widgets::Borders::ALL);
    frame.render_widget(block, area);
}
```

- [ ] **Step 5: Create stub `src/input.rs`**

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, InputMode, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Ctrl+C always quits.
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    match app.screen {
        Screen::MainMenu => handle_menu_key(app, key),
        Screen::InGame => handle_game_key(app, key),
        Screen::PostGame => handle_postgame_key(app, key),
    }
}

fn handle_menu_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => {
            let items = App::menu_items();
            app.menu_selection = (app.menu_selection + 1) % items.len();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let items = App::menu_items();
            app.menu_selection = (app.menu_selection + items.len() - 1) % items.len();
        }
        KeyCode::Enter => {
            match app.menu_selection {
                0 => app.start_new_game(), // Local Game
                1 => app.should_quit = true, // Quit
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_game_key(app: &mut App, key: KeyEvent) {
    match app.mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Input => handle_input_mode(app, key),
        InputMode::Command => handle_command_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // VIM navigation
        KeyCode::Char('h') | KeyCode::Left => app.move_cursor(-1, 0),
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor(0, -1),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor(0, 1),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(1, 0),

        // Select / confirm
        KeyCode::Enter | KeyCode::Char(' ') => {
            let sq = app.cursor_square();
            app.select_square(sq);
        }

        // Deselect
        KeyCode::Esc => app.deselect(),

        // Flip board
        KeyCode::Char('f') => app.board_flipped = !app.board_flipped,

        // Enter command mode
        KeyCode::Char(':') => {
            app.mode = InputMode::Command;
            app.command_buffer.clear();
        }

        // Algebraic input — typing a-h or piece letter enters Input mode
        KeyCode::Char(c) if ('a'..='h').contains(&c) || "NBRQKO".contains(c.to_ascii_uppercase()) => {
            app.mode = InputMode::Input;
            app.input_buffer.clear();
            // Re-feed this character through the input handler.
            handle_input_char(app, c);
        }

        _ => {}
    }
}

fn handle_input_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Char(c) => handle_input_char(app, c),
        KeyCode::Backspace => {
            app.input_buffer.pop();
            if app.input_buffer.is_empty() {
                app.mode = InputMode::Normal;
            }
        }
        _ => {}
    }
}

fn handle_input_char(app: &mut App, ch: char) {
    use crate::game::move_input::{MoveInputParser, InputResult};

    app.input_buffer.push(ch);

    let mut parser = MoveInputParser::new(app.game.board());
    // Feed the entire buffer to the parser.
    let mut result = InputResult::NeedMore(0);
    for c in app.input_buffer.chars() {
        result = parser.feed(c);
        match &result {
            InputResult::Exact(_) | InputResult::NoMatch => break,
            InputResult::NeedMore(_) => continue,
        }
    }

    match result {
        InputResult::Exact(mv) => {
            app.make_move(mv);
            app.mode = InputMode::Normal;
        }
        InputResult::NoMatch => {
            app.status_message = format!("No match for '{}'", app.input_buffer);
            app.input_buffer.clear();
            app.mode = InputMode::Normal;
        }
        InputResult::NeedMore(_) => {
            // Stay in input mode, waiting for more characters.
        }
    }
}

fn handle_command_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
            app.command_buffer.clear();
        }
        KeyCode::Enter => {
            execute_command(app);
            app.mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.command_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.command_buffer.pop();
            if app.command_buffer.is_empty() {
                app.mode = InputMode::Normal;
            }
        }
        _ => {}
    }
}

fn execute_command(app: &mut App) {
    let cmd = app.command_buffer.trim().to_lowercase();
    match cmd.as_str() {
        "q" | "quit" => app.should_quit = true,
        "resign" | "res" => {
            app.status_message = "Game resigned.".into();
            app.screen = Screen::PostGame;
        }
        "flip" | "f" => app.board_flipped = !app.board_flipped,
        "new" | "n" => app.start_new_game(),
        _ => {
            app.status_message = format!("Unknown command: {}", cmd);
        }
    }
    app.command_buffer.clear();
}

fn handle_postgame_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') => app.start_new_game(),
        KeyCode::Char('m') => {
            app.screen = Screen::MainMenu;
            app.menu_selection = 0;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo build 2>&1`
Expected: Compiles. The app should launch, show a blank bordered box, and quit with Ctrl+C.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/main.rs src/lib.rs src/input.rs src/ui/
git commit -m "feat: add app state machine, terminal setup, input handling"
```

---

### Task 7: Chess Board Widget

**Files:**
- Create: `src/ui/board.rs`
- Modify: `src/ui/mod.rs`

The heart of the TUI — renders the 8x8 chess board with pieces, highlighting, and cursor.

- [ ] **Step 1: Create `src/ui/board.rs`**

```rust
use cozy_chess::{Board, Color as ChessColor, Move, Piece, Square};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use crate::theme::Theme;

pub struct ChessBoardWidget<'a> {
    board: &'a Board,
    theme: &'a Theme,
    flipped: bool,
    cursor: Option<(u8, u8)>,       // (file, rank) in display coords
    selected: Option<Square>,
    legal_destinations: &'a [Move],
    last_move: Option<(Square, Square)>,
}

impl<'a> ChessBoardWidget<'a> {
    pub fn new(board: &'a Board, theme: &'a Theme) -> Self {
        Self {
            board,
            theme,
            flipped: false,
            cursor: None,
            selected: None,
            legal_destinations: &[],
            last_move: None,
        }
    }

    pub fn flipped(mut self, flipped: bool) -> Self {
        self.flipped = flipped;
        self
    }

    pub fn cursor(mut self, file: u8, rank: u8) -> Self {
        self.cursor = Some((file, rank));
        self
    }

    pub fn selected(mut self, sq: Option<Square>) -> Self {
        self.selected = sq;
        self
    }

    pub fn legal_moves(mut self, moves: &'a [Move]) -> Self {
        self.legal_destinations = moves;
        self
    }

    pub fn last_move(mut self, last: Option<(Square, Square)>) -> Self {
        self.last_move = last;
        self
    }
}

impl Widget for ChessBoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Board needs at least 19 cols (2*8 squares + 3 for labels) and 10 rows (8 + 1 label + 1 padding).
        if area.width < 19 || area.height < 9 {
            return;
        }

        let board_x = area.x + 2; // offset for rank labels
        let board_y = area.y;

        let in_check = !self.board.checkers().is_empty();
        let king_sq = if in_check {
            let side = self.board.side_to_move();
            let king_bb = self.board.pieces(Piece::King) & self.board.colors(side);
            king_bb.into_iter().next()
        } else {
            None
        };

        for display_rank in 0..8u8 {
            for display_file in 0..8u8 {
                let (file, rank) = if self.flipped {
                    (7 - display_file, display_rank)
                } else {
                    (display_file, 7 - display_rank)
                };

                let sq = Square::index(rank as usize * 8 + file as usize);

                // Determine background color (layer priority: check > selected > legal > last_move > base).
                let is_selected = self.selected == Some(sq);
                let is_legal_dest = self.legal_destinations.iter().any(|m| m.to == sq);
                let is_last_move = self.last_move.map_or(false, |(from, to)| sq == from || sq == to);
                let is_cursor = self.cursor == Some((display_file, display_rank));
                let is_check_king = king_sq == Some(sq);

                let base_bg = self.theme.square_bg(file, rank);

                let bg = if is_check_king {
                    self.theme.check_bg
                } else if is_cursor {
                    self.theme.cursor_bg
                } else if is_selected {
                    self.theme.selected_bg
                } else if is_legal_dest {
                    self.theme.legal_move_bg
                } else if is_last_move {
                    if (file + rank) % 2 == 0 { self.theme.last_move_dark } else { self.theme.last_move_light }
                } else {
                    base_bg
                };

                // Get piece character.
                let piece_char = self.piece_char_at(sq);
                let piece_color = if let Some(piece_chess_color) = self.piece_color_at(sq) {
                    match piece_chess_color {
                        ChessColor::White => self.theme.white_piece,
                        ChessColor::Black => self.theme.black_piece,
                    }
                } else {
                    bg // No piece, use bg color for the dot or space
                };

                let display_str = if let Some(ch) = piece_char {
                    format!("{} ", ch)
                } else if is_legal_dest {
                    "\u{00B7} ".to_string() // middle dot for legal move squares
                } else {
                    "  ".to_string()
                };

                let x = board_x + display_file as u16 * 2;
                let y = board_y + display_rank as u16;

                if x + 1 < area.x + area.width && y < area.y + area.height {
                    let style = Style::default().fg(piece_color).bg(bg);
                    buf.set_string(x, y, &display_str, style);
                }
            }

            // Rank label.
            let rank_label = if self.flipped {
                format!("{}", display_rank + 1)
            } else {
                format!("{}", 8 - display_rank)
            };
            let label_y = board_y + display_rank as u16;
            if label_y < area.y + area.height {
                buf.set_string(area.x, label_y, &rank_label, Style::default().fg(self.theme.text_dim));
            }
        }

        // File labels.
        let file_y = board_y + 8;
        if file_y < area.y + area.height {
            for display_file in 0..8u8 {
                let file = if self.flipped { 7 - display_file } else { display_file };
                let label = format!("{} ", (b'a' + file) as char);
                let x = board_x + display_file as u16 * 2;
                if x + 1 < area.x + area.width {
                    buf.set_string(x, file_y, &label, Style::default().fg(self.theme.text_dim));
                }
            }
        }
    }
}

impl ChessBoardWidget<'_> {
    fn piece_char_at(&self, sq: Square) -> Option<char> {
        let is_white = self.board.colors(ChessColor::White).has(sq);
        let is_black = self.board.colors(ChessColor::Black).has(sq);
        if !is_white && !is_black {
            return None;
        }

        let piece = if self.board.pieces(Piece::Pawn).has(sq) { Piece::Pawn }
            else if self.board.pieces(Piece::Knight).has(sq) { Piece::Knight }
            else if self.board.pieces(Piece::Bishop).has(sq) { Piece::Bishop }
            else if self.board.pieces(Piece::Rook).has(sq) { Piece::Rook }
            else if self.board.pieces(Piece::Queen).has(sq) { Piece::Queen }
            else if self.board.pieces(Piece::King).has(sq) { Piece::King }
            else { return None; };

        let ch = if is_white {
            match piece {
                Piece::King => '\u{2654}',   // ♔
                Piece::Queen => '\u{2655}',  // ♕
                Piece::Rook => '\u{2656}',   // ♖
                Piece::Bishop => '\u{2657}', // ♗
                Piece::Knight => '\u{2658}', // ♘
                Piece::Pawn => '\u{2659}',   // ♙
            }
        } else {
            match piece {
                Piece::King => '\u{265A}',   // ♚
                Piece::Queen => '\u{265B}',  // ♛
                Piece::Rook => '\u{265C}',   // ♜
                Piece::Bishop => '\u{265D}', // ♝
                Piece::Knight => '\u{265E}', // ♞
                Piece::Pawn => '\u{265F}',   // ♟
            }
        };

        Some(ch)
    }

    fn piece_color_at(&self, sq: Square) -> Option<ChessColor> {
        if self.board.colors(ChessColor::White).has(sq) {
            Some(ChessColor::White)
        } else if self.board.colors(ChessColor::Black).has(sq) {
            Some(ChessColor::Black)
        } else {
            None
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/ui/board.rs
git commit -m "feat: add chess board widget with highlights and Unicode pieces"
```

---

### Task 8: Supporting UI Widgets (Move List, Captured, Command Bar)

**Files:**
- Create: `src/ui/move_list.rs`
- Create: `src/ui/captured.rs`
- Create: `src/ui/command_bar.rs`
- Create: `src/ui/help.rs`

- [ ] **Step 1: Create `src/ui/move_list.rs`**

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use crate::game::state::MoveRecord;
use crate::game::notation::to_algebraic;
use crate::theme::Theme;

pub struct MoveListWidget<'a> {
    moves: &'a [MoveRecord],
    theme: &'a Theme,
    scroll: usize,
}

impl<'a> MoveListWidget<'a> {
    pub fn new(moves: &'a [MoveRecord], theme: &'a Theme, scroll: usize) -> Self {
        Self { moves, theme, scroll }
    }
}

impl Widget for MoveListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Moves ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_dim));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        for (i, pair) in self.moves.chunks(2).enumerate() {
            let move_num = i + 1;
            let white_san = to_algebraic(&pair[0].previous_board, &pair[0].mv);
            let black_san = if pair.len() > 1 {
                to_algebraic(&pair[1].previous_board, &pair[1].mv)
            } else {
                "...".to_string()
            };

            let line = Line::from(vec![
                Span::styled(format!("{:>2}. ", move_num), Style::default().fg(self.theme.text_dim)),
                Span::styled(format!("{:<8}", white_san), Style::default().fg(self.theme.text_primary)),
                Span::styled(black_san, Style::default().fg(self.theme.text_primary)),
            ]);
            lines.push(line);
        }

        // Apply scroll.
        let visible_lines: Vec<Line> = lines.into_iter()
            .skip(self.scroll / 2)
            .take(inner.height as usize)
            .collect();

        let paragraph = Paragraph::new(visible_lines);
        paragraph.render(inner, buf);
    }
}
```

- [ ] **Step 2: Create `src/ui/captured.rs`**

```rust
use cozy_chess::{Color as ChessColor, Piece};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};
use crate::theme::Theme;

pub struct CapturedWidget<'a> {
    white_captured: &'a [Piece],  // Pieces captured BY White
    black_captured: &'a [Piece],  // Pieces captured BY Black
    theme: &'a Theme,
}

impl<'a> CapturedWidget<'a> {
    pub fn new(white_captured: &'a [Piece], black_captured: &'a [Piece], theme: &'a Theme) -> Self {
        Self { white_captured, black_captured, theme }
    }

    fn pieces_to_string(pieces: &[Piece]) -> String {
        let mut sorted: Vec<Piece> = pieces.to_vec();
        sorted.sort_by_key(|p| match p {
            Piece::Queen => 0, Piece::Rook => 1, Piece::Bishop => 2,
            Piece::Knight => 3, Piece::Pawn => 4, Piece::King => 5,
        });
        sorted.iter().map(|p| match p {
            Piece::Queen => '♛', Piece::Rook => '♜', Piece::Bishop => '♝',
            Piece::Knight => '♞', Piece::Pawn => '♟', Piece::King => '♚',
        }).collect()
    }

    fn material_diff(white: &[Piece], black: &[Piece]) -> i32 {
        let val = |p: &Piece| match p {
            Piece::Queen => 9, Piece::Rook => 5, Piece::Bishop => 3,
            Piece::Knight => 3, Piece::Pawn => 1, Piece::King => 0,
        };
        let white_val: i32 = white.iter().map(val).sum();
        let black_val: i32 = black.iter().map(val).sum();
        white_val - black_val
    }
}

impl Widget for CapturedWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Captured ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_dim));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 { return; }

        let diff = Self::material_diff(self.white_captured, self.black_captured);
        let diff_str = if diff > 0 { format!(" +{}", diff) } else if diff < 0 { format!(" {}", diff) } else { String::new() };

        let w_str = Self::pieces_to_string(self.white_captured);
        let b_str = Self::pieces_to_string(self.black_captured);

        let w_line = format!("W: {}{}", w_str, if diff > 0 { &diff_str } else { "" });
        let b_line = format!("B: {}{}", b_str, if diff < 0 { &diff_str } else { "" });

        buf.set_string(inner.x, inner.y, &w_line, Style::default().fg(self.theme.text_primary));
        if inner.height > 1 {
            buf.set_string(inner.x, inner.y + 1, &b_line, Style::default().fg(self.theme.text_primary));
        }
    }
}
```

- [ ] **Step 3: Create `src/ui/command_bar.rs`**

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use crate::app::InputMode;
use crate::theme::Theme;

pub struct CommandBarWidget<'a> {
    mode: InputMode,
    input_buffer: &'a str,
    command_buffer: &'a str,
    status_message: &'a str,
    is_player_turn: bool,
    theme: &'a Theme,
}

impl<'a> CommandBarWidget<'a> {
    pub fn new(
        mode: InputMode,
        input_buffer: &'a str,
        command_buffer: &'a str,
        status_message: &'a str,
        is_player_turn: bool,
        theme: &'a Theme,
    ) -> Self {
        Self { mode, input_buffer, command_buffer, status_message, is_player_turn, theme }
    }
}

impl Widget for CommandBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 { return; }

        // Row 1: Input/status line.
        let input_y = area.y;
        let input_text = match self.mode {
            InputMode::Command => format!(":{}", self.command_buffer),
            InputMode::Input => format!(" {}", self.input_buffer),
            InputMode::Normal => {
                if !self.status_message.is_empty() {
                    self.status_message.to_string()
                } else {
                    "  hjkl move | Enter select | : command | ? help".to_string()
                }
            }
        };
        buf.set_string(area.x, input_y, &input_text, Style::default().fg(self.theme.text_primary));

        // Row 2: Mode indicator.
        let mode_y = area.y + 1;
        let (mode_str, mode_color) = match self.mode {
            InputMode::Normal => ("-- NORMAL --", self.theme.mode_normal),
            InputMode::Input => ("-- INPUT --", self.theme.mode_input),
            InputMode::Command => ("-- COMMAND --", self.theme.mode_command),
        };
        buf.set_string(area.x, mode_y, mode_str, Style::default().fg(mode_color));

        // Right-aligned version.
        let version = "chesstui v0.1.0";
        let version_x = area.x + area.width.saturating_sub(version.len() as u16);
        buf.set_string(version_x, mode_y, version, Style::default().fg(self.theme.text_dim));
    }
}
```

- [ ] **Step 4: Create `src/ui/help.rs`**

```rust
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use crate::theme::Theme;

pub struct HelpOverlay<'a> {
    theme: &'a Theme,
}

impl<'a> HelpOverlay<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }
}

impl Widget for HelpOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Center a 60x16 box.
        let w = 60u16.min(area.width.saturating_sub(4));
        let h = 16u16.min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let popup = Rect::new(x, y, w, h);

        Clear.render(popup, buf);

        let block = Block::default()
            .title(" Key Bindings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent));
        let inner = block.inner(popup);
        block.render(popup, buf);

        let help_text = vec![
            Line::from(Span::styled("NAVIGATION", Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD))),
            Line::from("  h/j/k/l     Move cursor"),
            Line::from("  Enter/Space  Select piece / confirm move"),
            Line::from("  Esc          Deselect / cancel"),
            Line::from("  f            Flip board"),
            Line::from(""),
            Line::from(Span::styled("MOVE INPUT", Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD))),
            Line::from("  Type algebraic: e4, Nf3, OO"),
            Line::from("  Type squares: e2e4"),
            Line::from("  Auto-executes when unique"),
            Line::from(""),
            Line::from(Span::styled("COMMANDS (:)", Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD))),
            Line::from("  :resign :flip :new :quit"),
            Line::from(""),
            Line::from(Span::styled("Press ? or Esc to close", Style::default().fg(self.theme.text_dim))),
        ];

        let paragraph = Paragraph::new(help_text);
        paragraph.render(inner, buf);
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add src/ui/move_list.rs src/ui/captured.rs src/ui/command_bar.rs src/ui/help.rs
git commit -m "feat: add move list, captured pieces, command bar, and help widgets"
```

---

### Task 9: Main Menu and In-Game Layout Rendering

**Files:**
- Create: `src/ui/menu.rs`
- Create: `src/ui/game.rs`
- Create: `src/ui/postgame.rs`
- Modify: `src/ui/mod.rs`

Wire everything together — the actual screen rendering.

- [ ] **Step 1: Create `src/ui/menu.rs`**

```rust
use ratatui::{
    layout::{Constraint, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;

const ASCII_TITLE: &str = r#"   _____ _                    _____ _   _ ___
  / ____| |                  |_   _| | | |_ _|
 | |    | |__   ___  ___ ___  | | | | | || |
 | |    | '_ \ / _ \/ __/ __| | | | | | || |
 | |____| | | |  __/\__ \__ \ | | | |_| || |
  \_____|_| |_|\___||___/___/ |_|  \___/|___|"#;

pub fn draw_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Min(8),       // Title
        Constraint::Length(4),    // Menu items
        Constraint::Fill(1),     // Spacer
        Constraint::Length(1),   // Hints
        Constraint::Length(1),   // Status bar
    ]).split(area);

    // Title.
    let title = if area.width >= 50 {
        ASCII_TITLE
    } else {
        "C H E S S T U I"
    };
    let title_para = Paragraph::new(title)
        .alignment(Alignment::Center)
        .style(Style::default().fg(app.theme.accent));
    frame.render_widget(title_para, layout[0]);

    // Menu items.
    let items = App::menu_items();
    let mut menu_lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let style = if i == app.menu_selection {
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text_dim)
        };
        let prefix = if i == app.menu_selection { "> " } else { "  " };
        menu_lines.push(Line::from(Span::styled(format!("{}{}", prefix, item), style)));
    }
    let menu = Paragraph::new(menu_lines).alignment(Alignment::Center);
    frame.render_widget(menu, layout[1]);

    // Hints.
    let hints = Paragraph::new("  j/k navigate   Enter select   q quit")
        .style(Style::default().fg(app.theme.text_dim));
    frame.render_widget(hints, layout[3]);

    // Status bar.
    let mode = format!("-- NORMAL --");
    let version = format!("chesstui v0.1.0   {}x{}", area.width, area.height);
    let status = Line::from(vec![
        Span::styled(mode, Style::default().fg(app.theme.mode_normal)),
        Span::raw("  "),
        Span::styled(
            format!("{:>width$}", version, width = (area.width as usize).saturating_sub(16)),
            Style::default().fg(app.theme.text_dim),
        ),
    ]);
    frame.render_widget(Paragraph::new(status), layout[4]);
}
```

- [ ] **Step 2: Create `src/ui/game.rs`**

```rust
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::board::ChessBoardWidget;
use crate::ui::move_list::MoveListWidget;
use crate::ui::captured::CapturedWidget;
use crate::ui::command_bar::CommandBarWidget;

pub fn draw_game(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Top-level vertical: player info | main content | command bar
    let main_layout = Layout::vertical([
        Constraint::Length(1),    // Turn indicator
        Constraint::Min(9),      // Board + side panels
        Constraint::Length(2),   // Command bar
    ]).split(area);

    // Turn indicator.
    let side = app.game.side_to_move();
    let turn_text = match side {
        cozy_chess::Color::White => "White to move",
        cozy_chess::Color::Black => "Black to move",
    };
    let turn = Paragraph::new(Line::from(vec![
        Span::styled(" \u{25CF} ", Style::default().fg(match side {
            cozy_chess::Color::White => app.theme.white_piece,
            cozy_chess::Color::Black => app.theme.black_piece,
        })),
        Span::styled(turn_text, Style::default().fg(app.theme.text_primary)),
    ]));
    frame.render_widget(turn, main_layout[0]);

    // Main content: board (left) + panels (right).
    let content_layout = Layout::horizontal([
        Constraint::Length(20),   // Board (2*8 + rank labels + padding)
        Constraint::Min(20),     // Right panel
    ]).split(main_layout[1]);

    // Board.
    let board_widget = ChessBoardWidget::new(app.game.board(), &app.theme)
        .flipped(app.board_flipped)
        .cursor(app.cursor_file, app.cursor_rank)
        .selected(app.selected_square)
        .legal_moves(&app.legal_moves_for_selected)
        .last_move(app.last_move);
    frame.render_widget(board_widget, content_layout[0]);

    // Right panel: move list + captured.
    let right_layout = Layout::vertical([
        Constraint::Min(5),      // Move list
        Constraint::Length(4),   // Captured pieces
    ]).split(content_layout[1]);

    let move_list = MoveListWidget::new(
        app.game.move_history(),
        &app.theme,
        app.move_list_scroll,
    );
    frame.render_widget(move_list, right_layout[0]);

    let captured = CapturedWidget::new(
        app.captured_by_white(),
        app.captured_by_black(),
        &app.theme,
    );
    frame.render_widget(captured, right_layout[1]);

    // Command bar.
    let cmd_bar = CommandBarWidget::new(
        app.mode,
        &app.input_buffer,
        &app.command_buffer,
        &app.status_message,
        true,
        &app.theme,
    );
    frame.render_widget(cmd_bar, main_layout[2]);
}
```

- [ ] **Step 3: Create `src/ui/postgame.rs`**

```rust
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::game::state::{GameResult, GameStatus};

pub fn draw_postgame(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Length(3),  // Result
        Constraint::Min(9),    // Board
        Constraint::Length(3), // Actions
        Constraint::Length(1), // Status
    ]).split(area);

    // Result banner.
    let result_text = match app.game.status() {
        GameStatus::Finished(GameResult::Checkmate(winner)) => {
            format!("CHECKMATE -- {} wins!", match winner {
                cozy_chess::Color::White => "White",
                cozy_chess::Color::Black => "Black",
            })
        }
        GameStatus::Finished(GameResult::Stalemate) => "STALEMATE -- Draw".to_string(),
        GameStatus::Finished(GameResult::Resignation(winner)) => {
            format!("{} wins by resignation", match winner {
                cozy_chess::Color::White => "White",
                cozy_chess::Color::Black => "Black",
            })
        }
        GameStatus::Finished(GameResult::DrawByInsufficientMaterial) => "Draw -- insufficient material".to_string(),
        _ => "Game Over".to_string(),
    };

    let result = Paragraph::new(result_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD));
    frame.render_widget(result, layout[0]);

    // Board in final position.
    let board_layout = Layout::horizontal([
        Constraint::Length(20),
        Constraint::Fill(1),
    ]).split(layout[1]);

    let board_widget = crate::ui::board::ChessBoardWidget::new(app.game.board(), &app.theme)
        .flipped(app.board_flipped)
        .last_move(app.last_move);
    frame.render_widget(board_widget, board_layout[0]);

    // Game stats on the right.
    let stats = vec![
        Line::from(format!("  Moves: {}", app.game.move_history().len())),
        Line::from(""),
    ];
    let stats_para = Paragraph::new(stats).style(Style::default().fg(app.theme.text_primary));
    frame.render_widget(stats_para, board_layout[1]);

    // Actions.
    let actions = Paragraph::new("  [n] New Game   [m] Menu   [q] Quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(app.theme.text_dim));
    frame.render_widget(actions, layout[2]);
}
```

- [ ] **Step 4: Update `src/ui/mod.rs` to wire everything**

```rust
pub mod board;
pub mod menu;
pub mod game;
pub mod move_list;
pub mod captured;
pub mod command_bar;
pub mod postgame;
pub mod help;

use ratatui::Frame;
use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::MainMenu => menu::draw_menu(frame, app),
        Screen::InGame => game::draw_game(frame, app),
        Screen::PostGame => postgame::draw_postgame(frame, app),
    }
}
```

- [ ] **Step 5: Add helper methods to App for captured pieces access**

Add to `src/app.rs`:

```rust
    pub fn captured_by_white(&self) -> &[cozy_chess::Piece] {
        self.game.captured_by(cozy_chess::Color::White)
    }

    pub fn captured_by_black(&self) -> &[cozy_chess::Piece] {
        self.game.captured_by(cozy_chess::Color::Black)
    }
```

- [ ] **Step 6: Add help overlay toggle to input handler**

In `src/input.rs`, add to `handle_normal_mode`:

```rust
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }
```

Add `pub show_help: bool` field to `App` in `src/app.rs`, defaulting to `false`.

In `src/ui/mod.rs`'s `draw` function, after the screen draw:

```rust
    if app.show_help {
        help::HelpOverlay::new(&app.theme).render(frame.area(), frame.buffer_mut());
    }
```

- [ ] **Step 7: Verify it compiles and runs**

Run: `cargo build 2>&1`
Run: `cargo run` (manually test: see menu, press Enter for Local Game, see board, move pieces with hjkl + Enter, type `e4`, press `:` then `quit`)

Expected: Full playable game in the terminal.

- [ ] **Step 8: Commit**

```bash
git add src/ui/ src/app.rs src/input.rs
git commit -m "feat: complete TUI with menu, game board, move list, and command bar"
```

---

### Task 10: Help Overlay Toggle and Polish

**Files:**
- Modify: `src/input.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add promotion popup**

When a pawn reaches the 8th rank and the user is using cursor selection (not algebraic), show a promotion choice. Add to `app.rs` `select_square`:

Replace the promotion default-to-queen logic with:

```rust
            if legal_dest.len() > 1 {
                // Check if these are promotion moves.
                let promos: Vec<_> = legal_dest.iter().filter(|m| m.promotion.is_some()).collect();
                if promos.len() > 1 {
                    // Set pending promotion state.
                    app.pending_promotion = Some(legal_dest);
                    app.promotion_choice = 0; // 0=Q, 1=R, 2=B, 3=N
                    return;
                }
            }
```

Add promotion fields to `App`:

```rust
    pub pending_promotion: Option<Vec<cozy_chess::Move>>,
    pub promotion_choice: usize,
```

Handle promotion keys in normal mode:

```rust
        // In handle_normal_mode, before other key handling:
        if app.pending_promotion.is_some() {
            match key.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    app.promotion_choice = app.promotion_choice.saturating_sub(1);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    app.promotion_choice = (app.promotion_choice + 1).min(3);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let piece = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight][app.promotion_choice];
                    if let Some(moves) = app.pending_promotion.take() {
                        if let Some(mv) = moves.iter().find(|m| m.promotion == Some(piece)) {
                            app.make_move(*mv);
                        }
                    }
                }
                KeyCode::Esc => { app.pending_promotion = None; }
                _ => {}
            }
            return;
        }
```

- [ ] **Step 2: Clear status message after a few ticks**

In `App`, add a counter. In the event loop after each draw, if `status_message` is non-empty, decrement a counter and clear when it reaches 0. Simple approach: clear on next keypress (already happens naturally since the input handler sets new messages).

- [ ] **Step 3: Final compile and manual test**

Run: `cargo run`

Test checklist:
- [ ] Menu renders with ASCII art
- [ ] j/k navigates menu, Enter starts game
- [ ] Board renders with colored squares and Unicode pieces
- [ ] hjkl moves cursor (highlighted square)
- [ ] Enter on own piece selects it, shows legal moves
- [ ] Enter on legal destination makes the move
- [ ] Typing `e4` in normal mode enters INPUT mode and executes the move
- [ ] `:resign` enters command mode and ends the game
- [ ] `f` flips the board
- [ ] `?` shows help overlay
- [ ] Post-game screen shows result
- [ ] `n` starts new game from post-game
- [ ] `q` quits

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add promotion popup and final polish for Phase 1"
```

---

### Task 11: Push to Remote

- [ ] **Step 1: Push to GitHub**

```bash
git push -u origin main
```

Expected: All code pushed to `git@github.com:millerc13/chesstui.git`

---

## Summary

After completing all 11 tasks, you will have:

- A Rust binary that launches a chess TUI in any terminal
- Main menu → local hotseat game → post-game flow
- VIM-style modal input (NORMAL/INPUT/COMMAND modes)
- Three move input methods: cursor selection, algebraic notation (auto-execute), and square-to-square
- Full chess rules via cozy-chess (castling, en passant, promotion, check/checkmate/stalemate)
- Unicode chess pieces with colored board squares
- Move history panel, captured pieces display, command bar
- Help overlay, board flipping, resignation
- Promotion picker popup
- All backed by tests for game state, notation, and input parsing
