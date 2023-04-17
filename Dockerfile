FROM rust:bookworm AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpython3-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml .
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --package botrs --release --bin botrs

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR app

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpython3.11 \
    python3-sympy \
    python3-numpy \
    python3-requests \
    python3-urllib3 \
    python3-bs4 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir /app/root

COPY --from=builder /app/target/release/botrs /usr/local/bin

USER 1000:0

ENTRYPOINT ["/usr/local/bin/botrs"]