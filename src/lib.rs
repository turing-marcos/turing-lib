mod instruction;
mod output;
mod turing;
mod warnings;

use std::{borrow::Cow, collections::HashMap};

pub use instruction::TuringInstruction;
pub use output::TuringOutput;
use pest::Parser;
use serde::{Deserialize, Serialize};
pub use turing::{Rule, TuringMachine, TuringParser};
pub use warnings::{CompilerError, CompilerWarning, ErrorPosition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub initial_state: Cow<'static, str>,
    pub final_state: Cow<'static, str>,
    pub used_states: Cow<'static, [Cow<'static, str>]>,
    pub code: Cow<'static, str>,
}

impl Library {
    pub fn get_instructions(
        &self,
    ) -> Result<HashMap<(String, bool), TuringInstruction>, CompilerError> {
        let mut instructions: HashMap<(String, bool), TuringInstruction> = HashMap::new();

        let file = match TuringParser::parse(Rule::instructions, self.code.as_ref()) {
            Ok(mut f) => f.next().unwrap(),
            Err(e) => panic!("{}", e),
        };

        for record in file.into_inner() {
            let tmp = match TuringInstruction::from(record.into_inner()) {
                Ok(i) => i,
                Err(e) => return Err(e),
            };
            instructions.insert(
                (tmp.from_state.clone(), tmp.from_value.clone()),
                tmp.clone(),
            );
        }

        Ok(instructions)
    }
}

/// Array of all the libraries that are included in the compiler.
/// # List of Libraries
///
/// ## sum
/// Adds two numbers together.
///
/// ## x2
/// Duplicates a number.
///
/// ## mod
/// Calculates the modulo of two numbers.
///
/// ## div2
/// Divides a number by two.
///
/// ## bound_diff
/// Calculates the difference between two numbers, but the result is always positive.
pub const LIBRARIES: [Library; 5] = [
    Library {
        name: Cow::Borrowed("sum"),
        description: Cow::Borrowed("x + y"),
        initial_state: Cow::Borrowed("q0"),
        final_state: Cow::Borrowed("q2"),
        used_states: Cow::Borrowed(&[
            Cow::Borrowed("q0"),
            Cow::Borrowed("q1"),
            Cow::Borrowed("q2"),
        ]),
        code: Cow::Borrowed(include_str!("./composition/sum.tm")),
    },
    Library {
        name: Cow::Borrowed("x2"),
        description: Cow::Borrowed("x * 2"),
        initial_state: Cow::Borrowed("q0"),
        final_state: Cow::Borrowed("qf"),
        used_states: Cow::Borrowed(&[
            Cow::Borrowed("q0"),
            Cow::Borrowed("q1"),
            Cow::Borrowed("q2"),
            Cow::Borrowed("q3"),
            Cow::Borrowed("q4"),
            Cow::Borrowed("q5"),
            Cow::Borrowed("qf"),
        ]),
        code: Cow::Borrowed(include_str!("./composition/duplicate.tm")),
    },
    Library {
        name: Cow::Borrowed("mod"),
        description: Cow::Borrowed("x mod y"),
        initial_state: Cow::Borrowed("q0"),
        final_state: Cow::Borrowed("qf"),
        used_states: Cow::Borrowed(&[
            Cow::Borrowed("q0"),
            Cow::Borrowed("q1"),
            Cow::Borrowed("q2"),
            Cow::Borrowed("q2"),
            Cow::Borrowed("q4"),
            Cow::Borrowed("q5"),
            Cow::Borrowed("q5"),
            Cow::Borrowed("q6"),
            Cow::Borrowed("q7"),
            Cow::Borrowed("q8"),
            Cow::Borrowed("q9"),
            Cow::Borrowed("q10"),
            Cow::Borrowed("q11"),
            Cow::Borrowed("qf"),
        ]),
        code: Cow::Borrowed(include_str!("./composition/mod.tm")),
    },
    Library {
        name: Cow::Borrowed("div2"),
        description: Cow::Borrowed("x div 2"),
        initial_state: Cow::Borrowed("q0"),
        final_state: Cow::Borrowed("qf"),
        used_states: Cow::Borrowed(&[
            Cow::Borrowed("q0"),
            Cow::Borrowed("q1"),
            Cow::Borrowed("q2"),
            Cow::Borrowed("qf"),
        ]),
        code: Cow::Borrowed(include_str!("./composition/div2.tm")),
    },
    Library {
        name: Cow::Borrowed("bound_diff"),
        description: Cow::Borrowed("x ∸ y"),
        initial_state: Cow::Borrowed("q0"),
        final_state: Cow::Borrowed("qf"),
        used_states: Cow::Borrowed(&[
            Cow::Borrowed("q0"),
            Cow::Borrowed("q1"),
            Cow::Borrowed("q2"),
            Cow::Borrowed("q3"),
            Cow::Borrowed("q4"),
            Cow::Borrowed("q5"),
            Cow::Borrowed("q6"),
            Cow::Borrowed("qf"),
        ]),
        code: Cow::Borrowed(include_str!("./composition/bound_diff.tm")),
    },
];

