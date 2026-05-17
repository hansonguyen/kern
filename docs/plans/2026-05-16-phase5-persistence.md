# Phase 5: Persistence — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** After each completed test, write the session result to `~/.config/kern/stats.json`; load history into `Model` on startup.

**Architecture:** `Command::SaveStats(StatsPayload)` carries pure metrics from `update` (pure) to `execute_command` (impure). `execute_command` stamps the timestamp, builds `SessionResult`, pushes to `model.history`, and calls `persistence::append`. History is loaded from disk in `main.rs` after `Model::default()` and stored in `model.history` for future display phases.

**Tech Stack:** `serde` + `serde_json` for JSON serialization; `thiserror` for domain errors; `tempfile` (dev-dep) for persistence integration tests.

---

### File Map

| File | Action | Responsibility |
|---|---|---|
| `Cargo.toml` | Modify | Add runtime deps (serde, serde_json, thiserror) and dev dep (tempfile) |
| `src/stats.rs` | Create | `SessionResult` struct with serde derives |
| `src/persistence.rs` | Create | `load`, `append`, `load_from`, `append_to`, `PersistError` |
| `src/model.rs` | Modify | Add `history: Vec<SessionResult>` field to `Model` |
| `src/commands.rs` | Modify | Add `StatsPayload`, `Command::SaveStats`, handler in `execute_command` |
| `src/update.rs` | Modify | Add `build_stats_payload` helper; return `SaveStats` in both Done paths; 2 new tests |
| `src/main.rs` | Modify | Declare `mod stats` + `mod persistence`; load history after `Model::default()` |

---

### Task 1: Add dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add runtime and dev dependencies**

In `Cargo.toml`, update `[dependencies]` and `[dev-dependencies]`:

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
anyhow = "1"
rand = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
proptest = "1.11"
insta = { version = "1.47", features = ["yaml"] }
tempfile = "3"
```

- [ ] **Step 2: Verify the build compiles**

Run: `cargo build`
Expected: Compiles successfully (no source changes yet, just deps resolved).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): add serde, serde_json, thiserror, tempfile"
```

---

### Task 2: Create `src/stats.rs`

**Files:**
- Create: `src/stats.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create the file**

Create `src/stats.rs` with exactly this content:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionResult {
    pub timestamp: i64,
    pub duration_secs: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
}
```

- [ ] **Step 2: Declare the module in `src/main.rs`**

In `src/main.rs`, add `mod stats;` to the module block:

Before:
```rust
mod commands;
mod generator;
mod input;
mod metrics;
mod model;
mod msg;
mod update;
mod view;
```

After:
```rust
mod commands;
mod generator;
mod input;
mod metrics;
mod model;
mod msg;
mod stats;
mod update;
mod view;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/stats.rs src/main.rs
git commit -m "feat(stats): add SessionResult type"
```

---

### Task 3: Create `src/persistence.rs` (TDD)

**Files:**
- Create: `src/persistence.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add module declaration to `src/main.rs`**

Add `mod persistence;` alongside the other module declarations:

```rust
mod commands;
mod generator;
mod input;
mod metrics;
mod model;
mod msg;
mod persistence;
mod stats;
mod update;
mod view;
```

- [ ] **Step 2: Write failing tests — create `src/persistence.rs` with stubs**

Create `src/persistence.rs` with `todo!()` stubs and the test module:

```rust
use std::path::{Path, PathBuf};

use crate::stats::SessionResult;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

fn stats_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("kern").join("stats.json")
}

pub fn load() -> Result<Vec<SessionResult>, PersistError> {
    load_from(&stats_path())
}

pub fn append(result: &SessionResult) -> Result<(), PersistError> {
    append_to(&stats_path(), result)
}

pub(crate) fn load_from(_path: &Path) -> Result<Vec<SessionResult>, PersistError> {
    todo!()
}

