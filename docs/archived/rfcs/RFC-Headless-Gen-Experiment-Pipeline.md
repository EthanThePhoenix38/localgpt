# RFC: Headless Generation, Experiment Queue, and Async Creative Pipeline

**Status:** Implemented (Archived 2026-03-25). See `crates/gen/src/experiment.rs` and `--headless` CLI flag.
**Author:** Yi  
**Date:** 2026-03-16  
**Target crates:** `localgpt-gen`, `localgpt-core` (heartbeat), `localgpt-world-types`  
**Depends on:** Gen mode MCP tools (P1–P5), world save/load (`gen_save_world`/`gen_load_world`)  
**Supersedes:** N/A (new capability)

---

## 1. Summary

Add headless (windowless) 3D world generation to LocalGPT Gen, integrate it with the heartbeat autonomous task runner as an experiment queue, redesign the memory system for gen-specific knowledge accumulation, expose headless generation via MCP server mode, and provide a gallery UI for browsing experiment results.

The product thesis: **"Queue experiments, generate overnight, explore results in the morning."** No competitor in the AI world-generation space — World Labs, Google Genie, HunyuanWorld, Roblox — offers asynchronous local-first generation with persistent creative memory. This capability is uniquely enabled by LocalGPT's single-binary, zero-cloud architecture.

---

## 2. Problem Statement

### 2.1 Synchronous generation is a bottleneck

Today, LocalGPT Gen operates exclusively in interactive mode: the user types a prompt, watches the AI issue tool calls, and sees the world materialize in real time. This works for single scenes but fails for creative workflows that require **variation and comparison** — the bread and butter of professional world design.

Building 5 lighting variations of a village means sitting through 5 sequential sessions. The user cannot queue "try this with sunset, moonlight, and overcast" and walk away. Every other creative tool — Photoshop actions, Blender batch rendering, CI/CD pipelines — solved this decades ago. Gen mode has no equivalent.

### 2.2 Memory is vestigial in gen mode

The memory system (MEMORY.md, daily logs, HEARTBEAT.md) was designed for CLI assistant workflows: remembering user preferences, project context, and todo items across chat sessions. Gen mode inherits this system via `Agent::new_with_tools()` but gains minimal value from it:

- MEMORY.md captures generic user preferences, not world-building style knowledge
- HEARTBEAT.md has no meaningful autonomous tasks for a synchronous world builder
- Daily logs dump raw conversation transcripts, not structured scene summaries
- Session compaction flushes context to MEMORY.md, but the flushed content is conversation text, not scene state

### 2.3 MCP server mode lacks batch capability

External AI backends (Claude CLI, Gemini CLI, Codex) can drive gen via the MCP server, but only one world at a time in an interactive session. There is no way for an external tool to submit a batch of generation tasks and retrieve results later.

### 2.4 No experiment comparison workflow

Even when a user manually builds multiple world variations, there is no gallery or comparison view. Each world is a folder in `skills/` with no thumbnail, no metadata summary, and no way to browse results without loading each one individually.

---

## 3. Design Principles

**P1 — Headless is a mode, not a separate binary.** Headless generation reuses the same `localgpt-gen` binary with a `--headless` flag. No separate build target, no code duplication. The single-binary constraint is preserved.

**P2 — HEARTBEAT.md remains the user-facing queue.** Users add experiments to HEARTBEAT.md in natural language, just like CLI tasks. The heartbeat runner parses and executes them. No new configuration format for the user to learn.

**P3 — Internal tracking uses structured state.** Behind the scenes, an experiment tracker (`state/gen-experiments.jsonl`) maintains machine-readable status for each queued task. This lives in XDG state (device-specific, not portable) while HEARTBEAT.md lives in XDG data (user-curated, portable).

**P4 — Memory becomes a creative journal, not a fact store.** Gen-specific memory records world-building style preferences, entity templates, experiment outcomes, and design decisions. The system prompt teaches the LLM what to remember for world creation.

**P5 — GPU exclusivity is respected.** Headless gen never runs concurrently with interactive gen on the same GPU. A lockfile prevents conflicts. Headless gen is designed for "generate while you're away."

**P6 — Each experiment is isolated.** Every headless generation task starts from a clean scene (`gen_clear_scene`), produces a self-contained world skill folder, and logs its own results. No cross-contamination between experiments.

**P7 — Thumbnails are first-class outputs.** Every completed experiment produces at least one screenshot. This enables gallery browsing without loading full worlds.

---

## 4. User Stories and Acceptance Criteria

### US-1: Queue world experiments from the terminal

> As a creator, I want to add world generation tasks to HEARTBEAT.md so they run automatically while I'm away.

**Acceptance criteria:**
- User adds a gen experiment entry to HEARTBEAT.md (freeform natural language)
- Heartbeat runner recognizes gen experiment tasks and dispatches them to headless gen
- Each experiment produces a saved world skill in `workspace/skills/<name>/`
- Each experiment produces at least one thumbnail screenshot
- Experiment status is logged to daily memory (`memory/YYYY-MM-DD.md`)
- Completed experiments are checked off in HEARTBEAT.md
- Failed experiments are marked with error reason, not silently dropped

### US-2: Queue variations of a single concept

> As a creator, I want to request "5 variations of a medieval village with different lighting" and get 5 separate world skills to compare.

**Acceptance criteria:**
- A single HEARTBEAT.md entry can specify N variations with a variation axis
- Each variation is generated as a separate world skill with a descriptive name suffix
- Variation parameters (lighting, density, palette) are recorded in the world's metadata
- Memory logs which variation the user ultimately preferred (when they review)

### US-3: Run headless gen from CLI

> As a developer, I want to generate a world without opening a window, for scripting and CI/CD.

**Acceptance criteria:**
- `localgpt-gen --headless --prompt "..." --output skills/my-world/` generates a world
- Exit code 0 on success, non-zero on failure
- stdout reports generation progress (tool calls, entity count)
- A thumbnail screenshot is saved alongside the world
- Works on headless Linux servers (no display required, software rendering fallback)

### US-4: Review experiment results in a gallery

> As a creator, I want to see thumbnails of all my generated worlds so I can quickly compare and pick winners.

**Acceptance criteria:**
- Interactive gen mode shows a gallery overlay (triggered by `/gallery` or keybind)
- Gallery displays world name, thumbnail, entity count, creation date, and style tags
- Clicking a thumbnail loads the world (`gen_load_world`)
- Gallery reads from `workspace/skills/*/` and their `world.ron` metadata
- Works without any new database — reads filesystem directly

### US-5: Memory learns my creative style

> As a creator, I want the AI to remember my preferred visual style, color palettes, and entity designs across sessions.

**Acceptance criteria:**
- Gen system prompt instructs the LLM to save style preferences via `memory_save`
- When the user praises or selects a particular style, the LLM records it
- New gen sessions begin by searching memory for the user's style preferences
- Entity templates (user-crafted designs) are saved as named entries in memory
- `memory_search("lantern")` returns the user's custom lantern design parameters

