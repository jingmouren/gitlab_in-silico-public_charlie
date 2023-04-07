## Epictetus

Epictetus is here to help you out with the investment process by reminding you that you should spend the majority of
your time analyzing investments, independently of outside thoughts and events. It will also remind you that you should
take into account unpredictable external events. In fact, Epictetus won't help you if you're analyzing a company for 
which you cannot think of at least small downside scenario.

Note that **Epictetus can give you advice, but he cannot give you conduct**. If you find yourself using this tool very
often by changing inputs to match your preconceived expectations, then you're probably on the wrong path.

### How should you work together with Epictetus?

You should work together with Epictetus in the following way:
- You spend 99.99% of your time thinking about companies that you'd like to invest it and providing different scenarios
  for each company. A scenario is described by your estimate of the intrinsic value and the probability that this
  intrinsic value will be realized at some point in the future.
- Epictetus will then take your inputs and calculate optimal allocation percentage for the candidate companies, by
  maximizing long-term growth rate of your assets. This is essentially equivalent to Kelly's approach, but generalized
  for multiple simultaneous investments in a focused-like portfolio.

### What Epictetus won't allow you to do?

Epictetus won't allow you to:
- Invest in companies that have negative expected return based on your inputs. In other words, it won't let you go
  short,
- Invest in companies that do not have a downside. If a company truly didn't have a downside, you should lever up
  infinitely and put all your assets into this company, which is usually not a good idea unless you have a founder-like
  insight into a fairly predictable business,
- Use leverage, because you shouldn't be in a hurry to get rich, although the purely mathematical solution would
  oftentimes imply use of leverage.

### Assumptions

During the mathematical derivation of the problem (optimizing long-term growth rate), an assumption that the number of
similar bets with similar outcomes is very high (tends to infinity). I don't have a hard mathematical proof that this is
completely ok for a focused investment strategy, although I feel confident it is iff one considers the following margins
of safety:
1. No shorting allowed,
2. No use of leverage allowed,
3. No companies without at least one downside scenario are allowed,
4. And by far most importantly, conservative input assumptions should be provided.

## Project

Project is written in the `rust` programming language and is implemented as a microservice with only a few endpoints
related to the portfolio allocation and analysis. The OpenAPI schema is generated using Dropshot and served through an
additional endpoint.

### Install rust

To install `rust` on a Unix-like system, do:
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
  - `model` contains data model classes
  - `validation` contains model for validation errors and the interface for performing validations
  - Other files contain business-level logic for solving the allocation process
  - Unit tests are separate modules in the same source files that they're testing
- `src/bin` directory contains two binaries: one for generating the OpenAPI schema and one for running a server
- `tests` directory contains integration tests
- `examples` directory contains 

### Execution

- Generate the schema: `cargo run --bin generate_schema`
- Start the server: `cargo run --release --bin run_server`
  - After the server is up and running, you can try running the two clients in `examples`:
    - `cargo run --example allocate_client`
    - `cargo run --example analyze_client`

## CI

The CI pipeline consists of format check, linter check, security check, unit and integration tests, and running the
server with two client examples. See `.gitlab-ci.yml` for details.

### OpenAPI schema update

In order to update the OpenAPI schema (after introducing changes to the request/response payloads), do the following:
- Generate the schema: `cargo run --bin generate_schema`
- Update the HTML index: `npx @redocly/cli build-docs schema/openapi.json  -o schema/index.html`

### Testing

To run both unit and integration tests with coverage, do:
- `cargo test`
- `cargo tarpaulin --ignore-tests`

