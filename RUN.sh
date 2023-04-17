#!/usr/bin/env bash

set -e

IMAGE_NAME="botrs"

test -e .env || (echo "Please create a .env file with the required environment variables" && exit 1)

ARGS=("$@")

# test if docker cli is installed
if ! command -v docker > /dev/null 2>&1; then
    echo "Docker is not installed. Please install it first."
    exit 1
fi

# test if docker is installed and running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start it first."
    exit 1
fi

# Test if the image exists
if ! docker image inspect $IMAGE_NAME > /dev/null 2>&1; then
    echo "Image $IMAGE_NAME not found. Build it first with ./BUILD.sh ."
fi

# Run the application
docker run -it --rm --name botrs \
    --env-file .env \
    $IMAGE_NAME "${ARGS[@]}"
