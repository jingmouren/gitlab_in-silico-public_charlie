# Multi-stage build to keep the final image small

# STAGE 1: BUILDER image
FROM rust:1.74 as BUILDER

# Create a blank (binary) project
RUN USER=root cargo new --bin /usr/src/charlie
WORKDIR /usr/src/charlie

# Create a lib.rs file because we have it in the project as well
RUN touch src/lib.rs

# Copy package dependencies first so that they're cached
COPY ./Cargo.toml ./Cargo.toml

# Build the project and delete dummy library
RUN cargo build --release
RUN rm -rf target/release/*charlie* target/release/deps/*charlie* target/release/.fingerprint/*charlie*

# Copy everything that's needed
COPY src src
COPY examples examples
COPY schema schema
COPY server_config.toml server_config.toml

# Build the whole project
RUN cargo build --release
RUN cargo build --example allocate_client --example analyze_client --example api_client --release

# STAGE 2: FINAL image
FROM debian:bookworm-slim as FINAL
RUN apt-get update && apt-get install -y tzdata ca-certificates && rm -rf /var/lib/apt/lists/*

# Expose port 8000, which will be exposed to the outside when running the container
EXPOSE 8000

# Copy schema directory and Cargo.toml because that's the directory structure we need for loading the files needed at
# runtime: Server configuration and schema definition
COPY --from=BUILDER /usr/src/charlie/server_config.toml /usr/local/bin/server_config.toml
COPY --from=BUILDER /usr/src/charlie/schema /usr/local/bin/schema
COPY --from=BUILDER /usr/src/charlie/Cargo.toml /usr/local/bin/Cargo.toml

# Copy the binaries
COPY --from=BUILDER /usr/src/charlie/target/release/run_server /usr/local/bin/
COPY --from=BUILDER /usr/src/charlie/target/release/examples/allocate_client /usr/local/bin/
COPY --from=BUILDER /usr/src/charlie/target/release/examples/analyze_client /usr/local/bin/
COPY --from=BUILDER /usr/src/charlie/target/release/examples/api_client /usr/local/bin/

# Run the server
CMD ["run_server"]
