FROM rust:bookworm AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG EXTRA_FEATURES=""

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpython3-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml .
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json --features="$EXTRA_FEATURES"
# Build application
COPY . .
RUN cargo build --release --bin sapiens_cli --features="$EXTRA_FEATURES"
RUN cargo build --release --bin sapiens_bot --features="$EXTRA_FEATURES"

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS base-runtime
WORKDIR app

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpython3.11 \
    python3-sympy \
    python3-numpy \
    python3-requests \
    python3-urllib3 \
    python3-bs4 \
    python3-feedparser \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

FROM base-runtime AS sapiens_cli

COPY --from=builder /app/target/release/sapiens_cli /usr/local/bin

USER 1000:0

ENTRYPOINT ["/usr/local/bin/sapiens_cli"]

FROM base-runtime AS sapiens_bot

COPY --from=builder /app/target/release/sapiens_bot /usr/local/bin

USER 1000:0

ENTRYPOINT ["/usr/local/bin/sapiens_bot"]
