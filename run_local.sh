#!/usr/bin/env bash
set -euo pipefail

NUM_BOTS=1
CARGO_PROFILE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --num-bots)
            NUM_BOTS="$2"
            shift 2
            ;;
        --release)
            CARGO_PROFILE="--release"
            shift
            ;;
        *)
            echo "Usage: $0 [--num-bots N] [--release]"
            exit 1
            ;;
    esac
done

if ! [[ "$NUM_BOTS" =~ ^[0-9]+$ ]] || [ "$NUM_BOTS" -lt 0 ] || [ "$NUM_BOTS" -gt 10 ]; then
    echo "Error: --num-bots must be between 0 and 10"
    exit 1
fi

export RUST_BACKTRACE=1

prefix_output() {
    local label="$1"
    local color="$2"
    sed -u "s/^/\x1b[${color}m[${label}]\x1b[0m /"
}

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
cargo run $CARGO_PROFILE -- server --headless --port 9989 2>&1 | prefix_output "server" "32" &
PIDS+=($!)

sleep 1

echo "Starting local manager..."
cargo run $CARGO_PROFILE -p manytris_game_manager --bin local_manager 2>&1 | prefix_output "manager" "34" &
PIDS+=($!)

for i in $(seq 1 "$NUM_BOTS"); do
    echo "Starting bot $i..."
    cargo run $CARGO_PROFILE -- bot --host localhost --port 9989 --headless 2>&1 | prefix_output "bot-$i" "33" &
    PIDS+=($!)
done

sleep 1

echo "Starting client..."
cargo run $CARGO_PROFILE -- client --manager-server http://localhost:3000 2>&1 | prefix_output "client" "35"
