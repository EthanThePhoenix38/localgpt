# RFC: XDG Base Directory Specification Compliance

**Status:** Draft
**Author:** Yi
**Created:** 2026-02-13
**Supersedes:** Monolithic `~/.localgpt/` layout
**Depends on:** RFC-LocalGPT-Security-Policy (for security file classification)

---

## 1. Summary

Migrate LocalGPT from a monolithic `~/.localgpt/` directory to an XDG Base Directory Specification (v0.8) compliant layout. Every file is classified into one of four XDG categories — config, data, state, or cache — based on its semantic purpose. The migration is automatic, backwards-compatible, and provides env var overrides at every level.

---

## 2. Motivation

The current layout stores everything under `~/.localgpt/`:

```
~/.localgpt/
├── config.toml
├── .device_key
├── .security_audit.jsonl
├── agents/main/sessions/
├── workspace/
│   ├── SOUL.md, MEMORY.md, HEARTBEAT.md, LocalGPT.md, ...
│   ├── .localgpt_manifest.json
│   ├── memory.sqlite
│   ├── memory/YYYY-MM-DD.md
│   ├── knowledge/
│   └── skills/
├── logs/
└── daemon.pid
```

This mirrors OpenClaw's `~/.openclaw/` monolithic approach. It works, but it fails users in concrete ways:

**Problem 1: No selective backup.** Users cannot back up their authored workspace files without also backing up multi-gigabyte session transcripts and regenerable search indexes. Everything is mixed together.

**Problem 2: No cache isolation.** The SQLite search index and embedding cache live alongside source markdown files. Users cannot symlink cache to a fast drive, place it on tmpfs, or delete it to reclaim space without risking their data.

