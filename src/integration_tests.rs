use std::time::Duration;

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
