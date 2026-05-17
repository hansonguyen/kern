use std::time::Duration;

use crate::stats::SessionResult;

pub const DURATION_OPTIONS: [u64; 3] = [15, 30, 60];

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Typing,
    Done,
    Quitting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Waiting, // words shown, nothing typed yet
    Running, // first keypress received
    Done,    // last word committed via Space
}

#[derive(Debug, Clone)]
pub struct Word {
    pub chars: Vec<char>, // original characters from generator
    // Raw typed input for this word. Capped at chars.len() — overtyping is not
    // supported in Phase 2 but this String makes Phase 4 corrected-char tracking easy.
    pub typed: String,
    pub committed: bool, // true once Space commits this word
}

impl Word {
    pub fn new(text: &str) -> Self {
        Word {
            chars: text.chars().collect(),
            typed: String::new(),
            committed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CursorStyle {
    Block, // filled background / inverted colors (default)
    #[expect(dead_code)]
    Underline, // not yet wired; Phase 6 polish
}

#[derive(Debug, Clone)]
pub struct Config {
    pub word_count: usize,
    pub cursor_style: CursorStyle,
    pub time_limit: Duration,
    // invariant: always a valid index into DURATION_OPTIONS; only mutated via modular arithmetic in update.rs
    pub selected_duration_idx: usize,
    #[expect(dead_code)]
    pub punctuation: bool, // stubbed; wired in Phase 3
    #[expect(dead_code)]
    pub numbers: bool, // stubbed; wired in Phase 3
}

impl Default for Config {
    fn default() -> Self {
        Config {
            word_count: 25,
            cursor_style: CursorStyle::Block,
            time_limit: Duration::from_secs(15),
            selected_duration_idx: 0,
            punctuation: false,
            numbers: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub words: Vec<Word>,
    pub current_word: usize,
    pub status: TestStatus,
    pub elapsed: Duration,
}

impl SessionState {
    pub fn new(words: Vec<Word>) -> Self {
        SessionState {
            words,
            current_word: 0,
            status: TestStatus::Waiting,
            elapsed: Duration::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub screen: Screen,
    pub session: SessionState,
    pub config: Config,
    pub history: Vec<SessionResult>,
}

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
