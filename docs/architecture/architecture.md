# LocalGPT Architecture

## Workspace Structure

LocalGPT is organized as a Cargo workspace with 10 crates:

```
crates/
├── core/        # localgpt-core — shared library (agent, memory, config, security)
├── cli/         # localgpt — binary with clap CLI, desktop GUI, dangerous tools
├── server/      # localgpt-server — HTTP/WS API, Telegram bot, BridgeManager
├── sandbox/     # localgpt-sandbox — Landlock/Seatbelt process sandboxing
├── mobile-ffi/  # localgpt-mobile-ffi — UniFFI bindings for iOS/Android
├── gen/         # localgpt-gen — Bevy 3D scene generation binary
└── bridge/      # localgpt-bridge — secure IPC protocol for bridge daemons

bridges/         # Standalone bridge binaries
├── telegram/    # localgpt-bridge-telegram — Telegram bot daemon
├── discord/     # localgpt-bridge-discord — Discord bot daemon
└── whatsapp/    # localgpt-bridge-whatsapp — WhatsApp bridge daemon

apps/            # Native mobile app projects (iOS, Android)
```

## Dependency Graph

```
                        ┌─────────────────┐
                        │ localgpt-core   │  (no internal deps)
                        └────────┬────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ localgpt-bridge │    │ localgpt-sandbox│    │ localgpt-gen    │
│ (no internal    │    │                 │    │                 │
│  deps)          │    └────────┬────────┘    └─────────────────┘
└────────┬────────┘             │
         │                      │
         ▼                      │
┌─────────────────┐             │
│ localgpt-server │◄────────────┘
│ (core + bridge) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ localgpt (CLI)  │
│ (core + server  │
│  + sandbox)     │
└─────────────────┘

Bridge daemons (all depend on core + bridge):
┌─────────────────────────┐
│ localgpt-bridge-telegram│
│ localgpt-bridge-discord │
│ localgpt-bridge-whatsapp│
└─────────────────────────┘

Mobile (core with local embeddings):
┌─────────────────────────┐
│ localgpt-mobile-ffi     │
│ (default-features=false,│
│  features=local+sqlite) │
└─────────────────────────┘
```

## Crate Summary

| Crate | Type | Dependencies | Purpose |
|-------|------|--------------|---------|
| `localgpt-core` | lib | None | Agent, memory, config, security (mobile-compatible) |
| `localgpt-bridge` | lib | None | IPC protocol for bridge daemons |
| `localgpt-sandbox` | lib | core | Landlock/Seatbelt process isolation |
| `localgpt-server` | lib | core, bridge | HTTP server, Telegram bot, BridgeManager |
| `localgpt` | bin | core, server, sandbox | CLI binary with all features |
| `localgpt-gen` | bin | core | 3D scene generation with Bevy |
| `localgpt-mobile-ffi` | lib+bin | core (minimal) | UniFFI bindings for iOS/Android |
| `localgpt-bridge-telegram` | bin | core, bridge | Telegram bot daemon |
| `localgpt-bridge-discord` | bin | core, bridge | Discord bot daemon |
| `localgpt-bridge-whatsapp` | bin | core, bridge | WhatsApp bridge daemon |

## Detailed Crate Descriptions

### Core Libraries

#### `localgpt-core`
Foundation library with zero platform-specific dependencies. Contains:
- **Agent**: LLM provider abstraction (OpenAI, Anthropic, Ollama, Claude CLI, GLM)
- **Memory**: SQLite FTS5 + markdown files + embeddings
- **Config**: TOML configuration with workspace resolution
- **Security**: HMAC signing, policy verification, audit logging
- **Heartbeat**: Autonomous task runner
- **Session**: Conversation management with compaction

#### `localgpt-bridge`
IPC protocol library for daemon-to-bridge communication:
- **tarpc-based RPC**: Async service definitions
- **Peer identity**: Unix socket UID/GID verification
- **Credential exchange**: Secure credential retrieval
- **Platform support**: Unix sockets (Unix), named pipes (Windows)

### Desktop Libraries

#### `localgpt-server`
HTTP/WebSocket server and daemon services:
- **Axum HTTP server**: REST API + embedded Web UI
- **Telegram bot**: teloxide-based bot with streaming
- **BridgeManager**: Unix socket server for bridge daemons
- **WebSocket handler**: Real-time chat streaming

