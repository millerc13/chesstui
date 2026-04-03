# ChessTUI Keyboard Control System -- Game Design Document

**Version**: 1.0  
**Date**: 2026-04-03  
**Status**: Design -- pre-implementation  
**Design Pillar**: Speed-to-move is king. Every keystroke must earn its place.

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [Modal System](#2-modal-system)
3. [Board Navigation](#3-board-navigation)
4. [Quick-Move System](#4-quick-move-system)
5. [Key Bindings Reference](#5-key-bindings-reference)
6. [Visual Feedback System](#6-visual-feedback-system)
7. [Speed Optimizations](#7-speed-optimizations)
8. [Onboarding](#8-onboarding)
9. [Tuning Variables](#9-tuning-variables)
10. [Edge Cases and Failure States](#10-edge-cases-and-failure-states)
11. [Changelog](#11-changelog)

---

## 1. Design Philosophy

### Design Pillars (non-negotiable)

1. **Keystroke economy**: The most common action (making a move) must require the fewest keystrokes. Opening moves like 1.e4 must be achievable in exactly 2 keystrokes.
2. **Zero-surprise VIM mapping**: Any key that exists in VIM must do the VIM-analogous thing. `h` goes left, not "help." `j` goes down, not "jump." No exceptions.
3. **Progressive disclosure**: A new user can play with just arrow keys + Enter. A VIM user immediately discovers hjkl + algebraic input. A power user discovers every shortcut through practice, not documentation.
4. **Speed parity with GUI**: A practiced user must be able to premove and execute moves at the same speed as lichess point-and-click. Target: sub-200ms per move in bullet.

### Fun Hypothesis

The core satisfaction is the feeling of "typing chess" -- making moves as fast as you can think them, with the same flow state as editing code in VIM. The keyboard should feel like an extension of chess intuition, not a translation layer.

---

## 2. Modal System

### Mode Architecture

```
                    +------------------+
        ESC         |                  |    :
   +--------------->|   NORMAL MODE    |--------+
   |                |  (board nav +    |        |
   |                |   piece select)  |        v
   |                +--------+---------+   +----------+
   |                         |             | COMMAND  |
   |                   any a-h, 1-8,      |  MODE    |
   |                   or piece letter     +----------+
   |                         |
   |                         v
   |                +------------------+
   |                |                  |
   +----------------+   INPUT MODE    |
                    |  (algebraic /    |
                    |   square entry)  |
                    +------------------+
```

### Mode Definitions

#### NORMAL MODE (default)

**Purpose**: Navigate the board, select pieces, browse the position.  
**Player experience goal**: "I am looking at the board, thinking about my move."  
**VIM analogy**: Normal mode -- the resting state. You observe and navigate.

Active capabilities:
- Cursor movement with `hjkl` or arrow keys
- Piece selection with `Enter` or `Space`
- Quick-move initiation by typing any algebraic character (transitions to Input mode)
- Game history navigation with `[` and `]`
- Command mode entry with `:`

The cursor is always visible on a specific square. The player can move it freely. Pressing `Enter`/`Space` on a square with a friendly piece selects it and highlights legal moves. Pressing `Enter`/`Space` on a legal destination square executes the move.

#### INPUT MODE (algebraic entry)

**Purpose**: Type a move in algebraic notation or square coordinates.  
**Player experience goal**: "I know my move. Let me type it and go."  
**VIM analogy**: Insert mode -- direct text entry with purpose.

Entry triggers:
- Typing any letter `a-h` (interpreted as start of a square or pawn move)
- Typing any piece letter `N`, `B`, `R`, `Q`, `K`, `O` (case-insensitive in input)
- Pressing `/` (opens a dedicated input line, like VIM search)

The input buffer appears in the status bar. As characters are typed, the system attempts to match a legal move. When a unique legal move is identified, it executes immediately -- no Enter required. If ambiguous, the player continues typing until disambiguated or presses Enter to see options.

Exit: `Escape` clears the buffer and returns to Normal mode. Successful move execution also returns to Normal mode.

#### COMMAND MODE

**Purpose**: Non-move game actions.  
**Player experience goal**: "I need to do something other than move a piece."  
**VIM analogy**: Command-line mode -- typed commands for system actions.

Entry: `:` from Normal mode.  
The command buffer appears at the bottom of the screen, prefixed with `:`.

Commands:
```
:resign       or  :res     -- Resign the game (requires confirmation)
:draw         or  :d       -- Offer a draw
:flip         or  :f       -- Flip the board orientation
:rematch      or  :re      -- Offer/accept rematch
:takeback     or  :tb      -- Request takeback
:abort        or  :ab      -- Abort game (if within first 2 moves)
:settings     or  :set     -- Open settings panel
:quit         or  :q       -- Quit application
:help         or  :h       -- Show help overlay
:theme <name> or  :t       -- Switch color theme
:clock        or  :cl      -- Toggle clock display
:eval         or  :ev      -- Toggle engine evaluation bar
:fen          or  :fen     -- Copy current FEN to clipboard
:pgn          or  :pgn     -- Copy game PGN to clipboard
:save         or  :s       -- Save game to file
:load <file>  or  :l       -- Load game from file
:new <time>   or  :n       -- Start new game (e.g. :new 3+2)
```

Exit: `Escape` or `Enter` (after command execution).

#### ANALYSIS MODE (optional -- entered from command)

**Purpose**: Explore variations without committing moves.  
**Player experience goal**: "What if I played this instead?"  
**VIM analogy**: Visual mode -- selecting and manipulating without permanent change.

Entry: `:analyze` or `a` in Normal mode (when reviewing a completed game or during analysis board).  
Not available during a live game against an opponent.

Capabilities:
- All Normal mode navigation and move input
- Branching variations (moves are stored in a tree, not committed)
- Arrow navigation through the variation tree
- `u` to undo the last analysis move
- `Ctrl-r` to redo

Exit: `Escape` returns to the game position. `:analyze` toggles off.

---

## 3. Board Navigation

### Cursor Movement

The cursor is a highlighted square that the player controls. It defaults to the player's king square at game start.

```
Board Orientation (White's perspective):

     a   b   c   d   e   f   g   h
   +---+---+---+---+---+---+---+---+
8  | r | n | b | q | k | b | n | r |  8
   +---+---+---+---+---+---+---+---+
7  | p | p | p | p | p | p | p | p |  7
   +---+---+---+---+---+---+---+---+
6  |   |   |   |   |   |   |   |   |  6
   +---+---+---+---+---+---+---+---+
5  |   |   |   |   |   |   |   |   |  5
   +---+---+---+---+---+---+---+---+
4  |   |   |   |   |   |   |   |   |  4
   +---+---+---+---+---+---+---+---+
3  |   |   |   |   |   |   |   |   |  3
   +---+---+---+---+---+---+---+---+
2  | P | P | P | P | P | P | P | P |  2
   +---+---+---+---+---+---+---+---+
1  | R | N | B | Q | K | B | N | R |  1
   +---+---+---+---+---+---+---+---+
     a   b   c   d   e   f   g   h

Cursor: [e1] (highlighted with inverse colors)
```

#### Basic movement (Normal mode)

| Key       | Action                 | VIM Analogy       |
|-----------|------------------------|-------------------|
| `h`       | Move cursor left       | VIM `h`           |
| `j`       | Move cursor down       | VIM `j`           |
| `k`       | Move cursor up         | VIM `k`           |
| `l`       | Move cursor right      | VIM `l`           |
| Arrows    | Same as hjkl           | Universal fallback|
| `w`       | Jump to next piece (clockwise scan) | VIM `w` word-jump |
| `b`       | Jump to previous piece  | VIM `b` back-word |

**Important note on board orientation**: `j` always moves the cursor toward rank 1 (visually down regardless of board flip), and `k` always moves toward rank 8 (visually up). When the board is flipped, `j` moves toward rank 8 and `k` toward rank 1 in algebraic terms, but the visual direction stays consistent. This matches VIM convention -- `j` is always "move the cursor down on screen."

#### Jump-to-square (Normal mode)

| Key       | Action                              | VIM Analogy       |
|-----------|-------------------------------------|-------------------|
| `g` + file + rank | Jump cursor directly to a square | VIM `g` go-to |
| Example: `ge4`    | Cursor jumps to e4              | Like `gg` or `G`  |

This is a two-character sequence after the `g` prefix. The cursor moves but no piece is selected. This is pure navigation.

#### Piece-centric jumps

| Key       | Action                              |
|-----------|-------------------------------------|
| `gk`      | Jump cursor to own king             |
| `gq`      | Jump cursor to own queen            |
| `0`       | Jump cursor to a1 (or h8 if black)  |
| `$`       | Jump cursor to h1 (or a8 if black)  |
| `gg`      | Jump cursor to a8 (top-left)        |
| `G`       | Jump cursor to h1 (bottom-right)    |

### Selection and Move Execution (Cursor Method)

This is **Method 3** from the requirements -- the cursor-based flow. It is the slowest but most visual method, ideal for beginners and for studying positions.

```
Step 1: Navigate cursor to a friendly piece
Step 2: Press ENTER or SPACE to select it
        --> Legal move squares are highlighted
        --> Selected piece square gets a distinct highlight
Step 3: Navigate cursor to a legal destination square
Step 4: Press ENTER or SPACE to confirm the move
        --> Move executes
        --> Highlights clear
        --> Return to Normal mode with cursor on destination
```

**Cancel**: Press `Escape` at any time to deselect and return to free navigation.  
**Reselect**: If a piece is selected and you press `Enter`/`Space` on another friendly piece, selection transfers to the new piece.  
**Illegal target**: If you press `Enter`/`Space` on a square that is not a legal destination, nothing happens. A subtle visual indicator (brief flash or border color change) signals "not a legal move."

---

## 4. Quick-Move System

Four input methods, ranked from fastest to slowest. All are available simultaneously. The system auto-detects which method the player is using based on the input pattern.

### Method 1: Algebraic Notation (fastest for experienced players)

**Input trigger**: Any letter `a-h`, piece letter `NBRQK`, or `O` typed in Normal mode.  
**Display**: Input buffer shown in the status bar at the bottom of the screen.

#### How it works

The player types standard algebraic notation. The system matches against the list of legal moves in real-time. When exactly one legal move matches the current input, it executes immediately.

```
Input Flow Examples:

"e4"  (pawn to e4)
  > Player types 'e' --> buffer shows "e_", system filters legal moves starting with e-pawn
  > Player types '4' --> buffer shows "e4", exactly one match --> EXECUTE
  > Total keystrokes: 2

"Nf3" (knight to f3)
  > Player types 'N' --> buffer shows "N_", system filters knight moves
  > Player types 'f' --> buffer shows "Nf_", may still be ambiguous (Nf3 vs Nf6? only if both legal)
  > Player types '3' --> buffer shows "Nf3", unique match --> EXECUTE
  > Total keystrokes: 3 (or 2 if only one N-to-f move is legal)

"O-O" (kingside castle)
  > Player types 'O' --> buffer shows "O_"
  > Player types 'O' --> buffer shows "OO", interpreted as O-O --> EXECUTE
  > Total keystrokes: 2 (dashes are optional)

"exd5" (pawn capture)
  > Player types 'e' --> buffer shows "e_"
  > Player types 'x' --> buffer shows "ex_" (capture)
  > Player types 'd' --> buffer shows "exd_"
  > Player types '5' --> buffer shows "exd5" --> EXECUTE
  > Total keystrokes: 4 (but often 3, see auto-complete below)
```

#### Auto-complete and Disambiguation

The system maintains a legal-move trie. As each character is typed:

1. Filter the legal moves to those matching the current prefix.
2. If exactly 1 move matches: **execute immediately**. No Enter needed.
3. If 0 moves match: **flash error**, clear buffer, return to Normal mode.
4. If 2+ moves match: **continue accepting input**. Show matching moves in a small overlay near the status bar.

**Smart auto-complete rules**:
- `x` (capture symbol) is optional. `ed5` is interpreted the same as `exd5` if unambiguous.
- Dashes in castling are optional. `OO` = `O-O`, `OOO` = `O-O-O`.
- The `=` in promotion is optional. `e8Q` = `e8=Q`.
- Piece prefix for pawns is never required. `e4` not `Pe4`.
- When two same-type pieces can reach the same square, the system waits for disambiguation (file, rank, or full square). Example: two rooks on a1 and f1, both can go to d1. Typing `Rd1` shows both options. Typing `Rad1` or `Rfd1` resolves it.

#### Input buffer display

```
+------------------------------------------------------+
|                                                      |
|           [Chess board here]                         |
|                                                      |
+------------------------------------------------------+
| e_        matches: e3, e4                       1:23 |
+------------------------------------------------------+
```

The buffer appears inline in the status bar. Matching moves are shown to the right. The clock remains visible.

### Method 2: Square-to-Square (fastest for board-vision players)

**Input trigger**: Detected when the player types a valid source square containing a friendly piece.

```
Input Flow:

"e2e4"
  > 'e' --> buffer: "e_" (could be algebraic or square-to-square, system waits)
  > '2' --> buffer: "e2" (this is a square with a white pawn -- system now expects destination)
  > 'e' --> buffer: "e2e_"
  > '4' --> buffer: "e2e4" --> EXECUTE
  > Total keystrokes: 4
```

**Disambiguation between Method 1 and Method 2**: The system distinguishes them by context:
- If the first two characters form a square with a friendly piece AND a valid algebraic move, the system waits for the third character. If the third character is a file letter (a-h), it is square-to-square. If it is anything else, it is algebraic.
- In practice, ambiguity is rare because algebraic pawn moves specify the destination, not the source.

### Method 3: Cursor Selection (described in Section 3)

Navigate with hjkl, select with Enter/Space, navigate to target, confirm with Enter/Space.

**Keystroke count**: Variable, 4-12+ depending on distance.  
**Best for**: Beginners, position study, when you want to visually verify before committing.

### Method 4: Smart Shortcuts

These are Normal-mode shortcuts that bypass Input mode entirely for common situations.

| Shortcut  | Action                                      | Condition                          |
|-----------|---------------------------------------------|------------------------------------|
| `p`       | Execute the only legal pawn move to this file | Cursor is on a file with exactly one legal pawn advance |
| `Enter` on last-moved piece destination | Premove accept (in premove mode) | Premove is queued |
| `.`       | Repeat last move type (e.g., if last move was e4, `.` tries e5) | VIM repeat analogy |
| `m` + square | Move the currently-selected piece to that square | Piece must be selected |
| `Tab`     | Cycle through legal moves for selected piece | A piece is selected |

#### Piece-prefix shortcuts (Normal mode)

When you type a piece letter and only one legal move exists for that piece type, it executes immediately:

```
Example: Only one knight move is legal (Nf3).
  > Player types 'N' --> only one legal knight move --> EXECUTE Nf3
  > Total keystrokes: 1
```

This is extremely powerful in the endgame and in forcing positions where most pieces have limited mobility.

---

## 5. Key Bindings Reference

### Complete Keybinding Table

#### Normal Mode

| Key           | Action                          | Category      |
|---------------|---------------------------------|---------------|
| `h` / Left    | Cursor left                     | Navigation    |
| `j` / Down    | Cursor down                     | Navigation    |
| `k` / Up      | Cursor up                       | Navigation    |
| `l` / Right   | Cursor right                    | Navigation    |
| `w`           | Jump to next friendly piece     | Navigation    |
| `b`           | Jump to previous friendly piece | Navigation    |
| `g` + sq      | Jump to specific square (e.g. `ge4`) | Navigation |
| `gg`          | Jump to top-left corner         | Navigation    |
| `G`           | Jump to bottom-right corner     | Navigation    |
| `0`           | Jump to leftmost file           | Navigation    |
| `$`           | Jump to rightmost file          | Navigation    |
| `gk`          | Jump to own king                | Navigation    |
| `gq`          | Jump to own queen               | Navigation    |
| `Enter`/`Space`| Select piece / confirm move    | Selection     |
| `Escape`      | Deselect piece / cancel         | Selection     |
| `Tab`         | Cycle legal moves (piece selected) | Selection  |
| `a-h`         | Begin algebraic input (-> Input mode) | Move input |
| `N,B,R,Q,K`   | Begin piece move input (-> Input mode) | Move input |
| `O`           | Begin castling input            | Move input    |
| `/`           | Open explicit input line        | Move input    |
| `.`           | Repeat last move pattern        | Move input    |
| `[`           | Step backward in move history   | History       |
| `]`           | Step forward in move history    | History       |
| `{`           | Jump to start of game           | History       |
| `}`           | Jump to end of game (current position) | History |
| `:`           | Enter Command mode              | Mode switch   |
| `?`           | Show keybinding help overlay    | UI            |
| `Ctrl-f`      | Flip board                      | UI            |
| `Ctrl-e`      | Toggle eval bar                 | UI            |
| `Ctrl-c`      | Toggle captured pieces panel    | UI            |
| `Ctrl-m`      | Toggle move list panel          | UI            |
| `Ctrl-t`      | Toggle clock display            | UI            |
| `Ctrl-p`      | Toggle premove mode             | UI            |

#### Input Mode (active while typing a move)

| Key           | Action                          |
|---------------|---------------------------------|
| `a-h`         | File input                      |
| `1-8`         | Rank input                      |
| `N,B,R,Q,K`   | Piece identifier                |
| `x`           | Capture indicator (optional)    |
| `+`           | Check indicator (optional, ignored) |
| `#`           | Checkmate indicator (optional, ignored) |
| `=`           | Promotion indicator (optional)  |
| `O`           | Castling character              |
| `Backspace`   | Delete last character in buffer |
| `Escape`      | Cancel input, return to Normal  |
| `Enter`       | Force-execute current buffer (if valid) |

#### Command Mode

| Key           | Action                          |
|---------------|---------------------------------|
| Any text      | Command input                   |
| `Tab`         | Auto-complete command            |
| `Backspace`   | Delete last character            |
| `Escape`      | Cancel, return to Normal         |
| `Enter`       | Execute command                  |
| Up/Down       | Command history navigation       |

#### Global Keys (work in any mode)

| Key           | Action                          |
|---------------|---------------------------------|
| `Ctrl-z`      | Undo last move (analysis only)  |
| `Ctrl-q`      | Quit application                |
| `F1`          | Help                            |
| `F2`          | Toggle coordinate labels        |
| `F11`         | Toggle fullscreen (if terminal supports) |

### Premove Keys

When premove mode is active (`Ctrl-p` to toggle), moves can be input during the opponent's turn. They queue and execute when it becomes the player's turn.

| Key           | Action                          |
|---------------|---------------------------------|
| Any move input| Queue a premove                 |
| `Escape`      | Cancel queued premove            |
| `Ctrl-p`      | Toggle premove mode on/off       |

---

## 6. Visual Feedback System

### 6.1 Cursor Position

The cursor square uses **inverted foreground/background colors** of the square's natural color. On a light square, the cursor is shown with a dark background and light text. On a dark square, the cursor is shown with a light background and dark text. This ensures visibility on both square colors without adding extra characters.

```
Normal square:      [  ]  or  [ p ]
Cursor on square:   [>><<] or [>p<]   (inverse colors + subtle bracket markers)
```

Implementation: The cell uses `crossterm::style::Attribute::Reverse` or equivalent. The brackets are conceptual -- the actual rendering uses full-cell background color inversion.

### 6.2 Legal Move Highlights

When a piece is selected, legal destination squares are highlighted:

| Square state               | Visual treatment                            |
|----------------------------|---------------------------------------------|
| Empty legal destination    | Dot in center of square (Unicode `*`)        |
| Capturable enemy piece     | Square background changes to capture-color (red tint) |
| Selected piece's square    | Distinct selection color (bright cyan/green) |
| Non-legal square           | Normal rendering (no change)                |

The dot-on-empty and color-on-capture pattern matches lichess conventions, making the mental model transferable.

### 6.3 Last Move Played

The source and destination squares of the last move are highlighted with a subtle background tint (yellow or amber). This persists until the next move is made. Both the "from" and "to" squares are tinted.

### 6.4 Check Indication

When a king is in check:
- The king's square background turns **red**.
- The status bar displays "CHECK" in bold.
- If the terminal supports it, a brief flash animation plays on the king's square (100ms inverse, 100ms normal, settle on red background).

### 6.5 Move Preview

Before confirming a cursor-based move (Method 3), the system shows a **ghost preview**:

- The piece appears on the destination square in a dimmed/transparent style (half-bright attribute).
- The source square shows as empty (or shows the piece with a strikethrough effect if the terminal supports it).
- An arrow or line connecting source to destination is drawn if terminal space allows.

This preview is visible between "navigate to destination" and "press Enter to confirm." It disappears on `Escape` (cancel) or `Enter` (confirm).

### 6.6 Input Mode Feedback

While in Input mode:

```
+------------------------------------------------------+
|                                                      |
|           [Board with real-time highlights]           |
|                                                      |
+------------------------------------------------------+
| > Nf_     | Nf3  Nf6                          [1:45] |
+------------------------------------------------------+
```

- The input buffer is prefixed with `>`.
- Matching legal moves are listed to the right.
- As the player types, matching destination squares on the board are highlighted in real time (same style as legal move highlights).
- The clock remains visible at all times.

### 6.7 Premove Feedback

Queued premoves are shown with a distinct visual treatment:
- Source square: dashed outline or dimmed original piece.
- Destination square: piece shown in a muted/ghosted color.
- A small indicator in the status bar: "PREMOVE: e2e4" (or whatever the queued move is).

---

## 7. Speed Optimizations

### 7.1 Keystroke Counts for Common Moves

| Move            | Method 1 (Algebraic) | Method 2 (Sq-to-Sq) | Method 3 (Cursor) |
|-----------------|----------------------|----------------------|--------------------|
| 1. e4           | **2** (`e4`)         | 4 (`e2e4`)           | ~6 (nav+sel+nav+confirm) |
| 1. d4           | **2** (`d4`)         | 4 (`d2d4`)           | ~6 |
| 2. Nf3          | **2-3** (`Nf3` or `N` if only one) | 4 (`g1f3`) | ~8 |
| 2...e5          | **2** (`e5`)         | 4 (`e7e5`)           | ~6 |
| O-O             | **2** (`OO`)         | 4 (`e1g1`)           | ~10 |
| O-O-O           | **3** (`OOO`)        | 4 (`e1c1`)           | ~12 |
| Bxf7+           | **3-4** (`Bf7` or `Bxf7`) | 4              | ~8 |
| exd5            | **3-4** (`ed5` or `exd5`) | 4              | ~6 |
| e8=Q            | **3** (`e8Q` -- auto-promotes to queen) | 5 (`e7e8q`) | ~6 |

**Target achieved**: All common moves are 2-3 keystrokes with algebraic input.

### 7.2 Promotion Handling

**Default**: Pawn reaching the 8th (or 1st) rank promotes to Queen automatically. No extra keystroke needed.

**Override**: To promote to a different piece, append the piece letter:
- `e8N` -- promote to knight (3 keystrokes)
- `e8R` -- promote to rook (3 keystrokes)
- `e8B` -- promote to bishop (3 keystrokes)

**Cursor method**: When a pawn reaches the promotion rank via cursor selection, a small popup appears:

```
  Promote to:
  [Q] Queen  (default, press Enter)
  [N] Knight
  [R] Rook
  [B] Bishop
```

Pressing `Q`, `N`, `R`, or `B` (or just `Enter` for queen) confirms. This popup has a `[PLACEHOLDER]` timeout of 3 seconds before defaulting to queen in bullet time controls.

### 7.3 Auto-Execute (the single biggest speed optimization)

The system never requires Enter to confirm a move in algebraic input when the move is unambiguous. This is the key differentiator from any text-based chess interface.

```
Traditional: type "e4" + press Enter = 3 actions
ChessTUI:    type "e4" = 2 actions, auto-executed on '4'
Savings:     33% fewer keystrokes on every single move
```

Over a 40-move game, this saves 40 keystrokes -- meaningful in bullet chess.

### 7.4 Premove Chains

In bullet/blitz, players often know their next 2-3 moves. The premove system allows:
1. Enter premove mode with `Ctrl-p` (or it is always-on in bullet time controls `[PLACEHOLDER]`).
2. Type your premove using any input method.
3. The premove queues. If the opponent's move makes it illegal, the premove is cancelled and the player is notified with a visual flash.
4. Only one premove at a time is supported in v1. `[PLACEHOLDER]` -- consider premove chains in v2.

### 7.5 Move Input While Opponent is Thinking

Even without formal premove mode, the player can begin typing their next move during the opponent's turn. The input buffer activates, destination squares highlight based on the current position, and the move executes the moment it becomes the player's turn if the input is complete and legal at that point.

---

## 8. Onboarding

### First-Time User Experience

#### Beat 1: First Contact (0-15 seconds)

The player sees the board rendered in Unicode with a blinking cursor on e2 (white) or e7 (black). A minimal hint appears in the status bar:

```
Arrow keys to move cursor | Enter to select | ? for help
```

No modal dialog. No tutorial popup. The player can immediately start exploring.

#### Beat 2: First Move (15-60 seconds)

When the player navigates the cursor to a piece and presses Enter, legal moves highlight automatically. This is the "discovery through exploration" moment. The player learns the select-navigate-confirm loop without reading instructions.

A subtle hint updates:

```
Navigate to a highlighted square and press Enter to move
```

#### Beat 3: Speed Discovery (1-5 minutes)

After the player makes their first cursor-based move, a brief non-blocking notification appears:

```
TIP: Type moves directly -- try typing "e4" or "Nf3"
```

This introduces Method 1. The tip appears once and does not repeat.

#### Beat 4: Help Overlay (on-demand)

Pressing `?` at any time shows a help overlay with the keybinding reference, grouped by category. The overlay is dismissible with `Escape` or `?` again.

### Onboarding Checklist

- [x] Core verb (move cursor) available within 0 seconds -- arrow keys work immediately
- [x] First success guaranteed -- cursor movement always works, no failure state
- [x] Each mechanic introduced in safe context -- legal move highlights prevent illegal moves
- [x] Discovery through exploration -- selecting a piece reveals legal moves visually
- [x] First session ends on a hook -- the speed difference between cursor and algebraic input motivates learning

---

## 9. Tuning Variables

| Variable                    | Value          | Min   | Max   | Rationale / Notes |
|-----------------------------|----------------|-------|-------|-------------------|
| Input buffer timeout        | None (no timeout) | -  | -     | Player controls pacing; timeout would cause misfire |
| Promotion popup timeout     | 3s `[PLACEHOLDER]` | 1s | 10s | Bullet needs fast default; classical needs time to choose |
| Check flash duration        | 200ms `[PLACEHOLDER]` | 100ms | 500ms | Must be noticeable but not disruptive |
| Legal move highlight opacity| 100% (full)    | 50%   | 100%  | Must be clearly visible on all themes |
| Cursor blink rate           | 530ms `[PLACEHOLDER]` | 250ms | 1000ms | Match terminal default for familiarity |
| Last-move highlight persist | Until next move | -     | -     | Standard chess UI convention |
| Auto-complete threshold     | 1 match = execute | -  | -     | Core speed mechanic -- do not add delay |
| Premove auto-enable         | Off `[PLACEHOLDER]` | -  | -     | Consider always-on for bullet (<2min) |
| Max premove queue depth     | 1 `[PLACEHOLDER]`  | 1  | 5     | v1 = 1, test demand for chains |
| Input case sensitivity      | Insensitive    | -     | -     | `nf3` = `Nf3`; reduces friction |
| Disambiguation display max  | 6 moves        | 3     | 10    | More than 6 is visually overwhelming |
| Ghost preview render delay  | 0ms (instant)  | -     | -     | Any delay breaks flow |

---

## 10. Edge Cases and Failure States

### 10.1 Ambiguous Input

**Scenario**: Player types `Nd2` but both knights can go to d2.  
**Handling**: System does not execute. Status bar shows: `Nd2? Disambiguate: Nbd2 / Nfd2`. Player types the next character to resolve. If they press Enter without disambiguating, nothing happens.

### 10.2 Invalid Input

**Scenario**: Player types `e9` or `Xf3` or gibberish.  
**Handling**: When the input buffer has no legal move matches, the buffer text turns red. Continuing to type does nothing productive. `Backspace` edits the buffer. `Escape` clears it entirely. An invalid buffer auto-clears after 2 seconds of inactivity `[PLACEHOLDER]`.

### 10.3 Input Collision: Algebraic vs. Navigation

**Scenario**: Player presses `b` in Normal mode. Is this "navigate to previous piece" (VIM `b`) or "start typing a pawn move on the b-file"?  
**Resolution**: This is the most significant design tension in the system. The resolution:

- `b` in Normal mode initiates Input mode (interpreted as b-file pawn move or Bishop move).
- `w` in Normal mode is "jump to next piece" -- this key is not a valid file and causes no conflict.
- "Jump to previous piece" is remapped to `B` (shift-b) in Normal mode, since `B` is also the Bishop piece prefix but is only used in Input mode.

Wait -- this creates another conflict. Let me resolve cleanly:

**Final resolution (no conflicts)**:
- `w` and `W`: Next/previous friendly piece jump. These are not chess file names, so no conflict.
- `b` (lowercase): Enters Input mode, starts algebraic input for b-file or Bishop.
- `B` (uppercase) in Normal mode: Jump to previous friendly piece. In Input mode, `B` is the Bishop prefix -- no conflict because the modes are separate.

Alternatively, use a `w`/`e` pairing for piece jumping:
- `w`: Jump to next friendly piece.
- `e`: Jump to previous friendly piece.

`e` conflicts with the e-file. So:

**Cleanest resolution**: Piece-jumping uses only `w` (forward) and `,` / `.` or `(` / `)` -- keys that have no algebraic meaning.

Final answer for piece jumping:
- `w`: Jump cursor to next friendly piece (wrapping).
- `W`: Jump cursor to previous friendly piece (wrapping).
- All file letters `a-h` are reserved for Input mode entry. No conflict.

### 10.4 Time Pressure Mistakes

**Scenario**: Player in bullet accidentally types a legal but unintended move due to auto-execute.  
**Mitigation**: 
- `Backspace` cannot undo an executed move (it already happened).
- Takeback request (`:tb`) is available but requires opponent agreement.
- In analysis mode, `u` undoes moves freely.
- `[PLACEHOLDER]` -- Consider an optional "confirm mode" setting that requires Enter even for unambiguous moves. Default OFF for speed, available for cautious players.

### 10.5 Castling Edge Cases

**Scenario**: Player types `O` -- could be start of `OO` (O-O) or `OOO` (O-O-O).  
**Handling**: 
- After first `O`, system waits.
- After second `O`: if O-O is legal but O-O-O is also legal, system waits for one more character.
- After third `O`: execute O-O-O.
- After second `O` + any non-`O` character: execute O-O (if legal), push the non-O character back to the buffer.
- If only O-O is legal (not O-O-O), `OO` auto-executes immediately.
- If only O-O-O is legal (not O-O), `OO` does not match O-O, system waits for third `O`.

### 10.6 Pawn Promotion Ambiguity

**Scenario**: Pawn on e7 can go to e8 or capture on d8 or f8, each with 4 promotion options.  
**Handling**: 
- `e8` is not immediately executed because promotion choice is needed.
- System shows promotion popup. Player types piece letter.
- `e8Q` = promote to queen (3 keystrokes total, or 2 + auto-queen after timeout).
- `ed8N` = capture on d8, promote to knight (4 keystrokes).

### 10.7 Simultaneous Key Events

**Scenario**: Player presses keys very rapidly, buffer receives `e4Nf3` as a stream.  
**Handling**: The system processes characters sequentially. `e4` matches and auto-executes immediately. The remaining `Nf3` enters the buffer as a new move. If it is now the opponent's turn, the `Nf3` is treated as premove input (if premove mode is active) or discarded (if premove mode is off).

---

## 11. Changelog

| Version | Date       | Changes                          |
|---------|------------|----------------------------------|
| 1.0     | 2026-04-03 | Initial design document          |

---

## Appendix A: System Interaction Matrix

| System A         | System B           | Interaction Type | Notes |
|------------------|--------------------|------------------|-------|
| Input Mode       | Normal Mode        | Intended         | Input mode is entered from Normal; ESC returns to Normal |
| Input Mode       | Clock              | Intended         | Clock always visible during input |
| Input Mode       | Premove            | Intended         | Input during opponent turn queues premove |
| Cursor Selection | Input Mode         | Exclusive        | Cannot use both simultaneously; entering one exits the other |
| Auto-execute     | Promotion          | Exception        | Auto-execute is suspended when promotion choice is needed |
| Auto-execute     | Castling           | Conditional      | Auto-execute only when castle direction is unambiguous |
| Board Flip       | Navigation         | Intended         | hjkl directions are screen-relative, not algebraic-relative |
| History Nav      | Input Mode         | Exclusive        | Cannot input moves while browsing history (unless analysis mode) |
| Analysis Mode    | Premove            | Incompatible     | Premove is meaningless in analysis; disabled |
| Command Mode     | Clock              | Intended         | Clock continues during command input |

## Appendix B: Input State Machine

```
                         [NORMAL]
                        /    |    \
                       /     |     \
                a-h,  /  Enter/  \  :
              NBRQKO /   Space    \
                    /      |       \
                   v       v        v
              [INPUT]  [SELECTED]  [COMMAND]
                |         |    \       |
       auto-exec|    hjkl |  Esc\  Enter|
          or Esc|  Enter  |     |    Esc|
                |    /    |     |      |
                v  v      v     v      v
              [NORMAL] [MOVE   [NORMAL] [NORMAL]
                       EXECUTED]
                          |
                          v
                       [NORMAL]
```

## Appendix C: Comparison with Existing Chess TUI/CLI Tools

| Feature              | ChessTUI (this design) | lichess (browser) | chess-tui (existing) |
|----------------------|------------------------|--------------------|-----------------------|
| Algebraic auto-exec  | Yes (0 confirm)        | No (mouse-based)   | No                    |
| VIM-style nav        | Full hjkl + motions    | N/A                | Partial               |
| Premove              | Yes                    | Yes                | No                    |
| Min keystrokes (e4)  | 2                      | 2 (click+click)    | 4+                    |
| Modal system         | Yes (Normal/Input/Cmd) | N/A                | No                    |
| Legal move preview   | Yes                    | Yes                | Limited               |
| Move preview ghost   | Yes                    | No                 | No                    |
