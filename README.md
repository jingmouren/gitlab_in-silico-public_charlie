# Portfolio

Tools to help with capital allocation process.

## TODO

- [x] Calculation of expected value
- [x] Calculation of probability of loss of capital
- [x] Improve test coverage of company_returns field
- [x] Allocation via Kelly's formula for multiple investments
- [x] Integration tests
- [x] Logging instead of printing
- [x] Refactor validation
- [ ] Server POC
- [ ] Remove AllValidationProblems struct
- [ ] Error handling
- [ ] API trait with controller and http client
- [ ] Command line interface
- [ ] Special (direct) handling of single company Kelly allocation


## Investment process

This software is rather simple, and it really serves two purposes, both of them related to capital allocation process:
1. Keep us honest when deciding on the amount of capital to allocate to a particular investment,
2. Automate the process, freeing up time for more useful things.

Therefore, I firmly believe that using (and even writing) this software should take less than 0.1% of the time when
making investment decisions. In other words, a vast majority of time should be spent on defining what goes into the
software as an input.

## Project

### Install rust

Software is written in the `rust` programming language. To install `rust` on a Unix-like system, do:
- Get `rustup` tool: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Open a new shell and verify: `rustc --version` (see details [here](https://www.rust-lang.org/tools/install))

### Clone and build the project

To clone and build the project, do:
- `git clone git@gitlab.com:in-silico-team/portfolio.git`
- `cd portfolio`
- `cargo build`

### Project structure

Project structure follows the recommended practices for `rust` projects:
- `src` directory contains the source code:
  - `lib.rs` contains the library (public) interface
  - `main.rs` contains the executables
  - `model` contains data classes
  - Unit tests are separate modules in the same source files that they're testing
- `tests` directory contains the integration tests

### Testing

To run both unit and integration tests with coverage, do:
- `cargo test`
- `cargo tarpaulin`
