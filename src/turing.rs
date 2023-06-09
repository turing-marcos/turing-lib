use log::{debug, error, info, warn};
use pest::{error::ErrorVariant, Parser, Position};
use pest_derive::Parser;
use std::{collections::HashMap, fmt::Write};

use crate::{instruction::Movement, TuringInstruction};

use super::TuringOutput;

#[derive(Parser)]
#[grammar = "../turing.pest"]
pub struct TuringParser;

#[derive(Debug, Clone)]
/// A Turing machine
pub struct TuringMachine {
    pub instructions: HashMap<(String, bool), TuringInstruction>,
    pub final_states: Vec<String>,
    pub current_state: String,
    pub tape_position: usize,
    pub tape: Vec<bool>,
    pub frequencies: HashMap<String, usize>,
    pub description: Option<String>,
    pub code: String,
}

impl TuringMachine {
    /// Create a new Turing machine from a string of code
    pub fn new(code: &str) -> Result<Self, pest::error::Error<Rule>> {
        let mut instructions: HashMap<(String, bool), TuringInstruction> = HashMap::new();
        let mut final_states: Vec<String> = Vec::new();
        let mut current_state: String = String::new();
        let mut tape: Vec<bool> = Vec::new();
        let mut description: Option<String> = None;

        let file = match TuringParser::parse(Rule::file, code) {
            Ok(mut f) => f.next().unwrap(),
            Err(e) => return Err(e),
        };

        for record in file.into_inner() {
            match record.as_rule() {
                Rule::description => {
                    let s = record.as_str();
                    if !s.is_empty() {
                        description = Some(String::from(s.replace("///", "").trim()));
                        debug!("Found description: \"{:?}\"", description);
                    }
                }
                Rule::COMMENT => debug!("Found comment: \"{:?}\"", record.as_str()),
                Rule::tape => {
                    debug!(
                        "Entered tape rule: {}",
                        record.clone().into_inner().as_str()
                    );

                    for r in record.into_inner() {
                        match r.as_rule() {
                            Rule::value => {
                                if tape.is_empty() && r.as_str() == "0" {
                                    info!("The tape started with a 0, skipping it");
                                } else {
                                    tape.push(r.as_str() == "1");
                                }
                            }
                            _ => warn!(
                                "Unhandled: ({:?}, {})",
                                r.as_rule(),
                                r.into_inner().as_str()
                            ),
                        }
                    }

                    debug!("Initial state: {}", current_state);
                    debug!("Tape: {:?}", tape);

                    if tape.is_empty() || !tape.contains(&true) {
                        error!("The tape did not contain at least a 1");
                        return Err(pest::error::Error::new_from_pos(
                            ErrorVariant::CustomError {
                                message: String::from("Expected at least a 1 in the tape"),
                            },
                            Position::from_start(""),
                        ));
                    }
                }
                Rule::initial_state => {
                    current_state = String::from(record.into_inner().as_str());
                    debug!("The initial tape state is \"{}\"", current_state);
                }
                Rule::final_state => {
                    final_states = record
                        .into_inner()
                        .map(|v| String::from(v.as_span().as_str()))
                        .collect();
                    debug!("The final tape state is {:?}", final_states);
                }
                Rule::instruction => {
                    let tmp = TuringInstruction::from(record.into_inner());
                    instructions.insert(
                        (tmp.from_state.clone(), tmp.from_value.clone()),
                        tmp.clone(),
                    );

                    debug!("Found instruction {}", tmp);
                }
                Rule::EOI => {
                    debug!("End of file");
                }
                _ => {
                    warn!("Unhandled: {}", record.into_inner().as_str());
                }
            }
        }

        let mut tape_position = 0;
        while tape_position <= 2 {
            tape.insert(0, false);
            tape_position += 1;
        }

        debug!("The instructions are {:?}", instructions);

        Ok(Self {
            instructions,
            final_states,
            current_state,
            tape_position,
            tape,
            frequencies: HashMap::new(),
            description,
            code: String::from(code),
        })
    }

    /// Create a new empty Turing machine
    pub fn none() -> Self {
        let state = String::from("f");
        let mut instructions: HashMap<(String, bool), TuringInstruction> = HashMap::new();
        instructions.insert(
            (String::from("F"), false),
            TuringInstruction {
                from_state: state.clone(),
                from_value: false,
                to_value: false,
                movement: Movement::HALT,
                to_state: state.clone(),
            },
        );
        let final_states: Vec<String> = vec![state.clone()];
        let current_state: String = state.clone();
        let tape: Vec<bool> = vec![false, false, false, false, false];
        let description: Option<String> = None;

        Self {
            instructions,
            final_states,
            current_state,
            tape_position: 2,
            tape,
            frequencies: HashMap::new(),
            description,
            code: String::new(),
        }
    }

    /// Parse a Turing machine code syntax error
    /// and print it to the console
    pub fn handle_error(e: pest::error::Error<Rule>) {
        error!("I found an error while parsing the file!");

        match e.clone().variant {
            pest::error::ErrorVariant::ParsingError {
                positives,
                negatives,
            } => error!("Expected {:?}, found {:?}", positives, negatives),
            pest::error::ErrorVariant::CustomError { message } => error!("\t{}", message),
        };

        let mut cols = (0, 0);
        match e.line_col {
            pest::error::LineColLocation::Pos((line, col)) => {
                error!("Line {}, column {}: ", line, col);
                cols.0 = col;
                cols.1 = col + 1;
            }
            pest::error::LineColLocation::Span((line1, col1), (line2, col2)) => {
                error!("From line {}:{} to {}:{}. Found:", line1, col1, line2, col2);
                cols.0 = col1;
                cols.1 = col2;
            }
        };

        error!("\t\"{}\"", e.line());
        error!(
            "\t {: ^width1$}{:^^width2$}{: ^width3$}",
            "^",
            " ",
            " ",
            width1 = cols.0 - 1,
            width2 = cols.1 - cols.0,
            width3 = e.line().len() - cols.1
        );

        println!("\nPress enter to exit");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_default();
    }

