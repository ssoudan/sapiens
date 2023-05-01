#!/usr/bin/env bash

set -e

IMAGE_NAME="sapiens_exp"
OUTPUT_DIR="experiments/data"

test -e .env || (echo "Please create a .env file with the required environment variables" && exit 1)

test -d $OUTPUT_DIR || mkdir $OUTPUT_DIR

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
docker run -it --rm --read-only --name ${IMAGE_NAME} \
    -v "$(pwd)"/${OUTPUT_DIR}:/app/experiments/data \
    --env-file .env \
    $IMAGE_NAME "${ARGS[@]}"