pub(crate) fn append_to(_path: &Path, _result: &SessionResult) -> Result<(), PersistError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_result() -> SessionResult {
        SessionResult {
            timestamp: 1_000_000,
            duration_secs: 15,
            wpm: 60.0,
            raw_wpm: 65.0,
            accuracy: 92.0,
        }
    }

    #[test]
    fn load_missing_file_returns_empty_vec() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("stats.json");
        let result = load_from(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn append_creates_file_and_loads_back() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("stats.json");
        let r = sample_result();
        append_to(&path, &r).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].timestamp, r.timestamp);
        assert!((loaded[0].wpm - r.wpm).abs() < 0.01);
    }

    #[test]
    fn append_accumulates_multiple_results() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("stats.json");
        append_to(&path, &sample_result()).unwrap();
        append_to(&path, &sample_result()).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.len(), 2);
    }
}
```

- [ ] **Step 3: Run tests to confirm they fail**

Run: `cargo nextest run load_missing_file_returns_empty_vec append_creates_file_and_loads_back append_accumulates_multiple_results`
Expected: 3 tests FAIL with `not yet implemented` panics.

- [ ] **Step 4: Implement `load_from` and `append_to`**

Replace the two stub functions in `src/persistence.rs` with the real implementations (keep everything else the same):

```rust
pub(crate) fn load_from(path: &Path) -> Result<Vec<SessionResult>, PersistError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

pub(crate) fn append_to(path: &Path, result: &SessionResult) -> Result<(), PersistError> {
    let mut entries = if path.exists() {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str::<Vec<SessionResult>>(&data)?
    } else {
        Vec::new()
    };
    entries.push(result.clone());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&entries)?)?;
    Ok(())
}
```

- [ ] **Step 5: Run the 3 persistence tests to confirm they pass**

Run: `cargo nextest run load_missing_file_returns_empty_vec append_creates_file_and_loads_back append_accumulates_multiple_results`
Expected: 3 tests PASS.

- [ ] **Step 6: Run the full suite to confirm nothing broke**

Run: `cargo nextest run`
Expected: 61 tests pass (58 original + 3 new).

- [ ] **Step 7: Commit**

```bash
git add src/persistence.rs src/main.rs
git commit -m "feat(persistence): add load and append for stats.json"
```

---

### Task 4: Update `src/model.rs` — add `history` field

**Files:**
- Modify: `src/model.rs`

- [ ] **Step 1: Add the import**

At the top of `src/model.rs`, add after the existing `use std::time::Duration;` line:

```rust
use crate::stats::SessionResult;
```

- [ ] **Step 2: Add the field to `Model`**

Update the `Model` struct:

Before:
```rust
#[derive(Debug, Clone)]
pub struct Model {
    pub screen: Screen,
    pub session: SessionState,
    pub config: Config,
}
```

After:
```rust
#[derive(Debug, Clone)]
pub struct Model {
    pub screen: Screen,
    pub session: SessionState,
    pub config: Config,
    pub history: Vec<SessionResult>,
}
```

- [ ] **Step 3: Update `Default` for `Model`**

Before:
```rust
impl Default for Model {
    // Starts with an empty session; main.rs fires Command::GenerateWords immediately
    // after construction to populate words before the first frame renders.
    fn default() -> Self {
        Model {
            screen: Screen::Typing,
            session: SessionState::new(Vec::new()),
            config: Config::default(),
        }
    }
}
```

After:
```rust
impl Default for Model {
    // Starts with an empty session; main.rs fires Command::GenerateWords immediately
    // after construction to populate words before the first frame renders.
    fn default() -> Self {
        Model {
            screen: Screen::Typing,
            session: SessionState::new(Vec::new()),
            config: Config::default(),
            history: Vec::new(),
        }
    }
}
```

- [ ] **Step 4: Run the full suite**

Run: `cargo nextest run`
Expected: 61 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/model.rs
git commit -m "feat(model): add history field to Model"
```

---

### Task 5: Update `src/commands.rs` — add `StatsPayload`, `Command::SaveStats`, handler

