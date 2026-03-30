# wordle-tui — PLAN.md

Terminal Wordle clone in Rust + ratatui. Looks sexy. Plays clean.

---

## Stack

- **Rust** (2021 edition)
- **ratatui** — TUI rendering
- **crossterm** — terminal backend + raw key events
- **rand** — word selection

No external word API. Word list bundled at compile time via `include_str!`.

---

## Project Structure

```
wordle-tui/
├── Cargo.toml
├── words.txt          # ~2000 common 5-letter words
└── src/
    ├── main.rs        # entry point, event loop
    ├── app.rs         # App state machine
    ├── game.rs        # game logic (guesses, validation, letter states)
    ├── ui.rs          # ratatui layout + widgets
    └── words.rs       # word list loader (include_str!)
```

---

## Game Logic (`game.rs`)

### State

```rust
pub enum LetterState {
    Correct,    // right letter, right position  → green
    Present,    // right letter, wrong position  → yellow
    Absent,     // not in word                   → gray
    Unknown,    // not yet guessed               → dim
}

pub struct Guess {
    pub letters: [char; 5],
    pub states:  [LetterState; 5],
}

pub struct Game {
    pub target:       [char; 5],
    pub guesses:      Vec<Guess>,       // max 6
    pub current:      Vec<char>,        // letters typed so far (0–5)
    pub keyboard:     HashMap<char, LetterState>,
    pub status:       GameStatus,       // Playing / Won / Lost
}
```

### Core fn

- `Game::new(word_list)` — pick random target
- `Game::type_char(c)` — push to current (max 5)
- `Game::backspace()` — pop from current
- `Game::submit()` — validate length, score letters, push Guess, check win/loss
- `score_guess(target, guess) -> [LetterState; 5]` — classic Wordle scoring (handle duplicate letters correctly)

---

## App State Machine (`app.rs`)

```rust
pub enum Screen {
    Playing,
    Won,
    Lost,
    Quit,
}

pub struct App {
    pub game:   Game,
    pub screen: Screen,
    pub words:  Vec<String>,
    pub shake:  Option<Instant>,   // invalid submission → brief shake animation
    pub msg:    Option<String>,    // "Not enough letters", "Not in word list", etc.
}
```

Event loop: `crossterm::event::read()` → match key → mutate App → re-render.

---

## UI Layout (`ui.rs`)

```
┌─────────────────────────────┐
│         W O R D L E         │  ← title, centered, bold+cyan
├─────────────────────────────┤
│                             │
│   [ ][ ][ ][ ][ ]           │  ← row 1 (past guess or empty)
│   [ ][ ][ ][ ][ ]           │
│   [ ][ ][ ][ ][ ]           │
│   [ ][ ][ ][ ][ ]           │
│   [ ][ ][ ][ ][ ]           │
│   [ ][ ][ ][ ][ ]           │  ← row 6
│                             │
│   Q W E R T Y U I O P       │  ← keyboard hint row
│    A S D F G H J K L        │
│      Z X C V B N M          │
│                             │
│   "Not enough letters"      │  ← ephemeral message (auto-clears after 1.5s)
└─────────────────────────────┘
```

### Tile rendering

Each tile = `Block` with border + centered letter char.

| State     | Border color | BG color  | Letter   |
|-----------|-------------|-----------|----------|
| Unknown   | dark gray    | none      | dim      |
| Active    | white        | none      | bold     |
| Correct   | green        | green     | bold     |
| Present   | yellow       | yellow    | bold     |
| Absent    | dark gray    | dark gray | dim      |

Active row tiles get a **white border** to signal focus. Current typing position gets a **blinking cursor effect** (alternate style every 500ms tick).

### Keyboard hint

Each letter rendered as a small colored `Span` based on its best known `LetterState`. Same color scheme as tiles.

### Win / Lost overlay

Centered `Paragraph` popup (no full clear):
- **Won**: `🎉 GENIUS / MAGNIFICENT / etc.` based on guess count + the word
- **Lost**: the target word revealed in green
- Both show `[R] new game  [Q] quit`

---

## Event Handling (`main.rs`)

```
loop {
    terminal.draw(|f| ui::render(f, &app))?;

    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_alphabetic() => app.type_char(c),
                KeyCode::Enter     => app.submit(),
                KeyCode::Backspace => app.backspace(),
                KeyCode::Char('r') if app.is_done() => app.new_game(),
                KeyCode::Char('q') | KeyCode::Esc    => break,
                _ => {}
            }
        }
    }

    app.tick(); // clear expired messages, advance cursor blink
}
```

---

## Word List

- `words.txt` bundled via `include_str!("../words.txt")`
- ~2000 common English 5-letter words (no obscure scrabble words)
- Loaded once at startup, stored in `App`
- Guess validation: check if input is in word list (or allow any valid 5-letter word — configurable)

---

## Cargo.toml deps

```toml
[dependencies]
ratatui   = "0.29"
crossterm = "0.28"
rand      = "0.8"
```

---

## Polish / Nice-to-haves (post-MVP)

- [ ] Flip animation on submit (tile border cycles through styles row by row, 80ms delay per tile)
- [ ] Shake animation on invalid word (jitter tile X offset ±1 for 3 frames)
- [ ] Streak counter persisted to `~/.local/share/wordle-tui/stats.json`
- [ ] `--hard` flag (must use confirmed hints in subsequent guesses)
- [ ] Color-blind mode flag (`--cb`) swaps green→blue, yellow→orange

---

## Build & Run

```bash
cargo run --release
```

One binary. No config needed. Works in any terminal that supports 256 colors.

---

## MVP Checklist

- [ ] `words.rs` — load + pick random word
- [ ] `game.rs` — scoring logic with correct duplicate handling
- [ ] `app.rs` — state machine, key dispatch
- [ ] `ui.rs` — grid + keyboard hint rendering
- [ ] `main.rs` — event loop + terminal setup/cleanup
- [ ] Win/Lost screens
- [ ] Ephemeral message system
- [ ] Cursor blink on active tile
