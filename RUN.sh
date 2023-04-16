#!/usr/bin/env bash

set -e

# Run the application
docker run -it --rm --name botrs \
    -v $(pwd)/.env:/app/.env:ro \
    botrs
