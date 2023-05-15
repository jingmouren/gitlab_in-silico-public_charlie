## Charlie

The software is named after Charlie Munger, and is here to help you out in portfolio allocation for a focused, value
investing approach.

The underlying philosophy is that you should spend the majority of your time analyzing investments and thinking about
intrinsic values and different scenarios that might play out, independently of outside thoughts and events. The software
will force you to think about unpredictable external events. In fact, it won't help you if you're analyzing a company
for which you cannot think of at least one downside scenario.

Note that **the software can give you advice, but it cannot give you conduct**. If you find yourself using this tool
very often by changing inputs to match your preconceived expectations, then you're probably on the wrong path.

### How should you work with this tool?

You should use this software in the following way:
- You spend 99.99% of your time thinking about companies that you'd like to invest in, and providing different scenarios
  for each company. A scenario is described by your estimate of the intrinsic value and the probability that this
  intrinsic value will be realized at some (undefined) point in the future.
- The tool will then take your inputs and calculate optimal allocation percentage for the candidate companies, by
  maximizing long-term growth rate of your assets. This is equivalent to Kelly's approach, but generalized for multiple
  simultaneous investments in a focused portfolio.

### What this tool won't allow you to do?

There are certain things that the tool won't allow you to do, in order to provide additional margins of safety. The
tool won't allow you to:
- Invest in companies that have negative expected return based on your inputs. In other words, it won't let you go
  short,
- Use leverage, because you shouldn't be in a hurry to get rich, although purely mathematical solution would
  oftentimes imply use of leverage,
- Invest in companies that do not have a downside. If a company truly didn't have a downside, you should lever up
  infinitely and put all your assets into it, which is usually not a good idea unless you have a founder-like insight
  into a fairly predictable business, in which case you shouldn't need this tool. Note that these insights probably
  represent the best investment opportunities, so you should consider investing in them outside of this framework.

### Disclaimer

Calculating intrinsic value of a company is more of an art than science, especially for a high-quality, growing
businesses within your circle of competence. And according to Charlie Munger, Warren Buffett, Mohnish Pabrai and the
like, one should focus precisely on getting such great businesses for a fair price. That means that you shouldn't take
what this tool says at face value, and you should probably use its guidance infrequently.

During the mathematical derivation of the problem (optimizing long-term growth rate), an assumption is made that the
number of similar bets with similar outcomes is very high (tends to infinity). I don't have a hard mathematical proof
that this is completely ok for a focused investment strategy, although I feel confident it is ok iff one considers the
following margins of safety:
1. No shorting allowed,
2. No use of leverage allowed,
3. No companies without at least one downside scenario are allowed,
4. And by far most importantly, conservative input assumptions should be provided. This is the tricky part, because this
   tool unfortunately can't protect you from yourself.

## Documentation

A white-paper describing the investment framework and the mathematics behind the software is found in
[doc directory](/doc). The white-paper is written in `LaTeX` and the .pdf can be compiled by running:
`pdflatex paper.tex && bibtex paper.aux && for i in {0..1}; do pdflatex paper.tex; done`

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
- `git clone git@gitlab.com:in-silico-team/charlie.git`
- `cd charlie`
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
- `examples` directory contains examples on client-side use for all endpoints/functionality

### Execution

- Generate the schema: `cargo run --bin generate_schema`
- Start the server: `cargo run --release --bin run_server`
  - After the server is up and running, you can try running the two clients in `examples`:
    - `cargo run --example allocate_client`
    - `cargo run --example analyze_client`

Server can also be run within a Docker container: `docker run --network="host" -v ${pwd}:/usr/src/charlie registry.gitlab.com/in-silico-team/charlie:latest`

## Continuous Integration

The CI pipeline consists of format check, linter check, security check, unit and integration tests, docker build and
running the server with two client examples inside Docker containers. See `.gitlab-ci.yml` for details.

### OpenAPI schema update

In order to update the OpenAPI schema (after introducing changes to the request/response payloads), do the following:
- Generate the schema: `cargo run --bin generate_schema`

After generating the JSON schema, the application also calls `npx` to generate new index.html based on the schema. 

### Testing

To run both unit and integration tests with coverage, do:
- `cargo test`
- `cargo tarpaulin --ignore-tests`

