use std::{fmt::Display, str::FromStr};

use crate::{turing::Rule, CompilerError, ErrorPosition};
use pest::iterators::Pairs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
/// The possible movements of the tape head
pub enum Movement {
    RIGHT,
    LEFT,
    HALT,
}

impl std::str::FromStr for Movement {
    type Err = String;

    /// Parse a movement from a string
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "R" | "D" => Ok(Self::RIGHT),
            "L" | "I" => Ok(Self::LEFT),
            "H" | "N" => Ok(Self::HALT),
            _ => Err(format!("\"{input}\" is an unknown movement")),
        }
    }
}

impl Display for Movement {
    /// Display a movement as a string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Movement::RIGHT => write!(f, "R"),
            Movement::LEFT => write!(f, "L"),
            Movement::HALT => write!(f, "H"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A Turing machine instruction
pub struct TuringInstruction {
    pub from_state: String,
    pub from_value: bool,
    pub to_value: bool,
    pub movement: Movement,
    pub to_state: String,
}

impl Display for TuringInstruction {
    /// Display an instruction as a string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {}, {})",
            self.from_state,
            if self.from_value { "1" } else { "0" },
            if self.to_value { "1" } else { "0" },
            self.movement,
            self.to_state
        )
    }
}

impl TuringInstruction {
    /// Create an instruction from a `Pairs<Rule>` object
    pub fn from(mut code: Pairs<Rule>) -> Result<Self, CompilerError> {
        let from_state = match code.next() {
            Some(s) => String::from(s.as_span().as_str()),
            None => panic!("The instruction lacks an initial state"),
        };
        let from_value = match code.next() {
            Some(s) => s.as_span().as_str() == "1",
            None => panic!("The instruction lacks an initial tape value"),
        };
        let to_value = match code.next() {
            Some(s) => s.as_span().as_str() == "1",
            None => panic!("The instruction lacks a target tape value"),
        };

        let movement = match code.next() {
            Some(s) => match Movement::from_str(s.as_span().as_str()) {
                Ok(m) => m,
                Err(message) => {
                    return Err(CompilerError::SyntaxError {
                        position: ErrorPosition::from(&s),
                        message,
                        code: String::from(s.as_str()),
                        expected: Rule::movement,
                        found: None,
                    })
                }
            },
            None => panic!("The instruction lacks an initial state"),
        };

        let to_state = match code.next() {
            Some(s) => String::from(s.as_span().as_str()),
            None => panic!("The instruction lacks a target state"),
        };

        Ok(Self {
            from_state,
            from_value,
            to_value,
            movement,
            to_state,
        })
    }

    /// Create a halt instruction when there is missing information
    pub fn halt(index: (String, bool)) -> Self {
        Self {
            from_state: index.0.clone(),
            from_value: index.1,
            to_value: index.1,
            movement: Movement::HALT,
            to_state: index.0,
        }
    }
}