### US-6: MCP server supports batch headless generation

> As an external AI backend operator, I want to submit multiple gen tasks via MCP and retrieve results, without needing a visible window.

**Acceptance criteria:**
- `localgpt-gen mcp-server --headless` runs without a window
- New MCP tools: `gen_queue_experiment`, `gen_list_experiments`, `gen_experiment_status`
- Experiments run sequentially (one at a time, GPU exclusivity)
- Results include world path, screenshot path, entity count, and generation time
- MCP clients can poll for completion

### US-7: Experiment history is searchable

> As a creator, I want to ask "what forest scenes did I generate last week?" and get answers from memory.

**Acceptance criteria:**
- `memory_search("forest scene")` returns experiment results with dates and outcomes
- Daily logs contain structured summaries of completed experiments (not raw transcripts)
- The LLM can reference past experiment results when building new worlds
- "Make it like the forest from last Tuesday" works via memory recall

---

## 5. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    User Interfaces                       │
│                                                         │
│  HEARTBEAT.md          CLI flags          MCP protocol  │
│  (experiment queue)    (--headless)       (batch tools)  │
└────────┬───────────────────┬───────────────────┬────────┘
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────┐
│              Experiment Dispatcher                       │
│                                                         │
│  Parses experiment tasks from any source                │
│  Maintains state/gen-experiments.jsonl                   │
│  Enforces GPU exclusivity via lockfile                  │
│  Routes to headless or interactive gen                   │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              Headless Bevy Runtime                       │
│                                                         │
│  MinimalPlugins + offscreen render target               │
│  GenPlugin (same as interactive, minus window)           │
│  Agent with gen tools + memory tools                    │
│  Offscreen screenshot capture                           │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              Output Layer                                │
│                                                         │
│  workspace/skills/<name>/                                │
│    ├── world.ron          (scene manifest)               │
│    ├── SKILL.md           (world metadata)               │
│    ├── history.jsonl      (tool call log)                │
│    └── screenshots/       (thumbnails)                   │
│                                                         │
│  MEMORY.md                (experiment journal)           │
│  memory/YYYY-MM-DD.md     (daily experiment log)         │
│  state/gen-experiments.jsonl (tracking state)            │
└─────────────────────────────────────────────────────────┘
```

---

## 6. Phase 1: Headless Bevy Gen Mode

### 6.1 Objective

Run the full gen tool pipeline (spawn, modify, audio, behaviors, save, export) without creating a visible window. Produce world skills and screenshots from the command line.

### 6.2 CLI Interface

```
localgpt-gen --headless [OPTIONS]

Options:
  --prompt <TEXT>        Initial generation prompt
  --output <PATH>        Output world skill directory (default: auto-named in workspace/skills/)
  --screenshot           Capture a thumbnail after generation (default: true)
  --screenshot-width <N>  Screenshot width in pixels (default: 1280)
  --screenshot-height <N> Screenshot height in pixels (default: 720)
  --timeout <DURATION>   Max generation time before abort (default: 5m)
  --agent <ID>           Agent ID for memory isolation (default: "gen-headless")
  --model <MODEL>        Override LLM model for this run
  --style <TEXT>         Style hint prepended to prompt (e.g., "low-poly, warm palette")
```

**Examples:**

```bash
# Generate a single world
localgpt-gen --headless --prompt "Build a cozy cabin in a snowy forest"

# Generate with style override
localgpt-gen --headless \
  --prompt "Medieval village marketplace" \
  --style "Studio Ghibli, watercolor feel" \
  --output skills/ghibli-village/

# Scriptable: generate and export
localgpt-gen --headless \
  --prompt "Sci-fi space station interior" \
  --output skills/station/ \
  && localgpt-gen --headless \
     --prompt "Export the scene as glTF" \
     --scene skills/station/
```

### 6.3 Headless Bevy Bootstrapping

The key architectural change: `run_bevy_app` gains a headless variant that replaces `DefaultPlugins` with a headless plugin set.

```rust
// crates/gen/src/main.rs

fn run_headless_bevy_app(
    channels: gen3d::GenChannels,
    workspace: PathBuf,
    screenshot_config: ScreenshotConfig,
) -> Result<()> {
    use bevy::prelude::*;
    use bevy::render::settings::{RenderCreation, WgpuSettings};

    let mut app = App::new();

    // Headless: use DefaultPlugins but without windowing,
    // with offscreen render target for screenshots.
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,  // No window
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // Allow software rendering on headless servers
                    backends: Some(wgpu::Backends::all()),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::asset::AssetPlugin {
                file_path: "/".to_string(),
                ..default()
            })
            .disable::<bevy::log::LogPlugin>(),
    );

    // Add offscreen render target for screenshot capture
    app.insert_resource(OffscreenRenderTarget {
        width: screenshot_config.width,
        height: screenshot_config.height,
        ..default()
    });

    gen3d::plugin::setup_gen_app(&mut app, channels, workspace, None);

    // Add headless-specific systems
    app.add_systems(Update, headless_completion_detector);

    app.run();
    Ok(())
}
```

### 6.4 Offscreen Rendering for Screenshots

Bevy's `headless_renderer` example provides the pattern. We create a render texture, attach a camera, and copy pixels back after rendering.

```rust
// crates/gen/src/gen3d/offscreen.rs

use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

/// Configuration for the offscreen render target.
#[derive(Resource)]
pub struct OffscreenRenderTarget {
    pub width: u32,
    pub height: u32,
    pub image_handle: Option<Handle<Image>>,
}

impl Default for OffscreenRenderTarget {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            image_handle: None,
        }
    }
}

/// Marker component for the offscreen camera.
#[derive(Component)]
pub struct OffscreenCamera;

/// Set up the offscreen render target and camera.
pub fn setup_offscreen_render(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut target: ResMut<OffscreenRenderTarget>,
) {
    let size = Extent3d {
        width: target.width,
        height: target.height,
        depth_or_array_layers: 1,
    };

    // Create the render target image
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("offscreen_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);
    target.image_handle = Some(image_handle.clone());

    // Spawn offscreen camera rendering to the texture
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: bevy::render::camera::RenderTarget::Image(image_handle),
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        OffscreenCamera,
    ));
}

/// Capture the offscreen render target to a PNG file.
pub fn capture_offscreen_screenshot(
    images: Res<Assets<Image>>,
    target: Res<OffscreenRenderTarget>,
    path: &str,
) -> Result<(), String> {
    let handle = target
        .image_handle
        .as_ref()
        .ok_or("No offscreen render target")?;

    let image = images
        .get(handle)
        .ok_or("Render target image not ready")?;

    // Convert to PNG via image crate
    let dynamic_image = image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(
            target.width,
            target.height,
            image.data.clone(),
        )
        .ok_or("Failed to create image buffer")?,
    );

    dynamic_image
        .save(path)
        .map_err(|e| format!("Failed to save screenshot: {}", e))?;

    Ok(())
}
```

### 6.5 Headless Completion Detection

In interactive mode, the Bevy app runs indefinitely. In headless mode, it must detect when generation is complete and exit.

```rust
// crates/gen/src/gen3d/headless.rs

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared flag between agent thread and Bevy main thread.
#[derive(Resource, Clone)]
pub struct HeadlessCompletionFlag {
    pub done: Arc<AtomicBool>,
    pub success: Arc<AtomicBool>,
}

