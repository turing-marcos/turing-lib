# Changelog

## [2.1.0] - 2023-07-22
### Added
- Syntax support for wrinting the definitions (`I`, `F`, `compose` and the tape) in any order.
- Error handling for when there are no final states or no intial state.

### Changed
- Updated minor version of dependencies
- Now parsing an instruction can return an error if the instruction is not valid.
- Now defining instructions is optional. If no instructions are defined, the turing machine will be empty apart from the composed libraries.

### Fixed
- Syntax support for the movements `I` (Izquierda) and `D` (Derecha).
- The `finished` function now takes into account the movement in addition to whether the state is final or not.

## [2.0.2] - 2023-07-18
### Fixed
- All of the libraries' code was missing a semicolon at the end of each line.

### Added
- A new test for checking that multiple libraries can be composed at once.
- A new test for checking that all libraries compile and are valid.

## [2.0.0] - 2023-07-16
### Added
- Syntax support for the movements `I` (Izquierda) and `D` (Derecha).
- Syntax support for composing libraries.
- Multiple integrated libraries for Turing Machines:
    - `sum`
    - `x2`
    - `mod`
    - `div2`
    - `bound_diff`
- A new test for checkinf that library names are recognized.
- A new test for checking that the composition syntax is recognized.
- Documentation
- Three new enums for returning errors and warnings from the compiler (`CompilerError` with `ErrorPosition` and `CompilerWarning`).

### Changed
- The return value of `TuringMachine::new`. It now can return either a [`CompilerError`](https://docs.rs/turing-lib/latest/turing_lib/enum.CompilerError.html) or a tuple with the turing machine and a vector of [`CompilerWarning`s](https://docs.rs/turing-lib/latest/turing_lib/enum.CompilerWarning.html).

### Fixed
- The function `handle_error` now just prints the error returned by the compiler.


## [1.1.3] - 2023-03-27
### Added
- The GPL-2.0 license.
- Contributing guidelines.
- The Readme file.
- The github workflow for testing the library on release.

## [1.1.2] - 2023-03-27
### Added
- Initial release of the library.