**Files:**
- Modify: `src/commands.rs`

- [ ] **Step 1: Replace `src/commands.rs` with the updated version**

```rust
use std::time::SystemTime;

use rand::rngs::SmallRng;

use crate::generator;
use crate::model::{Model, SessionState};
use crate::persistence;
use crate::stats::SessionResult;

#[derive(Debug)]
pub struct StatsPayload {
    pub duration_secs: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
}

#[derive(Debug)]
pub enum Command {
    None,
    GenerateWords { count: usize },
    SaveStats(StatsPayload),
}

// The only place side effects happen. update() returns a Command; main.rs calls this.
// rng lives in main.rs (seeded once at startup) and is passed in here — it is
// infrastructure, not app state.
pub fn execute_command(model: &mut Model, cmd: Command, rng: &mut SmallRng) {
    match cmd {
        Command::None => {}
        Command::GenerateWords { count } => {
            model.session = SessionState::new(generator::generate(count, rng));
        }
        Command::SaveStats(payload) => {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let result = SessionResult {
                timestamp,
                duration_secs: payload.duration_secs,
                wpm: payload.wpm,
                raw_wpm: payload.raw_wpm,
                accuracy: payload.accuracy,
            };
            model.history.push(result.clone());
            if let Err(e) = persistence::append(&result) {
                eprintln!("kern: failed to save stats: {e}");
            }
        }
    }
}
```

- [ ] **Step 2: Run the full suite**

Run: `cargo nextest run`
Expected: 61 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/commands.rs
git commit -m "feat(commands): add StatsPayload, SaveStats variant, and execute_command handler"
```

---

### Task 6: Update `src/update.rs` — return `SaveStats` on test completion (TDD)

**Files:**
- Modify: `src/update.rs`

- [ ] **Step 1: Write the failing tests**

In `src/update.rs`, add these two tests inside the existing `#[cfg(test)] mod tests` block, after the last existing test (`tick_after_done_is_noop`):

```rust
#[test]
fn space_on_last_word_returns_save_stats_command() {
    let mut model = model_with_words(&["hi"]);
    update(&mut model, Msg::Char('h'));
    let cmd = update(&mut model, Msg::Space);
    assert!(matches!(cmd, Command::SaveStats(_)));
}

#[test]
fn tick_at_time_limit_returns_save_stats_command() {
    let mut model = model_with_words(&["hello"]);
    update(&mut model, Msg::Char('h'));
    let cmd = update(&mut model, Msg::Tick(Duration::from_secs(15)));
    assert!(matches!(cmd, Command::SaveStats(_)));
}
```

- [ ] **Step 2: Run to confirm they fail**

Run: `cargo nextest run space_on_last_word_returns_save_stats_command tick_at_time_limit_returns_save_stats_command`
Expected: 2 tests FAIL — both arms still return `Command::None`.

- [ ] **Step 3: Update imports at the top of `src/update.rs`**

Before:
```rust
use std::time::Duration;

use crate::commands::Command;
use crate::model::{DURATION_OPTIONS, Model, Screen, TestStatus};
use crate::msg::Msg;
```

After:
```rust
use std::time::Duration;

use crate::commands::{Command, StatsPayload};
use crate::metrics;
use crate::model::{DURATION_OPTIONS, Model, Screen, TestStatus};
use crate::msg::Msg;
```

- [ ] **Step 4: Add `build_stats_payload` helper before `pub fn update`**

Insert this function before `pub fn update(model: &mut Model, msg: Msg) -> Command {`:

```rust
fn build_stats_payload(model: &Model) -> StatsPayload {
    let correct_words = metrics::count_correct_words(&model.session.words);
    let committed_words = metrics::count_committed_words(&model.session.words);
    let correct_chars = metrics::count_correct_chars(&model.session.words);
    let total_chars = metrics::count_total_chars_typed(&model.session.words);
    StatsPayload {
        duration_secs: DURATION_OPTIONS[model.config.selected_duration_idx],
        wpm: metrics::wpm(correct_words, model.session.elapsed),
        raw_wpm: metrics::raw_wpm(committed_words, model.session.elapsed),
        accuracy: metrics::accuracy(correct_chars, total_chars),
    }
}
```

