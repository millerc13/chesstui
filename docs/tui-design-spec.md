# ChessTUI -- Terminal User Interface Design Specification

This document defines every screen, layout, interaction pattern, color scheme,
and responsive behavior for the ChessTUI terminal client built with ratatui.

All measurements are in terminal cells (columns x rows).
Minimum supported terminal: **60 columns x 20 rows**.
Reference design target: **80 columns x 24 rows**.

---

## Table of Contents

1. [Global Conventions](#1-global-conventions)
2. [Main Menu](#2-main-menu)
3. [AI Difficulty Selection](#3-ai-difficulty-selection)
4. [Online Lobby](#4-online-lobby)
5. [In-Game Layout](#5-in-game-layout)
6. [Post-Game Screen](#6-post-game-screen)
7. [Settings Screen](#7-settings-screen)
8. [Responsive Design](#8-responsive-design)
9. [Color Scheme](#9-color-scheme)
10. [Ratatui Implementation Notes](#10-ratatui-implementation-notes)

---

## 1. Global Conventions

### 1.1 Modal Input System (VIM-style)

Three modes govern all input across the application:

```
 NORMAL     Default mode. Navigate with hjkl / arrow keys.
            Press : to enter COMMAND mode.
            Press i or / to enter INPUT mode (context-dependent).

 INPUT      Text entry (chat, game creation, search).
            Press Esc to return to NORMAL.

 COMMAND    Command-line at the bottom of the screen.
            :q  quit   :resign  resign game   :draw  offer draw
            :flip  flip board   :help  show help overlay
            Press Esc or Enter to return to NORMAL.
```

The current mode is always shown in the bottom-left of the status bar:

```
-- NORMAL --          (dim, unobtrusive)
-- INPUT --           (yellow text)
-- COMMAND --         (cyan text)
```

### 1.2 Key Binding Reference (Global)

```
Navigation
  j / Down        Move selection down
  k / Up          Move selection up
  h / Left        Move left / collapse
  l / Right       Move right / expand
  g g             Jump to top of list
  G               Jump to bottom of list
  Ctrl+d          Scroll half-page down
  Ctrl+u          Scroll half-page up

Actions
  Enter / Space   Select / confirm
  Esc             Back / cancel / exit mode
  q               Quit (from menus; requires confirm in-game)
  ?               Toggle help overlay
  :               Enter command mode
  Tab             Cycle focus between panels
```

### 1.3 Borders and Chrome

- Use ratatui `Block` with `Borders::ALL` and `BorderType::Rounded` for
  primary panels.
- Use `BorderType::Plain` for secondary/nested panels.
- Focused panel gets a highlighted border (bright white or accent color).
- Unfocused panels get dim borders (dark gray).

### 1.4 Status Bar (Global Footer)

Every screen has a single-line footer:

```
-- NORMAL --                          chesstui v0.1.0   80x24
```

Left: mode indicator. Right: version and terminal dimensions (useful for
debugging layout). Center: context-sensitive hints (varies per screen).

---

## 2. Main Menu

### 2.1 Layout (80x24)

```
                                                              80 cols
 +---------------------------------------------------------+  row 1
 |                                                         |
 |     _____ _                    _____ _   _ ___          |
 |    / ____| |                  |_   _| | | |_ _|         |
 |   | |    | |__   ___  ___ ___  | | | | | || |          |
 |   | |    | '_ \ / _ \/ __/ __| | | | | | || |          |
 |   | |____| | | |  __/\__ \__ \ | | | |_| || |          |
 |    \_____|_| |_|\___||___/___/ |_|  \___/|___|         |
 |                                                         |  row 9
 |                    v0.1.0                               |  row 10
 |                                                         |
 |              > Play Online                              |  row 12
 |                Play vs AI                               |  row 13
 |                Local Game                               |  row 14
 |                Settings                                 |  row 15
 |                Quit                                     |  row 16
 |                                                         |
 |                                                         |
 |                                                         |
 |                                                         |
 |                                                         |
 |      j/k navigate   Enter select   q quit   ? help     |  row 22
 |                                                         |
 +-- NORMAL -------------------------------------- 80x24 --+  row 24
```

### 2.2 Behavior

- ASCII art title is centered horizontally. On terminals narrower than 60
  columns, fall back to a simple text title: `C H E S S T U I`.
- Menu items are centered, one per row.
- Selected item is indicated with `>` prefix and **bold + accent color** text.
- Unselected items are dim white / gray.
- `j`/`k` wraps around (bottom wraps to top and vice versa).
- Pressing the first letter of an option jumps to it (P, S, Q).
- Row 22 shows context hints. These hints change if the user has not
  interacted for 5 seconds (rotate through tips).

### 2.3 Ratatui Structure

```
Layout::vertical([
    Constraint::Min(10),          // ASCII art + version
    Constraint::Length(7),        // Menu items (5 items + padding)
    Constraint::Fill(1),          // Spacer
    Constraint::Length(1),        // Hint bar
    Constraint::Length(1),        // Status bar
])
```

---

## 3. AI Difficulty Selection

### 3.1 Difficulty Levels

```
 Lvl  Name            ELO    Depth  Description
 ---  ----            ---    -----  -----------
  1   Beginner        400    1-2    Makes random legal moves. Perfect for learning.
  2   Novice          600    2-3    Captures hanging pieces. Blunders often.
  3   Casual          800    3      Knows basic tactics. Misses complex combos.
  4   Club Player    1000    4      Solid fundamentals. Occasional mistakes.
  5   Competitor     1200    5      Understands strategy. Few blunders.
  6   Expert         1400    6      Strong positional play. Punishes mistakes.
  7   Master         1600    7      Deep calculation. Dangerous in endgames.
  8   Grandmaster    1800    8+     Near-optimal play. Very few weaknesses.
  9   Engine         2000    10+    Full strength. No mercy.
 10   Impossible     2200+   max    Maximum depth, no personality dampening.
```

### 3.2 Layout (80x24)

```
 +--[ AI Difficulty ]------------------------------------------+
 |                                                             |
 |  Select your opponent:                                      |
 |                                                             |
 |   1  Beginner       ELO ~400   [*---------]                |
 |   2  Novice         ELO ~600   [**--------]                |
 |   3  Casual         ELO ~800   [***-------]                |
 | > 4  Club Player    ELO ~1000  [****------]   <-- selected |
 |   5  Competitor     ELO ~1200  [*****-----]                |
 |   6  Expert         ELO ~1400  [******----]                |
 |   7  Master         ELO ~1600  [*******---]                |
 |   8  Grandmaster    ELO ~1800  [********--]                |
 |   9  Engine         ELO ~2000  [*********-]                |
 |  10  Impossible     ELO ~2200+ [**********]                |
 |                                                             |
 +-------------------------------------------------------------+
 |  Club Player -- Solid fundamentals. Plays principled chess  |
 |  but sometimes misses deeper tactics. Thinks for 1-3s.     |
 |  Personality: Slightly aggressive, prefers open positions.  |
 +-------------------------------------------------------------+
 |  j/k select   Enter confirm   Esc back   1-9,0 jump        |
 +-- NORMAL ------------------------------------------ 80x24 --+
```

### 3.3 Visual Indicators

- **Difficulty bar**: 10-character gauge using filled/empty block characters.
  Color graduates from green (level 1) through yellow (5-6) to red (9-10).
  Uses ratatui `Gauge` or a custom `Span` sequence:

  ```
  Level  1: [#---------]  Green
  Level  5: [#####-----]  Yellow
  Level 10: [##########]  Red
  ```

  Actual characters: `\u{2588}` (full block) for filled, `\u{2591}` (light
  shade) for empty. Falls back to `#` and `-` on terminals without Unicode.

- **Selected row**: Bold text, accent-colored `>` prefix, entire row gets a
  subtle background highlight (using `Style::bg()`).

- **Preview panel** (bottom 3 rows): Updates dynamically as the user scrolls
  through levels. Shows description, typical think time, and personality notes.

### 3.4 Keyboard

- `j`/`k` or arrow keys to scroll selection.
- Number keys `1`-`9`, `0` (for 10) jump directly to that level.
- `Enter` confirms and starts the game.
- `Esc` returns to main menu.

---

## 4. Online Lobby

### 4.1 Layout (80x24)

```
 +--[ Online Lobby ]-------------------------------------------+
 | Players: 42    Games: 12    Ping: 23ms         [Create Game]|
 +------------------------------+------------------------------+
 |  Open Games                  |  Your Games                  |
 |  -------------------------   |  -------------------------   |
 |  stockfish99   5+3  2 spec   |  vs. knight_rider (active)  |
 |  pawnstar      10+0 0 spec   |  vs. AI-5 (your turn)       |
 |> darkbishop    3+2  5 spec   |                              |
 |  queenGambit   15+10 1 spec  |                              |
 |  rookie42      1+0  0 spec   |                              |
 |                               |                              |
 |                               |                              |
 |                               |                              |
 +------------------------------+------------------------------+
 |  Game Details                                               |
 |  darkbishop (ELO 1450) -- Blitz 3+2 -- 5 spectators        |
 |  Created 2m ago -- Rated -- No restrictions                 |
 +-------------------------------------------------------------+
 |  Enter join  s spectate  c create  r refresh  Tab switch    |
 +-- NORMAL ------------------------------------------ 80x24 --+
```

### 4.2 Panels

**Header row**: Server stats (player count, active games, latency). The
`[Create Game]` button is accessible with `c`.

**Left panel -- Open Games**: Scrollable list. Each row shows:
- Player name (truncated to 14 chars)
- Time control (e.g., `5+3` = 5 min + 3s increment)
- Spectator count

**Right panel -- Your Games**: Active/pending games you are involved in.

**Bottom panel -- Game Details**: Context for the currently selected game.
Updates as selection changes.

### 4.3 Create Game Flow

Pressing `c` opens a modal overlay (centered, 40x12):

```
 +----[ Create Game ]-----+
 |                        |
 |  Time Control          |
 |  Base:  [  5 ] min     |
 |  Incr:  [  3 ] sec     |
 |                        |
 |  Rated:    [x] Yes     |
 |  Color:    [ ] Random  |
 |                        |
 |  [ Create ]  [ Cancel ]|
 +------------------------+
```

- `Tab` cycles through fields.
- `h`/`l` adjusts numeric values.
- `Space` toggles checkboxes.
- `Enter` on Create submits. `Esc` cancels.

### 4.4 Keyboard

```
Tab         Switch focus between left/right panels
j/k         Scroll selected panel
Enter       Join selected game
s           Spectate selected game
c           Open create-game modal
r           Refresh game list
/           Filter/search games (enters INPUT mode)
Esc         Clear filter / close modal / back to menu
```

---

## 5. In-Game Layout

This is the core of the application. Every element must be carefully placed.

### 5.1 Panel Layout (80x24)

```
 +--[ White: player1 (1200) ]---+--[ Black: engine_L5 (1400) ]-+  row 1
 |   10:00          BLITZ 5+3   |   09:47                      |  row 2
 +------------------------------+-------------------------------+  row 3
 |                              |  Move History                 |
 |    8  r n b q k b n r        | +--------------------------+  |
 |    7  p p p p p p p p        | |  1. e4    e5             |  |
 |    6  . . . . . . . .        | |  2. Nf3   Nc6            |  |
 |    5  . . . . . . . .        | |  3. Bb5   a6             |  |
 |    4  . . . . P . . .        | |  4. Ba4   Nf6            |  |
 |    3  . . . . . N . .        | |  5. O-O   ...            |  |
 |    2  P P P P . P P P        | |                          |  |
 |    1  R N B Q K B . R        | |                          |  |
 |       a b c d e f g h        | +--------------------------+  |
 |                              +-------------------------------+
 |   Captured: BNP              |  Captured: p                 |
 +------------------------------+-------------------------------+
 |                                                              |
 |  e2 -> e4   (or algebraic: e4)                              |
 +-- NORMAL --  Your turn  ---- [?] help ------------- 80x24 --+
```

### 5.2 Detailed Panel Breakdown

#### 5.2.1 Status Bar (rows 1-2)

Two columns, one per player. Each contains:

- **Player name** with optional ELO in parentheses
- **Clock**: Large-ish digits showing remaining time
- **Time control label** centered between the two clocks (row 2)

The active player's panel gets a **highlighted border** (bright accent color).
The inactive player's panel uses dim borders.

When a clock falls below 30 seconds, the time display turns **red** and
blinks (using ratatui `Modifier::SLOW_BLINK` -- but only if the terminal
supports it; degrade gracefully to solid red).

```
Layout for status bar:
Layout::horizontal([
    Constraint::Percentage(50),  // White player
    Constraint::Percentage(50),  // Black player
])
```

#### 5.2.2 Chess Board (left panel, rows 3-15)

The board occupies the left half of the main area. Each square is rendered
as **2 characters wide by 1 character tall** for a total of 16 columns x 8
rows for the squares, plus rank/file labels.

**Board rendering**:

```
  Board cell: 2 chars wide, 1 char tall
  Total board: 16 cols x 8 rows = grid area
  With labels:  19 cols x 9 rows (rank numbers left, file letters below)
  With padding:  ~22 cols x 10 rows
```

**Unicode piece characters** (default):

```
  White: K=♔  Q=♕  R=♖  B=♗  N=♘  P=♙
  Black: K=♚  Q=♛  R=♜  B=♝  N=♞  P=♟
```

**ASCII fallback** (for terminals without Unicode):

```
  White: K Q R B N P  (uppercase)
  Black: k q r b n p  (lowercase)
```

**Square coloring**: Each cell gets a background color.
- Light squares: one bg color
- Dark squares: another bg color
- See Color Scheme section for exact values.

**Highlighting layers** (applied in priority order, highest first):

1. **Check**: King square gets a red background.
2. **Selected piece**: The piece you have picked up -- bright accent bg.
3. **Legal moves**: Squares the selected piece can move to -- dotted/dim
   accent bg. Empty legal squares show a center dot: `..`
   Capture squares show the target piece with a colored bg.
4. **Last move**: Source and destination of the most recent move -- subtle
   tint (slightly different shade of the square color).
5. **Premove** (if implemented): Queued move shown with a blue tint.

**Coordinate labels**:
- File letters (a-h) below the board.
- Rank numbers (1-8 or 8-1 depending on orientation) to the left.
- When board is flipped, ranks and files reverse accordingly.

#### 5.2.3 Move History (right panel, upper)

Scrollable list of moves in standard algebraic notation, displayed in two
columns (white move, black move) per row:

```
  1. e4    e5
  2. Nf3   Nc6
  3. Bb5   a6
```

- Current move is highlighted with a `>` marker or background color.
- User can scroll through history with `Ctrl+j`/`Ctrl+k` or `[`/`]` to
  review the game.
- When reviewing past positions, the board updates to show that position.
  A banner appears: `-- REVIEWING MOVE 3 -- Press Esc to return --`
- The move list auto-scrolls to keep the latest move visible during play.

#### 5.2.4 Captured Pieces (right panel, lower)

Compact display of captured pieces, grouped by color:

```
  White captured: ♟♟♞♝       (pieces White has taken)
  Black captured: ♙♙          (pieces Black has taken)
```

Material advantage shown as `+2` or `-1` next to the relevant side.
Pieces are ordered by value: Q R B N P.

#### 5.2.5 Input / Command Bar (rows 22-23)

Two lines at the bottom:

**Row 22 -- Input line**: Shows the current move being entered, or the
command being typed in COMMAND mode.

In NORMAL mode during the player's turn:
```
  Select a piece (use hjkl or type square, e.g., e2)
```

After selecting a piece:
```
  ♙ e2 selected -- move to? (hjkl or type square, e.g., e4)
```

In COMMAND mode:
```
  :resign
```

**Row 23 -- Context hints**: Changes based on current state.

```
  During your turn:    hjkl move cursor   Enter select   :resign   ? help
  Opponent's turn:     Waiting for opponent...   :resign   ? help
  Spectating:          [/] browse moves   q leave   ? help
```

### 5.3 Move Input Methods

Three ways to enter moves (all available simultaneously):

**Method 1: Cursor navigation (default)**
- `h`/`j`/`k`/`l` moves a cursor highlight around the board.
- `Enter` or `Space` on a piece selects it (shows legal moves).
- `Enter` or `Space` on a legal move square executes the move.
- `Esc` deselects the current piece.

**Method 2: Square typing**
- Type a square directly: `e2` then `e4`.
- The input bar shows what you are typing.
- If the move is unambiguous after 2 characters, it executes immediately.
- If ambiguous (e.g., two knights can go to the same square), prompt for
  clarification.

**Method 3: Algebraic notation (COMMAND mode)**
- `:Nf3`, `:e4`, `:O-O`, `:Bxe5`
- Parsed and validated. Error message if illegal.

**Promotion**: When a pawn reaches the 8th rank, a small popup appears:

```
  +-- Promote --+
  | Q  R  B  N  |
  +-------------+
```

Navigate with `h`/`l`, confirm with `Enter`.

### 5.4 In-Game Key Bindings

```
Movement & Selection
  h/j/k/l         Move board cursor
  Enter / Space    Select piece / confirm move
  Esc              Deselect piece / cancel

Board
  f                Flip board orientation
  [ / ]            Browse move history backward/forward
  Esc              Return to current position (when reviewing)

Game Actions
  :resign          Resign the game
  :draw            Offer a draw
  :abort           Abort (only in first 2 moves)
  :takeback        Request takeback

Interface
  Tab              Cycle focus (board -> move list -> captured)
  ?                Toggle help overlay
  :                Enter command mode
```

---

## 6. Post-Game Screen

### 6.1 Layout (80x24)

```
 +-------------------------------------------------------------+
 |                                                              |
 |                    CHECKMATE                                 |
 |               White wins by checkmate                        |
 |                                                              |
 +------------------------------+-------------------------------+
 |                              |  Game Summary                 |
 |    (final board position)    |  Moves: 34                    |
 |                              |  Duration: 12:45              |
 |                              |  White time: 3:22 remaining   |
 |                              |  Black time: 1:08 remaining   |
 |                              |                               |
 |                              |  AI Accuracy: 87%             |
 |                              |  Your Accuracy: 72%           |
 |                              |  Blunders: 2  Mistakes: 4     |
 +------------------------------+-------------------------------+
 |                                                              |
 |    [r] Review Moves   [n] Rematch   [m] Menu   [q] Quit     |
 |                                                              |
 +-- NORMAL ------------------------------------------ 80x24 --+
```

### 6.2 Result Display

The result banner (rows 2-4) varies by outcome:

```
  Checkmate:     "CHECKMATE -- White wins"
  Stalemate:     "STALEMATE -- Draw"
  Resignation:   "Black resigns -- White wins"
  Timeout:       "White wins on time"
  Draw agreed:   "Draw by agreement"
  Repetition:    "Draw by threefold repetition"
  50-move:       "Draw by fifty-move rule"
  Insufficient:  "Draw -- insufficient material"
  Disconnect:    "Black disconnected -- White wins"
```

Text is **bold**, centered, and uses the accent color.

### 6.3 Game Review Mode

Pressing `r` enters review mode: the board becomes interactive,
`[`/`]` step through moves, and the move list highlights the current
position. The right panel switches from summary to the full move list.

Press `Esc` to exit review and return to the post-game options.

### 6.4 Accuracy Stats (AI games only)

When playing against the AI, the engine evaluates each of the human player's
moves after the game to provide:

- **Accuracy percentage**: How close moves were to the engine's top choice.
- **Blunders**: Moves that lost more than 200 centipawns.
- **Mistakes**: Moves that lost 50-200 centipawns.
- **Best move count**: How many times the player found the engine's top move.

These stats are not shown for online games (no engine analysis on client).

---

## 7. Settings Screen

### 7.1 Layout (80x24)

```
 +--[ Settings ]-----------------------------------------------+
 |                                                             |
 |  Board Theme                                                |
 |    Piece style:    [ Unicode ]  ASCII  Fancy                |
 |    Light squares:  [=======]  #B58863                       |
 |    Dark squares:   [=======]  #6D4C2F                       |
 |    Board border:   [x] Show   [ ] Hide                     |
 |                                                             |
 |  Orientation                                                |
 |    Default:        [ ] White bottom  [x] Auto (play color)  |
 |                                                             |
 |  Gameplay                                                   |
 |    Premoves:       [x] Enabled                              |
 |    Move confirm:   [ ] Require Enter to confirm moves       |
 |    Show legal:     [x] Highlight legal moves                |
 |    Auto-queen:     [ ] Auto-promote to queen                |
 |                                                             |
 |  Sound                                                      |
 |    Move sound:     [x] Terminal bell on opponent's move     |
 |    Low time:       [x] Bell when clock < 30s                |
 |                                                             |
 |  j/k navigate   h/l adjust   Space toggle   Esc back       |
 +-- NORMAL ------------------------------------------ 80x24 --+
```

### 7.2 Behavior

- Settings are organized in sections with headers.
- `j`/`k` navigates between options (skipping section headers).
- `h`/`l` cycles through enum values (piece style, colors).
- `Space` toggles boolean checkboxes.
- Color values can be edited by pressing `Enter` on a color field to open
  a small hex input (INPUT mode).
- All changes are saved immediately to `~/.config/chesstui/config.toml`.
- `Esc` returns to the previous screen.

### 7.3 Key Binding Reference

Pressing `?` from the settings screen (or any screen) opens a full-screen
help overlay:

```
 +--[ Key Bindings ]-------------------------------------------+
 |                                                             |
 |  GLOBAL                          IN-GAME                    |
 |  j/k      Navigate up/down       h/j/k/l  Move cursor      |
 |  Enter     Select / confirm       Enter    Select/move       |
 |  Esc       Back / cancel          f        Flip board        |
 |  ?         Toggle help            [/]      Browse moves      |
 |  :         Command mode           :resign  Resign            |
 |  q         Quit                   :draw    Offer draw        |
 |  Tab       Cycle panels                                      |
 |                                                             |
 |  LOBBY                           COMMAND MODE               |
 |  c         Create game            :q       Quit              |
 |  s         Spectate               :flip    Flip board        |
 |  r         Refresh                :help    Show help         |
 |  /         Search/filter                                     |
 |                                                             |
 |                    Press ? or Esc to close                   |
 +-------------------------------------------------------------+
```

---

## 8. Responsive Design

### 8.1 Terminal Size Tiers

```
  Tier       Columns   Rows    Behavior
  ----       -------   ----    --------
  Minimum    60        20      Compact mode: board only, no side panels
  Compact    70        22      Board + abbreviated move list
  Standard   80        24      Full layout (reference design)
  Wide       100+      24+     Extra padding, wider move history panel
  Large      120+      30+     Full-width board (3-char squares), stats
```

### 8.2 Adaptive Layout Rules

**When width < 80 columns** (Compact):
1. Move history panel collapses to a single-line "last move" display below
   the board.
2. Captured pieces move to the status bar area.
3. Board remains full size (priority element).

**When width < 70 columns**:
1. File/rank labels use single-character padding.
2. Clock display abbreviates (`5:00` instead of `05:00`).
3. Hint bar shows fewer hints.

**When width < 60 columns** (Minimum):
1. Side panels are completely hidden.
2. Board fills available width.
3. Move history accessible only via overlay (`m` key).
4. Player info condensed to one line: `W: player1 10:00 | B: engine 09:47`

**When height < 20 rows**:
1. Help hints line is removed.
2. Status bar merges with input line.

**When width >= 100 columns** (Wide):
1. Move history panel widens; moves shown with more detail (timestamps,
   eval bars if available).
2. Extra padding around the board for visual breathing room.

**When width >= 120 and height >= 30** (Large):
1. Board squares become 3 characters wide for better piece visibility.
2. Evaluation bar appears beside the board (vertical bar graph showing
   engine eval over time).
3. Game info panel expands with additional statistics.

### 8.3 Resize Handling

- The application redraws on every `crossterm::event::Event::Resize`.
- Layout recalculation is immediate; no debouncing needed for terminal
  resize events.
- If the terminal drops below 40x15, display a centered message:
  `Terminal too small. Resize to at least 60x20.`

### 8.4 Priority Order for Space Allocation

When space is constrained, elements are hidden in this order (first hidden
first):

1. Help hints bar
2. Captured pieces display
3. Move history panel (replaced with single-line last move)
4. Clock display (merged into player name line)
5. File/rank labels on board
6. Board padding/centering

The board itself is never hidden or shrunk below 16x8.

---

## 9. Color Scheme

### 9.1 Design Principles

- Must be readable on both dark and light terminal backgrounds.
- Uses 256-color mode (Color::Indexed) as baseline, with true-color
  (Color::Rgb) as an enhancement.
- All colors are defined as constants in a `Theme` struct for easy swapping.
- Avoids pure white (#FFFFFF) and pure black (#000000) to work on both
  terminal backgrounds.

### 9.2 Default Color Palette

```rust
// Board colors
const LIGHT_SQUARE_BG: Color = Color::Rgb(181, 136, 99);   // warm tan
const DARK_SQUARE_BG:  Color = Color::Rgb(109, 76, 47);    // dark wood

// 256-color fallback
const LIGHT_SQUARE_256: Color = Color::Indexed(180);        // closest tan
const DARK_SQUARE_256:  Color = Color::Indexed(137);        // closest brown

// Piece colors
const WHITE_PIECE: Color = Color::Rgb(240, 235, 220);       // cream/off-white
const BLACK_PIECE: Color = Color::Rgb(40, 35, 30);          // near-black

// 256-color fallback for pieces
const WHITE_PIECE_256: Color = Color::Indexed(230);          // light cream
const BLACK_PIECE_256: Color = Color::Indexed(235);          // dark gray

// Highlight colors
const SELECTED_BG:   Color = Color::Rgb(130, 170, 100);     // muted green
const LEGAL_MOVE_BG: Color = Color::Rgb(100, 140, 80);      // dim green (dots)
const LAST_MOVE_BG_LIGHT: Color = Color::Rgb(205, 210, 106);// soft yellow-green
const LAST_MOVE_BG_DARK:  Color = Color::Rgb(170, 162, 58); // deeper yellow-green
const CHECK_BG:      Color = Color::Rgb(200, 50, 50);       // red alert
const PREMOVE_BG:    Color = Color::Rgb(70, 100, 170);      // steel blue

// UI chrome
const ACCENT:        Color = Color::Rgb(100, 160, 220);     // calm blue
const TEXT_PRIMARY:   Color = Color::Rgb(200, 200, 200);     // light gray
const TEXT_DIM:       Color = Color::Rgb(120, 120, 120);     // medium gray
const TEXT_BRIGHT:    Color = Color::Rgb(240, 240, 240);     // near-white
const BORDER_FOCUSED: Color = Color::Rgb(100, 160, 220);    // accent blue
const BORDER_DIM:     Color = Color::Rgb(60, 60, 60);       // dark gray
const BG_PANEL:       Color = Color::Reset;                  // terminal default

// Mode indicators
const MODE_NORMAL:  Color = Color::Rgb(120, 120, 120);      // dim gray
const MODE_INPUT:   Color = Color::Rgb(220, 180, 50);       // warm yellow
const MODE_COMMAND: Color = Color::Rgb(80, 180, 220);       // cyan

// Clock colors
const CLOCK_NORMAL:  Color = Color::Rgb(200, 200, 200);     // standard
const CLOCK_LOW:     Color = Color::Rgb(220, 160, 40);      // warning amber
const CLOCK_CRITICAL:Color = Color::Rgb(220, 50, 50);       // danger red

// Difficulty gauge
const DIFF_EASY:   Color = Color::Rgb(80, 180, 80);         // green
const DIFF_MEDIUM: Color = Color::Rgb(220, 180, 50);        // yellow
const DIFF_HARD:   Color = Color::Rgb(220, 70, 50);         // red
```

### 9.3 256-Color Fallback Table

For terminals that do not support true color (detected at startup via
`$COLORTERM` environment variable):

```
  Element              True Color       256-color Index
  -------              ----------       ---------------
  Light square bg      #B58863          180
  Dark square bg       #6D4C2F          137
  White pieces         #F0EBDC          230
  Black pieces         #28231E          235
  Selected bg          #82AA64          107
  Legal move bg        #648C50          65
  Last move light      #CDD26A          185
  Last move dark       #AAA23A          142
  Check bg             #C83232          160
  Accent               #64A0DC          74
  Text primary         #C8C8C8          251
  Text dim             #787878          243
  Border focused       #64A0DC          74
  Border dim           #3C3C3C          238
```

### 9.4 Alternative Board Themes

Users can select from preset themes in Settings:

```
  Classic:    Tan/brown        (default, shown above)
  Midnight:   Dark blue/navy   (#2B4570 / #1B2838)
  Forest:     Green/dark green (#6B8F4A / #3D5A27)
  Marble:     Light/med gray   (#D0D0D0 / #909090)
  Terminal:   Uses terminal default bg with dim/bright differentiation
```

---

## 10. Ratatui Implementation Notes

### 10.1 Application State Machine

```
enum AppScreen {
    MainMenu,
    AIDifficulty,
    OnlineLobby,
    CreateGame,          // modal overlay on lobby
    InGame(GameState),
    PostGame(GameResult),
    Settings,
    HelpOverlay,         // modal overlay on any screen
}

enum InputMode {
    Normal,
    Input,
    Command,
}

struct App {
    screen: AppScreen,
    mode: InputMode,
    command_buffer: String,
    terminal_size: (u16, u16),
    theme: Theme,
    config: Config,
}
```

### 10.2 Layout Hierarchy for In-Game Screen

```rust
// Top-level vertical split
let main_layout = Layout::vertical([
    Constraint::Length(2),        // Status bar (player names + clocks)
    Constraint::Min(10),         // Main content area
    Constraint::Length(1),       // Input line
    Constraint::Length(1),       // Status/mode bar
]);

// Main content: horizontal split
let content_layout = Layout::horizontal([
    Constraint::Length(22),       // Board (fixed width: 16 squares + labels + padding)
    Constraint::Min(20),         // Right panel
]);

// Right panel: vertical split
let right_layout = Layout::vertical([
    Constraint::Min(8),          // Move history (takes remaining space)
    Constraint::Length(3),       // Captured pieces
]);
```

### 10.3 Board Rendering Strategy

The board is rendered as a custom widget implementing `ratatui::widgets::Widget`:

```rust
struct ChessBoard<'a> {
    position: &'a Board,        // from cozy-chess
    orientation: Color,          // which color is at the bottom
    cursor: Option<(u8, u8)>,   // board cursor position (file, rank)
    selected: Option<Square>,   // currently selected piece
    legal_moves: Vec<Square>,   // legal destination squares
    last_move: Option<(Square, Square)>,
    in_check: bool,
    theme: &'a BoardTheme,
}

impl Widget for ChessBoard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 1. Draw square backgrounds (2 chars wide x 1 char tall per square)
        // 2. Apply highlight layers (last move, legal moves, selected, check)
        // 3. Draw pieces using Unicode or ASCII
        // 4. Draw rank/file labels
        // 5. Draw cursor indicator (if cursor mode)
    }
}
```

### 10.4 Event Loop Architecture

```rust
loop {
    // 1. Draw
    terminal.draw(|frame| {
        match app.screen {
            AppScreen::MainMenu => ui::draw_main_menu(frame, &app),
            AppScreen::InGame(ref state) => ui::draw_game(frame, &app, state),
            // ...
        }
        // Draw modal overlays on top
        if matches!(app.screen, AppScreen::HelpOverlay) {
            ui::draw_help_overlay(frame);
        }
    })?;

    // 2. Handle input
    if crossterm::event::poll(Duration::from_millis(100))? {
        match crossterm::event::read()? {
            Event::Key(key) => handle_key(&mut app, key),
            Event::Resize(w, h) => app.terminal_size = (w, h),
            _ => {}
        }
    }

    // 3. Tick (update clocks, check for server messages, AI progress)
    app.tick();
}
```

### 10.5 Module Organization

```
src/
  main.rs                   // Entry point, arg parsing (clap)
  app.rs                    // App struct, state machine, tick logic
  config.rs                 // Config file loading (~/.config/chesstui/config.toml)
  theme.rs                  // Color theme definitions and fallback logic
  input.rs                  // Key event handling, mode transitions
  ui/
    mod.rs                  // Re-exports
    menu.rs                 // Main menu rendering
    difficulty.rs           // AI difficulty selection screen
    lobby.rs                // Online lobby rendering
    game.rs                 // In-game layout orchestration
    board.rs                // Chess board widget
    move_list.rs            // Move history widget
    captured.rs             // Captured pieces widget
    clock.rs                // Clock display widget
    command_bar.rs          // Input/command bar widget
    postgame.rs             // Post-game screen
    settings.rs             // Settings screen
    help.rs                 // Help overlay
    modal.rs                // Generic modal overlay utility
  game/
    mod.rs                  // Game state management
    clock.rs                // Clock logic (countdown, increment)
    move_input.rs           // Move parsing and validation
    notation.rs             // Algebraic notation formatting
  engine/                   // AI engine (already exists)
    mod.rs
    evaluation.rs
    search.rs
    difficulty.rs
    opening_book.rs
    personality.rs
    tables.rs
  net/
    mod.rs                  // Network client
    protocol.rs             // Wire protocol (WebSocket messages)
    lobby.rs                // Lobby state sync
```

### 10.6 Performance Considerations

- Board rendering is lightweight (64 cells). No optimization needed.
- Move history list should use `ratatui::widgets::List` with `StatefulWidget`
  for efficient scrolling.
- AI computation runs on a separate thread; the UI thread never blocks.
  Communication via `mpsc::channel` or `Arc<Mutex<>>` for the result.
- Network I/O uses `tokio` in a background runtime; game state updates
  arrive via a channel to the main event loop.
- Target frame rate: 30fps (33ms per frame). The `poll(Duration::from_millis(100))`
  in the event loop means ~10fps idle, which is fine for a chess game. Increase
  poll frequency only when clocks are running and below 30 seconds.

---

## Appendix A: Unicode Chess Pieces Reference

```
  White King:   ♔  U+2654
  White Queen:  ♕  U+2655
  White Rook:   ♖  U+2656
  White Bishop: ♗  U+2657
  White Knight: ♘  U+2658
  White Pawn:   ♙  U+2659
  Black King:   ♚  U+265A
  Black Queen:  ♛  U+265B
  Black Rook:   ♜  U+265C
  Black Bishop: ♝  U+265D
  Black Knight: ♞  U+265E
  Black Pawn:   ♟  U+265F
```

Note: These glyphs are single-width in most terminal fonts, but some fonts
render them as double-width. The board widget must detect this at startup
(render a test character and measure cursor advancement) and adjust cell
width accordingly.

## Appendix B: ASCII Art Title Variants

**Full (60+ columns)**:
```
   _____ _                    _____ _   _ ___
  / ____| |                  |_   _| | | |_ _|
 | |    | |__   ___  ___ ___  | | | | | || |
 | |    | '_ \ / _ \/ __/ __| | | | | | || |
 | |____| | | |  __/\__ \__ \ | | | |_| || |
  \_____|_| |_|\___||___/___/ |_|  \___/|___|
```

**Compact (40-59 columns)**:
```
 C H E S S T U I
```

**Minimal (< 40 columns)**:
```
 ChessTUI
```
