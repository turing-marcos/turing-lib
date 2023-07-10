mod instruction;
mod output;
mod turing;

use std::borrow::Cow;

pub use instruction::TuringInstruction;
pub use output::TuringOutput;
use serde::{Serialize, Deserialize};
pub use turing::{Rule, TuringMachine, TuringParser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub initial_state: Cow<'static, str>,
    pub final_state: Cow<'static, str>,
    pub code: Cow<'static, str>,
}

pub const LIBRARIES: [Library; 2] = [
    Library {
        name: Cow::Borrowed("sum"),
        description: Cow::Borrowed("x + y"),
        initial_state: Cow::Borrowed("p0"),
        final_state: Cow::Borrowed("p2"),
        code: Cow::Borrowed(include_str!("./composition/sum.tm"))
    },
    Library {
        name: Cow::Borrowed("duplicate"),
        description: Cow::Borrowed("2x"),
        initial_state: Cow::Borrowed("p0"),
        final_state: Cow::Borrowed("pf"),
        code: Cow::Borrowed(include_str!("./composition/duplicate.tm"))
    },
];

#[cfg(test)]
mod test_parsing {
    use std::fs;

    use crate::Rule;
    use crate::TuringMachine;
    use crate::TuringParser;
    use pest::error::ErrorVariant;
    use pest::Position;
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

        let expected: pest::error::Error<Rule> = pest::error::Error::new_from_pos(
            ErrorVariant::CustomError {
                message: String::from("Expected at least a 1 in the tape"),
            },
            Position::from_start(""),
        );
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
        let tm = match TuringMachine::new(&unparsed_file) {
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
    use crate::TuringParser;
    use pest::{consumes_to, parses_to};

    #[test]
    fn parse_composition_function_name_valid(){
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
    fn parse_composition_valid(){
        let test = "sum_test()";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::composition,
            tokens: [
                composition(0, 10, [
                    function_name(0, 8)
                ])
            ]
        }
    }


    #[test]
    fn parse_composition_instruction_valid(){
        let test = "sum_test();";

        parses_to! {
            parser: TuringParser,
            input: test,
            rule: Rule::instruction,
            tokens: [
                instruction(0, 11, [
                    composition(0, 10, [
                        function_name(0, 8)
                    ])
                ])
            ]
        }
    }
}