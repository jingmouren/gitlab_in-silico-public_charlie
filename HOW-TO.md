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

The software provides two interfaces, a command line interface and a REST API.

#### Server application

The server can be started with:
```cargo run --release --bin run_server```

Server can also be run within a Docker container:
```docker run --network="host" -v ${pwd}:/usr/src/charlie registry.gitlab.com/in-silico-team/charlie:latest```

To re-generate the OpenAPI schema after updates to the interface, run:
```cargo run --bin generate_schema```

After generating the JSON schema, the application also calls `npx` to generate new index.html based on the schema.

#### CLI application

To check all the options in the CLI application, run:
```cargo run --release --bin charlie_cli -h```

### Tests

To run both unit and integration tests with coverage, do:
- `cargo test`
- `cargo tarpaulin --ignore-tests --timeout 120`