**Problem 3: Config not Git-trackable.** Users who manage dotfiles via Git (a common pattern in LocalGPT's target audience) cannot cleanly include `config.toml` without pulling in the entire `~/.localgpt/` tree.

**Problem 4: No privilege separation.** The device key, audit log, config, workspace, sessions, and cache all live under one directory with uniform permissions. XDG's directory separation provides natural permission boundaries.

**Problem 5: Ecosystem friction.** Linux distributions, NixOS, and system tools increasingly expect XDG compliance. Users who set `XDG_CONFIG_HOME=/custom/path` expect all well-behaved tools to respect it. Hardcoded `~/.localgpt/` ignores their preference.

**Problem 6: OpenClaw learned this the hard way.** OpenClaw has migrated directory names multiple times (`~/.clawdbot/` → `~/.moltbot/` → `~/.openclaw/`), accumulating legacy detection code each time. Getting the layout right from LocalGPT's early days avoids compounding migration debt.

---

## 3. XDG Base Directory Specification (v0.8)

The spec defines seven environment variables. LocalGPT uses five:

| Variable | Default | Purpose | LocalGPT uses |
|---|---|---|---|
| `XDG_CONFIG_HOME` | `$HOME/.config` | User-edited settings | ✅ |
| `XDG_DATA_HOME` | `$HOME/.local/share` | Important, portable user data | ✅ |
| `XDG_STATE_HOME` | `$HOME/.local/state` | Persistent but non-portable state | ✅ |
| `XDG_CACHE_HOME` | `$HOME/.cache` | Regenerable, deletable data | ✅ |
| `XDG_RUNTIME_DIR` | *(no default)* | Sockets, PIDs, ephemeral runtime | ✅ |
| `XDG_DATA_DIRS` | `/usr/local/share/:/usr/share/` | System-wide data search paths | — |
| `XDG_CONFIG_DIRS` | `/etc/xdg` | System-wide config search paths | — |

Key behavioral rules from the spec:

- All paths **must be absolute**. Relative paths must be ignored.
- Applications **must create directories with mode 0700** when they don't exist.
- When variables are **not set or empty**, applications must use the defined defaults.
- `XDG_RUNTIME_DIR` must be owned by the user, mode 0700, created at login, removed at full logout.

---

## 4. Classification of Every LocalGPT File

### 4.1 Decision Framework

Each file is classified by answering three questions:

1. **Does the user directly edit this?** → Config
2. **Is this important, portable content the user authored or curates?** → Data
3. **Does this persist across restarts but isn't portable?** → State
4. **Can this be deleted and regenerated without data loss?** → Cache

### 4.2 File Classification Table

| File | Current Location | XDG Category | Rationale |
|---|---|---|---|
| `config.toml` | `~/.localgpt/config.toml` | **Config** | User-edited settings, version-controllable |
| `SOUL.md` | `~/.localgpt/workspace/` | **Data** | User-authored identity file |
| `MEMORY.md` | `~/.localgpt/workspace/` | **Data** | User-curated knowledge |
| `HEARTBEAT.md` | `~/.localgpt/workspace/` | **Data** | User-defined autonomous tasks |
| `LocalGPT.md` | `~/.localgpt/workspace/` | **Data** | User-authored persistent instructions |
| `USER.md` | `~/.localgpt/workspace/` | **Data** | User profile (OpenClaw compat) |
| `IDENTITY.md` | `~/.localgpt/workspace/` | **Data** | Agent identity (OpenClaw compat) |
| `TOOLS.md` | `~/.localgpt/workspace/` | **Data** | Tool notes (OpenClaw compat) |
| `AGENTS.md` | `~/.localgpt/workspace/` | **Data** | Operating instructions (OpenClaw compat) |
| `.localgpt.manifest.json` | `~/.localgpt/workspace/` | **Data** | Travels with LocalGPT.md as integrity pair |
| `memory/YYYY-MM-DD.md` | `~/.localgpt/workspace/memory/` | **Data** | Daily logs, user-valuable history |
| `knowledge/**` | `~/.localgpt/workspace/knowledge/` | **Data** | User-provided knowledge base |
| `skills/**` | `~/.localgpt/workspace/skills/` | **Data** | Custom skill definitions |
| `.gitignore` (workspace) | `~/.localgpt/workspace/.gitignore` | **Data** | Part of workspace |
| `localgpt.device.key` | `~/.localgpt/.device_key` | **Data** | Cryptographic key, must not be lost. Outside workspace subtree |
| `sessions.json` | `~/.localgpt/agents/main/sessions/` | **State** | Session metadata, device-specific |
| `*.jsonl` (transcripts) | `~/.localgpt/agents/main/sessions/` | **State** | Conversation history, not portable |
| `localgpt.audit.jsonl` | `~/.localgpt/.security_audit.jsonl` | **State** | Action history, device-specific |
| `logs/**` | `~/.localgpt/logs/` | **State** | Application logs |
| `memory.sqlite` | `~/.localgpt/workspace/memory.sqlite` | **Cache** | FTS5 + vector index, fully regenerable from markdown |
| Embedding model files | `~/.localgpt/cache/embeddings/` | **Cache** | Downloaded models, re-downloadable |
| `daemon.pid` | `~/.localgpt/daemon.pid` | **Runtime** | PID lock, ephemeral |
| Unix socket (future) | — | **Runtime** | IPC socket, ephemeral |

### 4.3 Non-Obvious Classification Decisions

**`localgpt.device.key` → Data, not Config.**
The user never edits this file. It's machine-generated and critical — losing it invalidates all signatures. Analogous to GPG private keys, which live in `XDG_DATA_HOME/gnupg/`. It must reside outside the workspace subtree (agent sandbox boundary) but within the data directory.

**`memory.sqlite` → Cache, not Data.**
This is the most impactful reclassification. The SQLite database containing FTS5 indexes, chunked content, and vector embeddings is **fully regenerable** from the source markdown files. Users should be able to `rm -rf` the cache directory and have everything rebuilt on next startup. Currently it lives inside the workspace, which makes workspace backups unnecessarily large and slow.

**Session transcripts → State, not Data.**
The spec lists "actions history" as the canonical state example. Transcripts are device-specific conversation logs that persist across restarts but aren't portable between machines. A user migrating to a new device would start fresh sessions.

**`localgpt.audit.jsonl` → State, not Data.**
Same reasoning — it's an action history, device-specific, persistent but not portable. Placing it in state (separate from data) adds a defense-in-depth layer: the agent's workspace-scoped tools cannot reach state directory paths.

**Daily logs (`memory/YYYY-MM-DD.md`) → Data, not State.**
Although the spec lists "recently used files" as state, daily logs are user-valuable content that the user may want to back up, search, and reference long-term. They're semantically equivalent to journal entries, not UI layout state.

---

## 5. Target Layout

### 5.1 Linux (XDG Native)

```
~/.config/localgpt/                       # XDG_CONFIG_HOME/localgpt/
└── config.toml                           # User-edited settings

~/.local/share/localgpt/                  # XDG_DATA_HOME/localgpt/
├── localgpt.device.key                   # HMAC signing key (0600)
└── workspace/                            # Memory workspace
    ├── SOUL.md
    ├── MEMORY.md
    ├── HEARTBEAT.md
    ├── LocalGPT.md
    ├── USER.md                           # OpenClaw compat
    ├── IDENTITY.md                       # OpenClaw compat
    ├── TOOLS.md                          # OpenClaw compat
    ├── AGENTS.md                         # OpenClaw compat
    ├── .localgpt.manifest.json         # Dot prefix: hidden in workspace ls
    ├── .gitignore
    ├── memory/
    │   └── YYYY-MM-DD.md
    ├── knowledge/
    └── skills/

~/.local/state/localgpt/                  # XDG_STATE_HOME/localgpt/
├── agents/
│   └── main/
│       └── sessions/
│           ├── sessions.json
│           └── <sessionId>.jsonl
├── localgpt.audit.jsonl
└── logs/
    └── *.log

~/.cache/localgpt/                        # XDG_CACHE_HOME/localgpt/
├── memory.sqlite                         # FTS5 + vector search index
├── memory.sqlite-wal
├── memory.sqlite-shm
└── embeddings/                           # Downloaded embedding models
    └── BAAI--bge-small-en-v1.5/

/run/user/$UID/localgpt/                  # XDG_RUNTIME_DIR/localgpt/
├── daemon.pid
└── localgpt.sock                         # Future: IPC socket
```

### 5.2 macOS

Use XDG-style paths (via `etcetera` crate's `choose_base_strategy()`), not Apple-native `~/Library/` paths. Rationale: LocalGPT's audience is CLI-oriented developers who expect `~/.config/` paths. This follows the precedent set by bat, fd, eza, alacritty, and most Rust CLI tools on macOS.

```
~/.config/localgpt/config.toml
~/.local/share/localgpt/workspace/...
~/.local/share/localgpt/localgpt.device.key
~/.local/state/localgpt/agents/...
~/.cache/localgpt/memory.sqlite
```

No `XDG_RUNTIME_DIR` on macOS. Fall back to `$TMPDIR/localgpt-$UID/` (see §7.3).

### 5.3 Windows

Use platform-native paths via `etcetera` or `directories` crate:

```
%APPDATA%\localgpt\config.toml
%APPDATA%\localgpt\workspace\...
%APPDATA%\localgpt\localgpt.device.key
%LOCALAPPDATA%\localgpt\state\agents\...
%LOCALAPPDATA%\localgpt\cache\memory.sqlite
```

No `XDG_RUNTIME_DIR` on Windows. Use `%TEMP%\localgpt-<username>\` for PID files.

---

## 6. Path Resolution

### 6.1 Resolution Order

Every directory is resolved through a three-level fallback:

```
1. LocalGPT-specific env var  (LOCALGPT_CONFIG_DIR, etc.)
2. XDG env var                (XDG_CONFIG_HOME, etc.)
3. Platform default           (~/.config, etc.)
```

### 6.2 Environment Variable Overrides

| Env Var | Overrides | Example |
|---|---|---|
| `LOCALGPT_CONFIG_DIR` | Config directory | `/custom/config/localgpt` |
| `LOCALGPT_DATA_DIR` | Data directory (contains workspace + localgpt.device.key) | `/custom/data/localgpt` |
| `LOCALGPT_STATE_DIR` | State directory (sessions, audit, logs) | `/custom/state/localgpt` |
| `LOCALGPT_CACHE_DIR` | Cache directory (indexes, models) | `/custom/cache/localgpt` |
| `LOCALGPT_WORKSPACE` | Workspace subdirectory only (existing, preserved) | `/projects/ai-workspace` |
| `LOCALGPT_PROFILE` | Creates profile-specific workspace (existing, preserved) | `work` |

`LOCALGPT_WORKSPACE` overrides only the workspace path within the data directory. All other directories (config, state, cache) remain at their resolved locations. This preserves the existing behavior where users point to a custom workspace while keeping infrastructure elsewhere.

### 6.3 Implementation: `Paths` Struct

```rust
/// Resolved directory paths for the entire application.
///
/// Created once at startup, passed through the application.
/// All paths are absolute and guaranteed to exist (created with 0700).
pub struct Paths {
    /// Config directory: config.toml lives here
    pub config_dir: PathBuf,

    /// Data directory root: contains workspace/ and localgpt.device.key
    pub data_dir: PathBuf,

    /// Workspace: markdown files, knowledge, skills
    /// May be overridden independently via LOCALGPT_WORKSPACE
    pub workspace: PathBuf,

    /// State directory: sessions, audit log, logs
    pub state_dir: PathBuf,

    /// Cache directory: search index, embedding models
    pub cache_dir: PathBuf,

    /// Runtime directory: PID file, sockets
    /// None if XDG_RUNTIME_DIR not set and no fallback available
    pub runtime_dir: Option<PathBuf>,
}
```

### 6.4 Resolution Function

```rust
use etcetera::BaseStrategy;

impl Paths {
    pub fn resolve() -> Result<Self> {
        let strategy = etcetera::choose_base_strategy()?;

        let config_dir = env_or("LOCALGPT_CONFIG_DIR",
            || strategy.config_dir().join("localgpt"));

        let data_dir = env_or("LOCALGPT_DATA_DIR",
            || strategy.data_dir().join("localgpt"));

        let state_dir = env_or("LOCALGPT_STATE_DIR",
            || strategy.state_dir()
                .unwrap_or_else(|| strategy.data_dir()) // Fallback for macOS/Windows
                .join("localgpt"));

        let cache_dir = env_or("LOCALGPT_CACHE_DIR",
            || strategy.cache_dir().join("localgpt"));

        // Workspace: independent override or default under data_dir
        let workspace = env_or("LOCALGPT_WORKSPACE",
            || data_dir.join("workspace"));

        // Runtime: XDG_RUNTIME_DIR or platform fallback
        let runtime_dir = resolve_runtime_dir();

        let paths = Self {
            config_dir, data_dir, workspace,
            state_dir, cache_dir, runtime_dir,
        };

        // Create all directories with mode 0700 per XDG spec
        paths.ensure_dirs()?;

        Ok(paths)
    }

    /// Convenience accessors for specific files
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn device_key(&self) -> PathBuf {
        self.data_dir.join("localgpt.device.key")
    }

    pub fn audit_log(&self) -> PathBuf {
        self.state_dir.join("localgpt.audit.jsonl")
    }

    pub fn search_index(&self) -> PathBuf {
        self.cache_dir.join("memory.sqlite")
    }

    pub fn sessions_dir(&self, agent_id: &str) -> PathBuf {
        self.state_dir.join("agents").join(agent_id).join("sessions")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.state_dir.join("logs")
    }

    pub fn pid_file(&self) -> Option<PathBuf> {
        self.runtime_dir.as_ref().map(|d| d.join("daemon.pid"))
    }
}

fn env_or(var: &str, default: impl FnOnce() -> PathBuf) -> PathBuf {
    std::env::var(var)
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute()) // XDG spec: ignore relative paths
        .unwrap_or_else(default)
}

fn resolve_runtime_dir() -> Option<PathBuf> {
    // Try XDG_RUNTIME_DIR first
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir).join("localgpt"));
        }
    }

    // Fallback: /tmp/localgpt-$UID on Unix, %TEMP%\localgpt-<user> on Windows
    #[cfg(unix)]
    {
        let uid = unsafe { libc::getuid() };
        Some(PathBuf::from(format!("/tmp/localgpt-{}", uid)))
    }

    #[cfg(windows)]
    {
        std::env::var("TEMP").ok().map(|t| {
            let user = std::env::var("USERNAME").unwrap_or_else(|_| "user".into());
            PathBuf::from(t).join(format!("localgpt-{}", user))
        })
    }
}
```

### 6.5 CLI: `localgpt paths`

Print all resolved paths for debugging and scripting:

```
$ localgpt paths

