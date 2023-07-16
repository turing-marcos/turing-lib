use log::{debug, error, info, warn};
use pest::Parser;
use pest_derive::Parser;
use std::{collections::HashMap, fmt::Write};

use crate::{
    instruction::Movement, warnings::ErrorPosition, CompilerError, CompilerWarning, Library,
    TuringInstruction,
};

use super::TuringOutput;

#[derive(Parser)]
#[grammar = "../turing.pest"]
pub struct TuringParser;

#[derive(Debug, Clone)]
/// A Turing machine
pub struct TuringMachine {
    /// The dictionary of instructions for the machine.
    pub instructions: HashMap<(String, bool), TuringInstruction>,

    /// The final states of the machine. If the machine reaches one of these states, it will stop.
    pub final_states: Vec<String>,

    /// The current state of the machine.
    pub current_state: String,

    /// The position of the head on the tape.
    pub tape_position: usize,

    /// The binary tape of the machine.
    pub tape: Vec<bool>,

    /// The frequencies of the states. Used to detect infinite loops.
    pub frequencies: HashMap<String, usize>,

    /// The description of the machine. Found in the `///` comments at the top of the file.
    pub description: Option<String>,

    /// The composed libraries that the machine uses.
    /// Used only as information, since their instructions are already compiled into the machine.
    pub composed_libs: Vec<Library>,

    /// The actual code of the machine. Used for resetting the machine and debugging.
    pub code: String,
}

impl TuringMachine {
    /// Create a new Turing machine from a string of code
    pub fn new(code: &str) -> Result<(Self, Vec<CompilerWarning>), CompilerError> {
        let mut instructions: HashMap<(String, bool), TuringInstruction> = HashMap::new();
        let mut final_states: Vec<String> = Vec::new();
        let mut current_state: String = String::new();
        let mut tape: Vec<bool> = Vec::new();
        let mut description: Option<String> = None;
        let mut composed: Vec<Library> = Vec::new();
        let mut warnings: Vec<CompilerWarning> = Vec::new();

        let file = match TuringParser::parse(Rule::file, code) {
            Ok(mut f) => f.next().unwrap(),
            Err(error) => return Err(CompilerError::FileRuleError { error }),
        };

        for record in file.into_inner() {
            let record_span = &record.as_span();

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

                    // Used to extract the position of the error (if any)
                    // A span contains the start and end position of the error, while a Pair only contains the start position
                    let span = record.line_col();

                    let code = record.clone().into_inner().as_str();

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

                        return Err(CompilerError::SyntaxError {
                            position: span.into(),
                            message: String::from("Expected at least a 1 in the tape"),
                            code: String::from(code),
                            expected: Rule::tape,
                            found: None,
                        });
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
                Rule::composition => {
                    debug!("Entered composition rule");
                    for r in record.into_inner() {
                        match r.as_rule() {
                            Rule::function_name => {
                                debug!("Found composition of: {}", r.as_str());

                                let mut lib: Option<Library> = None;

                                for l in super::LIBRARIES {
                                    if l.name == r.as_str() {
                                        lib = Some(l);
                                        break;
                                    }
                                }

                                if let Some(library) = lib {
                                    debug!("Found the library, composing...");

                                    instructions.extend(library.get_instructions());

                                    composed.push(library.clone());
                                } else {
                                    error!("Could not find the library \"{}\"", r.as_str());

                                    let (line, column) = r.line_col();

                                    return Err(CompilerError::SyntaxError {
                                        position: ErrorPosition::new((line, column), None),
                                        message: format!(
                                            "Could not find the library \"{}\"",
                                            r.as_str()
                                        ),
                                        code: String::from(r.as_str()),
                                        expected: r.as_rule(),
                                        found: None,
                                    });
                                }
                            }
                            _ => warn!(
                                "Unhandled: ({:?}, {})",
                                r.as_rule(),
                                r.into_inner().as_str()
                            ),
                        }
                    }
                }
                Rule::instruction => {
                    let tmp = TuringInstruction::from(record.into_inner());

                    if instructions.contains_key(&(tmp.from_state.clone(), tmp.from_value.clone()))
                    {
                        warn!("Instruction {} already exists, overwriting it", tmp.clone());

                        warnings.push(CompilerWarning::StateOverwrite {
                            position: record_span.into(),
                            state: tmp.from_state.clone(),
                            value_from: tmp.from_value.clone(),
                        })
                    }
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

        Ok((
            Self {
                instructions,
                final_states,
                current_state,
                tape_position,
                tape,
                frequencies: HashMap::new(),
                description,
                composed_libs: composed,
                code: String::from(code),
            },
            warnings,
        ))
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
            composed_libs: Vec::new(),
            code: String::new(),
        }
    }

    /// Parse a Turing machine code syntax error
    /// and print it to the console
    pub fn handle_error(error: CompilerError) {
        error!("I found an error while parsing the file!");

        let position = error.position();

        debug!("Error position: {:?}", position);

        error!(
            "Error at {}: {}\n\t{}\n\t{:~>width1$}{:^<width2$}{:~<width3$}",
            position,
            error.message(),
            error.code(),
            "~",
            "^",
            "~",
            width1 = position.start.1,
            width2 = position.end.unwrap_or((0, position.start.1 +1)).1 - position.start.1,
            width3 = error.code().len() - position.end.unwrap_or((0, position.start.1 +1)).1
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
