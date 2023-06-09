mod instruction;
mod output;
mod turing;

pub use instruction::TuringInstruction;
pub use output::TuringOutput;
pub use turing::{Rule, TuringMachine, TuringParser};

#[cfg(test)]
mod tests {
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