Config:    ~/.config/localgpt/
Data:      ~/.local/share/localgpt/
Workspace: ~/.local/share/localgpt/workspace/
State:     ~/.local/state/localgpt/
Cache:     ~/.cache/localgpt/
Runtime:   /run/user/1000/localgpt/

Config file:   ~/.config/localgpt/config.toml
Device key:    ~/.local/share/localgpt/localgpt.device.key
Search index:  ~/.cache/localgpt/memory.sqlite
Audit log:     ~/.local/state/localgpt/localgpt.audit.jsonl
Sessions:      ~/.local/state/localgpt/agents/main/sessions/
Logs:          ~/.local/state/localgpt/logs/
PID file:      /run/user/1000/localgpt/daemon.pid
```

---

## 7. Migration

### 7.1 Detection

On startup, `Paths::resolve()` checks for the legacy layout:

```rust
fn detect_legacy_layout() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let legacy = home.join(".localgpt");

    // Confirm it's the old monolithic layout, not just a stale directory
    if legacy.join("config.toml").exists()
        || legacy.join("workspace").exists()
    {
        Some(legacy)
    } else {
        None
    }
}
```

### 7.2 Automatic Migration

Migration runs once, automatically, on first startup after upgrade:

```
Step 1: Detect ~/.localgpt/ exists with old layout
Step 2: Resolve new XDG paths
Step 3: Move files to new locations (see mapping below)
Step 4: Write ~/.localgpt/.migrated marker file
Step 5: Leave ~/.localgpt/ in place (non-destructive)
Step 6: Print migration summary
```

**File migration mapping:**

| Old Path | New Path |
|---|---|
| `~/.localgpt/config.toml` | `$XDG_CONFIG_HOME/localgpt/config.toml` |
| `~/.localgpt/.device_key` | `$XDG_DATA_HOME/localgpt/localgpt.device.key` |
| `~/.localgpt/workspace/**` | `$XDG_DATA_HOME/localgpt/workspace/**` |
| `~/.localgpt/.security_audit.jsonl` | `$XDG_STATE_HOME/localgpt/localgpt.audit.jsonl` |
| `~/.localgpt/agents/**` | `$XDG_STATE_HOME/localgpt/agents/**` |
| `~/.localgpt/logs/**` | `$XDG_STATE_HOME/localgpt/logs/**` |
| `~/.localgpt/daemon.pid` | `$XDG_RUNTIME_DIR/localgpt/daemon.pid` |
| `~/.localgpt/workspace/memory.sqlite*` | `$XDG_CACHE_HOME/localgpt/memory.sqlite*` |

### 7.3 Migration Strategy: Copy-then-Symlink

To avoid breaking running daemons or concurrent processes:

1. **Copy** files to new XDG locations (not move).
2. **Verify** copied files match originals (size + mtime check).
3. **Replace** old files with symlinks to new locations.
4. **Write** `~/.localgpt/.migrated` with timestamp and version.

This means `~/.localgpt/config.toml` becomes a symlink to `~/.config/localgpt/config.toml`. Any code still referencing the old path continues to work. The symlinks serve as a breadcrumb trail that can be cleaned up later.

### 7.4 Rollback

If anything fails during migration:

1. Stop immediately — do not continue with partial migration.
2. Remove any newly created XDG directories that are empty.
3. Leave `~/.localgpt/` untouched.
4. Log the error and continue startup using the legacy layout.
5. Print: `⚠ XDG migration failed: {error}. Using legacy layout. Run localgpt migrate --retry to try again.`

### 7.5 Manual Migration Command

```bash
localgpt migrate              # Run migration (auto-detects if needed)
localgpt migrate --dry-run    # Show what would be moved
localgpt migrate --retry      # Retry after previous failure
localgpt migrate --cleanup    # Remove old ~/.localgpt/ symlinks and marker
```

### 7.6 OpenClaw Migration Update

The existing OpenClaw migration (`src/config/migrate.rs`) must be updated to write files to XDG locations instead of `~/.localgpt/`:

```rust
// Old: ~/.localgpt/config.toml
// New: XDG_CONFIG_HOME/localgpt/config.toml
let config_path = paths.config_file();

// Old: ~/.localgpt/workspace/
// New: XDG_DATA_HOME/localgpt/workspace/
let workspace = paths.workspace.clone();
```

---

## 8. Config File Changes

### 8.1 Updated `config.toml` Template

```toml
# LocalGPT Configuration
# Location: ~/.config/localgpt/config.toml

[agent]
default_model = "claude-cli/opus"
context_window = 128000
reserve_tokens = 8000

[providers.claude_cli]
command = "claude"

[heartbeat]
enabled = true
interval = "30m"

[memory]
# Workspace directory for memory files.
# Default: $XDG_DATA_HOME/localgpt/workspace/
# Override: set LOCALGPT_WORKSPACE env var, or change this value.
# workspace = "~/.local/share/localgpt/workspace"

# Chunk size for indexing (in tokens)
chunk_size = 400
chunk_overlap = 80

# Embedding provider: "local", "openai", "gguf", or "none"
embedding_provider = "local"
embedding_model = "BAAI/bge-small-en-v1.5"

# Additional paths to index (outside workspace)
# [[memory.paths]]
# path = "~/projects/notes"
# pattern = "**/*.md"

