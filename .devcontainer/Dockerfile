FROM rust:1.99.1-bookworm
RUN apt-get update \
    && apt-get install -y --no-install-recommends git openssh-client pkg-config libssl-dev libsqlite3-dev curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt clippy \
    && rustup target add wasm32-unknown-unknown \
    && cargo install trunk --version 0.21.14 --locked
