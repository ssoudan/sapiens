#!/usr/bin/env bash

set -e 

ALL="sapiens_bot sapiens_cli sapiens sapiens_derive sapiens_tools sapiens_exp"

cargo smart-release --update-crates-index ${ALL}

echo ""
echo "RUN: cargo smart-release --update-crates-index ${ALL} --execute"
echo ""