/// System that checks the completion flag and triggers Bevy exit.
pub fn headless_completion_detector(
    flag: Res<HeadlessCompletionFlag>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    if flag.done.load(Ordering::Relaxed) {
        exit.send(bevy::app::AppExit::Success);
    }
}
```

The agent loop sets `done = true` after:
1. The LLM finishes generating (no more tool calls)
2. `gen_save_world` has been called
3. A screenshot has been captured (if enabled)

### 6.6 Exit Code Contract

| Exit code | Meaning |
|-----------|---------|
| 0 | World generated and saved successfully |
| 1 | Generation failed (LLM error, tool error, timeout) |
| 2 | Invalid arguments or configuration |
| 3 | GPU not available (headless server without compatible GPU) |

---

## 7. Phase 2: Experiment Queue and Heartbeat Integration

### 7.1 Objective

Enable the heartbeat runner to parse gen experiment tasks from HEARTBEAT.md, dispatch them to headless gen, and log results.

### 7.2 HEARTBEAT.md Experiment Format

Experiments are written as checkbox items under a `## Gen Experiments` heading. The format is natural language with optional structured hints in parentheses.

```markdown
## Gen Experiments

- [ ] Build an enchanted forest — sparse trees, open clearings, wildflowers
- [ ] Build an enchanted forest — dense canopy, mushrooms, dark atmosphere
- [ ] Build an enchanted forest — medium density, winding paths, dappled sunlight
- [ ] Medieval village with 3 lighting variations (variation: lighting = dawn, noon, sunset)
- [ ] Sci-fi corridor in my "neon cyberpunk" style (style: from memory)
```

**Variation syntax:** `(variation: <axis> = <value1>, <value2>, ...)` expands into N separate experiments. The heartbeat agent interprets this and generates each variation as a separate world.

**Style reference:** `(style: from memory)` tells the agent to search MEMORY.md for the named style before generating.

### 7.3 Experiment State Tracking

Internal state lives at `<state_dir>/gen-experiments.jsonl` (XDG state — device-specific, not portable).

```rust
// crates/gen/src/experiment.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique experiment ID (e.g., "exp-20260316-143022-enchanted-forest")
    pub id: String,
    /// Original prompt from HEARTBEAT.md
    pub prompt: String,
    /// Optional style hint (from memory or inline)
    pub style: Option<String>,
    /// Current status
    pub status: ExperimentStatus,
    /// Output world skill path (set on completion)
    pub output_path: Option<String>,
    /// Screenshot path (set on completion)
    pub screenshot_path: Option<String>,
    /// Entity count in final scene
    pub entity_count: Option<usize>,
    /// Generation duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Error message (set on failure)
    pub error: Option<String>,
    /// Timestamp when queued
    pub queued_at: DateTime<Utc>,
    /// Timestamp when started
    pub started_at: Option<DateTime<Utc>>,
    /// Timestamp when completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Variation group ID (if part of a variation set)
    pub variation_group: Option<String>,
    /// Variation axis and value (e.g., "lighting" = "sunset")
    pub variation: Option<(String, String)>,
    /// LLM model used for generation
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Append-only experiment log.
pub struct ExperimentTracker {
    path: PathBuf,
}

impl ExperimentTracker {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            path: state_dir.join("gen-experiments.jsonl"),
        }
    }

    /// Append a new or updated experiment record.
    pub fn append(&self, experiment: &Experiment) -> anyhow::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(experiment)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Read all experiments, deduplicating by ID (last entry wins).
    pub fn read_all(&self) -> anyhow::Result<Vec<Experiment>> {
        use std::collections::HashMap;
        let content = std::fs::read_to_string(&self.path).unwrap_or_default();
        let mut map: HashMap<String, Experiment> = HashMap::new();
        for line in content.lines() {
            if let Ok(exp) = serde_json::from_str::<Experiment>(line) {
                map.insert(exp.id.clone(), exp);
            }
        }
        let mut experiments: Vec<Experiment> = map.into_values().collect();
        experiments.sort_by_key(|e| e.queued_at);
        Ok(experiments)
    }

    /// Get pending experiments.
    pub fn pending(&self) -> anyhow::Result<Vec<Experiment>> {
        Ok(self
            .read_all()?
            .into_iter()
            .filter(|e| e.status == ExperimentStatus::Pending)
            .collect())
    }
}
```

### 7.4 Gen-Aware Heartbeat Runner

Extend the existing `HeartbeatRunner` with a gen experiment dispatch path.

```rust
// crates/gen/src/heartbeat_gen.rs

use localgpt_core::config::Config;

/// Factory function that creates gen tools for a headless Bevy instance.
///
/// Called by HeartbeatRunner's ToolFactory when a gen experiment is detected.
pub fn create_headless_gen_tool_factory() -> localgpt_core::heartbeat::ToolFactory {
    Box::new(|config: &Config| {
        let (bridge, channels) = gen3d::create_gen_channels();

        // Boot headless Bevy on a dedicated thread
        let workspace = config.workspace_path();
        std::thread::spawn(move || {
            if let Err(e) = run_headless_bevy_app(
                channels,
                workspace,
                ScreenshotConfig::default(),
            ) {
                tracing::error!("Headless Bevy failed: {}", e);
            }
        });

        // Wait for Bevy to initialize (channels become ready)
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Return gen tools pointing at the headless bridge
        let mut tools = gen3d::tools::create_gen_tools(bridge.clone());
        tools.extend(mcp::avatar_tools::create_character_tools(bridge.clone()));
        tools.extend(mcp::interaction_tools::create_interaction_tools(
            bridge.clone(),
        ));
        tools.extend(mcp::terrain_tools::create_terrain_tools(bridge.clone()));
        tools.extend(mcp::ui_tools::create_ui_tools(bridge.clone()));
        tools.extend(mcp::physics_tools::create_physics_tools(bridge));
        Ok(tools)
    })
}
```

### 7.5 Heartbeat Experiment Detection

The heartbeat prompt builder needs to detect gen experiment entries and route them appropriately.

```rust
// Extension to crates/core/src/agent/mod.rs :: build_heartbeat_prompt

/// Detect if HEARTBEAT.md contains gen experiment entries.
pub fn has_gen_experiments(heartbeat_content: &str) -> bool {
    let lower = heartbeat_content.to_lowercase();
    lower.contains("## gen experiments")
        || lower.contains("## world experiments")
        || (lower.contains("- [ ]")
            && (lower.contains("build a")
                || lower.contains("generate a")
                || lower.contains("create a world")
                || lower.contains("variation")))
}
```

When gen experiments are detected, the heartbeat runner:

