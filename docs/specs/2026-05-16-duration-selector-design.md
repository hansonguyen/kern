# Phase 4: Time Duration Selector — Design Spec

**Date:** 2026-05-16  
**Status:** Approved

## Overview

Add a duration selector to Kern's header that lets users cycle through three test durations — 15s, 30s, 60s — using the Tab key before a test starts. The selected duration is displayed as a mini tab strip in the header and on the results screen. Tab during a running test still restarts as today.

## Data Model (`model.rs`)

Two additions to the existing code, nothing removed:

```rust
pub const DURATION_OPTIONS: [u64; 3] = [15, 30, 60];

pub struct Config {
    pub word_count: usize,
    pub cursor_style: CursorStyle,
    pub time_limit: Duration,          // kept in sync with selected_duration_idx
    pub selected_duration_idx: usize,  // NEW: index into DURATION_OPTIONS
    pub punctuation: bool,
    pub numbers: bool,
}
```

`Default` sets `selected_duration_idx: 0` (15s). `DURATION_OPTIONS` is the canonical list of valid durations. `time_limit` is always derived from it and is never set independently.

No existing tests inspect `selected_duration_idx`, so all 53 tests compile without modification.

## Update Logic (`update.rs`)

`Msg::Tab` becomes context-aware on `session.status`:

```rust
Msg::Tab => {
    if model.session.status != TestStatus::Running {
        let next_idx = (model.config.selected_duration_idx + 1) % DURATION_OPTIONS.len();
        model.config.selected_duration_idx = next_idx;
        model.config.time_limit = Duration::from_secs(DURATION_OPTIONS[next_idx]);
    }
    model.screen = Screen::Typing;
    return Command::GenerateWords { count: model.config.word_count };
}
```

Behavior table:

| State | Tab effect |
|---|---|
| `Waiting` | Cycle 15→30→60→15, reset session with fresh words |
| `Running` | Restart with current duration (no cycle) |
| `Done` | Cycle duration + restart with fresh words |

Existing Tab tests at lines 173 and 189 of `update.rs` remain valid — both use a default model (status=Waiting) and only assert on `Command::GenerateWords` and `Screen::Typing`, neither of which changes.

## View (`view.rs`)

### Strip Helper

A shared pure function produces the duration strip for both screens:

```rust
fn duration_strip<'a>(selected_idx: usize, dimmed: bool) -> Line<'a>
```

Produces spans like `  15  [30]  60  `. The active item is bold; inactive items are dim. When `dimmed=true` (Running state), the entire strip is rendered dim to signal non-interactivity.

### Typing Screen Header

Replaces the current `kern  15s  [tab] restart` header:

| Status | Header |
|---|---|
| Waiting | `kern  15  [30]  60   [tab] restart` |
| Running | `kern  15  [30]  60  ·  22s   [tab] restart` |

The `·` separates the strip from the live countdown. Strip is full-brightness when Waiting, dimmed when Running.

### Done Screen

Gains a strip row at the top of its vertical layout, plus an updated footer hint:

```
  15  [30]  60

          kern

  wpm       raw wpm        acc
   62           71          88%

   [tab] change/restart   [esc] quit
```

The vertical layout adds one `Constraint::Length(1)` for the strip row and one `Constraint::Length(1)` spacer before the existing `kern` title row.

## Testing

- All 53 existing behavioral tests pass without modification.
- Snapshot tests (`typing_screen_snapshot`, `results_screen_snapshot`) are updated by running `INSTA_UPDATE=always cargo nextest run` after implementation — the frames change and new snapshots replace the old ones.
- New unit tests cover the Tab cycling logic:
  - Tab while Waiting cycles idx 0→1→2→0 and updates `time_limit`
  - Tab while Running does not cycle `selected_duration_idx`
  - Tab while Done cycles idx

## Files Touched

| File | Change |
|---|---|
| `src/model.rs` | Add `DURATION_OPTIONS`, add `selected_duration_idx` to `Config` and `Default` |
| `src/update.rs` | Add cycling logic to `Msg::Tab` branch; add new unit tests |
| `src/view.rs` | Add `duration_strip` helper; update `render_typing` header; update `render_results` layout and footer |
| `src/msg.rs` | No changes |
| `src/input.rs` | No changes |
| `src/metrics.rs` | No changes |
