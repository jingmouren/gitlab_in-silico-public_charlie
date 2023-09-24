## How-To Guide

### Install rust

To install `rust` on a Unix-like system, do:
- Get `rustup` tool: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Open a new shell and verify: `rustc --version` (see details [here](https://www.rust-lang.org/tools/install))

### Clone and build the project

To clone and build the project, do:
- `git clone git@gitlab.com:in-silico-team/charlie.git`
- `cd charlie`
- `cargo build --release`

### Execute

The software consists of two applications with two interfaces, a CLI application and a server with REST API. The build
process produces two applications in the [target directory](/target/release).

#### CLI application

To check the options in the CLI application, run:
```./target/release/charlie_cli -h```

Via the CLI, there are two options:
1. `allocate` -> Solves the allocation problem by providing candidate companies,
2. `analyze` -> Prints out useful information about a portfolio.

#### Server application

The server can be started with:
```./target/release/run_server```

Server can also be run within a Docker container:
```docker run --network="host" -v ${pwd}:/usr/src/charlie registry.gitlab.com/in-silico-team/charlie:latest```

To re-generate the OpenAPI schema after updates to the interface, run:
```cargo run --bin generate_schema```

After generating the JSON schema, the application also calls `npx` to generate new index.html based on the schema.

### Tests

To run both unit and integration tests with coverage, do:
- `cargo test`
- `cargo tarpaulin --ignore-tests --timeout 120`
