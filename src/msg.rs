#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Tick,
    Char(char),
    Backspace,
    Space,
    Tab,
    Esc,
}
