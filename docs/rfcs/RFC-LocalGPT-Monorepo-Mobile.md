# RFC: LocalGPT Monorepo Restructure for Mobile

**Status:** Draft
**Author:** LocalGPT Team
**Date:** 2026-02-16
**Target version:** v0.3.0

---

## 1. Goal

Extract the shared Rust logic from the current `localgpt` crate into a `localgpt-core` library crate that can be consumed by the existing CLI/desktop/server targets **and** new iOS and Android apps via UniFFI-generated bindings. The restructure must be zero-regression for existing users — `cargo install localgpt` must continue to produce the same single binary.

---

## 2. Current State

Today the crate is a single `localgpt` package that compiles to one binary. Platform-specific code is gated behind `cfg` attributes and feature flags:

```
localgpt/                     # workspace root
├── Cargo.toml                # workspace: members = ["gen"]
├── src/
│   ├── main.rs               # binary entry point
│   ├── lib.rs                # re-exports everything
│   ├── agent/                # LLM providers, session, tools, sanitize, security
│   ├── memory/               # SQLite FTS5, sqlite-vec, fastembed, workspace
│   ├── heartbeat/            # runner.rs (HEARTBEAT.md poller)
│   ├── server/               # axum HTTP, WebSocket, telegram bot
│   ├── desktop/              # egui GUI (feature = "desktop")
│   ├── cli/                  # clap subcommands
│   ├── config/               # TOML config, env expansion, paths
│   ├── commands.rs           # slash commands shared by CLI + Telegram
│   ├── concurrency.rs        # TurnGate, workspace locks
│   ├── paths.rs              # XDG path resolution
│   ├── sandbox/              # Landlock, seccomp, Seatbelt
│   └── security/             # policy signing, audit chain
└── gen/                      # localgpt-gen subcrate (Bevy 3D)
```

Problems this causes for mobile:

- **`main.rs` pulls in daemonize, rustyline, clap** — none of which compile for iOS/Android.
- **`server/` depends on axum, teloxide** — unnecessary weight for a mobile library.
- **`sandbox/` uses Linux Landlock and macOS Seatbelt** — irrelevant on mobile (apps are already sandboxed by the OS).
- **`fastembed` → `ort` → ONNX Runtime** — needs special cross-compilation for mobile targets. Must be optional.
- **Agent is not `Send+Sync`** (noted in CLAUDE.md) — mobile UIs dispatch from the main thread and need async access.

---

## 3. Target State

```
localgpt/
├── Cargo.toml                        # workspace root
│
├── crates/
│   ├── core/                         # localgpt-core (THE shared library)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent/                # providers, session, tools, sanitize
│   │       ├── memory/               # index, workspace, watcher
│   │       ├── heartbeat/            # runner (no daemon, no daemonize)
│   │       ├── config/               # Config, Paths, env expansion
│   │       ├── concurrency.rs
│   │       └── security/             # policy, signing (no sandbox)
│   │
│   ├── cli/                          # localgpt-cli → `localgpt` binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── chat.rs
│   │       ├── ask.rs
│   │       ├── daemon.rs             # daemonize lives here
│   │       ├── commands.rs           # slash commands
│   │       └── desktop/              # egui (feature = "desktop")
│   │
│   ├── server/                       # localgpt-server (axum + telegram)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── http.rs
│   │       ├── telegram.rs
│   │       └── websocket.rs
│   │
│   ├── sandbox/                      # localgpt-sandbox (Linux/macOS only)
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── mobile/                       # localgpt-mobile (UniFFI bindings)
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs                # UniFFI exports
│   │   └── localgpt.udl              # interface definition
│   │
│   └── gen/                          # localgpt-gen (Bevy 3D, moved from gen/)
│       ├── Cargo.toml
│       └── src/
│
├── mobile/                           # native app projects (NOT Rust)
│   ├── ios/
│   │   ├── LocalGPT.xcodeproj
│   │   ├── LocalGPT/                 # SwiftUI sources
│   │   └── scripts/
│   │       └── build-rust.sh         # cargo build + uniffi-bindgen
│   └── android/
│       ├── build.gradle.kts
│       ├── app/src/main/             # Jetpack Compose sources
│       └── gradle/
│           └── rust.gradle           # cargo-ndk integration
│
├── CLAUDE.md
├── README.md
└── Cargo.lock                        # single lockfile for everything
```

---

## 4. Workspace Cargo.toml