#[cfg(test)]
mod test_parsing {
    use std::fs;

    use crate::warnings::ErrorPosition;
    use crate::CompilerError;
    use crate::Rule;
    use crate::TuringMachine;
    use crate::TuringParser;
    use pest::{consumes_to, parses_to};

    #[test]
    fn parse_description() {
        let test = "/// a + b\r\n";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::description,
            tokens: [
                description(0, 11),
            ]
        }
    }

    #[test]
    fn parse_tape_valid() {
        let test = "{111011};";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::tape,
            tokens: [
                tape(0, 9, [
                    value(1, 2),
                    value(2, 3),
                    value(3, 4),
                    value(4, 5),
                    value(5, 6),
                    value(6, 7),
                ]),
            ]
        }
    }

    #[test]
    // Test that the parser fails when the tape does not contain a 1
    fn parse_tape_zeros() {
        let test = "
        {000};
        I = {q0};
        F = {q2};
        
        (q0, 1, 0, R, q1);
        
        (q1, 1, 1, R, q1);
        (q1, 0, 0, R, q2);
        
        (q2, 1, 0, H, q2);
        (q2, 0, 0, H, q2);
        ";

        let tm_error = TuringMachine::new(test);

        let expected: CompilerError = CompilerError::SyntaxError {
            position: ErrorPosition::new((1, 9), None), // FIXME: Positions are not correct
            message: String::from("Expected at least a 1 in the tape"),
            code: String::from("000"),
            expected: Rule::tape,
            found: None,
        };

        assert_eq!(tm_error.unwrap_err(), expected);
    }

    #[test]
    fn parse_initial_state() {
        let test = "I = {q0};";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::initial_state,
            tokens: [
                initial_state(0, 9, [
                    state(5, 7)
                ])
            ]
        }
    }

    #[test]
    fn parse_final_state() {
        let test = "F = {q2};";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::final_state,
            tokens: [
                final_state(0, 9, [
                    state(5, 7)
                ])
            ]
        }
    }

    #[test]
    fn parse_instruction() {
        let test = "(q0, 1, 0, R, q1);";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::instruction,
            tokens: [
                instruction(0, 18, [
                    state(1, 3),
                    value(5, 6),
                    value(8, 9),
                    movement(11, 12),
                    state(14, 16)
                ]),
            ]
        }
    }

    #[test]
    fn parse_file() {
        let unparsed_file = fs::read_to_string("Examples/Example1.tm").expect("cannot read file");
        let (tm, _) = match TuringMachine::new(&unparsed_file) {
            Ok(t) => t,
            Err(e) => {
                TuringMachine::handle_error(e);
                std::process::exit(1);
            }
        };

        assert_eq!(
            tm.to_string(),
            "0 0 0 1 1 1 1 1 0 1 1 \n      ^               "
        )
    }
}

#[cfg(test)]
mod test_composition {
    use crate::Rule;
    use crate::TuringMachine;
    use crate::TuringOutput;
    use crate::TuringParser;
    use crate::LIBRARIES;
    use pest::{consumes_to, parses_to};

    #[test]
    fn parse_composition_function_name_valid() {
        let test = "sum_test";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::function_name,
            tokens: [
                function_name(0, 8)
            ]
        }
    }

    #[test]
    fn parse_composition_valid() {
        let test = "compose = { sum_test };";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::composition,
            tokens: [
                composition(0, 23, [
                    function_name(12, 20)
                ])
            ]
        }
    }

    #[test]
    fn parse_multiple_compositions() {
        let test = "compose = {sum, diff};";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::composition,
            tokens: [
                composition(0, 22, [
                    function_name(11, 14),
                    function_name(16, 20)
                ])
            ]
        }
    }

    #[test]
    /// Test that all the libraries are correctly parsed
    /// (should not panic)
    fn libraries() {
        for lib in LIBRARIES {
            let _ = lib.get_instructions();
        }
    }

    #[test]
    /// Test compiling a program that uses composition and nothing else (no extra code)
    /// Also tests that you can write the `compose`, tape (`{111011}`), initial state (`I = {q0}`) and final state (`F = {q2}`) in any order
    fn composition() {
        let test = "
        compose = {sum};
        
        F = {q2};
        {111011};
        I = {q0};
        ";

        let mut tm = match TuringMachine::new(test) {
            Ok(t) => t.0,
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        };

        assert_eq!(tm.final_result(), TuringOutput::Defined((5, 3)));

        assert_eq!(
            tm.to_string(),
            "0 0 0 0 1 1 0 0 1 0 0 \n              ^       "
        );
    }
}
