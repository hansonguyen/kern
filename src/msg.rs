use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Tick(Duration), // elapsed since test start; fired every frame by main.rs
    Char(char),
    Backspace,
    Space,
    Tab,
    Esc,
}
