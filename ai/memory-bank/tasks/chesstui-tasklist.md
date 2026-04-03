# ChessTUI Development Roadmap

## Project Context

**Type**: Solo developer passion project -- multiplayer chess TUI in Rust
**Binary name**: `chesstui` (single binary: `chesstui`, `chesstui server`, `chesstui client`)
**Server port**: 7600 (TCP), self-hosted on static IP
**State directory**: `/var/lib/chesstui`

**Tech Stack**:
- Rust (2021 edition)
- `ratatui` -- TUI rendering
- `crossterm` -- terminal backend
- `tokio` -- async runtime, TCP networking
- `cozy-chess` -- bitboard move generation (already chosen, see `src/engine/mod.rs`)
- `serde` / `serde_json` -- serialization
- `clap` -- CLI argument parsing

**What already exists** (as of project start):
- `.github/workflows/ci.yml` -- fmt, clippy, test matrix (ubuntu/macos/windows), smoke test
- `.github/workflows/release.yml` -- 5-target build matrix, GitHub Release, Homebrew tap, crates.io publish, auto-deploy server
- `deploy/` -- systemd service file (port 7600), setup-server.sh, deploy.sh (with cross-compilation support)
- `scripts/` -- install.sh (curl installer with checksum verification), install.ps1, pre-commit hook, setup-hooks.sh
- `src/engine/mod.rs` -- AI engine skeleton (ChessAI struct, AIResult, module declarations for evaluation/search/difficulty/opening_book/personality/tables)

**What does NOT exist yet**:
- `Cargo.toml`
- Chess rules / board representation (delegated to `cozy-chess`)
- TUI rendering
- Networking / protocol
- Any playable code

---

## Critical Path

```
Phase 1 (Chess Core) --> Phase 2 (TUI) --> Phase 3 (AI) --> Phase 4 (Networking) --> Phase 5 (Features)
                                                                                          |
Phase 6 (Polish) runs in parallel from Phase 2 onward, but final push after Phase 5 ------+
```

**Prototype first**: Get a playable local game as fast as possible (Phase 1.3 + Phase 2.3). A human-vs-human hotseat game with basic board rendering proves the core loop works. Target: weekend #1.

**Second milestone**: Human-vs-AI locally (Phase 3.4). This is the "show it to friends" milestone. Target: weekend #3-4.

**Third milestone**: Two people play over the network (Phase 4.5). This is where the project becomes real. Target: weekend #6-7.

---

## Phase 1: Chess Core Library

**Scope**: Medium
**Duration estimate**: 1-2 weekends
**Dependencies**: None
**Key decision**: Use `cozy-chess` for move generation and board state, or write from scratch?
**Decision made**: Use `cozy-chess` (already imported in engine/mod.rs). This handles bitboard representation, legal move generation, and position hashing. We build game-level logic on top.

### [ ] 1.1 Project scaffolding

**Description**: Initialize Cargo workspace, set up crate structure, get `cargo build` and `cargo test` passing in CI.

**Deliverables**:
- `Cargo.toml` with workspace members and dependencies
- Crate structure: single binary crate for now (split into `chesstui-core` lib later if needed)
- Module tree: `main.rs`, `lib.rs`, `chess/mod.rs`, `engine/mod.rs`, `tui/mod.rs`, `net/mod.rs`
- `clap` CLI with subcommands: `chesstui` (default: local game), `chesstui server`, `chesstui client <host>`
- All existing CI checks pass (fmt, clippy, test, smoke)

**Acceptance criteria**:
- `cargo build --release` succeeds
- `cargo test` runs (even if no tests yet)
- `./target/release/chesstui --help` prints usage
- `./target/release/chesstui server --help` and `client --help` work
- CI green on all 3 OS targets

**Files to create**:
- `Cargo.toml`
- `src/main.rs`
- `src/lib.rs`
- `src/chess/mod.rs` (empty module)
- `src/tui/mod.rs` (empty module)
- `src/net/mod.rs` (empty module)

### [ ] 1.2 Game state manager

**Description**: Build the game-level chess logic on top of `cozy-chess`. This is NOT the board/move-gen (cozy-chess does that). This is: game lifecycle, move history, draw detection, result tracking, FEN/PGN I/O.

