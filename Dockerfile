# Multi-stage build to keep the final image small
FROM rust:latest as builder
WORKDIR /opt/charlie
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /opt/charlie/target/release/run_server /usr/local/bin
CMD ["run_server"]