[logging]
level = "info"

[server]
enabled = false
port = 3580
bind = "127.0.0.1"
```

### 8.2 Removed from Config

The following paths are **removed from config** because they're now derived from XDG resolution:

| Old Config Key | Replacement |
|---|---|
| `memory.workspace` | `LOCALGPT_WORKSPACE` env var or `Paths::workspace` |
| `memory.embedding_cache_dir` | `Paths::cache_dir` / `embeddings/` |

The `memory.workspace` key is preserved for backwards compatibility but deprecated. If set, it overrides the XDG-derived workspace path (same as `LOCALGPT_WORKSPACE`). A deprecation warning is printed.

---

## 9. Crate Selection

### 9.1 Recommendation: `etcetera`

Use the `etcetera` crate with `choose_base_strategy()`:

| Crate | XDG on macOS | State dir | Project dirs | License |
|---|---|---|---|---|
| `dirs` | No (Apple paths) | Returns None on macOS | No | MIT/Apache-2.0 |
| `directories` | No (Apple paths) | Returns None on macOS | Yes | MIT/Apache-2.0 |
| **`etcetera`** | **Yes** (`choose_base_strategy`) | **Falls back to data** | Yes | MIT/Apache-2.0 |

`etcetera` is the only crate that gives XDG paths on macOS out of the box, which is what CLI users expect. It also provides `choose_native_strategy()` for future GUI builds that should use Apple-native paths.

### 9.2 Dependency Changes

| Action | Crate | Note |
|---|---|---|
| **Add** | `etcetera` | Primary path resolution |
| **Keep** | `directories` | Still used in `sandbox/policy.rs` for `BaseDirs::home_dir()` |
| **Keep** | `dirs` | Transitive dependency, minimal |

The `directories` crate is currently used in `src/config/mod.rs` and `src/sandbox/policy.rs`. Migrate `config/mod.rs` to `etcetera`. Keep `directories` in sandbox code where it's only used for home directory detection.

---

## 10. Security Implications

### 10.1 Agent Sandbox Boundaries

The XDG layout creates natural sandbox boundaries that strengthen the security model:

```
AGENT CAN READ:        workspace/          (XDG_DATA_HOME/.../workspace/)
AGENT CAN WRITE:       workspace/          (minus protected files)
AGENT CANNOT REACH:    localgpt.device.key  (XDG_DATA_HOME/.../localgpt.device.key — outside workspace subtree)
AGENT CANNOT REACH:    localgpt.audit.jsonl (XDG_STATE_HOME/... — entirely different tree)
AGENT CANNOT REACH:    config.toml          (XDG_CONFIG_HOME/... — entirely different tree)
```

Previously, the device key and audit log were "outside the workspace" but still under `~/.localgpt/`. With XDG, they're in completely separate directory trees, making path traversal attacks from workspace-scoped tools structurally impossible.

### 10.2 Updated Protected Files

```rust
// Protected workspace files (within workspace subtree)
pub const PROTECTED_FILES: &[&str] = &[
    "LocalGPT.md",
    ".localgpt.manifest.json",
    "IDENTITY.md",
];

