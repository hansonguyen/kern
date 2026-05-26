use std::time::Duration;

use crate::stats::SessionResult;
use crate::theme::Theme;

pub const DURATION_OPTIONS: [u64; 3] = [15, 30, 60];
pub const WORD_COUNT_OPTIONS: [usize; 4] = [10, 25, 50, 100];

#[derive(Debug, Clone, PartialEq)]
pub enum TestMode {
    Time,
    Words,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Typing,
    Done,
    Quitting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Waiting,
    Running,
    Done,
}

#[derive(Debug, Clone)]
pub struct Word {
    pub chars: Vec<char>,
    pub typed: String,
    pub committed: bool,
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

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaretStyle {
    Off,
    Default,
    #[default]
    Block,
    Underline,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalKind {
    CustomTime,
    CustomWords,
}

#[derive(Debug, Clone)]
pub struct ModalState {
    pub kind: ModalKind,
    pub input: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub test_mode: TestMode,
    pub caret_style: CaretStyle,
    // time mode
    pub time_limit: Duration,
    // valid in 0..=DURATION_OPTIONS.len(); DURATION_OPTIONS.len() = custom slot
    pub selected_duration_idx: usize,
    pub custom_time_secs: Option<u64>,
    // words mode
    pub word_count: usize,
    // valid in 0..=WORD_COUNT_OPTIONS.len(); WORD_COUNT_OPTIONS.len() = custom slot
    pub selected_word_count_idx: usize,
    pub custom_word_count: Option<usize>,
    #[expect(dead_code)]
    pub punctuation: bool,
    #[expect(dead_code)]
    pub numbers: bool,
}

impl Config {
    pub fn initial_word_count(&self) -> usize {
        match self.test_mode {
            TestMode::Time => 50,
            TestMode::Words => {
                if self.selected_word_count_idx == WORD_COUNT_OPTIONS.len() {
                    match self.custom_word_count {
                        None | Some(0) => 50,
                        Some(n) => n,
                    }
                } else {
                    self.word_count
                }
            }
        }
    }

    pub fn is_infinite_words(&self) -> bool {
        self.selected_word_count_idx == WORD_COUNT_OPTIONS.len()
            && self.custom_word_count == Some(0)
    }

    pub fn is_infinite_time(&self) -> bool {
        self.selected_duration_idx == DURATION_OPTIONS.len() && self.custom_time_secs == Some(0)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            test_mode: TestMode::Time,
            caret_style: CaretStyle::Block,
            time_limit: Duration::from_secs(15),
            selected_duration_idx: 0,
            custom_time_secs: None,
            word_count: WORD_COUNT_OPTIONS[1], // 25
            selected_word_count_idx: 1,
            custom_word_count: None,
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
    pub total_chars_typed: u64,
    pub total_errors: u64,
    pub wpm_history: Vec<f64>,
    pub error_history: Vec<u64>,
}

impl SessionState {
    pub fn new(words: Vec<Word>) -> Self {
        SessionState {
            words,
            current_word: 0,
            status: TestStatus::Waiting,
            elapsed: Duration::ZERO,
            total_chars_typed: 0,
            total_errors: 0,
            wpm_history: Vec::new(),
            error_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub screen: Screen,
    pub session: SessionState,
    pub config: Config,
    pub history: Vec<SessionResult>,
    pub pending_update: Option<String>,
    pub theme: Theme,
    pub modal: Option<ModalState>,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            screen: Screen::Typing,
            session: SessionState::new(Vec::new()),
            config: Config::default(),
            history: Vec::new(),
            pending_update: None,
            theme: Theme::default(),
            modal: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_style_default_is_block() {
        let cfg = Config::default();
        assert_eq!(cfg.caret_style, CaretStyle::Block);
    }

    #[test]
    fn caret_style_serializes_to_lowercase() {
        let s = serde_json::to_string(&CaretStyle::Underline).unwrap();
        assert_eq!(s, "\"underline\"");
        let s = serde_json::to_string(&CaretStyle::Off).unwrap();
        assert_eq!(s, "\"off\"");
    }

    #[test]
    fn caret_style_deserializes_from_lowercase() {
        let v: CaretStyle = serde_json::from_str("\"block\"").unwrap();
        assert_eq!(v, CaretStyle::Block);
        let v: CaretStyle = serde_json::from_str("\"underline\"").unwrap();
        assert_eq!(v, CaretStyle::Underline);
    }

    #[test]
    fn custom_fields_default_to_none() {
        let cfg = Config::default();
        assert!(cfg.custom_time_secs.is_none());
        assert!(cfg.custom_word_count.is_none());
    }

    #[test]
    fn initial_word_count_infinite_words_returns_50() {
        let mut cfg = Config::default();
        cfg.test_mode = TestMode::Words;
        cfg.selected_word_count_idx = WORD_COUNT_OPTIONS.len(); // custom slot
        cfg.custom_word_count = Some(0); // infinite
        assert_eq!(cfg.initial_word_count(), 50);
    }

    #[test]
    fn initial_word_count_custom_words_returns_count() {
        let mut cfg = Config::default();
        cfg.test_mode = TestMode::Words;
        cfg.selected_word_count_idx = WORD_COUNT_OPTIONS.len();
        cfg.custom_word_count = Some(42);
        assert_eq!(cfg.initial_word_count(), 42);
    }

    #[test]
    fn modal_defaults_to_none() {
        let model = Model::default();
        assert!(model.modal.is_none());
    }
}
