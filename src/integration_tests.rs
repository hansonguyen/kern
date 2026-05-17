use rand::SeedableRng;
use rand::rngs::SmallRng;
use tempfile::TempDir;

use crate::commands::{Command, execute_command};
use crate::model::{Config, Model, Screen, SessionState, TestStatus, Word};
use crate::msg::Msg;
use crate::persistence;
use crate::stats::SessionResult;
use crate::update::update;

fn two_word_model() -> Model {
    Model {
        screen: Screen::Typing,
        session: SessionState::new(vec![Word::new("hi"), Word::new("ok")]),
        config: Config::default(),
        history: Vec::new(),
    }
}

#[test]
fn full_session_via_word_completion() {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut model = two_word_model();

    // Type one char of "hi", commit with Space → advances to "ok"
    update(&mut model, Msg::Char('h'));
    let cmd = update(&mut model, Msg::Space);
    execute_command(&mut model, cmd, &mut rng);

    // Type one char of "ok", commit with Space → last word → Done + SaveStats
    update(&mut model, Msg::Char('o'));
    let cmd = update(&mut model, Msg::Space);
    execute_command(&mut model, cmd, &mut rng);

    assert_eq!(model.screen, Screen::Done);
    assert_eq!(model.history.len(), 1);
}

#[test]
fn timer_expiry_saves_stats() {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut model = two_word_model();

    // Start the session (Waiting → Running)
    update(&mut model, Msg::Char('h'));
    assert_eq!(model.session.status, TestStatus::Running);

    // Tick exactly at the time limit → triggers Done via timer path (not Space)
    let time_limit = model.config.time_limit;
    let cmd = update(&mut model, Msg::Tick(time_limit));
    assert!(matches!(cmd, Command::SaveStats(_)));
    execute_command(&mut model, cmd, &mut rng);

    assert_eq!(model.screen, Screen::Done);
    assert_eq!(model.session.status, TestStatus::Done);
    assert_eq!(model.history.len(), 1);
}

#[test]
fn tab_from_done_resets_session() {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut model = two_word_model();

    // Drive to Done via word completion
    update(&mut model, Msg::Char('h'));
    let cmd = update(&mut model, Msg::Space);
    execute_command(&mut model, cmd, &mut rng);
    update(&mut model, Msg::Char('o'));
    let cmd = update(&mut model, Msg::Space);
    execute_command(&mut model, cmd, &mut rng);
    assert_eq!(model.screen, Screen::Done);

    // Tab from Done: cycles duration (Done status != Running) and resets session
    let cmd = update(&mut model, Msg::Tab);
    assert!(matches!(cmd, Command::GenerateWords { .. }));
    execute_command(&mut model, cmd, &mut rng);

    assert_eq!(model.screen, Screen::Typing);
    assert_eq!(model.session.status, TestStatus::Waiting);
    assert_eq!(model.session.words.len(), model.config.word_count);
    // Duration cycles: 0 → 1 (15s → 30s)
    assert_eq!(model.config.selected_duration_idx, 1);
}

#[test]
fn persistence_end_to_end() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("stats.json");

    let result = SessionResult {
        timestamp: 1_234_567_890,
        duration_secs: 15,
        wpm: 42.5,
        raw_wpm: 48.0,
        accuracy: 88.5,
    };

    persistence::append_to(&path, &result).unwrap();
    let loaded = persistence::load_from(&path).unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].timestamp, result.timestamp);
    assert_eq!(loaded[0].duration_secs, result.duration_secs);
    assert!((loaded[0].wpm - result.wpm).abs() < 1e-9);
    assert!((loaded[0].raw_wpm - result.raw_wpm).abs() < 1e-9);
    assert!((loaded[0].accuracy - result.accuracy).abs() < 1e-9);
}
