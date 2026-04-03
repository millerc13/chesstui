# Chess AI System Design

Self-contained, multi-level chess engine for the chesstui Rust binary.
No external engines, no cloud calls, no subprocess spawning.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  TUI Game Loop (main thread, async)                     │
│  ┌───────────┐  ┌───────────┐  ┌─────────────────────┐ │
│  │  Renderer  │  │  Input    │  │  AI Controller      │ │
│  │  (ratatui) │  │  Handler  │  │  (channel-based)    │ │
│  └───────────┘  └───────────┘  └──────────┬──────────┘ │
│                                           │ mpsc/oneshot│
└───────────────────────────────────────────┼─────────────┘
                                            │
┌───────────────────────────────────────────┼─────────────┐
│  AI Thread (std::thread, blocking)        │             │
│  ┌────────────────────────────────────────▼───────────┐ │
│  │  ChessAI                                          │ │
│  │  ┌──────────┐  ┌────────────┐  ┌───────────────┐  │ │
│  │  │ Opening  │  │  Search    │  │  Personality  │  │ │
│  │  │ Book     │  │  Engine    │  │  System       │  │ │
│  │  │ (probe)  │  │  (ID+AB)  │  │  (blunders)   │  │ │
│  │  └──────────┘  └─────┬──────┘  └───────────────┘  │ │
│  │                      │                            │ │
│  │              ┌───────▼──────┐                     │ │
│  │              │  Evaluation  │                     │ │
│  │              │  Function    │                     │ │
│  │              │  (configurable components)         │ │
│  │              └──────────────┘                     │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  Board Representation: cozy-chess (bitboards)           │
│  Transposition Table: Vec<Option<TTEntry>>              │
└─────────────────────────────────────────────────────────┘
```

## 1. Board Representation

**Decision: Use `cozy-chess` crate for move generation. Build search/eval on top.**

Rationale:
- `cozy-chess` uses bitboards internally (12 u64s: one per piece per color)
- Legal move generation is correct and tested (this is the hardest part to get right)
- Safe Rust (no `unsafe` blocks)
- ~2-3 million legal move generations per second, sufficient for depth 8-10
- The TUI can maintain its own simple `[[Option<Piece>; 8]; 8]` for rendering
  and convert to/from `cozy_chess::Board` at the boundary

**Why not write our own?**
A correct move generator handles castling through check, en passant, promotion,
pins, discovered check, and dozens of edge cases. It takes weeks to get right and
months to debug. The search and evaluation -- where difficulty personality lives --
is where our time should go.

## 2. Search Algorithm

### Iterative Deepening + Alpha-Beta + PVS + Quiescence

```
for depth in 1..=max_depth:
    score = alpha_beta(root, depth, -INF, +INF)
    if time_expired: break
    save best_move from this iteration
