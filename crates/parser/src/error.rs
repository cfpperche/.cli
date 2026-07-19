use std::fmt;

/// A lex or parse error, anchored to a source position (1-based).
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self { line, col, message: message.into() }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for ParseError {}
