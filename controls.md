  Method 1: Algebraic Notation (2-3 keystrokes)

  Just start typing the move. No prefix, no Enter needed:
  - e4 — 2 keystrokes, auto-executes when unique match found
  - Nf3 — 2-3 keystrokes (2 if only one knight move is legal)
  - OO — 2 keystrokes for castling (dashes optional)
  - exd5 — can be typed as ed5 (capture x optional)
  - Promotion: e8Q auto-executes (defaults to queen, = optional)

  The system maintains a legal-move trie — as you type each character, it filters legal moves. When exactly 1 move matches, it fires
  instantly.

  Method 2: Square-to-Square (e2e4)

  Type source square + destination square. 4 keystrokes, always unambiguous.

  Method 3: Cursor Navigation

  hjkl to move cursor → Enter/Space to select piece → legal moves highlight → navigate to target → Enter/Space to confirm. Slowest but most
  visual, great for beginners.

  Method 4: Smart Piece Jumps

  - w / b — jump cursor to next/previous friendly piece
  - gk — jump to your king
  - gq — jump to your queen
  - ge4 — jump cursor directly to e4

  Full Key Bindings

  ┌─────────────┬──────────────────────────────────┬─────────────┐
  │     Key     │              Action              │ VIM Analogy │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ h/j/k/l     │ Move cursor left/down/up/right   │ Same        │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ w           │ Jump to next piece               │ w word-jump │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ b           │ Jump to previous piece           │ b back-word │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ Enter/Space │ Select piece / confirm move      │ —           │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ Esc         │ Cancel selection / exit mode     │ Same        │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ g + square  │ Jump cursor to square            │ gg / G      │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ 0 / $       │ Jump to a1 / h1                  │ Same        │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ [ / ]       │ Browse move history back/forward │ —           │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ u           │ Undo (analysis mode)             │ Same        │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ Ctrl-r      │ Redo (analysis mode)             │ Same        │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ :           │ Enter command mode               │ Same        │
  ├─────────────┼──────────────────────────────────┼─────────────┤
  │ /           │ Open algebraic input line        │ VIM search  │
  └─────────────┴──────────────────────────────────┴─────────────┘