**Deliverables**:
- `GameState` struct wrapping `cozy_chess::Board` with:
  - Move history (`Vec<Move>` + SAN strings)
  - Half-move clock and full-move counter (from FEN)
  - Game result enum: `InProgress`, `Checkmate(Color)`, `Stalemate`, `DrawBy(Reason)`
  - Draw detection: fifty-move rule, threefold repetition (hash history), insufficient material
  - `make_move(&mut self, mv: Move) -> Result<(), IllegalMove>`
  - `legal_moves(&self) -> Vec<Move>`
  - `is_check(&self) -> bool`
  - `status(&self) -> GameStatus`
- FEN support: `from_fen(fen: &str)`, `to_fen(&self) -> String` (mostly delegated to cozy-chess)
- PGN export: `to_pgn(&self) -> String` with headers (Event, Date, Result, etc.)

**Acceptance criteria**:
- Unit tests for each draw condition (stalemate, 50-move, threefold, insufficient material)
- Unit tests for checkmate detection
- Round-trip FEN: parse -> export -> parse produces identical board
- PGN export generates valid PGN that external tools can read
- Scholar's mate test: 1.e4 e5 2.Bc4 Nc6 3.Qh5 Nf6 4.Qxf7# results in `Checkmate(White)`

**Files to create/edit**:
- `src/chess/mod.rs`
- `src/chess/game.rs`
- `src/chess/pgn.rs`
- `tests/chess_rules.rs` (integration tests)

### [ ] 1.3 Move parsing and coordinate system

**Description**: Bridge between human input (algebraic notation, square coordinates) and `cozy-chess::Move`. Support both standard algebraic ("Nf3") and coordinate ("g1f3") notation.

**Deliverables**:
- Parse algebraic notation to `cozy_chess::Move` given a board position
- Parse coordinate notation ("e2e4") to `Move`
- Format `Move` as SAN and as coordinate string
- Disambiguation for SAN (R1a3 vs Ra3 when two rooks can reach a3)
- Promotion parsing: "e8=Q" or "e7e8q"

**Acceptance criteria**:
- All standard algebraic notation cases parse correctly (pieces, captures, castling O-O/O-O-O, promotion, en passant, check/checkmate suffixes)
- Round-trip: move -> SAN -> parse -> same move
- Invalid notation returns clear error

**Files to create/edit**:
- `src/chess/notation.rs`
- Tests in `src/chess/notation.rs` or `tests/notation.rs`

### Phase 1 Definition of Done
- `cargo test` passes with 30+ unit tests covering rules, notation, FEN, PGN
- A simple main() can play through a game programmatically (sequence of moves ending in checkmate)
- No TUI, no AI, no networking -- just the chess logic library

---

## Phase 2: TUI and Controls

**Scope**: Large
**Duration estimate**: 2-3 weekends
**Dependencies**: Phase 1 complete
**Key decisions**:
1. VIM modal system: how many modes? Recommendation: NORMAL (navigate board), COMMAND (`:` commands), INPUT (type algebraic notation). Keep it to 3.
2. Board orientation: always white at bottom, or flip for black? Support both with a toggle.
3. Unicode pieces vs ASCII? Default to Unicode (`♔♕♖♗♘♙`), fall back to ASCII (`K Q R B N P`) if terminal doesn't support it.

### [ ] 2.1 Board renderer

**Description**: Render an 8x8 chess board in the terminal using ratatui. This is the core visual component.

**Deliverables**:
- Board widget that renders in a ratatui `Frame`
- Colored squares (dark/light) using background colors
- Piece rendering with Unicode symbols (and ASCII fallback via config flag)
- Rank/file labels (a-h, 1-8)
- Highlighted squares: selected piece, legal move destinations, last move, king in check
- Board flipping (white/black perspective)
- Minimum terminal size detection and graceful handling

**Acceptance criteria**:
- Board renders correctly at the starting position
- Board renders correctly from FEN positions (various mid-game states)
- Selected piece highlights legal moves
- Last move is visually distinct
- King in check gets a red highlight
- Board looks correct when flipped
- Renders without panic on 80x24 terminal minimum

**Files to create/edit**:
- `src/tui/mod.rs`
- `src/tui/board.rs`
- `src/tui/theme.rs` (color definitions)

### [ ] 2.2 Layout and panels

