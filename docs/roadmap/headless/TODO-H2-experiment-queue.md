# Headless 2: Experiment Queue & Heartbeat Integration

**Turns headless gen into an autonomous pipeline.** Users add experiments to HEARTBEAT.md in natural language; the heartbeat runner parses, dispatches, and logs results while the user is away.

**Source:** RFC-Headless-Gen-Experiment-Pipeline.md, Phase 2 (Section 7)

**Dependencies:** H1 (Headless Bevy Gen Mode), H1.5 (Offscreen Screenshots)

**Priority:** 3 of 6 (~25h)

---

## Spec H2.1: `Experiment` Struct and `ExperimentTracker`

**Goal:** Define the experiment data model and append-only JSONL tracker for machine-readable state.

### Data Model

```rust
pub struct Experiment {
    pub id: String,                              // "exp-20260316-143022-enchanted-forest"
    pub prompt: String,                          // original prompt
    pub style: Option<String>,                   // style hint or memory reference
    pub status: ExperimentStatus,                // Pending | Running | Completed | Failed | Cancelled
    pub output_path: Option<String>,             // world skill path
    pub screenshot_path: Option<String>,         // thumbnail path
    pub entity_count: Option<usize>,             // final entity count
    pub duration_ms: Option<u64>,                // generation time
    pub error: Option<String>,                   // failure reason
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub variation_group: Option<String>,          // group ID for variation sets
    pub variation: Option<(String, String)>,      // axis + value
    pub model: Option<String>,                   // LLM model used
}
```

### Implementation

1. File location: `<state_dir>/gen-experiments.jsonl` (XDG state — device-specific)
2. `append()`: serialize experiment as JSON, append line to file
3. `read_all()`: read all lines, deduplicate by ID (last entry wins), sort by `queued_at`
4. `pending()`: filter for `ExperimentStatus::Pending`
5. ID format: `exp-YYYYMMDD-HHMMSS-<slug>` from prompt

### Acceptance Criteria

- [ ] Experiments can be appended and read back correctly
- [ ] Deduplication works (updating status appends new line, read_all returns latest)
- [ ] File is valid JSONL (one JSON object per line)

---

## Spec H2.2: `GpuLock` — File-Based GPU Exclusivity

**Goal:** Prevent concurrent Bevy instances from fighting over the GPU.

### Implementation

1. Lock file at `<runtime_dir>/localgpt/gen-gpu.lock` (XDG runtime — ephemeral)
2. Use `fs2::FileExt::try_lock_exclusive` for non-blocking acquisition
3. Write PID to lock file for diagnostics
4. Lock released automatically when `GpuLockGuard` (holding the `File`) is dropped
5. `is_locked()` check for skip-if-busy logic

### Acceptance Criteria

- [ ] Only one headless gen instance can hold the lock at a time
- [ ] `try_acquire()` returns `None` when lock is held by another process
- [ ] Lock is released when the process exits (even on crash — OS releases file lock)

---

## Spec H2.3: `has_gen_experiments` — HEARTBEAT.md Detection

**Goal:** Detect whether HEARTBEAT.md contains gen experiment entries that need dispatching.

### Detection Heuristics

Match if content contains:
- `## Gen Experiments` or `## World Experiments` section header
- Unchecked items (`- [ ]`) with gen-like verbs: "build a", "generate a", "create a world", "variation"

### Implementation

1. Case-insensitive string matching on heartbeat content
2. Return `bool` — detection only, not parsing (parsing happens in heartbeat runner)

### Acceptance Criteria

- [ ] Detects `## Gen Experiments` section
- [ ] Detects unchecked items with generation verbs
- [ ] Does not false-positive on regular CLI tasks like "build the project"

---

## Spec H2.4: `create_headless_gen_tool_factory`

**Goal:** Factory function that creates gen tools backed by a headless Bevy instance, for use by the heartbeat runner.

### Implementation

1. Create `GenChannels` (bridge between agent and Bevy)
2. Spawn headless Bevy on a dedicated thread via `run_headless_bevy_app`
3. Wait for Bevy initialization (~500ms or channel handshake)
4. Return all gen tool sets: core tools, avatar tools, interaction tools, terrain tools, UI tools, physics tools
5. Bevy thread shuts down when the returned bridge is dropped

### Acceptance Criteria

- [ ] Factory returns a complete set of gen tools
- [ ] Tools are functional (can issue gen commands and get responses)
- [ ] Bevy thread exits cleanly when tools/bridge are dropped

---

## Spec H2.5: Heartbeat Runner Gen Experiment Dispatch

**Goal:** Extend `HeartbeatRunner` to process gen experiments from HEARTBEAT.md.

### Experiment Lifecycle per Tick

1. Check `has_gen_experiments` on current HEARTBEAT.md content
2. If yes, check GPU lock — if locked, skip to next tick
3. Acquire GPU lock
4. Parse next unchecked experiment entry from HEARTBEAT.md
5. Create experiment record (status: Pending → Running)
6. Boot headless Bevy via tool factory
7. Create agent with gen tools + memory tools
8. Search memory for user style preferences
9. Send prompt to LLM, let it issue tool calls
10. After LLM finishes: `gen_save_world`, capture screenshot
11. Update experiment tracker (status: Completed/Failed)
12. Log results to daily memory
13. Check off the HEARTBEAT.md entry (or mark with error)
14. Shut down headless Bevy, release GPU lock

### Key Constraint

Process ONE experiment per heartbeat tick, not all at once. Respects the heartbeat interval for interleaving with other tasks.

### Acceptance Criteria

- [ ] Heartbeat runner detects and processes gen experiments
- [ ] One experiment per tick
- [ ] GPU lock acquired before gen, released after
- [ ] HEARTBEAT.md entry checked off on success
- [ ] Failed experiments marked with error in HEARTBEAT.md
- [ ] Experiment tracker updated at each status transition

---

## Spec H2.6: Variation Expansion

**Goal:** A single HEARTBEAT.md entry with variation syntax expands into N separate experiments.

### Syntax

```
- [ ] Medieval village with 3 lighting variations (variation: lighting = dawn, noon, sunset)
```

Expands to:
- `Medieval village — lighting: dawn`
- `Medieval village — lighting: noon`
- `Medieval village — lighting: sunset`

### Implementation

1. Parse `(variation: <axis> = <v1>, <v2>, ...)` from the prompt text
2. Generate N experiment records sharing the same `variation_group` ID
3. Each gets the base prompt with the variation axis/value appended
4. Each produces a separate world skill with name suffix (e.g., `village-dawn`, `village-noon`)

### Acceptance Criteria

- [ ] Variation syntax is parsed correctly
- [ ] N experiments created with shared variation_group
- [ ] Each variation has the correct axis and value recorded
- [ ] Output world names include the variation value

---

## Spec H2.7: End-to-End Heartbeat Experiment Test

**Goal:** Validate the full pipeline: HEARTBEAT.md entry → heartbeat tick → headless gen → world save → check-off.

### Test Plan

1. Write a gen experiment entry to HEARTBEAT.md
2. Trigger a heartbeat tick
3. Assert: GPU lock was acquired and released
4. Assert: world skill directory exists with `world.ron`
5. Assert: experiment tracker has a Completed record
6. Assert: HEARTBEAT.md entry is checked off (`- [x]`)
7. Assert: daily memory log contains experiment summary

### Acceptance Criteria

- [ ] Full lifecycle works end-to-end
- [ ] No resource leaks (GPU lock, Bevy thread)