1. Acquires the GPU lockfile (`<runtime_dir>/localgpt-gen-gpu.lock`)
2. Creates a headless gen agent with gen tools via the factory
3. Processes one experiment per heartbeat tick (not all at once — respects interval)
4. After each experiment: saves world, captures screenshot, updates experiment tracker, logs to memory
5. Releases GPU lock

### 7.6 GPU Exclusivity

```rust
// crates/gen/src/gpu_lock.rs

use std::path::PathBuf;

/// File-based GPU lock to prevent concurrent Bevy instances.
pub struct GpuLock {
    lock_path: PathBuf,
}

impl GpuLock {
    pub fn new() -> Self {
        // Use XDG_RUNTIME_DIR if available, else /tmp
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("localgpt");
        let _ = std::fs::create_dir_all(&runtime_dir);
        Self {
            lock_path: runtime_dir.join("gen-gpu.lock"),
        }
    }

    /// Try to acquire the GPU lock (non-blocking).
    /// Returns None if another gen instance holds it.
    pub fn try_acquire(&self) -> Option<GpuLockGuard> {
        use fs2::FileExt;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.lock_path)
            .ok()?;

        file.try_lock_exclusive().ok()?;

        // Write PID for diagnostics
        use std::io::Write;
        let mut f = file;
        let _ = write!(f, "{}", std::process::id());

        Some(GpuLockGuard { _file: f })
    }

    /// Check if the GPU is currently locked by another process.
    pub fn is_locked(&self) -> bool {
        self.try_acquire().is_none()
    }
}

pub struct GpuLockGuard {
    _file: std::fs::File,
    // Lock released automatically when File is dropped (fs2 unlock)
}
```

### 7.7 Experiment Lifecycle

```
HEARTBEAT.md entry created by user
        │
        ▼
Heartbeat tick fires
        │
        ├─ No gen experiments? → Normal heartbeat behavior (CLI tasks)
        │
        ├─ Gen experiment found, GPU locked? → Skip, try next tick
        │
        ├─ Gen experiment found, GPU available:
        │       │
        │       ▼
        │   Acquire GPU lock
        │       │
        │       ▼
        │   Boot headless Bevy (background thread)
        │       │
        │       ▼
        │   Create agent with gen tools + memory tools
        │       │
        │       ▼
        │   Search memory for user style preferences
        │       │
        │       ▼
        │   Send prompt to LLM → LLM issues gen tool calls
        │       │
        │       ▼
        │   gen_save_world → world skill saved
        │       │
        │       ▼
        │   Capture offscreen screenshot → thumbnail saved
        │       │
        │       ▼
        │   Log results to memory (MEMORY.md + daily log)
        │       │
        │       ▼
        │   Update experiment tracker (gen-experiments.jsonl)
        │       │
        │       ▼
        │   Check off HEARTBEAT.md entry
        │       │
        │       ▼
        │   Shut down headless Bevy, release GPU lock
        │
        ▼
   Next tick
```

---

## 8. Phase 3: Gen-Specific Memory Integration

### 8.1 Objective

Redesign how memory is used in gen mode so that it accumulates world-building knowledge: style preferences, entity templates, experiment outcomes, and design patterns.

### 8.2 Gen System Prompt Overlay

Replace the generic memory guidance when the agent is in gen mode. This is the highest-ROI change — no new code, just different instructions.

```rust
// crates/gen/src/system_prompt.rs

pub const GEN_MEMORY_PROMPT: &str = r#"
## Memory (World Creator Mode)

You have access to a persistent memory system that helps you learn
this creator's preferences over time.

### Before building a new scene:
1. Use `memory_search` to check for:
   - Style preferences (colors, lighting, aesthetics, mood)
   - Saved entity templates (custom designs the user has built before)
   - Past experiment outcomes (what worked, what didn't)
2. Apply any discovered preferences as defaults for the new scene.

### After the user approves a scene or experiment:
Use `memory_save` to record:
- **Style preferences**: palette, lighting setup, fog settings, camera angles
  Format: `## Style: <name>\n- Palette: ...\n- Lighting: ...\n- Mood: ...`
- **Entity templates**: reusable designs the user iterated on
  Format: `## Entity: <name>\n- Shape: ...\n- Color: ...\n- Behaviors: ...`
- **Experiment outcomes**: what was tried, what the user preferred
  Format: `## Experiment: <date> <topic>\n- Variation A: ...\n- Winner: ...`

### When the user references past work:
- "like the forest from last week" → `memory_search("forest")`
- "use my lantern design" → `memory_search("entity lantern")`
- "my usual style" → `memory_search("style preference")`

### Daily log:
After each session, use `memory_log` to record a structured summary:
```
World: <name> | Entities: <N> | Style: <brief> | Outcome: <user reaction>
```

