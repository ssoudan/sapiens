#!/usr/bin/env bash

set -e 

ALL="sapiens_bot sapiens_cli sapiens sapiens_derive sapiens_tools"

cargo smart-release --update-crates-index ${ALL}
