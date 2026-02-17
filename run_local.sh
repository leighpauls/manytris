#!/usr/bin/env bash
set -euo pipefail

PIDS=()

cleanup() {
    echo "Cleaning up..."
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
}

trap cleanup EXIT INT TERM

echo "Starting game server..."
cargo run -- server --headless --port 9989 &
PIDS+=($!)

sleep 1

echo "Starting local manager..."
cargo run -p manytris_game_manager --bin local_manager &
PIDS+=($!)

echo "Starting bot..."
cargo run -- bot --host localhost --port 9989 --headless &
PIDS+=($!)

sleep 1

echo "Starting client..."
cargo run -- client --manager-server http://localhost:3000