Do NOT log raw conversation text. Log scene state and design decisions.
"#;
```

### 8.3 Entity Template Memory Format

When the LLM detects that a user has iterated on an entity design and is satisfied, it saves a template:

```markdown
## Entity: Glowing Lantern
- Created: 2026-03-15
- Shape: box 0.15×0.4×0.15
- Color: #FFD700, emissive: 2.0
- Light: point, color #FFB347, intensity 800, radius 8
- Behavior: bob amplitude 0.1, speed 0.5
- Tags: lighting, decoration, medieval, warm
- Notes: User iterated 3x on color before settling on warm gold
```

Retrieval example:
```
User: "Add some of my lanterns along the path"
LLM: [calls memory_search("entity lantern")]
LLM: [gets back the template above]
LLM: [calls gen_spawn_primitive with exact saved parameters, 5 times along path]
```

### 8.4 Style Preference Memory Format

```markdown
## Style: Ghibli Forest
- Created: 2026-03-14
- Palette: earth tones (#8B7355, #D4A574, #556B2F, #87CEEB)
- Lighting: warm amber point lights, soft directional from 45°
- Sky: sunset preset, fog distance 80-120
- Atmosphere: magical realism, cozy, slightly oversaturated
- Camera: third-person, offset [0, 4, -8]
- Audio: forest ambience (birds, wind), volume 0.3
- Tags: nature, warm, ghibli, fantasy
```

### 8.5 Experiment Result Memory Format

```markdown
## Experiment: 2026-03-16 Enchanted Forest Variations
- Concept: Three density variations of an enchanted forest
- Variations:
  - sparse-forest (skills/sparse-forest/): 8 trees, 3 bushes, open clearings
  - medium-forest (skills/medium-forest/): 20 trees, 12 bushes, winding paths ★ WINNER
  - dense-forest (skills/dense-forest/): 45 trees, 25 bushes, too dark
- Findings: Medium density with dappled light felt best. Dense needs stronger ambient.
- Style used: Ghibli Forest (from memory)
```

### 8.6 Session Summarization Override

Override `save_session_to_memory` for gen sessions to produce structured summaries instead of raw transcripts.

```rust
// crates/gen/src/memory_gen.rs

use localgpt_core::agent::Agent;
use crate::gen3d::{GenBridge, GenCommand, GenResponse};

/// Generate a structured gen session summary using the LLM.
///
/// Called instead of the default transcript dump when a gen session ends.
pub async fn save_gen_session_summary(
    agent: &mut Agent,
    bridge: &GenBridge,
) -> anyhow::Result<Option<std::path::PathBuf>> {
    // Get current scene state
    let scene_info = bridge
        .send_command(GenCommand::SceneInfo)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get scene info: {}", e))?;

    let entity_count = match &scene_info {
        GenResponse::SceneInfo { entities, .. } => entities.len(),
        _ => 0,
    };

    // Ask the LLM to summarize the session as structured memory
    let summary_prompt = format!(
        "The world-building session is ending. The final scene has {} entities.\n\
         Summarize this session for your memory in the following format:\n\
         ```\n\
         World: <name> | Entities: {} | Style: <brief description>\n\
         Key entities: <list the main things you built>\n\
         Design decisions: <what the user liked or asked to change>\n\
         ```\n\
         Use `memory_log` to save this summary. Do NOT save raw conversation.",
        entity_count, entity_count
    );

    let response = agent.chat(&summary_prompt).await?;
    tracing::debug!("Gen session summary: {}", response);

    Ok(None) // memory_log tool call handles the actual save
}
```

### 8.7 Memory Partitioning Strategy

Gen memory and CLI memory share the same MEMORY.md file but use distinct section headers for searchability:

```markdown
# MEMORY.md

## User Info
- Name: Yi
- Timezone: ...

## Gen Styles
### Style: Ghibli Forest
...
### Style: Neon Cyberpunk
...

## Gen Entity Templates
### Entity: Glowing Lantern
...
### Entity: Pine Tree
...

## Gen Experiment Log
### 2026-03-16: Enchanted Forest Variations
...
```

The `memory_search` tool's FTS5 index handles section-level retrieval naturally — searching "entity lantern" will rank the Entity: Glowing Lantern section highest. No schema change needed.

**Scaling path:** If the combined file grows past the embedding chunk window (typically 512 tokens per chunk), introduce a separate `GEN_MEMORY.md`. This is controlled by the `[gen.memory] separate_file` config option (default: false for v1).

---

## 9. Phase 4: Gallery UI

### 9.1 Objective

Provide an in-app gallery for browsing generated worlds, comparing experiment results, and loading worlds for further editing.

### 9.2 Gallery Data Source

The gallery reads directly from the filesystem — no database required.

```rust
// crates/gen/src/gallery.rs

use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct WorldGalleryEntry {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub entity_count: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub thumbnail_path: Option<PathBuf>,
    pub style_tags: Vec<String>,
    pub variation_group: Option<String>,
    /// Source: "interactive", "headless", "experiment", "mcp"
    pub source: String,
}

/// Scan workspace/skills/ for world skills and build gallery entries.
pub fn scan_world_gallery(workspace: &Path) -> Vec<WorldGalleryEntry> {
    let skills_dir = workspace.join("skills");
    let mut entries = Vec::new();

    if !skills_dir.exists() {
        return entries;
    }

    for dir_entry in std::fs::read_dir(&skills_dir).into_iter().flatten() {
        let Ok(entry) = dir_entry else { continue };
        let path = entry.path();

        // Must have world.ron to be a world skill
        let ron_path = path.join("world.ron");
        if !ron_path.exists() {
            continue;
        }

        // Parse world.ron for metadata
        if let Ok(ron_str) = std::fs::read_to_string(&ron_path) {
            if let Ok(manifest) =
                ron::from_str::<localgpt_world_types::WorldManifest>(&ron_str)
            {
                let thumbnail_path = find_thumbnail(&path);
                let created_at = std::fs::metadata(&ron_path)
                    .ok()
                    .and_then(|m| m.created().ok())
                    .map(DateTime::<Utc>::from);

                entries.push(WorldGalleryEntry {
                    name: manifest.meta.name.clone(),
                    path: path.clone(),
                    description: manifest.meta.description.clone(),
                    entity_count: manifest.entities.len(),
                    created_at,
                    thumbnail_path,
                    style_tags: manifest.meta.tags.clone().unwrap_or_default(),
                    variation_group: manifest.meta.variation_group.clone(),
                    source: manifest
                        .meta
                        .source
                        .clone()
                        .unwrap_or_else(|| "interactive".to_string()),
                });
            }
        }
    }

    // Sort by creation date, newest first
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    entries
}

/// Find the best thumbnail for a world skill folder.
fn find_thumbnail(world_path: &Path) -> Option<PathBuf> {
    let screenshots_dir = world_path.join("screenshots");
    if screenshots_dir.exists() {
        // Return the most recent screenshot
        let mut screenshots: Vec<PathBuf> = std::fs::read_dir(&screenshots_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .is_some_and(|ext| ext == "png" || ext == "jpg")
            })
            .collect();
        screenshots.sort();
        screenshots.last().cloned()
    } else {
        None
    }
}
```

### 9.3 WorldManifest Metadata Extensions

Add fields to `WorldManifest.meta` to support gallery and experiment tracking:

```rust
// crates/world-types/src/lib.rs (extension to existing WorldMeta)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMeta {
    // ... existing fields ...
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    // New fields for gallery + experiments:

    /// Free-form style tags for gallery filtering
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Generation source: "interactive", "headless", "experiment", "mcp"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// If part of a variation experiment, the group ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variation_group: Option<String>,

    /// Variation axis and value (e.g., ("lighting", "sunset"))
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variation: Option<(String, String)>,

    /// Original prompt used to generate this world
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// LLM model used for generation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Generation duration in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_duration_ms: Option<u64>,

    /// Style name from memory (if applied)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_ref: Option<String>,
}
```

### 9.4 Gallery UI (egui overlay)

The gallery renders as an egui overlay in interactive gen mode, triggered by `/gallery` command or the `G` keybind.

```rust
// crates/gen/src/gen3d/gallery_ui.rs

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

use super::gallery::{WorldGalleryEntry, scan_world_gallery};
use super::plugin::GenWorkspace;

#[derive(Resource, Default)]
pub struct GalleryState {
    pub visible: bool,
    pub entries: Vec<WorldGalleryEntry>,
    pub filter: String,
    pub selected: Option<usize>,
    /// Loaded thumbnail textures (lazy-loaded)
    pub thumbnails: HashMap<std::path::PathBuf, egui::TextureHandle>,
}