return best_move
```

**Why iterative deepening?**
- Natural time control: stop after the last completed depth
- Move ordering from shallower iterations makes deeper iterations faster
- The overhead of re-searching is small (~10%) due to better pruning

**Alpha-beta pruning** reduces the branching factor from ~35 to ~6 with good
move ordering. **PVS** (Principal Variation Search) improves this further by
using null-window searches for non-PV moves.

**Quiescence search** prevents the "horizon effect" -- the engine stops searching
at a depth limit, but what if a queen capture is the very next move? Quiescence
continues searching capture-only moves until the position is "quiet."

### Node Budget by Depth

| Depth | Approx Nodes (with pruning) | Time at 2M NPS |
|-------|---------------------------|-----------------|
| 1     | 35                        | <1ms            |
| 2     | 200                       | <1ms            |
| 3     | 1,500                     | <1ms            |
| 4     | 12,000                    | 6ms             |
| 5     | 80,000                    | 40ms            |
| 6     | 500,000                   | 250ms           |
| 7     | 2,500,000                 | 1.2s            |
| 8     | 10,000,000                | 5s (with TT: ~2s)|
| 10    | 50,000,000                | needs TT + time limit |

At depth 8+ the transposition table is critical for staying under 2 seconds.

## 3. Evaluation Function

See `src/engine/evaluation.rs` for the full implementation. Components:

| Component          | Weight | Description                              | Active at Level |
|-------------------|--------|------------------------------------------|-----------------|
| Material          | Base   | Piece values (P=100, N=320, B=330, R=500, Q=900) | 1+ |
| Piece-Square Tables | +/-50 | Positional bonus per piece per square    | 2+              |
| Center Control    | +10/sq | Bonus for occupying e4/d4/e5/d5          | 2+              |
| King Safety       | +/-50  | Pawn shield, open files near king        | 3+              |
| Pawn Structure    | +/-20  | Doubled, isolated pawn penalties         | 4+              |
| Mobility          | +1-3/sq| Legal moves per piece type               | 4+              |
| Bishop Pair       | +30    | Bonus for having both bishops            | 5+              |
| Rook on Open File | +25    | Rook on file with no friendly pawns      | 5+              |
| Passed Pawns      | +10-120| Bonus by rank advancement                | 6+              |
| Endgame Knowledge | varies | King centralization, pawn racing          | 7+              |

### Endgame vs Middlegame

The evaluation switches between middlegame and endgame piece-square tables
for the king. In the middlegame, the king wants to hide behind pawns (castled).
In the endgame, the king becomes an active piece and wants the center.

Detection: endgame when total non-pawn material <= 14 "points"
(queen=9, rook=5, minor=3).

## 4. Difficulty Levels

| Lvl | Name                | ELO  | Depth | QS  | Eval Components     | Blunder% | Noise | TT   |
|-----|---------------------|------|-------|-----|---------------------|----------|-------|------|
| 1   | Pawn Pusher         | 400  | 1     | 0   | Material            | 35%      | 150cp | None |
| 2   | Coffee Shop         | 600  | 2     | 1   | + PST, Center       | 25%      | 120cp | None |
| 3   | Park Bench          | 800  | 3     | 2   | + King Safety       | 18%      | 90cp  | None |
| 4   | Casual Player       | 1000 | 3     | 3   | + Pawns, Mobility   | 12%      | 60cp  | 64K  |
| 5   | Tournament Hopeful  | 1200 | 4     | 4   | + Bishop Pair, Rooks| 7%       | 40cp  | 128K |
| 6   | Rated Player        | 1400 | 5     | 6   | + Passed Pawns      | 4%       | 25cp  | 256K |
| 7   | Club Veteran        | 1600 | 6     | 8   | Full                | 2%       | 15cp  | 512K |
| 8   | Club Champion       | 1800 | 7     | 10  | Full                | 0%       | 8cp   | 1M   |
| 9   | Expert              | 1950 | 8     | 12  | Full                | 0%       | 0cp   | 1M   |
| 10  | Grandmaster Wannabe | 2100 | 10    | 16  | Full                | 0%       | 0cp   | 2M   |

### Three Knobs for Difficulty

1. **Search depth**: Lower levels literally cannot see multi-move tactics
2. **Evaluation complexity**: Lower levels misjudge positions (no king safety = walks into attacks)
3. **Personality blunders**: Even when the engine finds the best move, it sometimes plays something worse

These three dimensions create qualitatively different play at each level.

## 5. Making Weak AI Feel Natural

This is the hardest and most important design problem. See `src/engine/personality.rs`.

### Anti-Patterns (what NOT to do)

- **Random move injection**: A depth-4 engine that plays randomly 30% of the time
  produces schizophrenic play -- brilliant moves alternating with insane ones.
  Real beginners don't do this.
- **Uniform random selection**: Picking uniformly from all legal moves produces
  non-human patterns (king walks, pointless rook shuffles).

### Human-Like Weakness Strategies

**Strategy 1: Evaluation Noise (Misjudging Positions)**

Add random noise to the static evaluation. This makes the AI think a slightly
worse position is slightly better, leading to subtle positional errors -- exactly
like a human who doesn't fully understand pawn structure or king safety.

```
eval_noise_cp = 60  // at level 4
actual_eval = evaluate(position)
perceived_eval = actual_eval + random(-60, +60)
```

This is the most effective single technique. At 60cp noise, the AI will sometimes
prefer a slightly worse pawn structure or miss that a piece is slightly misplaced.

**Strategy 2: Motivated Blunders**

When the AI does blunder, it picks from categories that match human error patterns:
- **Tempting moves** (40%): Captures and checks that look good but aren't best.
  Humans love taking free pieces even when there's a better quiet move.
- **Lazy moves** (25%): Development moves when a tactic is available.
  Humans often play "natural" moves and miss tactics.
- **Greedy moves** (20%): Taking material even at a positional cost.
  Humans grab pawns when they shouldn't.
- **Second-best moves** (15%): Just slightly inaccurate.
  The most common human error -- playing a reasonable but non-optimal move.

**Strategy 3: Blind Spots**

Lower levels literally cannot see certain patterns because their search depth
is too shallow:
- Level 1 (depth 1): Cannot see ANY tactics. Will hang pieces.
- Level 2 (depth 2): Sees one-move captures but not two-move combinations.
- Level 3 (depth 3): Sees forks and pins but not complex sacrifices.
- Levels 1-2: Cannot see back-rank mates (their king safety eval is off).

This is actually the most natural form of weakness -- it's how humans improve
(by learning to see deeper).

**Strategy 4: Opening Book as Strength Limiter**

- Levels 1-2: No opening book. They play weird openings like 1. a4 or 1... f5.
  This looks natural for a beginner.
- Levels 3+: Opening book. They play reasonable openings, then their weakness
  shows in the middlegame. This avoids the "the AI played the Italian Game
  perfectly then hung its queen" problem.

**Strategy 5: Human-Like Timing**

Even when the AI computes instantly (depth 1 takes microseconds), it waits
500ms-1.2s before moving. This prevents the eerie instant-response that
breaks immersion. Higher levels respond more quickly because stronger players
often play quickly in clear positions.

## 6. Opening Book

**Design: Hardcoded move sequences, stored as hash-to-move map.**

24 common openings, ~6 moves deep each. Total data: ~3 KB in the binary.
No Polyglot .bin file needed.

The book is built at startup by replaying each opening line on a board and
recording `board.hash() -> next_move` mappings. Lookup is O(1) by hash.

Weighted random selection allows variety -- the AI won't always play the
same opening. The Italian Game and Ruy Lopez have higher weights than the
Alekhine Defense.

### Why Not Polyglot?

A Polyglot book adds 5-30 MB to the binary and requires a parser. For a TUI
game, 24 openings provide plenty of variety. If you want more variety later,
you can embed a Polyglot file with `include_bytes!()` and add a parser (~200
lines of code).

## 7. Performance Budget

### Target: 2M nodes/second in Rust

This is conservative for a Rust engine with bitboard move generation.
For reference:
- Stockfish: ~100M NPS (heavily optimized C++, NNUE)
- A basic Rust engine with cozy-chess: 2-5M NPS
- With good move ordering and TT: effectively 5-10M NPS (due to pruning)

### Transposition Table Sizing

| Level | TT Entries | Entry Size | Total RAM |
|-------|-----------|------------|-----------|
| 1-3   | 0         | -          | 0         |
| 4     | 64K       | 24 bytes   | 1.5 MB    |
| 5     | 128K      | 24 bytes   | 3 MB      |
| 6     | 256K      | 24 bytes   | 6 MB      |
| 7     | 512K      | 24 bytes   | 12 MB     |
| 8-9   | 1M        | 24 bytes   | 24 MB     |
| 10    | 2M        | 24 bytes   | 48 MB     |

The TT is a fixed-size vector with hash-based indexing (no dynamic allocation
during search). Collisions are handled by replacement -- newer entries overwrite
older ones.

### Time Budget Breakdown

At level 10 (depth 10, 2 second limit):
- Depth 1-6: <500ms (completed quickly, provides fallback move)
- Depth 7-8: 500ms-1.5s (main work)
- Depth 9-10: may or may not complete, depends on position complexity
- Iterative deepening guarantees we always have a move from the last completed depth

## 8. Integration with Game Loop

See `src/engine/integration.rs` for the full implementation.

### Threading Model

```
Main Thread (async, tokio)          AI Thread (blocking, std::thread)
├── Render UI                       ├── Wait for ThinkRequest
├── Handle Input                    ├── Run search (CPU-bound)
├── Send ThinkRequest ──────────►   ├── Apply personality
├── Show "thinking..." animation    ├── Apply min_think_time delay
├── Poll for AI response            ├── Send AIResult back
├── Play AI's move   ◄──────────    └── Wait for next request
└── Continue game loop
```

Key design decisions:
- **Separate OS thread**, not `tokio::spawn_blocking`: The search is CPU-bound
  and can run for up to 2 seconds. Using a dedicated thread avoids starving
  the tokio runtime.
- **Channel-based communication**: `mpsc::Sender` for requests,
  `oneshot::Sender` for responses. Non-blocking on the game loop side.
- **Cancellation via `AtomicBool`**: The search checks this flag every 4096
  nodes. The UI can cancel instantly (e.g., on undo or quit).

### Thinking Animation

While `ai_controller.is_thinking()` returns true, the TUI renders a cycling
animation: `"Thinking."` -> `"Thinking.."` -> `"Thinking..."`. This is driven
by the render loop's 50ms tick, not by the AI thread.

### AI in Multiplayer

The AI controller is a generic `AIController` -- you can instantiate one per
empty seat. For a "Player vs AI" game, create one controller. For "AI vs AI"
analysis mode, create two controllers at different difficulty levels.

## 9. Crate Recommendations

| Crate       | Purpose                        | Recommendation |
|-------------|-------------------------------|----------------|
| `cozy-chess`| Move generation (bitboards)    | USE -- saves weeks of work |
| `rand`      | Personality RNG, book selection| USE            |
| `ratatui`   | TUI rendering                 | USE (already planned) |
| `crossterm` | Terminal backend               | USE (already planned) |
| `tokio`     | Async runtime, channels        | USE (already planned) |

**Do NOT use:**
- `shakmaty`: Good crate but `cozy-chess` is simpler and sufficient
- `chess`: Older, less maintained than `cozy-chess`
- Any NNUE or neural network crate: Overkill, adds huge binary size

### What to Write from Scratch

- **Search algorithm**: Must be custom (this is where difficulty tuning lives)
- **Evaluation function**: Must be custom (component-based, difficulty-controlled)
- **Personality system**: Entirely custom (this is the game design, not the engine)
- **Opening book**: Custom (just hardcoded move lists)

### What to Use a Crate For

- **Move generation**: `cozy-chess` (correctness is critical, testing takes months)
- **Random numbers**: `rand` (don't roll your own)

## 10. File Structure

```
src/engine/
├── mod.rs           # ChessAI top-level controller, AIResult
├── difficulty.rs    # 10 difficulty levels with all parameters
├── evaluation.rs    # Static position evaluation (configurable components)
├── search.rs        # Iterative deepening + alpha-beta + quiescence + TT
├── personality.rs   # Human-like mistake generation
├── opening_book.rs  # Compact inline opening book (24 lines)
├── tables.rs        # Piece values, piece-square tables, constants
└── integration.rs   # Game loop integration (threading, channels)
```

## 11. Future Enhancements (Not in v1)

- **Null move pruning**: Skip a move to get a quick lower bound. Massive speedup
  at depth 7+. Add after v1 is working.
- **Late move reduction (LMR)**: Search unpromising moves at reduced depth.
  Another significant speedup.
- **Killer move heuristic**: Remember moves that caused beta cutoffs at each ply.
  Better move ordering.
- **Aspiration windows**: Narrow the alpha-beta window based on previous iteration.
  Small speedup.
- **Syzygy tablebases**: Perfect endgame play with 6 or fewer pieces. Adds ~150 MB
  to the binary via `include_bytes!()`. Only worth it for level 10.
- **NNUE evaluation**: Neural network evaluation for stronger positional play.
  Adds ~40 MB to the binary. Would push level 10 to ~2400 ELO.