- [ ] **Step 5: Update the `Msg::Space` arm**

Find the `is_last` branch inside `Msg::Space`. Before:

```rust
            if is_last {
                session.status = TestStatus::Done;
                model.screen = Screen::Done;
            } else {
                session.current_word += 1;
            }
```

After:

```rust
            if is_last {
                session.status = TestStatus::Done;
                model.screen = Screen::Done;
                return Command::SaveStats(build_stats_payload(model));
            } else {
                session.current_word += 1;
            }
```

- [ ] **Step 6: Update the `Msg::Tick` arm**

Find the time-limit check inside `Msg::Tick`. Before:

```rust
            if elapsed >= model.config.time_limit {
                model.session.status = TestStatus::Done;
                model.screen = Screen::Done;
            }
```

After:

```rust
            if elapsed >= model.config.time_limit {
                model.session.status = TestStatus::Done;
                model.screen = Screen::Done;
                return Command::SaveStats(build_stats_payload(model));
            }
```

- [ ] **Step 7: Run the two new tests to confirm they pass**

Run: `cargo nextest run space_on_last_word_returns_save_stats_command tick_at_time_limit_returns_save_stats_command`
Expected: 2 tests PASS.

- [ ] **Step 8: Run the full suite**

Run: `cargo nextest run`
Expected: 63 tests pass (61 previous + 2 new).

- [ ] **Step 9: Commit**

```bash
git add src/update.rs
git commit -m "feat(update): return SaveStats command on test completion"
```

---

### Task 7: Update `src/main.rs` — load history on startup

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add history loading to the `run` function**

In `src/main.rs`, find the start of `fn run`:

```rust
fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut rng: SmallRng = rand::make_rng();
    let mut model = Model::default();
    // timer_start is infrastructure — not app state. Owned here alongside rng.
    let mut timer_start: Option<Instant> = None;
```

Update to:

```rust
fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut rng: SmallRng = rand::make_rng();
    let mut model = Model::default();
    match persistence::load() {
        Ok(history) => model.history = history,
        Err(e) => eprintln!("kern: failed to load stats: {e}"),
    }
    // timer_start is infrastructure — not app state. Owned here alongside rng.
    let mut timer_start: Option<Instant> = None;
```

(`persistence` is already in scope as a sibling module declared earlier in this file — no `use` needed.)

- [ ] **Step 2: Run the full suite**

Run: `cargo nextest run`
Expected: 63 tests pass.

- [ ] **Step 3: Build the binary**

Run: `cargo build`
Expected: Compiles without errors or warnings.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): load stats history on startup"
```

---

### Task 8: Final verification

**Files:** None modified.

- [ ] **Step 1: Run the full test suite**

Run: `cargo nextest run`
Expected: 63 tests pass (58 original + 3 persistence + 2 update).

- [ ] **Step 2: Lint**

Run: `cargo clippy -- -D warnings`
Expected: No warnings or errors.

- [ ] **Step 3: Format**

Run: `cargo fmt`
Expected: Apply any formatting changes.

- [ ] **Step 4: Smoke-test the binary**

Run: `cargo run`

Play a test to completion (either let the timer expire or type all words and press Space). Then quit with Esc.

Verify the stats file was created:

```bash
cat ~/.config/kern/stats.json
```

Expected output (values will vary):
```json
[
  {
    "timestamp": 1747440000,
    "duration_secs": 15,
    "wpm": 45.2,
    "raw_wpm": 50.1,
    "accuracy": 90.3
  }
]
```

- [ ] **Step 5: Commit formatting changes if any**

```bash
git add -u
git commit -m "style: apply cargo fmt"
```

(Skip this step if `cargo fmt` made no changes.)