```toml
[workspace]
members = [
    "crates/core",
    "crates/cli",
    "crates/server",
    "crates/sandbox",
    "crates/mobile",
    "crates/gen",
]
default-members = ["crates/cli"]
resolver = "3"

[workspace.package]
version = "0.3.0"
edition = "2024"
license = "Apache-2.0"
repository = "https://github.com/localgpt-app/localgpt"

[workspace.dependencies]
# Shared dependency versions — crates use `dep.workspace = true`
tokio = { version = "1.49", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
rusqlite = { version = "0.38", features = ["bundled", "functions", "vtab"] }
sqlite-vec = "0.1.7-alpha.2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.20", features = ["v4"] }
async-trait = "0.1"
reqwest = { version = "0.13", features = ["json", "stream"] }
thiserror = "2.0"
```

---

## 5. Crate Dependency Graph

```
                   ┌─────────────────────┐
                   │    localgpt-cli      │
                   │  (binary: localgpt)  │
                   └──────┬──────────────┘
                          │ depends on
            ┌─────────────┼─────────────────┐
            ▼             ▼                 ▼
  ┌──────────────┐ ┌──────────────┐  ┌──────────────┐
  │ localgpt-core│ │localgpt-server│  │localgpt-sandbox│
  │  (library)   │ │  (library)   │  │  (library)   │
  └──────┬───────┘ └──────┬───────┘  └──────────────┘
         │                │ depends on core
         │                ▼
         │         ┌──────────────┐
         │         │ localgpt-core│
         │         └──────────────┘
         │
         │ also consumed by:
         │
  ┌──────┴───────┐              ┌──────────────┐
  │localgpt-mobile│              │ localgpt-gen │
  │ (UniFFI lib) │              │  (Bevy 3D)   │
  └──────────────┘              └──────────────┘
```

Rule: **`localgpt-core` has zero platform-specific dependencies.** No `daemonize`, no `landlock`, no `seccompiler`, no `nix`, no `eframe`, no `axum`, no `teloxide`, no `rustyline`, no `clap`. It compiles cleanly for `aarch64-apple-ios` and `aarch64-linux-android` with no feature gates.

---

## 6. What Goes Where

### 6.1 `localgpt-core` — the mobile-safe shared library

Everything a consumer of LocalGPT's brain needs, nothing else:

| Module | Contents | Key Types |
|--------|----------|-----------|
| `agent/providers.rs` | `LLMProvider` trait, Anthropic, OpenAI, Ollama providers | `LLMProvider`, `ChatMessage`, `StreamChunk` |
| `agent/session.rs` | Conversation state, compaction | `Session`, `Usage` |
| `agent/tools.rs` | Tool trait + safe tools (memory_search, memory_get) | `Tool`, `ToolResult` |
| `agent/system_prompt.rs` | Context assembly from SOUL/MEMORY/HEARTBEAT | `build_system_prompt()` |
| `agent/sanitize.rs` | Injection detection, output sanitization | `sanitize_tool_output()` |
| `memory/` | MemoryManager, MemoryIndex, workspace init | `MemoryManager`, `SearchResult` |
| `heartbeat/` | HeartbeatRunner (timer logic, no daemon) | `HeartbeatRunner`, `HeartbeatStatus` |
| `config/` | Config, Paths, env expansion | `Config`, `Paths` |
| `concurrency.rs` | TurnGate, workspace locks | `TurnGate` |
| `security/` | Policy signing, audit chain (no sandbox) | `PolicyVerification` |

**Excluded from core** (stays in platform crates):

| Component | Reason | Destination |
|-----------|--------|-------------|
| `ClaudeCliProvider` | Spawns `claude` subprocess — no CLI on mobile | `crates/cli` |
| `sandbox/` | Landlock, seccomp, Seatbelt — mobile OS handles this | `crates/sandbox` |
| `server/` | axum, teloxide — mobile doesn't serve HTTP | `crates/server` |
| `desktop/` | egui — replaced by native UI on mobile | `crates/cli` |
| `cli/` | clap, rustyline, daemon — binary-specific | `crates/cli` |
| Dangerous tools (bash, write_file, edit_file) | Security risk on mobile — defer to v0.4 | `crates/cli` |

### 6.2 `localgpt-core` feature flags

