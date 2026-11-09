use std::fmt;

use super::location::Location;

/// Error produced while lexing or parsing a Gherkin document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Approximate source location of the problem.
    pub location: Location,
    /// Human-readable description.
    pub message: String,
}

impl ParseError {
    /// Create a parse error at `location`.
    pub fn new(location: Location, message: impl Into<String>) -> Self {
        Self {
            location,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.location.line, self.location.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}