pub fn gallery_ui_system(
    mut contexts: EguiContexts,
    mut gallery: ResMut<GalleryState>,
    workspace: Res<GenWorkspace>,
) {
    if !gallery.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("World Gallery")
        .resizable(true)
        .default_size([800.0, 600.0])
        .show(ctx, |ui| {
            // Filter bar
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut gallery.filter);
                if ui.button("Refresh").clicked() {
                    gallery.entries = scan_world_gallery(&workspace.path);
                }
            });

            ui.separator();

            // Grid of world cards
            let filtered: Vec<_> = gallery
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    gallery.filter.is_empty()
                        || e.name
                            .to_lowercase()
                            .contains(&gallery.filter.to_lowercase())
                        || e.style_tags.iter().any(|t| {
                            t.to_lowercase()
                                .contains(&gallery.filter.to_lowercase())
                        })
                })
                .collect();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (idx, entry) in &filtered {
                        ui.vertical(|ui| {
                            // Thumbnail placeholder (200x150)
                            let thumb_size = egui::vec2(200.0, 150.0);
                            if let Some(ref _thumb_path) = entry.thumbnail_path {
                                // TODO: Load and cache texture from PNG file
                                // For v1, show a colored placeholder
                                let (rect, _) = ui.allocate_exact_size(
                                    thumb_size,
                                    egui::Sense::click(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    4.0,
                                    egui::Color32::from_gray(60),
                                );
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    thumb_size,
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    4.0,
                                    egui::Color32::from_gray(40),
                                );
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "No Preview",
                                    egui::FontId::default(),
                                    egui::Color32::GRAY,
                                );
                            }

                            // World name
                            ui.strong(&entry.name);

                            // Metadata line
                            ui.label(format!(
                                "{} entities | {}",
                                entry.entity_count, entry.source
                            ));

                            // Tags
                            if !entry.style_tags.is_empty() {
                                ui.horizontal(|ui| {
                                    for tag in &entry.style_tags {
                                        ui.small(tag);
                                    }
                                });
                            }

                            // Load button
                            if ui.button("Load").clicked() {
                                gallery.selected = Some(*idx);
                                // TODO: Send gen_load_world command
                            }
                        });
                    }
                });
            });
        });
}

/// Toggle gallery visibility with G key.
pub fn gallery_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut gallery: ResMut<GalleryState>,
    workspace: Res<GenWorkspace>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        gallery.visible = !gallery.visible;
        if gallery.visible && gallery.entries.is_empty() {
            gallery.entries = scan_world_gallery(&workspace.path);
        }
    }
}
```

### 9.5 Gallery Keybinds and Commands

| Trigger | Action |
|---------|--------|
| `G` key (when not typing) | Toggle gallery overlay |
| `/gallery` slash command | Toggle gallery overlay |
| `/gallery refresh` | Rescan skills/ and reload entries |
| `/gallery filter <tag>` | Filter gallery by tag |
| Click "Load" on a card | Load world into scene via `gen_load_world` |

---

## 10. Phase 5: MCP Server Headless Mode

### 10.1 Objective

Allow external AI backends to drive headless generation via MCP, including batch experiment submission and status polling.

### 10.2 CLI Interface

```bash
# Headless MCP server (no window)
localgpt-gen mcp-server --headless

# Headless MCP server with specific GPU backend
localgpt-gen mcp-server --headless --gpu-backend vulkan
```

### 10.3 New MCP Tools

Three new tools are added to the MCP server when running in headless or standard mode:

```rust
// crates/gen/src/mcp/experiment_tools.rs

use serde_json::json;
use localgpt_core::agent::providers::ToolSchema;

/// Queue a world generation experiment for background processing.
pub struct GenQueueExperimentTool { /* ... */ }

// Tool schema for gen_queue_experiment
fn queue_experiment_schema() -> ToolSchema {
    ToolSchema {
        name: "gen_queue_experiment".to_string(),
        description: "Queue a world generation experiment. The experiment \
            will be processed in the background. Use gen_experiment_status \
            to check progress.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "World generation prompt"
                },
                "name": {
                    "type": "string",
                    "description": "World name (used for skill folder name)"
                },
                "style": {
                    "type": "string",
                    "description": "Optional style hint or memory reference"
                },
                "variations": {
                    "type": "object",
                    "description": "Optional variation spec",
                    "properties": {
                        "axis": {
                            "type": "string",
                            "description": "Variation axis (e.g., 'lighting', 'density')"
                        },
                        "values": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Values to try for each variation"
                        }
                    }
                },
                "screenshot": {
                    "type": "boolean",
                    "default": true,
                    "description": "Capture thumbnail after generation"
                }
            },
            "required": ["prompt", "name"]
        }),
    }
}

/// List all experiments and their statuses.
// Tool schema for gen_list_experiments
fn list_experiments_schema() -> ToolSchema {
    ToolSchema {
        name: "gen_list_experiments".to_string(),
        description: "List all queued, running, and completed experiments \
            with their statuses, paths, and thumbnails.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["all", "pending", "running", "completed", "failed"],
                    "default": "all"
                },
                "limit": {
                    "type": "integer",
                    "default": 20,
                    "description": "Max experiments to return"
                }
            }
        }),
    }
}

/// Get detailed status of a specific experiment.
// Tool schema for gen_experiment_status
fn experiment_status_schema() -> ToolSchema {
    ToolSchema {
        name: "gen_experiment_status".to_string(),
        description: "Get detailed status of a specific experiment by ID, \
            including output path, screenshot, entity count, and duration.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Experiment ID"
                }
            },
            "required": ["id"]
        }),
    }
}
```

### 10.4 MCP Headless Processing Loop

In headless MCP mode, experiments are processed sequentially between MCP message handling:

```rust
// crates/gen/src/mcp_server.rs (extended)

use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;

use crate::experiment::{ExperimentStatus, ExperimentTracker};
use crate::gen3d::GenBridge;

pub async fn run_headless_mcp_server(
    bridge: Arc<GenBridge>,
    config: localgpt_core::config::Config,
) -> anyhow::Result<()> {
    let tools = create_mcp_tools(bridge.clone(), &config)?;

    // Add experiment queue tools
    let tracker = Arc::new(ExperimentTracker::new(&config.paths.state_dir));
    let experiment_tools = create_experiment_tools(tracker.clone(), bridge.clone());
    let mut all_tools = tools;
    all_tools.extend(experiment_tools);

    // Spawn experiment processor on a background task
    let processor_bridge = bridge.clone();
    let processor_config = config.clone();
    let processor_tracker = tracker.clone();
    tokio::spawn(async move {
        experiment_processor_loop(
            processor_bridge,
            processor_config,
            processor_tracker,
        )
        .await;
    });

    localgpt_core::mcp::server::run_mcp_stdio_server(all_tools, "localgpt-gen").await
}

/// Background loop that processes pending experiments sequentially.
async fn experiment_processor_loop(
    bridge: Arc<GenBridge>,
    config: localgpt_core::config::Config,
    tracker: Arc<ExperimentTracker>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        let pending = match tracker.pending() {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to read pending experiments: {}", e);
                continue;
            }
        };

        if let Some(experiment) = pending.first() {
            tracing::info!("Processing experiment: {}", experiment.id);

            let mut exp = experiment.clone();
            exp.status = ExperimentStatus::Running;
            exp.started_at = Some(Utc::now());
            let _ = tracker.append(&exp);

            match run_single_experiment(&bridge, &config, &exp).await {
                Ok(result) => {
                    exp.status = ExperimentStatus::Completed;
                    exp.completed_at = Some(Utc::now());
                    exp.output_path = Some(result.world_path);
                    exp.screenshot_path = result.screenshot_path;
                    exp.entity_count = Some(result.entity_count);
                    exp.duration_ms = Some(result.duration_ms);
                    let _ = tracker.append(&exp);
                    tracing::info!(
                        "Experiment {} completed: {} entities",
                        exp.id,
                        result.entity_count
                    );
                }
                Err(e) => {
                    exp.status = ExperimentStatus::Failed;
                    exp.completed_at = Some(Utc::now());
                    exp.error = Some(e.to_string());
                    let _ = tracker.append(&exp);
                    tracing::error!("Experiment {} failed: {}", exp.id, e);
                }
            }
        }
    }
}

