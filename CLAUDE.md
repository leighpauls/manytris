# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Manytris is a Rust-based multiplayer Tetris game built with the Bevy game engine. It supports standalone single-player, multiplayer server/client architecture, AI bot players with GPU-accelerated move computation (Vulkan & Metal), WebAssembly compilation for web clients, and headless server mode for containerized deployments.

## Build Commands

```bash
# Build and run standalone (single-player)
cargo run

# Build and run headless server
cargo run -- server --headless --port 9989

# Build and run client connecting to server
cargo run -- client

# Build and run AI bot against a server
cargo run -- bot --host localhost --port 9989

# Build with GPU bot support (Vulkan)
cargo build --release --features bot_vulkan

# Build for WebAssembly
cargo build --target wasm32-unknown-unknown --release

# Format code
cargo fmt

# Run clippy
cargo clippy
```

## Testing Multiplayer Changes

Use `run_local.sh` to verify changes to multiplayer features. It starts a headless server, local game manager, a bot client, and a human client all together:

```bash
./run_local.sh
```

This launches (in order): a headless game server on port 9989, the local manager on port 3000, a headless bot, and a graphical client connected to the manager. Ctrl-C cleans up all processes.

## Docker Deployment

```bash
cd docker
make game_runtime_prod    # Build production game container
make manager_prod         # Build production manager container
make publish_prod         # Push to registry
```

## Architecture

### Workspace Structure (8 crates)

- **manytris** (root): CLI entry point, parses `ExecCommand` (server/client/bot)
- **manytris_core**: Core game logic - field, game state, shapes, tetrominoes (no rendering)
- **manytris_bevy**: Bevy ECS implementation - rendering, input, networking, plugins
- **manytris_bot**: CPU-based AI with game tree search
- **manytris_bot_vulkan**: GPU-accelerated bot using Vulkan compute shaders
- **manytris_bot_metal**: GPU-accelerated bot using Metal (macOS)
- **manytris_bot_demo**: Bot testing/demo application
- **manytris_game_manager**: Kubernetes/Docker orchestration REST API
- **manytris_game_manager_proto**: Shared protocol definitions for manager

### Execution Modes

Configured via CLI subcommands in `src/main.rs`:
- **StandAlone**: Single player, generates own shapes locally
- **Server**: Hosts game on WebSocket, generates shapes, manages multiple clients
- **Client (Human)**: Connects to server, keyboard input
- **Client (Bot)**: Connects to server, AI-generated inputs

### Key Data Flow

1. `GameState` (manytris_core) receives `TickMutation` events (shift, rotate, drop, hold)
2. Returns `TickResult` (lock piece, clear lines, add garbage)
3. In multiplayer, `NetMessage` serialized via MessagePack over WebSocket
4. Server broadcasts game state updates to all connected clients

### Bevy Plugin Architecture (manytris_bevy)

Key plugins in `plugins.rs`:
- `StatesPlugin`: Manages `PlayingState` (MainMenu → Connecting → Playing → Restarting)
- `root.rs`: Core tick/lock timers, pause handling
- `game_container.rs`: Manages multiple local game instances (tiled 4x3 grid)
- `input.rs`: Keyboard input with key repeat handling
- `net_client.rs` / `net_listener.rs`: WebSocket client/server networking
- `field_blocks.rs` / `window_blocks.rs`: Field and block rendering
- `bot_input.rs`: AI move generation (feature-gated)

### Networking

- WebSocket-based (ewebsock for WASM-compatible client, tungstenite for server)
- MessagePack serialization (rmp-serde)
- `ClientControlEvent` / `ServerControlEvent` in `net_client.rs` / `net_listener.rs`
- Game manager REST API (Axum) at port 3000 for server orchestration

### GPU Bot Architecture

Both Vulkan and Metal implementations use compute shaders for parallel move evaluation:
- `MoveResultScore`: Evaluates game_over, lines_cleared, height, covered_blocks
- Exponential search through move candidates with configurable depth
- `--bot-millis` controls decision period

## Feature Flags

```toml
bot          # Enable CPU bot
bot_vulkan   # Enable Vulkan GPU bot (includes bot)
stats_server # Enable Axum stats HTTP server on port 9990
```

## Key Files

- `manytris_core/src/game_state.rs`: Main game state machine
- `manytris_core/src/field.rs`: 2D field with occupied blocks
- `manytris_bevy/src/plugins.rs`: Plugin orchestration
- `manytris_bevy/src/states.rs`: `PlayingState` and `ExecType` resources
- `manytris_bevy/src/root.rs`: Core game tick/lock logic
- `src/main.rs`: CLI parsing and app initialization
