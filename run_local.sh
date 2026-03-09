#!/usr/bin/env bash
set -euo pipefail

NUM_BOTS=1

while [[ $# -gt 0 ]]; do
    case "$1" in
        --num-bots)
            NUM_BOTS="$2"
            shift 2
            ;;
        *)
            echo "Usage: $0 [--num-bots N]"
            exit 1
            ;;
    esac
done

if ! [[ "$NUM_BOTS" =~ ^[0-9]+$ ]] || [ "$NUM_BOTS" -lt 0 ] || [ "$NUM_BOTS" -gt 10 ]; then
    echo "Error: --num-bots must be between 0 and 10"
    exit 1
fi

export RUST_BACKTRACE=1

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

for i in $(seq 1 "$NUM_BOTS"); do
    echo "Starting bot $i..."
    cargo run -- bot --host localhost --port 9989 --headless &
    PIDS+=($!)
done

sleep 1

echo "Starting client..."
cargo run -- client --manager-server http://localhost:3000