**Description**: Full screen layout with board + info panels.

**Deliverables**:
- Layout: board (center), move history (right panel), status bar (bottom), captured pieces (above/below board)
- Move history panel: scrollable, shows move pairs (1. e4 e5), highlights current move
- Status bar: current turn, game status, mode indicator (NORMAL/COMMAND/INPUT)
- Captured pieces display
- Clock/timer display area (placeholder -- timers implemented in Phase 5)
- Responsive layout that adapts to terminal width (hide side panel if too narrow)

**Acceptance criteria**:
- All panels render with correct data
- Move history scrolls when it exceeds panel height
- Mode indicator shows current VIM mode
- Layout degrades gracefully on small terminals

**Files to create/edit**:
- `src/tui/layout.rs`
- `src/tui/panels.rs`
- `src/tui/status_bar.rs`

### [ ] 2.3 VIM-style input system

**Description**: Modal keyboard input inspired by VIM. This is the signature UX feature. Keep it intuitive for VIM users but learnable for non-VIM users.

**Deliverables**:
- **NORMAL mode** (default):
  - `h/j/k/l` -- move cursor on board (left/down/up/right)
  - `Enter` or `Space` -- select piece / confirm move destination
  - `Escape` -- deselect piece
  - `f` -- flip board
  - `/` -- enter search mode (filter legal moves by typing, e.g., `/Nf` shows knight moves to f-files)
  - `u` -- undo last move (local games only)
  - `r` -- redo
  - `:` -- enter COMMAND mode
  - `?` -- show help overlay
  - `g` -- go to first move in history
  - `G` -- go to last move in history
  - `n/N` -- next/previous move in history (review mode)
  - `1-8` -- quick jump to rank
- **COMMAND mode** (`:` prefix, like VIM ex commands):
  - `:q` / `:quit` -- quit game
  - `:resign` -- resign current game
  - `:draw` -- offer draw
  - `:fen` -- display/copy current FEN
  - `:pgn` -- export PGN
  - `:flip` -- flip board
  - `:new` -- new game
  - `:ai <level>` -- start game vs AI at given level
  - `:connect <host>` -- connect to server
  - `:set <option>` -- configure options
  - Tab completion for commands
- **INPUT mode** (type algebraic notation directly):
  - `i` from NORMAL mode enters INPUT mode
  - Type move in algebraic notation ("Nf3", "e4", "O-O")
  - `Enter` -- submit move
  - `Escape` -- cancel and return to NORMAL mode
  - Auto-complete/suggest as you type

**Acceptance criteria**:
- All NORMAL mode bindings work
- Cursor movement wraps correctly at board edges (or stops, configurable)
- Piece selection -> legal move highlights -> move confirmation flow works
- Command mode accepts and executes all listed commands
- Input mode parses algebraic notation and makes the move
- Mode transitions are visually clear (status bar updates)
- Unknown commands show error message, don't crash
- Help overlay (`?`) shows all keybindings

**Files to create/edit**:
- `src/tui/input.rs` (input handler, mode state machine)
- `src/tui/commands.rs` (command parser and executor)
- `src/tui/help.rs` (help overlay)
- `src/tui/search.rs` (move search/filter)

### [ ] 2.4 Game loop and local play

**Description**: Wire everything together into a playable local game (hotseat -- two humans at one terminal).

**Deliverables**:
- Main game loop: render -> handle input -> update state -> repeat
- Tokio runtime setup (single-threaded for now, needed later for networking)
- Crossterm raw mode, alternate screen, mouse capture
- Graceful terminal restore on panic (catch_unwind or panic hook)
- New game flow: choose color, choose opponent (human hotseat / AI)
- Sound: terminal bell on check, illegal move (optional, behind flag)
- Clean shutdown on Ctrl+C / `:q`

**Acceptance criteria**:
- Two humans can play a complete chess game to checkmate/stalemate/resignation
- Terminal is always restored cleanly, even on crash
- Game detects and announces checkmate, stalemate, draws
- Undo/redo works in local games
- `:new` starts a fresh game without restarting the binary

**Files to create/edit**:
- `src/tui/app.rs` (application state, game loop)
- `src/main.rs` (wire up runtime and app)

