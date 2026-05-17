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
            model.history.push(result);
            if let Err(e) = persistence::append(model.history.last().unwrap()) {
                eprintln!("kern: failed to save stats: {e}");
            }
        }
    }
}
