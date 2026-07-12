# syntax=docker/dockerfile:1.7@sha256:a57df69d0ea827fb7266491f2813635de6f17269be881f696fbfdf2d83dda33e

# Build/test only. This image is not a product runtime and never runs NWN EE.
FROM node:24.15.0-bookworm@sha256:f22d6a1f082c02f292e86929b5b0442ac2e5eaf438a5dea9b1566601c3e05940 AS node_toolchain

RUN test "$(node --version)" = "v24.15.0"

FROM rust:1.96.1-bookworm@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663 AS rust_toolchain

COPY --from=node_toolchain /usr/local/bin/node /usr/local/bin/node

RUN rustup component add rustfmt clippy \
    && rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack --version 0.15.0 --locked \
    && rustc --version | grep -E '^rustc 1\.96\.1 ' \
    && cargo --version | grep -E '^cargo 1\.96\.1 ' \
    && test "$(node --version)" = "v24.15.0" \
    && test "$(wasm-pack --version)" = "wasm-pack 0.15.0"

FROM rust_toolchain AS quality

WORKDIR /workspace

# Keep the context auditable: only workspace inputs required by the gates enter
# the image. Retail/CEP assets, proof outputs and host configuration are never
# copied.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/m2a-core/Cargo.toml crates/m2a-core/Cargo.toml
COPY crates/m2a-core/src crates/m2a-core/src
COPY crates/m2a-core/tests crates/m2a-core/tests
COPY crates/m2a-wasm/Cargo.toml crates/m2a-wasm/Cargo.toml
COPY crates/m2a-wasm/src crates/m2a-wasm/src

RUN rustc --version \
    && cargo --version \
    && node --version \
    && wasm-pack --version \
    && cargo fmt --all -- --check \
    && cargo clippy --locked --workspace --all-targets -- -D warnings \
    && cargo test --locked --workspace \
    && cargo build --locked -p m2a-wasm --target wasm32-unknown-unknown \
    && wasm-pack test --node crates/m2a-wasm
