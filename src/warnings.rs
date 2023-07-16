use std::fmt::Display;

use log::error;
use pest::{iterators::Pair, Span};

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
    FileRuleError { error: pest::error::Error<Rule> },
}

impl CompilerError {
    /// Log the error to the console
    /// with the format `Syntax error At position {position}: {message} - Expected {expected:?}, got {found:?}` or `Syntax error: {error}` if the error is a `FileRuleError`
    pub fn log_error(&self) {
        match self {
            CompilerError::SyntaxError {
                position,
                message,
                expected,
                found,
                ..
            } => {
                error!(
                    "Syntax error At position {position}: {message} - Expected {expected:?}, got {:?}",
                    found.unwrap_or(Rule::EOI)
                );
            }
            CompilerError::FileRuleError { error, .. } => {
                error!("Syntax error: {}", error);
            }
        }
    }

    /// Get the expected message. If the error is a `FileRuleError`, the message will be extracted from `pest::error::Error`, otherwise it will be `Expected {expected:?}, got {found:?}`
    pub fn get_message_expected(&self) -> String {
        match &self {
            CompilerError::SyntaxError {
                expected, found, ..
            } => format!("Expected {:?}, found {:?}", expected, found),
            CompilerError::FileRuleError { error } => String::from(error.variant.message()),
        }
    }

    /// Get the code that caused the error. If the error is a `FileRuleError`, the code will be extracted from `pest::error::Error`
    pub fn code(&self) -> String {
        match self {
            CompilerError::SyntaxError { code, .. } => code.clone(),
            CompilerError::FileRuleError { error, .. } => String::from(error.line()),
        }
    }

    /// Get the error message
    pub fn message(&self) -> String {
        match self {
            CompilerError::SyntaxError { message, .. } => String::from(message),
            CompilerError::FileRuleError { error, .. } => error.variant.message().to_string(),
        }
    }

    /// Get the line of the error. If the error is a `FileRuleError`, the line will be `0`
    pub fn line(&self) -> usize {
        match self {
            CompilerError::SyntaxError { position, .. } => position.start.0,
            CompilerError::FileRuleError { .. } => 0,
        }
    }

    /// Get the position of the error. It extracts the position from the `pest::error::Error` if the error is a `FileRuleError`
    pub fn position(&self) -> ErrorPosition {
        match self {
            CompilerError::SyntaxError { position, .. } => position.clone(),
            CompilerError::FileRuleError { error, .. } => match error.line_col {
                pest::error::LineColLocation::Pos((line, col)) => ErrorPosition {
                    start: (line, col),
                    end: None,
                },
                pest::error::LineColLocation::Span((line1, col1), (line2, col2)) => ErrorPosition {
                    start: (line1 - 1, col1),
                    end: Some((line2 - 1, col2)),
                },
            },
        }
    }

    /// Get the expected rule
    pub fn expected(&self) -> Rule {
        match self {
            CompilerError::SyntaxError { expected, .. } => *expected,
            CompilerError::FileRuleError { error, .. } => match &error.variant {
                pest::error::ErrorVariant::ParsingError { positives, .. } => {
                    positives.first().unwrap().clone()
                }
                _ => Rule::EOI,
            },
        }
    }

    /// Get the found rule
    pub fn found(&self) -> Option<Rule> {
        match self {
            CompilerError::SyntaxError { found, .. } => *found,
            CompilerError::FileRuleError { error, .. } => match &error.variant {
                pest::error::ErrorVariant::ParsingError { positives, .. } => {
                    Some(positives.first().unwrap().clone())
                }
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A struct to store the position of an error
pub struct ErrorPosition {
    /// The start position of the error. The first value is the line, the second is the column
    pub start: (usize, usize),

    /// The end position of the error. The first value is the line, the second is the column.
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
    /// Convert a `pest::error::LineColLocation` to an `ErrorPosition`
    fn from(e: pest::error::LineColLocation) -> Self {
        match e {
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

impl From<pest::error::Error<Rule>> for ErrorPosition {
    /// Convert a `pest::error::Error` to an `ErrorPosition`
    /// Only a `pest::error::LineColLocation` has an end position, so the end position will be `None` otherwise
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

impl From<Span<'_>> for ErrorPosition {
    /// Convert a `pest::Span` to an `ErrorPosition`
    /// This operation on a `pest::Span` is `O(n)`, so you better use pair.line_col() instead if it has no end position
    fn from(e: Span) -> Self {
        ErrorPosition {
            start: (e.start_pos().line_col().0 - 1, e.start_pos().line_col().1),
            end: Some((e.end_pos().line_col().0 - 1, e.end_pos().line_col().1)),
        }
    }
}

impl From<&Span<'_>> for ErrorPosition {
    /// Convert a `&pest::Span` to an `ErrorPosition`
    /// This operation on a `pest::Span` is `O(n)`, so you better use pair.line_col() instead if it has no end position
    fn from(e: &Span) -> Self {
        ErrorPosition {
            start: (e.start_pos().line_col().0 - 1, e.start_pos().line_col().1),
            end: Some((e.end_pos().line_col().0 - 1, e.end_pos().line_col().1)),
        }
    }
}

impl From<&Pair<'_, Rule>> for ErrorPosition {
    /// Convert a `pest::Pair` to an `ErrorPosition`.
    /// Note that a `pest::Pair` has no end position, so the end position will be `None`
    fn from(e: &Pair<Rule>) -> Self {
        ErrorPosition {
            start: (e.line_col().0 - 1, e.line_col().1),
            end: None,
        }
    }
}

impl From<(usize, usize)> for ErrorPosition {
    /// Convert a `(usize, usize)` to an `ErrorPosition`.
    /// Note that a `(usize, usize)` has no end position, so the end position will be `None`
    fn from(e: (usize, usize)) -> Self {
        ErrorPosition {
            start: (e.0 - 1, e.1),
            end: None,
        }
    }
}