#### `localgpt-sandbox`
Process isolation for shell command execution:
- **Linux**: Landlock + seccomp filters
- **macOS**: Seatbelt (sandbox-init) profiles
- **Windows**: Restricted token creation
- **Graceful degradation**: Falls back to warning on unsupported systems

### Binaries

#### `localgpt` (CLI)
Primary user-facing binary:
- **Commands**: chat, ask, daemon, memory, config, bridge, gen, auth, init
- **Features**: Desktop GUI (eframe), daemon mode, dangerous tools
- **Target**: End users on desktop systems

#### `localgpt-gen`
3D scene generation with Bevy engine:
- **Tools**: spawn_entity, modify_entity, set_material, set_ambience
- **Audio**: Procedural environmental sounds via FunDSP
- **Export**: glTF/GLB scene export

### Mobile

#### `localgpt-mobile-ffi`
UniFFI bindings for iOS/Android:
- **Library**: `staticlib` + `cdylib` for native linking
- **Binary**: `uniffi-bindgen` for generating Swift/Kotlin bindings
- **Features**: Uses `embeddings-local` + `sqlite-vec` (local embeddings work on mobile)
- **Runtime**: Embedded tokio runtime

```
localgpt-mobile-ffi
    │
    ├── lib.rs          # UniFFI proc-macro exports
    │   └── LocalGPTClient → AgentHandle (Arc<Mutex<Agent>>)
    │
    ├── uniffi-bindgen.rs  # Binary for generating bindings
    │
    └── Generated/
        ├── Swift (iOS/macOS) # apps/apple/Generated/
        └── Kotlin (Android) # apps/android/Generated/
```

### Bridge Daemons

#### `localgpt-bridge-telegram`
Standalone Telegram bot daemon:
- Connects to LocalGPT daemon via Unix socket
- Supports streaming responses with edit updates
- 6-digit pairing auth flow

#### `localgpt-bridge-discord`
Standalone Discord bot daemon:
- serenity-based Discord gateway client
- Same IPC protocol as Telegram bridge

#### `localgpt-bridge-whatsapp`
WhatsApp bridge daemon:
- Uses baileys (Node.js) via embedded process
- Axum server for baileys webhook events

## Critical Design Rules

1. **`localgpt-core` must have zero platform-specific dependencies**
   - Must compile cleanly for `aarch64-apple-ios` and `aarch64-linux-android`
   - No `clap`, `eframe`, `axum`, `teloxide`, `landlock`, `nix`, `tarpc`, etc.
   - No dependency on `localgpt-bridge`

2. **Mobile uses local embeddings**
   - `default-features = false, features = ["embeddings-local", "sqlite-vec"]`
   - Excludes `claude-cli` (subprocess execution not available on mobile)
   - fastembed (ONNX) compiles successfully for iOS/Android

3. **Bridge management lives in server**
   - `BridgeManager` is in `localgpt-server`, not `localgpt-core`
   - Desktop-only functionality (Unix sockets, peer identity)

---

## Relationship with OpenClaw

LocalGPT is a **fresh Rust implementation** inspired by OpenClaw's architecture, stripped down to essential components for local-only AI interaction.

### Design Philosophy

| Aspect | OpenClaw | LocalGPT |
|--------|----------|----------|
| Language | TypeScript | Rust |
| LOC (estimated) | ~200k | ~3k |
| Remote channels | 20+ (Telegram, Discord, Slack, etc.) | 0 |
| Dependencies | 500+ npm packages | ~30 crates |
| Memory system | Markdown + SQLite | Same approach |
| Heartbeat | HEARTBEAT.md driven | Same approach |
| Startup time | ~2-3s | <100ms |

### Borrowed Concepts

The following concepts were directly inspired by OpenClaw:

1. **Memory System Design**
   - `MEMORY.md` as curated long-term knowledge
   - `memory/*.md` for daily append-only logs
   - `HEARTBEAT.md` for pending tasks/reminders
   - SQLite for indexing with FTS5

2. **Heartbeat Runner**
   - Periodic autonomous execution
   - Active hours configuration
   - Simple prompt-based task checking

3. **Session Management**
   - Context compaction with summarization
   - Pre-compaction memory flush prompts
   - JSONL transcript storage