### Phase 2 Definition of Done
- A human can launch `chesstui`, see a board, move pieces with hjkl+enter or algebraic notation, and play to completion
- VIM users feel at home; non-VIM users can figure it out with `?` help
- No networking, no AI -- just the board and input system

---

## Phase 3: AI Engine

**Scope**: Large
**Duration estimate**: 2-3 weekends
**Dependencies**: Phase 1 complete; Phase 2 nice-to-have (can develop AI without TUI, test via unit tests)
**Key decisions**:
1. Search algorithm: Negamax with alpha-beta pruning. Add iterative deepening, move ordering, transposition table.
2. Evaluation: Piece-square tables + material + basic positional (king safety, pawn structure, mobility). Don't try to compete with Stockfish -- aim for "fun to play against."
3. Difficulty: 8-10 levels. Low levels = shallow search + random mistakes. High levels = deeper search + better eval.

**Note**: `src/engine/mod.rs` already defines the module structure and `ChessAI` / `AIResult` types. The sub-modules need implementation.

### [ ] 3.1 Evaluation function

**Description**: Static board evaluation. Given a position, return a centipawn score.

**Deliverables**:
- Material counting (standard values: P=100, N=320, B=330, R=500, Q=900)
- Piece-square tables for all pieces (middlegame and endgame)
  - Tables in `src/engine/tables.rs`
- Tapered evaluation (blend middlegame/endgame based on remaining material)
- Basic positional factors:
  - Pawn structure: doubled, isolated, passed pawns
  - King safety: pawn shield, open files near king
  - Mobility: count of legal moves
  - Bishop pair bonus
  - Rook on open/semi-open file
- Evaluation from White's perspective (negate for Black, or use side-to-move convention)

**Acceptance criteria**:
- Starting position evaluates near 0 (within +/- 30 cp)
- Position with extra queen evaluates ~900 cp advantage
- Passed pawn on 7th rank evaluates significantly higher than pawn on 2nd rank
- Endgame king centralization is rewarded
- Eval is fast: < 1 microsecond per call

**Files to create/edit**:
- `src/engine/evaluation.rs`
- `src/engine/tables.rs` (piece-square tables)
- `tests/eval.rs`

### [ ] 3.2 Search engine

**Description**: Implement the tree search that finds the best move.

**Deliverables**:
- Negamax with alpha-beta pruning
- Iterative deepening (search depth 1, then 2, then 3... until time runs out)
- Move ordering for better pruning:
  - Hash move (from transposition table)
  - Captures ordered by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
  - Killer moves (quiet moves that caused beta cutoff)
  - History heuristic
- Quiescence search (search captures at leaf nodes to avoid horizon effect)
- Transposition table (hash table, Zobrist hashing -- cozy-chess provides board hashing)
- Time management: allocate time per move based on remaining time, abort search when time expires
- Cancellation support via `AtomicBool` (already in ChessAI struct)
- Search info reporting: depth, nodes, nps, PV line

