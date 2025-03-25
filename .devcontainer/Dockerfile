FROM mcr.microsoft.com/devcontainers/base:bookworm

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    lldb \
    libpython3-dev \
    pkg-config \
    protobuf-compiler \
    libprotobuf-dev \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/*

RUN rm -f /usr/lib/python3.*/EXTERNALLY-MANAGED

USER vscode

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

ENV PATH="/home/vscode/.cargo/bin:${PATH}"

RUN rustup update

RUN rustup component add clippy rustfmt rust-src rust-analysis

RUN cargo install cargo-edit cargo-machete cargo-update