```toml
[features]
default = ["embeddings-local"]

# Local embeddings via fastembed (ONNX). Desktop default.
# Requires ort cross-compilation for mobile — disable if not ready.
embeddings-local = ["fastembed"]

# GGUF embeddings via llama.cpp (optional, C++ compiler required)
embeddings-gguf = ["llama-cpp-2"]

# OpenAI API embeddings (no native deps, always works on mobile)
embeddings-openai = []    # uses reqwest, already in core

# Disable all embeddings — FTS5 keyword search only
embeddings-none = []
```

This lets mobile builds start with `default-features = false, features = ["embeddings-openai"]` to avoid the ONNX cross-compilation problem entirely, and upgrade to `embeddings-local` once that pipeline is proven.

### 6.3 `localgpt-mobile` — UniFFI binding layer

```toml
[package]
name = "localgpt-mobile"
version.workspace = true

[lib]
crate-type = ["staticlib", "cdylib"]   # .a for iOS, .so for Android
name = "localgpt_mobile"

[dependencies]
localgpt-core = { path = "../core", default-features = false, features = ["embeddings-openai"] }
uniffi = { version = "0.29", features = ["cli"] }

[build-dependencies]
uniffi = { version = "0.29", features = ["build"] }
```

The UDL file (`localgpt.udl`) or proc-macro annotations expose a **minimal surface** for v0.3:

```webidl
namespace localgpt {
    // Config
    Config load_config(string data_dir);
    void   save_config(Config config, string data_dir);

    // Provider management
    void   configure_provider(string provider, string api_key, string? base_url);
    sequence<string> list_providers();

    // Chat (single-turn for now; streaming in v0.4)
    string chat(string message, string? model);

    // Memory
    sequence<SearchResult> memory_search(string query, u32 max_results);
    string memory_get(string filename);

    // Workspace files
    string get_soul();
    void   set_soul(string content);
    string get_memory();
    string get_heartbeat();
    void   set_heartbeat(string content);
};
```

---

## 7. The `Send + Sync` Problem

The Agent struct contains a `MemoryManager` which holds an `Arc<Mutex<rusqlite::Connection>>`. This is already `Send` via the Mutex. However, some code paths hold references across await points.

**Fix (in this PR):**

1. Ensure `MemoryIndex.conn` stays as `Arc<Mutex<Connection>>` (already the case as of v0.1.1).
2. Wrap `Agent` access in `localgpt-core` behind an `AgentHandle`:

```rust
/// Thread-safe handle to an Agent. Mobile and server code use this.
pub struct AgentHandle {
    inner: Arc<Mutex<Agent>>,
}

impl AgentHandle {
    pub async fn chat(&self, message: &str) -> Result<String> {
        let mut agent = self.inner.lock().map_err(|e| anyhow!("{e}"))?;
        agent.chat(message).await
    }

    pub async fn memory_search(&self, query: &str, max: u32) -> Result<Vec<SearchResult>> {
        let agent = self.inner.lock().map_err(|e| anyhow!("{e}"))?;
        agent.memory.search(query, max).await
    }
}
```

The CLI and desktop continue using `Agent` directly for their single-threaded use cases. `AgentHandle` is the mobile and server interface.

---

## 8. Implementation Plan

### Phase 1: Extract `localgpt-core` (Week 1–2)

This is the critical-path work. Everything else depends on it.

**Step 1.1: Create crate skeleton** (Day 1)

```bash
mkdir -p crates/{core,cli,server,sandbox,mobile}
```

Create `crates/core/Cargo.toml` with workspace dependencies. Create `crates/core/src/lib.rs` re-exporting the modules.

**Step 1.2: Move modules** (Day 2–3)

Move these directories from `src/` → `crates/core/src/`:

- `agent/` (minus `ClaudeCliProvider`)
- `memory/`
- `heartbeat/`
- `config/`
- `concurrency.rs`
- `paths.rs`
- `security/` (minus sandbox)

For each moved module:
1. `git mv` to preserve history.
2. Update `use` paths — change `crate::` to `localgpt_core::` in consuming crates.
3. Ensure `pub` visibility on everything the CLI needs.

**Step 1.3: Create `crates/cli`** (Day 3–4)

- Move `src/main.rs` → `crates/cli/src/main.rs`
- Move `src/cli/` → `crates/cli/src/`
- Move `src/desktop/` → `crates/cli/src/desktop/`
- Move `src/commands.rs` → `crates/cli/src/commands.rs`
- Move `ClaudeCliProvider` → `crates/cli/src/claude_cli_provider.rs` and register it as a provider via a trait extension.
- `Cargo.toml` depends on `localgpt-core`, `clap`, `rustyline`, `daemonize`, `eframe`, etc.
- Binary name stays `localgpt` via `[[bin]] name = "localgpt"`.