// Protected files are now in separate XDG directories.
// The agent's file tools are scoped to the workspace path.
// localgpt.device.key and localgpt.audit.jsonl are structurally unreachable.
//
// PROTECTED_EXTERNAL_PATHS is no longer needed for security
// (kept for defense-in-depth bash heuristic checking only).
pub const PROTECTED_EXTERNAL_PATHS: &[&str] = &[
    "localgpt.device.key",
    "localgpt.audit.jsonl",
    "config.toml",
];
```

### 10.3 Device Key Permissions

```rust
fn ensure_device_key(data_dir: &Path) -> Result<()> {
    let key_path = data_dir.join("localgpt.device.key");
    if key_path.exists() { return Ok(()); }

    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    fs::write(&key_path, key)?;

    #[cfg(unix)]
    fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;

    Ok(())
}
```

---

## 11. Naming Convention Enforcement

Per the naming convention decisions:

| Context | Convention | Examples |
|---|---|---|
| User-facing workspace identity files | SCREAMING_CASE.md | `SOUL.md`, `MEMORY.md`, `LocalGPT.md` |
| Hidden workspace implementation files | dot + `localgpt.<purpose>.<ext>` | `.localgpt.manifest.json`, `.gitignore` |
| Everything outside workspace | `localgpt.<purpose>.<ext>`, no dot | `localgpt.device.key`, `localgpt.audit.jsonl`, `config.toml`, `memory.sqlite` |

`LocalGPT.md` is the branded exception to SCREAMING_CASE — it uses PascalCase for brand recognition.

---

## 12. Affected Code Paths

### 12.1 Must Change

| File | Change |
|---|---|
| `src/config/mod.rs` | Replace `Config::config_path()` with `Paths::config_file()`. Remove hardcoded `~/.localgpt/`. Deprecate `memory.workspace` config key. |
| `src/config/migrate.rs` | Update OpenClaw migration to write to XDG paths. |
| `src/memory/workspace.rs` | `init_workspace()` receives `Paths` instead of deriving from `workspace.parent()`. `init_state_dir()` uses `Paths::state_dir`. |
| `src/memory/mod.rs` | `MemoryManager` takes `Paths` for index location (`cache_dir/memory.sqlite`). |
| `src/memory/index.rs` | DB path comes from `Paths::search_index()` instead of `workspace.join("memory.sqlite")`. |
| `src/agent/mod.rs` | `Agent::new()` takes `Paths`. Security policy verification uses `Paths::workspace` and `Paths::data_dir`. |
| `src/security/signing.rs` | `read_device_key()` uses `Paths::device_key()`. |
| `src/security/audit.rs` | `append_audit_entry()` uses `Paths::audit_log()`. |
| `src/security/protected_files.rs` | Update path checks for XDG layout. Simplify external paths. |
| `src/cli/md.rs` | All security CLI commands derive paths from `Paths`. |
| `src/daemon.rs` (if exists) | PID file from `Paths::pid_file()`. |
| `docs/memory-system.md` | Update all path references. |
| `CLAUDE.md` | Update directory structure documentation. |

### 12.2 New Files

| File | Purpose |
|---|---|
| `src/paths.rs` | `Paths` struct, resolution logic, migration |
| `src/cli/paths.rs` | `localgpt paths` subcommand |
| `src/cli/migrate.rs` | `localgpt migrate` subcommand |

---

## 13. Implementation Order

```
Phase 1: Foundation
  1. Add `etcetera` dependency
  2. Create `src/paths.rs` with Paths struct and resolution logic
  3. Add `localgpt paths` CLI subcommand
  4. Tests for path resolution on all platforms