4. **Tool System**
   - Bash execution
   - File operations (read, write, edit)
   - Memory search and append

### Key Differences

1. **Bridge-Based Channels**: OpenClaw has built-in remote channels; LocalGPT uses standalone bridge binaries (Telegram, Discord, WhatsApp) that connect to the daemon via secure IPC
2. **No Plugin System**: OpenClaw has an extension architecture; LocalGPT is monolithic
3. **Bridge Architecture**: OpenClaw's gateway handles multi-channel routing; LocalGPT uses a bridge daemon model with encrypted credential exchange
4. **Web UI**: OpenClaw has a full web-based chat UI; LocalGPT has an embedded web UI served by the daemon, plus a desktop GUI (egui)

---

## Current Implementation Status

### Completed (MVP)

- [x] CLI interface (`chat`, `ask`, `daemon`, `memory`, `config` commands)
- [x] LLM providers (OpenAI, Anthropic, Ollama)
- [x] Memory files (MEMORY.md, memory/*.md, HEARTBEAT.md)
- [x] Memory search (FTS5)
- [x] Daemon mode with heartbeat
- [x] Basic tool set (bash, file ops, memory ops, web fetch)
- [x] TOML configuration
- [x] HTTP server with REST API
- [x] Session management with compaction

### Gaps and Remaining Work

#### High Priority

1. **Vector Search / Embeddings**
   - Currently: FTS5 keyword search only
   - Needed: Semantic search with embeddings
   - Options: `sqlite-vec`, local ONNX embeddings, or API-based
   - Effort: Medium

2. **Thread Safety for Agent**
   - Currently: Agent contains SQLite connection (not `Send`/`Sync`)
   - Impact: HTTP server creates new agent per request (no session persistence)
   - Fix: Wrap connection in `Arc<Mutex<>>` or use connection pooling
   - Effort: Medium

3. **Streaming Responses**
   - Currently: Full response only
   - Needed: SSE/WebSocket streaming for real-time output
   - WebSocket handler is stubbed but not connected
   - Effort: Medium

4. **Proper Token Counting**
   - Currently: Rough estimate (4 chars = 1 token)
   - Needed: Use `tiktoken-rs` properly per model
   - Effort: Low

#### Medium Priority

5. **Background Daemonization**
   - Currently: `--foreground` mode only
   - Needed: Proper Unix daemon with `fork()`
   - Effort: Low

6. **Memory Flush Prompts**
   - Currently: Pre-compaction flush is basic
   - Needed: Better integration with daily logs
   - Effort: Low

7. **Session Resume in HTTP**
   - Currently: Each HTTP request is stateless
   - Needed: Session ID tracking across requests
   - Effort: Medium

8. **Error Handling**
   - Currently: Basic `anyhow` errors
   - Needed: Structured error types, better user messages
   - Effort: Medium

#### Low Priority / Nice to Have

9. **Local LLM Inference**
   - Add `llama-cpp-rs` for fully offline operation
   - Effort: High

10. **TUI Interface**
    - Rich terminal UI with `ratatui`
    - Effort: High

11. **Shell Completions**
    - Bash/Zsh/Fish completion scripts
    - Effort: Low

12. **Systemd/Launchd Integration**
    - Service files for auto-start
    - Effort: Low

13. **Metrics/Observability**
    - Token usage tracking
    - Request latency metrics
    - Effort: Medium

14. **Multi-Session Support**
    - Multiple concurrent chat sessions
    - Session listing and management
    - Effort: Medium

---

## File Structure Reference

```
~/.localgpt/
├── config.toml              # Main configuration
├── workspace/
│   ├── MEMORY.md            # Curated long-term knowledge
│   ├── HEARTBEAT.md         # Pending tasks/reminders
│   └── memory/
│       ├── 2024-01-15.md    # Daily append-only logs
│       └── ...
├── sessions/
│   └── <session_id>.jsonl   # Conversation transcripts
├── memory.sqlite            # FTS index
└── logs/
    └── agent.log            # Operation logs
```

---

## Contributing

When working on LocalGPT:

1. Keep the codebase small and focused
2. Prefer simplicity over features
3. Maintain Rust idioms and safety
4. Test with `cargo test`
5. Format with `cargo fmt`
6. Lint with `cargo clippy`