**Acceptance criteria**:
- Finds mate-in-1 instantly
- Finds mate-in-2 within depth 4
- Finds mate-in-3 within depth 6
- Does not crash or hang on any legal position
- Respects time limits (aborts search within 50ms of deadline)
- Transposition table improves search speed measurably (benchmark before/after)
- At depth 6+, plays reasonable chess (doesn't hang pieces)

**Files to create/edit**:
- `src/engine/search.rs`
- `tests/search.rs`

### [ ] 3.3 Opening book

**Description**: Small embedded opening book so the AI plays recognizable openings at higher difficulty levels.

**Deliverables**:
- Compact opening book format (hash map of position hash -> list of weighted moves)
- Cover ~200-500 positions from common openings (Sicilian, Ruy Lopez, Queen's Gambit, Italian, etc.)
- Book selection with some randomness (weighted random from book moves)
- Book can be disabled per difficulty level

**Acceptance criteria**:
- AI plays 1.e4, 1.d4, or 1.Nf3 from starting position (not random garbage)
- After 1.e4 e5, AI plays reasonable responses (Nf3, Bc4, f4, etc.)
- Book exits gracefully when position is not in book
- Book data compiles into the binary (no external files)

**Files to create/edit**:
- `src/engine/opening_book.rs`
- Build script or const data for book positions

### [ ] 3.4 Difficulty levels and personality

**Description**: Make the AI fun to play against at all skill levels. This is what separates "chess engine" from "chess game."

**Deliverables**:
- `DifficultyLevel` struct with parameters:
  - `max_depth: u8` (search depth limit)
  - `time_limit: Duration` (per-move time budget)
  - `use_opening_book: bool`
  - `mistake_probability: f32` (chance to play suboptimal move)
  - `mistake_max_cp_loss: i32` (how bad the mistakes can be)
  - `eval_noise: i32` (random noise added to evaluation)
  - `use_endgame_tables: bool`
- 10 levels with these approximate profiles:
  - Level 1: depth 1, high mistakes, large noise -- "my kid can beat this"
  - Level 3: depth 2-3, occasional mistakes -- casual player
  - Level 5: depth 4, rare mistakes -- club player
  - Level 7: depth 6, no deliberate mistakes, opening book -- strong amateur
  - Level 10: max depth, full time, opening book, no noise -- "good luck"
- Personality system: at low levels, occasionally play human-like mistakes (not random moves, but second/third best moves)
- ELO estimation per level (rough, for display purposes)

**Acceptance criteria**:
- Level 1 loses to a beginner who knows basic tactics
- Level 5 feels like a real opponent for an intermediate player
- Level 10 is the strongest the engine can play
- Difficulty can be changed mid-session via `:ai <level>`
- Each level feels distinct (not just "slightly faster/slower")

**Files to create/edit**:
- `src/engine/difficulty.rs`
- `src/engine/personality.rs`

### [ ] 3.5 AI integration with TUI

**Description**: Wire AI into the game loop so humans can play against it.

**Deliverables**:
- AI thinks on a background `tokio::task::spawn_blocking` thread
- "Thinking..." indicator with elapsed time and search depth
- Cancel AI thinking with `Escape`
- AI move is animated (brief highlight of from/to squares)
- Post-game analysis mode: AI evaluates each move after the game

**Acceptance criteria**:
- Human can play a full game against AI at any level
- UI remains responsive while AI thinks (can quit, resize terminal)
- AI respects time limits at all difficulty levels
- No deadlocks or race conditions

**Files to create/edit**:
- `src/tui/app.rs` (add AI opponent mode)
- `src/tui/ai_panel.rs` (thinking indicator, eval display)

### Phase 3 Definition of Done
- Human can launch `chesstui`, type `:ai 5`, and play a full game against a level-5 AI
- AI plays reasonable chess at all 10 levels
- AI thinking is non-blocking and cancellable

---

## Phase 4: Networking and Multiplayer

**Scope**: Large
**Duration estimate**: 2-3 weekends
**Dependencies**: Phase 1 and Phase 2 complete; Phase 3 is independent
**Key decisions**:
1. Protocol: Custom binary protocol over TCP? Or JSON lines? Recommendation: JSON lines over TCP with length-prefix framing. Easy to debug with `nc`/`telnet`, fast enough for chess. Can optimize later if needed.
2. Authentication: None for v1. Players pick a display name on connect. Add auth later.
3. Server architecture: Single-process async Tokio server. Each game is a task. Lobby is shared state behind `Arc<Mutex<>>` or actor pattern.

### [ ] 4.1 Protocol definition

**Description**: Define the client-server message protocol. Document it thoroughly -- this is the contract.

**Deliverables**:
- Message types (serde-serializable enums):
  ```
  ClientMessage: Authenticate, CreateGame, JoinGame, MakeMove, Resign, OfferDraw, AcceptDraw, DeclineDraw, Chat, ListGames, SpectateGame, LeaveGame
  ServerMessage: Welcome, GameCreated, GameStarted, MoveMade, GameOver, Error, GameList, ChatMessage, SpectatorUpdate, TimeUpdate
  ```
- Framing: 4-byte length prefix (big-endian u32) + JSON payload
- Protocol version field in Authenticate/Welcome for forward compatibility
- Heartbeat/ping-pong for connection liveness

**Acceptance criteria**:
- All message types serialize/deserialize correctly (round-trip tests)
- Protocol documented in `docs/protocol.md`
- Message size is reasonable (< 1KB for typical messages)

**Files to create/edit**:
- `src/net/protocol.rs`
- `src/net/framing.rs` (length-prefix codec)
- `docs/protocol.md`

### [ ] 4.2 Server core

**Description**: TCP server that accepts connections and manages game state.

**Deliverables**:
- `chesstui server --bind 0.0.0.0:7600` starts the server
- Accept TCP connections with `tokio::net::TcpListener`
- Per-connection task that reads/writes framed messages
- Lobby state: connected players, active games, game history
- Game rooms: create, join, leave, spectate
- Move validation on server side (authoritative -- never trust the client)
- Graceful shutdown on SIGTERM (finish active games or save state)
- Logging with `tracing` crate

**Acceptance criteria**:
- Server starts and accepts connections
- Multiple clients can connect simultaneously
- Server validates all moves (rejects illegal moves with error)
- Server detects checkmate/stalemate/draw and notifies both players
- Server handles client disconnection gracefully (opponent wins on disconnect, or game pauses)
- No panics on malformed messages (fuzzing-friendly error handling)

**Files to create/edit**:
- `src/net/server.rs`
- `src/net/lobby.rs`
- `src/net/game_room.rs`

### [ ] 4.3 Client networking

**Description**: Client-side networking that connects to the server and syncs game state.

**Deliverables**:
- `chesstui client <host>` or `:connect <host>` connects to server
- Async message send/receive integrated with the TUI event loop
- Reconnection logic (attempt reconnect on connection drop, with backoff)
- Latency display
- Server message handling (update local game state, show opponent moves)

**Acceptance criteria**:
- Client connects to server and receives Welcome message
- Client can create a game, opponent can join
- Moves made by either player appear on both screens
- Connection drop is detected and reported to the user
- Reconnection works (game resumes if both players reconnect)

**Files to create/edit**:
- `src/net/client.rs`
- `src/tui/app.rs` (integrate networking into game loop)

### [ ] 4.4 Game synchronization

**Description**: Ensure both clients and the server agree on game state at all times.

**Deliverables**:
- Server is authoritative: client sends move intent, server validates and broadcasts
- Board state hash verification (client and server compare hashes periodically)
- Handle race conditions: both players try to move at same time (server orders by arrival)
- Move confirmation: client shows "pending" state until server acknowledges

**Acceptance criteria**:
- Game state never diverges between server and clients
- Illegal move on client side shows error, does not desync
- Server crash mid-game: clients detect and report
- Rapid moves don't cause desync

**Files to create/edit**:
- `src/net/sync.rs`
- Tests: simulated game between two clients and a server

### [ ] 4.5 End-to-end multiplayer test

**Description**: Integration test that verifies two clients can play a full game through the server.

**Deliverables**:
- Integration test that starts a server, connects two clients, plays a complete game
- Manual testing checklist for real-network play
- Basic load test: 10+ simultaneous games on the server

**Acceptance criteria**:
- Automated test: two clients play Scholar's Mate through the server, both see checkmate
- Server handles 10 simultaneous games without degradation
- Full game (50+ moves) completes without desync

**Files to create/edit**:
- `tests/multiplayer.rs`
- `tests/load_test.rs` (or benchmark)

### Phase 4 Definition of Done
- Two humans on different machines can play chess over TCP through the self-hosted server
- Server validates all moves, detects game end conditions
- Connection issues are handled gracefully (no panics, no desync)

---

## Phase 5: Game Features

**Scope**: Medium
**Duration estimate**: 2 weekends
**Dependencies**: Phase 4 complete for multiplayer features; Phase 2 complete for local features
**Note**: These are independent features. Can be implemented in any order. Prioritize what's most fun.

### [ ] 5.1 Lobby system

**Description**: Server lobby where players can see available games, create games with settings, and find opponents.

**Deliverables**:
- Lobby TUI screen (separate from game board):
  - List of open games (waiting for opponent)
  - List of active games (spectatable)
  - Player list (who's online)
  - Create game dialog: time control, color preference, rated/unrated
  - Join game by selecting from list
- Server-side lobby management
- Game creation with settings: time control, increment, color assignment

**Acceptance criteria**:
- Player connects and sees the lobby
- Player can create a game and wait for opponent
- Another player can see and join the game
- Game settings (time control) are applied correctly
- Lobby updates in real-time as games are created/completed

**Files to create/edit**:
- `src/tui/lobby.rs`
- `src/net/lobby.rs` (extend server lobby)

### [ ] 5.2 Chess clocks / time controls

**Description**: Implement chess clocks with standard time controls.

**Deliverables**:
- Time controls: bullet (1+0, 2+1), blitz (3+2, 5+0, 5+3), rapid (10+0, 15+10), classical (30+0)
- Clock display in the TUI (countdown timers for both players)
- Clock starts after first move (or after a configurable delay)
- Time increment (Fischer increment) added after each move
- Flag fall: player loses on time (server enforces in multiplayer)
- Low time warning (visual + optional terminal bell when under 30s, 10s)

**Acceptance criteria**:
- Clocks count down during the active player's turn
- Increment is added after each move
- Player loses on time (game ends)
- Clocks pause during opponent's turn
- Server enforces time in multiplayer (client clock is display-only)
- Clock display updates every 100ms for smooth countdown

**Files to create/edit**:
- `src/chess/clock.rs`
- `src/tui/clock_widget.rs`
- Server-side time tracking in `src/net/game_room.rs`

### [ ] 5.3 Spectating

**Description**: Allow connected clients to watch active games in real-time.

**Deliverables**:
- `:spectate <game_id>` or select from lobby
- Spectators see the board update in real-time
- Spectator count shown to players
- Spectators can chat (separate spectator chat)
- Board auto-flips based on last move (or fixed perspective, togglable)

**Acceptance criteria**:
- Spectator sees all moves in real-time
- Spectator cannot make moves (server enforces)
- Multiple spectators can watch the same game
- Spectator can leave without affecting the game
- Spectator joining mid-game sees the current position (not just moves from their join point)

**Files to create/edit**:
- `src/net/spectator.rs`
- `src/tui/spectator_view.rs`

### [ ] 5.4 Game history and persistence

**Description**: Save completed games and allow review.

**Deliverables**:
- Server saves completed games to disk (JSON or SQLite in `/var/lib/chesstui/`)
- Game history list: `:history` shows recent games
- Game replay: load a completed game and step through moves (n/N keys)
- PGN export of any completed game
- Player stats: wins/losses/draws (basic, not ELO -- ELO is scope creep for v1)

**Acceptance criteria**:
- Completed games are persisted across server restarts
- Player can list and replay their past games
- PGN export works for any saved game
- Stats are accurate

**Files to create/edit**:
- `src/net/storage.rs` (game persistence)
- `src/tui/history.rs` (history browser)
- `src/tui/replay.rs` (game replay view)

### [ ] 5.5 Chat

**Description**: In-game text chat between players.

**Deliverables**:
- Chat panel in the TUI (below or beside the board)
- Chat input (`:chat <message>` or dedicated chat mode)
- Server relays chat messages
- Chat history scrollable
- Mute option

**Acceptance criteria**:
- Both players can send and receive chat messages
- Chat persists for the duration of the game
- Messages are attributed to the correct player
- Chat does not interfere with game input

**Files to create/edit**:
- `src/tui/chat.rs`
- `src/net/protocol.rs` (already has Chat message type)

### Phase 5 Definition of Done
- Full multiplayer experience: lobby, game creation, spectating, chat, timers
- Games are saved and reviewable
- The experience feels like a complete chess platform, not a tech demo

---

## Phase 6: Polish and Distribution

**Scope**: Medium (ongoing)
**Duration estimate**: 1-2 weekends focused, then continuous
**Dependencies**: Core features from Phases 1-5; CI/CD scaffolding already exists
**Note**: Some of this can happen in parallel with earlier phases. Error handling and testing should be continuous, not deferred.

### [ ] 6.1 Error handling audit

**Description**: Replace all `unwrap()`, `expect()`, and `panic!()` with proper error handling.

**Deliverables**:
- Custom error types with `thiserror`
- All network errors are recoverable (show error, don't crash)
- All TUI errors restore terminal before exiting
- Structured error logging with `tracing`
- User-facing error messages are helpful (not raw Rust error strings)

**Acceptance criteria**:
- `grep -r "unwrap()" src/` returns zero results (or only in tests)
- Server handles any client input without panicking (fuzz-tested)
- Client handles server disconnection gracefully
- Terminal is always restored on exit (even on panic)

### [ ] 6.2 Configuration system

**Description**: User configuration file for persistent settings.

**Deliverables**:
- Config file at `~/.config/chesstui/config.toml`
- Configurable: default server, display name, board colors, piece style (unicode/ascii), key bindings (stretch), default AI level
- Config loaded at startup, `:set` commands modify runtime config
- `:config` command opens config in $EDITOR

**Acceptance criteria**:
- Config file is created on first run with defaults
- All visual settings can be changed via config
- Invalid config shows error and falls back to defaults

**Files to create/edit**:
- `src/config.rs`

### [ ] 6.3 Cross-platform testing

**Description**: Verify the binary works correctly on all target platforms.

**Deliverables**:
- Manual testing on macOS (ARM + Intel), Linux (x86_64), Windows
- Terminal compatibility: iTerm2, Terminal.app, Alacritty, Windows Terminal, tmux, screen
- Fix any platform-specific rendering issues
- Windows: verify crossterm works correctly (no ANSI escape issues)
- Document known limitations per platform

**Acceptance criteria**:
- All CI targets build and pass tests
- Manual smoke test on at least macOS, Linux, Windows
- No rendering glitches in tested terminals
- README documents supported terminals

### [ ] 6.4 Installation and distribution

**Description**: Make installation easy for end users. Most of this scaffolding already exists.

**Deliverables**:
- Verify and test: `curl | bash` installer (already exists)
- Verify and test: PowerShell installer for Windows (already exists)
- Verify and test: Homebrew tap (already in release workflow)
- Verify and test: crates.io publish (already in release workflow)
- AUR package (nice to have)
- `chesstui --version` shows version, git hash, build date
- README with installation instructions, screenshots, usage guide

**Acceptance criteria**:
- `cargo install chesstui` works
- `brew install cjmiller/tap/chesstui` works
- curl installer works on Linux and macOS
- PowerShell installer works on Windows
- Installed binary runs correctly

### [ ] 6.5 Performance and optimization

**Description**: Profile and optimize hot paths.

**Deliverables**:
- Profile AI search with `cargo bench` (criterion)
- TUI render time: target < 16ms per frame (60fps)
- Server: handle 100+ simultaneous connections
- Binary size optimization: `strip`, LTO, `opt-level = "s"` if size is a concern
- Memory usage profiling for long-running server

**Acceptance criteria**:
- AI level 10 searches 100k+ nodes/second
- TUI renders at 60fps on commodity hardware
- Server handles 50+ simultaneous games
- Binary size < 10MB (release, stripped)

### Phase 6 Definition of Done
- The project is installable, documented, and works reliably on all target platforms
- Error handling is robust -- no panics in production paths
- Performance is good enough that it never feels sluggish

---

## Summary Table

| Phase | Scope | Est. Weekends | Depends On | Critical Path? |
|-------|-------|---------------|------------|----------------|
| 1. Chess Core | M | 1-2 | None | Yes |
| 2. TUI & Controls | L | 2-3 | Phase 1 | Yes |
| 3. AI Engine | L | 2-3 | Phase 1 | No (parallel with Phase 2) |
| 4. Networking | L | 2-3 | Phase 1, 2 | Yes |
| 5. Game Features | M | 2 | Phase 4 | No |
| 6. Polish | M | 1-2+ | All | No |

**Total estimate**: 10-16 weekends for a solo developer. Realistically 4-6 months of weekend work.

**Critical path**: Phase 1 -> Phase 2 -> Phase 4 -> Phase 5. Phase 3 (AI) is parallelizable with Phase 2.

## Prototype-First Strategy

**Weekend 1**: Tasks 1.1, 1.2, 1.3, 2.1 -- Get a board on screen with pieces. Move pieces with hjkl. No rules enforcement yet if pressed for time, but rules engine is the higher priority.

**Weekend 2**: Tasks 2.2, 2.3, 2.4 -- Full local hotseat game. This is the "show people" milestone.

**Weekend 3-4**: Tasks 3.1, 3.2, 3.4, 3.5 -- AI opponent. Skip opening book (3.3) initially.

**Weekend 5-6**: Tasks 4.1, 4.2, 4.3, 4.4 -- Two people play over the network.

**Weekend 7+**: Phase 5 features in order of what's most fun. Timers and lobby first, then spectating, then history.
