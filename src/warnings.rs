use std::fmt::Display;

use log::error;
use pest::Span;

use crate::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerWarning {
    /// Warning for when an instruction overwrites another
    StateOverwrite {
        position: ErrorPosition,
        /// The state that is being overwritten
        state: String,
        value_from: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    /// A generic syntax error
    SyntaxError {
        position: ErrorPosition,
        /// The error message
        message: String,
        /// The line of code that caused the error
        code: String,
        expected: Rule,
        found: Option<Rule>,
    },

    /// An error when parsing the file rule
    FileRuleError {
        error: pest::error::ErrorVariant<Rule>,
    },
}

impl CompilerError {
    pub fn log_error(&self) {
        match self {
            CompilerError::SyntaxError {
                position,
                message,
                code: _,
                expected,
                found,
            } => {
                error!(
                    "Syntax error At position {}: {} - Expected {:?}, got {:?}",
                    position,
                    message,
                    expected,
                    found.unwrap_or(Rule::EOI)
                );
            }
            CompilerError::FileRuleError { error } => {
                error!("Syntax error: {}", error);
            }
        }
    }

    pub fn get_message(&self) -> String {
        match self {
            CompilerError::SyntaxError {
                position: _,
                message,
                code: _,
                expected: _,
                found: _,
            } => String::from(message),
            CompilerError::FileRuleError { error } => error.message().to_string(),
        }
    }

    pub fn get_position(&self) -> ErrorPosition {
        match self {
            CompilerError::SyntaxError { position, .. } => position.clone(),
            CompilerError::FileRuleError { .. } => ErrorPosition {
                start: (0, 0),
                end: None,
            },
        }
    }

    pub fn line(&self) -> usize {
        match self {
            CompilerError::SyntaxError { position, .. } => position.start.0,
            CompilerError::FileRuleError { .. } => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorPosition {
    pub start: (usize, usize),
    pub end: Option<(usize, usize)>,
}

impl ErrorPosition {
    pub fn new(start: (usize, usize), end: Option<(usize, usize)>) -> Self {
        ErrorPosition { start, end }
    }
}

impl Display for ErrorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.end {
            Some(end) => write!(
                f,
                "{}:{} to {}:{}",
                self.start.0, self.start.1, end.0, end.1
            ),
            None => write!(f, "{}:{}", self.start.0, self.start.1),
        }
    }
}

impl From<pest::error::LineColLocation> for ErrorPosition {
    fn from(e: pest::error::LineColLocation) -> Self {
        match e {
            pest::error::LineColLocation::Pos((line, col)) => {
                // error!("Line {}, column {}: ", line, col);
                ErrorPosition {
                    start: (line - 1, col),
                    end: None,
                }
            }
            pest::error::LineColLocation::Span((line1, col1), (line2, col2)) => {
                // error!("From line {}:{} to {}:{}. Found:", line1, col1, line2, col2);
                ErrorPosition {
                    start: (line1 - 1, col1),
                    end: Some((line2 - 1, col2)),
                }
            }
        }
    }
}

impl From<Span<'_>> for ErrorPosition {
    fn from(e: Span) -> Self {
        ErrorPosition {
            start: (e.start_pos().line_col().0 - 1, e.start_pos().line_col().1),
            end: Some((e.end_pos().line_col().0 - 1, e.end_pos().line_col().1)),
        }
    }
}

impl From<&Span<'_>> for ErrorPosition {
    fn from(e: &Span) -> Self {
        ErrorPosition {
            start: (e.start_pos().line_col().0 - 1, e.start_pos().line_col().1),
            end: Some((e.end_pos().line_col().0 - 1, e.end_pos().line_col().1)),
        }
    }
}

impl From<(usize, usize)> for ErrorPosition {
    fn from(e: (usize, usize)) -> Self {
        ErrorPosition {
            start: (e.0 - 1, e.1),
            end: None,
        }
    }
}

impl From<pest::error::Error<Rule>> for ErrorPosition {
    fn from(e: pest::error::Error<Rule>) -> Self {
        match e.line_col {
            pest::error::LineColLocation::Pos((line, col)) => ErrorPosition {
                start: (line - 1, col),
                end: None,
            },
            pest::error::LineColLocation::Span((line1, col1), (line2, col2)) => ErrorPosition {
                start: (line1 - 1, col1),
                end: Some((line2 - 1, col2)),
            },
        }
    }
}