Phase 2: Wire Through
  5. Update Config::load() to use Paths::config_file()
  6. Update MemoryManager to use Paths::search_index()
  7. Update Agent to use Paths for security file locations
  8. Update workspace init to use Paths
  9. Update all CLI commands to resolve Paths once and pass through

Phase 3: Migration
  10. Implement legacy layout detection
  11. Implement copy-then-symlink migration
  12. Implement `localgpt migrate` CLI
  13. Auto-migration on startup
  14. Update OpenClaw migration to target XDG paths

Phase 4: Cleanup
  15. Update all documentation (CLAUDE.md, docs/*.md, README)
  16. Deprecation warning for memory.workspace config key
  17. Remove hardcoded ~/.localgpt/ references
```

---

## 14. Testing

### 14.1 Unit Tests

```rust
#[test]
fn default_paths_are_xdg_compliant() {
    // Clear all env vars, verify defaults
    let paths = Paths::resolve_with_env(HashMap::new(), home);
    assert!(paths.config_dir.ends_with(".config/localgpt"));
    assert!(paths.data_dir.ends_with(".local/share/localgpt"));
    assert!(paths.state_dir.ends_with(".local/state/localgpt"));
    assert!(paths.cache_dir.ends_with(".cache/localgpt"));
}

#[test]
fn localgpt_env_vars_override_xdg() {
    let env = HashMap::from([
        ("LOCALGPT_CONFIG_DIR", "/custom/config"),
    ]);
    let paths = Paths::resolve_with_env(env, home);
    assert_eq!(paths.config_dir, PathBuf::from("/custom/config"));
}

#[test]
fn xdg_env_vars_override_defaults() {
    let env = HashMap::from([
        ("XDG_CONFIG_HOME", "/xdg/config"),
    ]);
    let paths = Paths::resolve_with_env(env, home);
    assert_eq!(paths.config_dir, PathBuf::from("/xdg/config/localgpt"));
}

#[test]
fn relative_paths_are_ignored() {
    let env = HashMap::from([
        ("LOCALGPT_CONFIG_DIR", "relative/path"),
    ]);
    let paths = Paths::resolve_with_env(env, home);
    // Should fall back to XDG default, not use relative path
    assert!(paths.config_dir.is_absolute());
}

#[test]
fn workspace_override_independent_of_data_dir() {
    let env = HashMap::from([
        ("LOCALGPT_WORKSPACE", "/projects/my-workspace"),
    ]);
    let paths = Paths::resolve_with_env(env, home);
    assert_eq!(paths.workspace, PathBuf::from("/projects/my-workspace"));
    // data_dir should still be at XDG default (not derived from workspace)
    assert!(paths.data_dir.ends_with(".local/share/localgpt"));
}
```

### 14.2 Integration Tests

- Legacy migration: create `~/.localgpt/` layout in tempdir, run migration, verify all files moved correctly.
- Symlink integrity: verify old paths resolve to new locations after migration.
- Cache deletion: delete cache dir, verify index rebuilds on next startup.
- Profile isolation: `LOCALGPT_PROFILE=work` creates separate workspace but shares config/state/cache.
- OpenClaw migration: create `~/.openclaw/` layout, verify auto-migration targets XDG paths.

---

## 15. OpenClaw Comparison

| Aspect | OpenClaw | LocalGPT (after this RFC) |
|---|---|---|
| Layout | Monolithic `~/.openclaw/` | XDG-compliant (4 directories) |
| Config location | `~/.openclaw/openclaw.json` | `$XDG_CONFIG_HOME/localgpt/config.toml` |
| Config override | `OPENCLAW_CONFIG_PATH` | `LOCALGPT_CONFIG_DIR` + XDG |
| Data/state separation | None — all in one tree | Full: data (workspace + key), state (sessions + audit), cache (index) |
| Cache deletable? | No — index mixed with workspace | Yes — `rm -rf ~/.cache/localgpt/` is safe |
| Dotfile-repo friendly | No — `~/.openclaw/` is too large | Yes — `~/.config/localgpt/` contains only config.toml |
| XDG compliance | None | Full (spec v0.8) |
| Selective backup | Copy entire `~/.openclaw/` | Back up `$XDG_DATA_HOME/localgpt/` only |
| Profile env var | `OPENCLAW_PROFILE` | `LOCALGPT_PROFILE` (preserved) |
| Legacy migration | Manual: copy `~/.openclaw/` | Automatic: copy-then-symlink with rollback |

---

## Appendix A: Quick Reference Card

For documentation, README, and `localgpt paths --help`:

```
LocalGPT stores files in four directories following the XDG Base Directory Specification:

  CONFIG    Settings you edit           ~/.config/localgpt/
  DATA      Your workspace and keys     ~/.local/share/localgpt/
  STATE     Sessions, logs, audit       ~/.local/state/localgpt/
  CACHE     Search index, models        ~/.cache/localgpt/

Override any directory:
  LOCALGPT_CONFIG_DIR=/path    or    XDG_CONFIG_HOME=/path
  LOCALGPT_DATA_DIR=/path      or    XDG_DATA_HOME=/path
  LOCALGPT_STATE_DIR=/path     or    XDG_STATE_HOME=/path
  LOCALGPT_CACHE_DIR=/path     or    XDG_CACHE_HOME=/path

Override just the workspace:
  LOCALGPT_WORKSPACE=/path/to/workspace

Show all resolved paths:
  localgpt paths
```
