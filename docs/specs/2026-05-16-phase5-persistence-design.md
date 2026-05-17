# Phase 5: Persistence — Design Spec

**Date:** 2026-05-16  
**Status:** Approved

## Overview

After each completed test (`Screen::Done`), write the session result to `~/.config/kern/stats.json`. On startup, load the history into `Model` so it is available for a future best-score display. The save is a side effect — triggered by `Command::SaveStats(StatsPayload)` returned from `update`, executed in `execute_command`. All existing 58 tests continue to pass.

## Data Types (`stats.rs`)

New file `src/stats.rs` with a single serializable struct:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionResult {
    pub timestamp: i64,      // unix epoch seconds, stamped in execute_command
    pub duration_secs: u64,  // DURATION_OPTIONS[selected_duration_idx]
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,       // 0.0–100.0
}
```

`timestamp` is `i64` (no `chrono` dep) — computed via `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64` in `execute_command`. `chrono` is deferred to a phase that needs date formatting or display.

## Persistence (`persistence.rs`)

New file `src/persistence.rs`:

```rust
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

pub fn load() -> Result<Vec<SessionResult>, PersistError>
pub fn append(result: &SessionResult) -> Result<(), PersistError>
```

`load`: Returns `Ok(vec![])` if the file is missing (not an error). Deserializes the JSON array otherwise.

`append`: Reads existing entries (empty vec if missing), pushes the new result, serializes the whole vec back. `fs::create_dir_all` ensures `~/.config/kern/` exists before writing. Full read-then-write is fine — the file stays small (typing test history, thousands of entries at most).

## Command & Execute (`commands.rs`)

`StatsPayload` carries pure metrics computed in `update` (no I/O):

```rust
pub struct StatsPayload {
    pub duration_secs: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
}

pub enum Command {
    None,
    GenerateWords { count: usize },
    SaveStats(StatsPayload),     // NEW
}
```

`execute_command` stamps the timestamp (only I/O-adjacent call), builds `SessionResult`, pushes to `model.history`, and persists. Errors are logged to stderr and swallowed — the results screen is never disturbed:

```rust
Command::SaveStats(payload) => {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let result = SessionResult { timestamp, duration_secs: payload.duration_secs,
        wpm: payload.wpm, raw_wpm: payload.raw_wpm, accuracy: payload.accuracy };
    model.history.push(result.clone());
    if let Err(e) = persistence::append(&result) {
        eprintln!("kern: failed to save stats: {e}");
    }
}
```

## Update (`update.rs`)

Both paths that transition to `Screen::Done` return `Command::SaveStats` instead of `Command::None`. A private helper keeps them DRY:

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

Called at both transition sites:

| Path | Trigger | Change |
|---|---|---|
| `Msg::Space` on last word | All words committed | Return `Command::SaveStats(build_stats_payload(model))` |
| `Msg::Tick` at time limit | `elapsed >= time_limit` | Return `Command::SaveStats(build_stats_payload(model))` |

No existing tests break — they assert on `model.screen` / `model.session.status`, not the returned command.

## Model & Main

**`model.rs`** — one new field, defaulting to empty:

```rust
pub struct Model {
    pub screen: Screen,
    pub session: SessionState,
    pub config: Config,
    pub history: Vec<SessionResult>,  // populated by main.rs after construction
}
```

`Default` sets `history: Vec::new()`. I/O stays out of `Default`.

**`main.rs`** — load history after model construction, before the event loop:

```rust
let mut model = Model::default();
match persistence::load() {
    Ok(history) => model.history = history,
    Err(e) => eprintln!("kern: failed to load stats: {e}"),
}
```

**`Cargo.toml`** — three new runtime dependencies:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

And one dev dependency:

```toml
tempfile = "3"
```

## Testing

**Existing tests (58):** All pass without modification.

**New unit tests in `update.rs`:**
- `space_on_last_word_returns_save_stats_command` — asserts returned command is `SaveStats`
- `tick_at_time_limit_returns_save_stats_command` — same for timer-expiry path

**New integration tests in `persistence.rs`** (real filesystem via temp path, no mocks):
- `append_creates_file_and_loads_back` — round-trip one result, assert fields match
- `append_accumulates_multiple_results` — append twice, load, assert `len == 2`
- `load_missing_file_returns_empty_vec` — no file present → `Ok(vec![])`

## Files Touched

| File | Change |
|---|---|
| `src/stats.rs` | NEW — `SessionResult` struct |
| `src/persistence.rs` | NEW — `load`, `append`, `PersistError` |
| `src/commands.rs` | Add `StatsPayload`, `Command::SaveStats`, handle in `execute_command` |
| `src/model.rs` | Add `history: Vec<SessionResult>` to `Model` and `Default` |
| `src/update.rs` | Add `build_stats_payload` helper; return `SaveStats` in two Done-transition arms; two new tests |
| `src/main.rs` | Load history on startup; add `persistence` and `stats` to module list |
| `Cargo.toml` | Add `serde`, `serde_json`, `thiserror`; add `tempfile` to dev-deps |
