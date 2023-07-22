# Turing Machine Backend Library

A powerful and efficient Turing Machine backend library written in Rust! This library contains the essential components, compiler, and abstractions needed to create, run, and manage Turing Machines.

## Demo

Check out the [online demo](https://turing.coldboard.net) (here is [the code](https://github.com/turing-marcos/turing-machine))

## Features

- Efficient Turing Machine implementation
- Compiler for custom Turing Machine code
- Abstractions to create and modify Turing Machines programmatically
- Support for both deterministic and non-deterministic Turing Machines
- Cross-platform compatibility

## Installation

Add the following to your `Cargo.toml` file under `[dependencies]`:

```toml
turing_lib = "^2.1"
```
or
```toml
turing_lib = { git = "https://github.com/turing-marcos/turing-lib/" }
```
for the git version.

Then, run cargo build to download and compile the library.

## Usage

To use the Turing Machine Backend Library in your Rust project, simply import it:
```Rust
use turing_lib::TuringMachine;

fn main() {
    let unparsed_file = fs::read_to_string(&"./some_file").expect("cannot read file");

    let (tm, warnings) = match TuringMachine::new(&unparsed_file) {
        Ok(t) => t,
        Err(e: CompilerError) => {
            handle_error(e, file);
            std::process::exit(1);
        }
    };
}
```

Refer to the [API documentation](https://docs.rs/turing-lib/latest/turing_lib/) for detailed information on the available methods and structures.

## Examples

You can find examples on how to use this library in the examples folder of this repository.

## Contributing

We welcome contributions! Feel free to submit pull requests, issues, or suggestions. Please follow the [contributing guidelines](https://github.com/turing-marcos/turing-lib/)