    /// Gets the current instruction, or a halt instruction if the current state is a final state
    /// even if there is no instruction for the current state and value
    fn get_instruction(&self) -> Option<TuringInstruction> {
        let current_val: bool = self.tape[self.tape_position];
        let index = (self.current_state.clone(), current_val);

        match self.instructions.get(&index) {
            Some(i) => Some(i.to_owned()),
            None => {
                if !self.final_states.contains(&self.current_state) {
                    return None;
                }

                Some(TuringInstruction::halt(index))
            }
        }
    }

    /// Gets the current instruction
    pub fn get_current_instruction(&self) -> Option<TuringInstruction> {
        let current_val: bool = self.tape[self.tape_position];
        let index = (self.current_state.clone(), current_val);

        self.instructions.get(&index).cloned()
    }

    /// Returns true if the current state is undefined
    /// (i.e. there is no instruction for the current state and value)
    /// except if the current state is a final state
    pub fn is_undefined(&self) -> bool {
        self.get_instruction().is_none()
    }

    /// Calculates the next step of the Turing machine and returns true if the current state is a final state
    pub fn step(&mut self) -> bool {
        let current_val: bool = self.tape[self.tape_position];

        let Some(instruction) = self.get_instruction() else {
            if self.final_states.contains(&self.current_state) {
                return true;
            }

            error!(
                "No instruction given for state ({}, {})",
                self.current_state.clone(),
                if current_val {"1"} else {"0"}
            );

            return true;
        };
        self.tape[self.tape_position] = instruction.to_value;

        match instruction.movement {
            Movement::LEFT => {
                if self.tape_position == 0 {
                    self.tape.insert(0, false);
                } else {
                    self.tape_position -= 1;
                }
            }
            Movement::RIGHT => {
                if self.tape_position == self.tape.len() - 1 {
                    self.tape.push(false);
                }

                self.tape_position += 1;
            }
            Movement::HALT => {}
        }

        while self.tape_position <= 2 {
            self.tape.insert(0, false);
            self.tape_position += 1;
        }

        while self.tape_position >= self.tape.len() - 3 {
            self.tape.push(false);
        }

        self.update_state(instruction.to_state.clone())
    }

    /// Updates the current state and returns true if the current state is a final state
    fn update_state(&mut self, state: String) -> bool {
        self.current_state = state.clone();

        if self.frequencies.contains_key(&state) {
            let Some(f) = self.frequencies.get_mut(&state) else {
                return self.final_states.contains(&self.current_state);
            };
            *f += 1;
        } else {
            self.frequencies.insert(state.clone(), 1);
        }

        return self.final_states.contains(&self.current_state);
    }

    /// Returns true if the current state has been reached more times than the given threshold
    pub fn is_infinite_loop(&self, threshold: usize) -> bool {
        for (_, v) in self.frequencies.iter() {
            if *v > threshold {
                return true;
            }
        }

        return false;
    }

    /// Resets the frequencies of the states
    pub fn reset_frequencies(&mut self) {
        self.frequencies = HashMap::new();
    }

    /// Returns true if the current state is a final state
    pub fn finished(&self) -> bool {
        return self.final_states.contains(&self.current_state);
    }

    /// Returns the values of the tape
    /// (i.e. the number of 1s between each 0)
    pub fn values(&self) -> Vec<u32> {
        let tmp: String = self
            .tape
            .iter()
            .map(|v| if *v { "1" } else { "0" })
            .collect();

        tmp.split("0")
            .filter_map(|s| {
                if s.len() > 0 {
                    Some(s.len() as u32 - 1)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the string representation of the tape
    pub fn to_string(&self) -> String {
        let mut tmp1 = String::new();
        let mut tmp2 = String::new();

        for (i, v) in self.tape.iter().enumerate() {
            write!(&mut tmp1, "{} ", if v.clone() { "1" } else { "0" }).unwrap();

            if i == self.tape_position {
                tmp2 += "^ ";
            } else {
                tmp2 += "  ";
            }
        }

        format!("{}\n{}", tmp1, tmp2)
    }

    /// Returns the current output of the Turing machine
    /// (i.e. the number of steps and the number of 1s on the tape,
    /// or undefined if the Turing machine is in an undefined state)
    pub fn tape_value(&self) -> TuringOutput {
        if self.is_undefined() {
            return TuringOutput::Undefined(0);
        }

        TuringOutput::Defined((0, self.tape.iter().map(|v| if *v { 1 } else { 0 }).sum()))
    }

    /// Returns the final output of the Turing machine directly
    /// (i.e. keeps calculating the next step until the current state is a final state)
    pub fn final_result(&mut self) -> TuringOutput {
        let mut steps = 0;

        while !self.finished() {
            self.step();
            steps += 1;
        }

        TuringOutput::Defined((
            steps,
            self.tape.iter().map(|v| if *v { 1 } else { 0 }).sum(),
        ))
    }
}