**Step 1.4: Create `crates/server`** (Day 4)

- Move `src/server/` → `crates/server/src/`
- Depends on `localgpt-core`, `axum`, `tower-http`, `teloxide`.
- `crates/cli` depends on `localgpt-server` and starts it in daemon mode.

**Step 1.5: Create `crates/sandbox`** (Day 4)

- Move `src/sandbox/` → `crates/sandbox/src/`
- Depends on `landlock`, `seccompiler`, `nix` (all behind `cfg(unix)` / `cfg(target_os = "linux")`).
- `crates/cli` depends on `localgpt-sandbox` and wires it into tool execution.

**Step 1.6: Validation gate** (Day 5)

Before proceeding, all of the following must pass:

```bash
# Existing functionality unchanged
cargo build -p localgpt-cli --release
cargo test --workspace
cargo clippy --workspace

# Core compiles for mobile targets (no linking yet, just type-checking)
cargo check -p localgpt-core --target aarch64-apple-ios
cargo check -p localgpt-core --target aarch64-linux-android

# Headless build still works
cargo build -p localgpt-cli --no-default-features --release
```

The iOS/Android `cargo check` is the key new gate. If `localgpt-core` passes `check` for both mobile targets, the architecture is sound. Any failures here are blockers — they indicate platform-specific code leaked into core.

**Step 1.7: Move `gen/` into `crates/gen/`** (Day 5)

Update workspace members. Adjust the `localgpt` dependency path. Trivial.

### Phase 2: UniFFI Bindings (Week 3)

**Step 2.1: Set up `crates/mobile`** (Day 1)

- Add UniFFI dependency.
- Write `localgpt.udl` with the minimal surface from §6.3.
- Implement the exported functions wrapping `localgpt-core` types.
- Generate Swift and Kotlin bindings: `cargo run -p uniffi-bindgen generate crates/mobile/src/localgpt.udl --language swift --language kotlin`.

**Step 2.2: iOS build script** (Day 2)

`mobile/ios/scripts/build-rust.sh`:

```bash
#!/bin/bash
set -euo pipefail

TARGETS="aarch64-apple-ios aarch64-apple-ios-sim"  # device + simulator
PROFILE="${1:-release}"

for TARGET in $TARGETS; do
    cargo build -p localgpt-mobile --target "$TARGET" --profile "$PROFILE"
done

# Create universal binary for simulator (arm64 + x86_64 if needed)
# Generate Swift bindings
cargo run -p uniffi-bindgen generate \
    crates/mobile/src/localgpt.udl \
    --language swift \
    --out-dir mobile/ios/LocalGPT/Generated/

# Create XCFramework
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/${PROFILE}/liblocalgpt_mobile.a \
    -headers mobile/ios/LocalGPT/Generated/ \
    -library target/aarch64-apple-ios-sim/${PROFILE}/liblocalgpt_mobile.a \
    -headers mobile/ios/LocalGPT/Generated/ \
    -output mobile/ios/LocalGPTCore.xcframework
```

**Step 2.3: Android build script** (Day 2)

`mobile/android/gradle/rust.gradle`:

```groovy
// Uses cargo-ndk plugin
cargoNdk {
    module = "../../crates/mobile"
    targets = ["arm64-v8a", "armeabi-v7a", "x86_64"]  // device + emulators
    librariesNames = ["liblocalgpt_mobile.so"]
}

// Copy generated Kotlin bindings
task generateUniFFIBindings(type: Exec) {
    commandLine "cargo", "run", "-p", "uniffi-bindgen", "generate",
        "../../crates/mobile/src/localgpt.udl",
        "--language", "kotlin",
        "--out-dir", "app/src/main/java/app/localgpt/core/"
}
```

**Step 2.4: Smoke test** (Day 3)

Write a minimal iOS Swift test and Android Kotlin test that:

1. Calls `load_config()` with a temp directory.
2. Calls `set_soul("You are a helpful assistant.")`.
3. Calls `memory_search("test", 5)` — returns empty, but proves SQLite + FTS5 work.
4. Calls `list_providers()` — returns the available providers.

This does **not** require a working LLM — it validates the Rust core → UniFFI → native bridge.

### Phase 3: CI Pipeline (Week 3–4)

**GitHub Actions matrix:**

