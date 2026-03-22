# Headless 1: Headless Bevy Gen Mode

**Foundation for all async generation.** Enables `localgpt-gen --headless --prompt "..."` to generate a complete world skill without opening a window. Every subsequent phase depends on this.

**Source:** RFC-Headless-Gen-Experiment-Pipeline.md, Phase 1 (Section 6)

**Dependencies:** Gen mode MCP tools (P1–P5), `gen_save_world`/`gen_load_world`

**Priority:** 1 of 6 — do this first (~12h)

---

## Spec H1.1: `--headless` CLI Flag and Arg Parsing

**Goal:** Add `--headless` flag to `localgpt-gen` with all supporting options from Section 6.2.

### CLI Interface

```
localgpt-gen --headless [OPTIONS]

Options:
  --prompt <TEXT>         Initial generation prompt
  --output <PATH>         Output world skill directory (default: auto-named in workspace/skills/)
  --screenshot            Capture a thumbnail after generation (default: true)
  --screenshot-width <N>  Screenshot width in pixels (default: 1280)
  --screenshot-height <N> Screenshot height in pixels (default: 720)
  --timeout <DURATION>    Max generation time before abort (default: 5m)
  --agent <ID>            Agent ID for memory isolation (default: "gen-headless")
  --model <MODEL>         Override LLM model for this run
  --style <TEXT>          Style hint prepended to prompt
```

### Implementation

1. Add clap args to the existing gen CLI parser
2. Route `--headless` to `run_headless_bevy_app` instead of `run_bevy_app`
3. Validate: `--prompt` is required when `--headless` is set
4. Default `--output` to `workspace/skills/<auto-name>/` using prompt slug + timestamp

### Acceptance Criteria

- [ ] `localgpt-gen --headless --prompt "test"` is accepted by the CLI parser
- [ ] `localgpt-gen --headless` without `--prompt` prints an error
- [ ] All options parse correctly and are passed to the headless runtime

---

## Spec H1.2: `run_headless_bevy_app` — Windowless Bevy Bootstrap

**Goal:** Boot Bevy with full render pipeline but no visible window. Reuse the same `GenPlugin` as interactive mode.

### Implementation

1. Use `DefaultPlugins` with `WindowPlugin { primary_window: None, exit_condition: DontExit }`
2. Set `wgpu::Backends::all()` for software rendering fallback on headless servers
3. Set `AssetPlugin { file_path: "/" }` for absolute path asset loading
4. Disable `LogPlugin` (gen uses tracing directly)
5. Call `gen3d::plugin::setup_gen_app(&mut app, channels, workspace, None)`
6. Add `headless_completion_detector` system

### Key Constraint

Bevy must run on the main thread (macOS requirement). The agent loop runs on a background thread, same pattern as interactive mode.

### Acceptance Criteria

- [ ] Headless Bevy boots without creating a window
- [ ] GenPlugin initializes successfully (all tools available)
- [ ] Agent loop can issue gen tool calls against the headless Bevy instance
- [ ] App exits cleanly when generation completes

---

## Spec H1.3: `HeadlessCompletionFlag` — Agent-to-Bevy Shutdown Signal

**Goal:** Shared atomic flag that the agent thread sets when generation is complete, causing Bevy to exit gracefully.

### Implementation

```rust
#[derive(Resource, Clone)]
pub struct HeadlessCompletionFlag {
    pub done: Arc<AtomicBool>,
    pub success: Arc<AtomicBool>,
}
```

1. Insert as Bevy resource before `app.run()`
2. Clone into agent thread
3. Agent sets `done = true` after: LLM finishes, `gen_save_world` called, screenshot captured
4. `headless_completion_detector` system checks flag each frame, sends `AppExit::Success`

### Acceptance Criteria

- [ ] Agent completing generation triggers Bevy exit within 1 frame
- [ ] Exit code reflects success/failure from the flag
- [ ] No race condition between flag set and Bevy shutdown

---

## Spec H1.4: Timeout Watchdog

**Goal:** Prevent headless gen from running forever if the LLM loops or hangs.

### Implementation

1. Spawn a watchdog thread at startup
2. Sleep for `--timeout` duration (default 5m)
3. If still running: set `HeadlessCompletionFlag.done = true`, `success = false`
4. Log timeout error to stderr

### Acceptance Criteria

- [ ] Generation that exceeds timeout is terminated
- [ ] Exit code is 1 (failure) on timeout
- [ ] Timeout value is configurable via `--timeout`

---

## Spec H1.5: Exit Code Contract

**Goal:** Deterministic exit codes for scripting and CI integration.

| Exit code | Meaning |
|-----------|---------|
| 0 | World generated and saved successfully |
| 1 | Generation failed (LLM error, tool error, timeout) |
| 2 | Invalid arguments or configuration |
| 3 | GPU not available (headless server without compatible GPU) |

### Implementation

1. Map `HeadlessCompletionFlag.success` to exit code 0 vs 1
2. Catch argument validation errors → exit code 2
3. Catch wgpu initialization failure → exit code 3
4. Use `std::process::exit()` after Bevy returns

### Acceptance Criteria

- [ ] Successful generation exits with code 0
- [ ] Failed generation exits with code 1
- [ ] Invalid args exit with code 2
- [ ] GPU failure exits with code 3

---

## Spec H1.6: End-to-End Headless Gen Test

**Goal:** Validate the full pipeline: CLI → headless Bevy → agent → tool calls → world save → exit.

### Test Plan

1. `localgpt-gen --headless --prompt "Build a red cube" --output /tmp/test-world/`
2. Assert exit code 0
3. Assert `/tmp/test-world/world.ron` exists and is valid RON
4. Assert entity count > 0 in the world manifest
5. Assert stdout contains progress messages

### Acceptance Criteria

- [ ] Headless gen produces a valid `world.ron` with at least one entity
- [ ] Process exits with code 0
- [ ] No window is created during generation