/// Result of a single experiment run.
struct ExperimentResult {
    world_path: String,
    screenshot_path: Option<String>,
    entity_count: usize,
    duration_ms: u64,
}

/// Run a single experiment against the headless Bevy instance.
async fn run_single_experiment(
    bridge: &Arc<GenBridge>,
    config: &localgpt_core::config::Config,
    experiment: &crate::experiment::Experiment,
) -> anyhow::Result<ExperimentResult> {
    use localgpt_core::agent::Agent;
    use localgpt_core::agent::tools::create_safe_tools;
    use localgpt_core::memory::MemoryManager;

    let start = std::time::Instant::now();

    // Clear existing scene
    bridge.send_command(crate::gen3d::GenCommand::ClearScene).await?;

    // Create an agent for this experiment
    let memory = MemoryManager::new_with_agent(&config.memory, "gen-experiment")?;
    let memory = Arc::new(memory);
    let mut tools = create_safe_tools(config, Some(memory.clone()))?;
    tools.extend(crate::gen3d::tools::create_gen_tools(bridge.clone()));

    let mut agent = Agent::new_with_tools(
        config.clone(),
        "gen-experiment",
        memory,
        tools,
    )?;
    agent.new_session().await?;

    // Build prompt with optional style
    let prompt = if let Some(ref style) = experiment.style {
        format!(
            "Style: {}. Build the following world: {}",
            style, experiment.prompt
        )
    } else {
        experiment.prompt.clone()
    };

    // Generate
    let _response = agent.chat(&prompt).await?;

    // Save world
    let world_name = experiment
        .id
        .split('-')
        .skip(3)
        .collect::<Vec<_>>()
        .join("-");
    let save_prompt = format!(
        "Save this world with gen_save_world. Name: \"{}\"",
        world_name
    );
    let _save_response = agent.chat(&save_prompt).await?;

    // Get final entity count
    let scene_info = bridge.send_command(crate::gen3d::GenCommand::SceneInfo).await?;
    let entity_count = match &scene_info {
        crate::gen3d::GenResponse::SceneInfo { entities, .. } => entities.len(),
        _ => 0,
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let world_path = config
        .workspace_path()
        .join("skills")
        .join(&world_name)
        .to_string_lossy()
        .into_owned();

    Ok(ExperimentResult {
        world_path,
        screenshot_path: None, // TODO: wire offscreen screenshot
        entity_count,
        duration_ms,
    })
}
```

---

## 11. Risk Analysis

### 11.1 Technical Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Bevy headless rendering instability | High | Medium | Start with `MinimalPlugins` (no rendering) for tool-call-only gen; add offscreen rendering incrementally. Screenshot capture can be a Phase 1.5 feature. |
| GPU contention between interactive and headless | High | Low (by design) | GPU lockfile prevents concurrent instances. Heartbeat skips if lock held. |
| Offscreen rendering on headless Linux (no display) | Medium | Medium | wgpu supports Vulkan without display via `VK_KHR_headless_surface`. Fall back to software rendering (llvmpipe) as last resort. Test in CI with `DISPLAY=`. |
| LLM generates invalid/incomplete worlds in headless | Medium | Medium | Timeout safety net (default 5m). Validation pass on `gen_save_world`. Retry logic with different prompt temperature on failure. |
| Experiment tracker JSONL grows unbounded | Low | High | Compact on startup: keep last 1000 entries, archive older ones. State file, so deletion is safe. |
| Memory pollution between gen and CLI modes | Medium | Medium | Section headers in MEMORY.md provide soft partitioning. Phase 2 optimization: separate GEN_MEMORY.md if needed. |

### 11.2 Product Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Users don't discover async generation | Medium | First-run tutorial suggests it. `/help` lists experiment commands. HEARTBEAT.md template includes a gen experiment example. |
| Experiment results are low quality without user feedback loop | Medium | Capture screenshot + scene info for quick review. Memory accumulates style preferences over time, improving quality. |
| Gallery UI is too basic for serious comparison | Low | Gallery v1 is functional (thumbnails + load). v2 can add side-by-side comparison, diff view, and rating system. |

### 11.3 Platform-Specific Risks

| Platform | Risk | Mitigation |
|----------|------|------------|
| macOS | Bevy requires main thread for windowing/GPU even in headless | Headless mode still runs Bevy on main thread, just without a window. Agent loop on background thread (same pattern as interactive). |
| Linux (headless server) | No GPU available | Software rendering fallback via llvmpipe. Document minimum GPU requirements. |
| Mobile (future) | No headless gen possible | Mobile clients submit experiments to a local daemon or desktop instance. Out of scope for this RFC. |

---

## 12. XDG File Layout Changes

New files introduced by this RFC, classified per the XDG RFC:

| File | XDG Category | Location | Rationale |
|------|-------------|----------|-----------|
| `gen-experiments.jsonl` | **State** | `<state_dir>/gen-experiments.jsonl` | Device-specific tracking, not portable |
| `gen-gpu.lock` | **Runtime** | `<runtime_dir>/localgpt/gen-gpu.lock` | Ephemeral lock, cleared at logout |
| World skill thumbnails | **Data** | `<data_dir>/workspace/skills/<name>/screenshots/` | User-valuable outputs |
| `GEN_MEMORY.md` (future) | **Data** | `<data_dir>/workspace/GEN_MEMORY.md` | User-curated creative knowledge |

All existing files (MEMORY.md, HEARTBEAT.md, daily logs, world.ron) retain their current locations and classifications.

---

## 13. Configuration

New `config.toml` sections:

```toml
[gen]
# Default screenshot dimensions for headless generation
screenshot_width = 1280
screenshot_height = 720

# Maximum generation time before timeout (headless mode)
headless_timeout = "5m"

# Maximum concurrent experiments (always 1 for GPU exclusivity, future: multi-GPU)
max_concurrent_experiments = 1

[gen.experiments]
# Enable experiment processing in heartbeat
enabled = true

# Maximum experiment history entries before compaction
max_history = 1000

# Auto-capture screenshot after each headless generation
auto_screenshot = true

[gen.memory]
# Use separate GEN_MEMORY.md instead of shared MEMORY.md
# Default: false (shared MEMORY.md with section headers)
separate_file = false

# Maximum entity templates to keep in memory
max_entity_templates = 50

# Maximum style entries to keep in memory
max_styles = 20
```

---

## 14. Implementation Phases and Timeline

### Phase 1: Headless Bevy Gen Mode (Weeks 1–2)

**Goal:** `localgpt-gen --headless --prompt "..."` works end-to-end.

| Task | Effort | Depends on |
|------|--------|------------|
| Add `--headless` CLI flag and arg parsing | 2h | — |
| Implement `run_headless_bevy_app` (no window, `DontExit`) | 4h | — |
| Wire `HeadlessCompletionFlag` between agent thread and Bevy thread | 2h | — |
| Implement timeout watchdog | 1h | — |
| Add exit code contract | 1h | — |
| Test: headless gen produces valid `world.ron` | 2h | All above |
| **Subtotal** | **~12h** | |

### Phase 1.5: Offscreen Screenshots (Week 2)

| Task | Effort | Depends on |
|------|--------|------------|
| Implement `OffscreenRenderTarget` resource and setup system | 4h | Phase 1 |
| Implement `capture_offscreen_screenshot` (render target → PNG) | 3h | Offscreen target |
| Wire screenshot into headless gen pipeline (after save, before exit) | 2h | Screenshot capture |
| Test: headless gen produces valid PNG thumbnail | 2h | All above |
| Fallback: software rendering on headless Linux | 3h | Screenshot capture |
| **Subtotal** | **~14h** | |

### Phase 2: Experiment Queue + Heartbeat (Weeks 3–4)

| Task | Effort | Depends on |
|------|--------|------------|
| Define `Experiment` struct and `ExperimentTracker` | 3h | — |
| Implement `GpuLock` (file-based, `fs2::try_lock_exclusive`) | 2h | — |
| Implement `has_gen_experiments` detection for HEARTBEAT.md | 2h | — |
| Create `create_headless_gen_tool_factory` | 4h | Phase 1 |
| Extend `HeartbeatRunner` to dispatch gen experiments | 6h | Tool factory + GPU lock |
| Implement variation expansion (single entry → N experiments) | 3h | Experiment struct |
| HEARTBEAT.md check-off after experiment completion | 2h | Heartbeat dispatch |
| Test: heartbeat processes a gen experiment end-to-end | 3h | All above |
| **Subtotal** | **~25h** | |

### Phase 3: Gen-Specific Memory (Weeks 4–5)

| Task | Effort | Depends on |
|------|--------|------------|
| Write `GEN_MEMORY_PROMPT` system prompt overlay | 2h | — |
| Wire gen system prompt into agent creation for gen mode | 1h | System prompt |
| Override `save_session_to_memory` for gen sessions | 4h | — |
| Define entity template memory format (markdown conventions) | 1h | — |
| Define style preference memory format (markdown conventions) | 1h | — |
| Define experiment result memory format (markdown conventions) | 1h | — |
| Test: LLM saves entity template to memory, retrieves in new session | 3h | All above |
| Test: experiment results are logged as structured memory | 2h | Phase 2 + memory |
| **Subtotal** | **~15h** | |

### Phase 4: Gallery UI (Weeks 5–6)

| Task | Effort | Depends on |
|------|--------|------------|
| Implement `scan_world_gallery` filesystem scanner | 3h | — |
| Extend `WorldMeta` with gallery fields (tags, source, variation) | 2h | — |
| Implement egui gallery overlay (`gallery_ui_system`) | 8h | Gallery scanner |
| Implement thumbnail loading and caching in egui | 4h | Gallery overlay |
| Wire `/gallery` command and `G` keybind | 1h | Gallery overlay |
| Wire "Load" button to `gen_load_world` | 2h | Gallery overlay |
| Test: gallery shows all generated worlds with thumbnails | 2h | All above |
| **Subtotal** | **~22h** | |

### Phase 5: MCP Server Headless Mode (Weeks 6–7)

| Task | Effort | Depends on |
|------|--------|------------|
| Add `--headless` flag to `mcp-server` subcommand | 1h | Phase 1 |
| Implement `gen_queue_experiment` MCP tool | 3h | Phase 2 |
| Implement `gen_list_experiments` MCP tool | 2h | Phase 2 |
| Implement `gen_experiment_status` MCP tool | 2h | Phase 2 |
| Implement `experiment_processor_loop` for MCP mode | 4h | Phase 2 |
| Test: external MCP client queues and retrieves experiments | 3h | All above |
| **Subtotal** | **~15h** | |

### Total Estimated Effort

| Phase | Hours | Weeks (solo dev) |
|-------|-------|-------------------|
| Phase 1: Headless gen | ~12h | 1 |
| Phase 1.5: Offscreen screenshots | ~14h | 1 |
| Phase 2: Experiment queue | ~25h | 2 |
| Phase 3: Memory integration | ~15h | 1 |
| Phase 4: Gallery UI | ~22h | 1.5 |
| Phase 5: MCP headless | ~15h | 1 |
| **Total** | **~103h** | **~7.5 weeks** |

---

## 15. Future Extensions (Out of Scope)

These are natural follow-ups but are explicitly out of scope for this RFC:

- **Distributed generation**: Multiple machines processing experiments in parallel
- **Cloud experiment queue**: Submit from mobile, generate on desktop
- **A/B testing UI**: Side-by-side comparison view with preference tracking
- **Auto-evolution**: Genetic algorithm over world parameters using experiment results
- **Style transfer**: Apply one world's style to another world's layout
- **Template marketplace**: Share entity templates and styles between users
- **CI/CD integration**: GitHub Actions workflow for world generation pipelines
- **Notification system**: Push notification when experiments complete (Telegram bot, system notification)

---

## 16. Open Questions

1. **Bevy headless rendering maturity.** The `headless_renderer` example exists but is not heavily tested in production. Should Phase 1 ship without screenshots (tool-calls-only headless) and add rendering in 1.5?

2. **Single vs persistent headless Bevy.** Should each experiment boot a fresh Bevy app (clean isolation, ~200-500ms overhead) or reuse a persistent instance with `gen_clear_scene` between experiments? Fresh boot is safer; persistent is faster.

3. **Memory file separation.** Should gen memory use a separate `GEN_MEMORY.md` from day one, or start in shared MEMORY.md with section headers and split later if needed?

4. **Gallery framework.** Should the gallery use egui (consistent with existing gen UI), or should it be a separate web UI served via the existing HTTP server for richer thumbnail rendering?

5. **Experiment priority.** Should HEARTBEAT.md support experiment priority (e.g., `(priority: high)`)? Or is FIFO sufficient for v1?

---

## 17. References

- Bevy `headless_renderer` example: `examples/app/headless_renderer.rs`
- Bevy offscreen rendering discussion: `bevyengine/bevy#1053`
- wgpu headless surface: `VK_KHR_headless_surface`
- Existing gen save/load: `crates/gen/src/gen3d/world.rs`
- Existing heartbeat runner: `crates/core/src/heartbeat/runner.rs`
- Existing MCP server: `crates/gen/src/mcp_server.rs`
- XDG layout RFC: `RFC-XDG-Directory-Layout.md`
- SpacetimeDB data model RFC: `RFC-SpacetimeDB-3D-Audio-Data-Model.md`