```yaml
jobs:
  # Existing
  test-linux:
    runs-on: ubuntu-latest
    steps:
      - cargo test --workspace
      - cargo clippy --workspace

  test-macos:
    runs-on: macos-latest
    steps:
      - cargo test --workspace

  # NEW: Mobile target checks
  check-ios:
    runs-on: macos-latest
    steps:
      - rustup target add aarch64-apple-ios aarch64-apple-ios-sim
      - cargo check -p localgpt-core --target aarch64-apple-ios
      - cargo check -p localgpt-mobile --target aarch64-apple-ios
      # Optionally: xcodebuild the iOS project

  check-android:
    runs-on: ubuntu-latest
    steps:
      - rustup target add aarch64-linux-android
      - cargo install cargo-ndk
      - cargo ndk -t arm64-v8a check -p localgpt-core
      - cargo ndk -t arm64-v8a check -p localgpt-mobile

  # Release binary (unchanged behavior)
  release:
    steps:
      - cargo build -p localgpt-cli --release
      # produces the same `localgpt` binary as before
```

---

## 9. Migration Checklist for Existing Contributors

After this lands, contributors need to know:

| Before | After |
|--------|-------|
| `cargo build` | `cargo build` (builds CLI by default, same as before) |
| `cargo test` | `cargo test --workspace` (tests all crates) |
| Edit `src/agent/providers.rs` | Edit `crates/core/src/agent/providers.rs` |
| `use crate::memory::MemoryManager` | `use localgpt_core::memory::MemoryManager` (in cli/server) |
| `cargo install localgpt` | `cargo install localgpt-cli` (or alias: `cargo install --path crates/cli`) |
| Feature flag `--no-default-features` | Same — applies to cli crate, disables egui |

The crates.io package name changes from `localgpt` to `localgpt-cli`. Add a `localgpt` shim crate that just depends on and re-exports `localgpt-cli` if you want to preserve `cargo install localgpt`.

---

## 10. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `fastembed`/`ort` won't cross-compile for iOS | High | Medium | Feature-gate embeddings. Ship mobile with `embeddings-openai` first. Add local embeddings in v0.4 after proving the ONNX pipeline. |
| Breaking `cargo install localgpt` for existing users | Medium | High | Publish a `localgpt` shim crate on crates.io that depends on `localgpt-cli`. Or keep the binary name `localgpt` regardless of crate name. |
| `Agent` Send+Sync issues surface in edge cases | Medium | Medium | `AgentHandle` wraps in `Arc<Mutex<>>`. Add `static_assertions::assert_impl_all!(AgentHandle: Send, Sync);` compile-time check. |
| UniFFI version compatibility with Swift/Kotlin codegen | Low | Medium | Pin `uniffi` version in workspace. Test generated code in CI on every PR. |
| Repo size grows with Xcode/Gradle files | Low | Low | `.gitignore` build artifacts aggressively. No model files in repo. |

---

## 11. What This RFC Does NOT Cover (Deferred to v0.4+)

- **On-device LLM inference** (llama.cpp integration for offline models)
- **Streaming chat over UniFFI** (requires callback/async patterns)
- **Dangerous tool execution on mobile** (bash, write_file)
- **Mobile UI implementation** (SwiftUI / Jetpack Compose — separate RFC)
- **App Store / Play Store packaging and distribution**
- **HEARTBEAT.md background execution** (requires iOS BGProcessingTask / Android WorkManager — separate spec)
- **Push notification integration** for heartbeat alerts
- **Model download and management UI**

---

## 12. Definition of Done

Phase 1 is complete when:

- [ ] `cargo build -p localgpt-cli --release` produces the same binary as today
- [ ] `cargo test --workspace` passes with zero regressions
- [ ] `cargo check -p localgpt-core --target aarch64-apple-ios` succeeds
- [ ] `cargo check -p localgpt-core --target aarch64-linux-android` succeeds
- [ ] `localgpt-core` has zero dependencies on: clap, rustyline, daemonize, eframe, axum, teloxide, landlock, seccompiler, nix
- [ ] CI pipeline runs mobile target checks on every PR

Phase 2 is complete when:

- [ ] `uniffi-bindgen` generates valid Swift and Kotlin bindings
- [ ] iOS XCFramework builds from `build-rust.sh`
- [ ] Android `.so` builds from `cargo-ndk`
- [ ] Smoke tests pass on both iOS Simulator and Android Emulator
- [ ] `memory_search` returns results over the UniFFI bridge
