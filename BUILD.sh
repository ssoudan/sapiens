#!env /bin/bash

set -e

RED='\033[0;31m'
BLUE='\033[0;34m'
NORMAL='\033[0m'
GREEN='\033[0;32m'


echo -e "${BLUE}Testing...${NORMAL}"
cargo test --all --all-features || (echo -e "$RED [Tests failed] $NORMAL" && exit 1)

echo -e "${BLUE}Building...${NORMAL}"
cargo build --all --all-features || (echo -e "$RED [Build failed] $NORMAL" && exit 1)

echo -e "${BLUE}Checking...${NORMAL}"
cargo check --all --all-features --tests --benches --examples || (echo -e "$RED [Check failed] $NORMAL" && exit 1)

echo -e "${BLUE}Clippying...${NORMAL}"
cargo clippy --all --all-features --tests --benches --examples -- -D clippy::all || (echo -e "$RED [Clippy failed] $NORMAL" && exit 1)

echo -e "${BLUE}Formatting...${NORMAL}"
cargo +nightly fmt --all -- --check || (echo -e "$RED [Format failed] $NORMAL" && exit 1)

echo -e "${BLUE}Licensing...${NORMAL}"
cargo deny check || (echo -e "$RED [License check failed] $NORMAL" && exit 1)

echo -e "${BLUE}Udeps...${NORMAL}"
cargo +nightly udeps || (echo -e "$RED [Udep failed] $NORMAL" && exit 1)

#echo -e "${BLUE}Benchmarking...${NORMAL}"
#cargo criterion --all --features=unstable

#open target/criterion/reports/index.html

echo -e "${BLUE}Build container...${NORMAL}"
docker build -t botrs .

echo -e "$GREEN === OK === $NORMAL"
