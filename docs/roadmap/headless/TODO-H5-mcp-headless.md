# Headless 5: MCP Server Headless Mode

**Enables external AI backends to drive batch generation.** Claude CLI, Gemini CLI, Codex, and other MCP clients can submit experiment batches via `gen_queue_experiment`, poll status, and retrieve results — all without a visible window.

**Source:** RFC-Headless-Gen-Experiment-Pipeline.md, Phase 5 (Section 10)

**Dependencies:** H1 (Headless Bevy Gen), H2 (Experiment Queue)

**Priority:** 6 of 6 (~15h)

---

## Spec H5.1: `--headless` Flag for MCP Server

**Goal:** `localgpt-gen mcp-server --headless` runs the MCP server without creating a window.

### CLI Interface

```bash
localgpt-gen mcp-server --headless
localgpt-gen mcp-server --headless --gpu-backend vulkan
```

### Implementation

1. Add `--headless` flag to the `mcp-server` subcommand
2. When set, boot Bevy via `run_headless_bevy_app` instead of `run_bevy_app`
3. All existing MCP tools work identically (same GenBridge, same tool implementations)
4. Additionally register experiment queue tools (H5.2–H5.4)

### Acceptance Criteria

- [ ] MCP server starts without a window when `--headless` is passed
- [ ] All existing gen MCP tools function correctly in headless mode
- [ ] MCP stdio transport works (stdin/stdout for tool calls)

---

## Spec H5.2: `gen_queue_experiment` MCP Tool

**Goal:** Queue a world generation experiment for background processing.

### MCP Tool Schema

```json
{
  "name": "gen_queue_experiment",
  "parameters": {
    "prompt": { "type": "string", "required": true },
    "name": { "type": "string", "required": true },
    "style": { "type": "string", "optional": true },
    "variations": {
      "type": "object", "optional": true,
      "properties": {
        "axis": { "type": "string" },
        "values": { "type": "array", "items": { "type": "string" } }
      }
    },
    "screenshot": { "type": "boolean", "default": true }
  }
}
```

### Implementation

1. Create `Experiment` record(s) with status `Pending`
2. If `variations` specified, expand into N experiments (same as H2.6)
3. Append to experiment tracker
4. Return experiment ID(s) for status polling

### Acceptance Criteria

- [ ] Queuing returns experiment ID(s) immediately
- [ ] Experiments appear in tracker with `Pending` status
- [ ] Variation expansion creates N separate experiments
- [ ] Style hint is stored in experiment record

---

## Spec H5.3: `gen_list_experiments` MCP Tool

**Goal:** List experiments with optional status filter.

### MCP Tool Schema

```json
{
  "name": "gen_list_experiments",
  "parameters": {
    "status": { "type": "string", "enum": ["all", "pending", "running", "completed", "failed"], "default": "all" },
    "limit": { "type": "integer", "default": 20 }
  }
}
```

### Implementation

1. Read all experiments from tracker
2. Filter by status if specified
3. Limit results
4. Return: id, prompt, status, output_path, screenshot_path, entity_count, duration_ms

### Acceptance Criteria

- [ ] Lists experiments with correct metadata
- [ ] Status filter works correctly
- [ ] Limit parameter caps results

---

## Spec H5.4: `gen_experiment_status` MCP Tool

**Goal:** Get detailed status of a specific experiment by ID.

### MCP Tool Schema

```json
{
  "name": "gen_experiment_status",
  "parameters": {
    "id": { "type": "string", "required": true }
  }
}
```

### Implementation

1. Look up experiment by ID in tracker
2. Return full experiment record including all fields
3. Return error if ID not found

### Acceptance Criteria

- [ ] Returns complete experiment details for valid ID
- [ ] Returns error for unknown ID
- [ ] Status reflects current state (pending/running/completed/failed)

---

## Spec H5.5: `experiment_processor_loop` — Background Processing

**Goal:** Background async loop that processes pending experiments sequentially.

### Implementation

1. Run as a `tokio::spawn` task alongside the MCP server
2. Poll every 5 seconds for pending experiments
3. Process one at a time (GPU exclusivity via GpuLock)
4. For each experiment:
   - Clear scene (`gen_clear_scene`)
   - Create agent with gen tools + memory tools
   - Send prompt, let LLM generate
   - Save world, capture screenshot
   - Update tracker status
5. Continue polling after each experiment completes

### Key Constraint

MCP message handling continues while experiments process. Clients can queue new experiments and poll status concurrently with an active generation.

### Acceptance Criteria

- [ ] Pending experiments are picked up and processed automatically
- [ ] MCP server remains responsive during experiment processing
- [ ] Experiments run sequentially (one at a time)
- [ ] Failed experiments don't block the queue

---

## Spec H5.6: End-to-End MCP Headless Test

**Goal:** Validate: external MCP client queues experiments, polls status, and retrieves results.

### Test Plan

1. Start `localgpt-gen mcp-server --headless`
2. Send `gen_queue_experiment` via MCP (prompt: "Build a red cube", name: "test-cube")
3. Poll `gen_experiment_status` until status is `Completed`
4. Assert: output_path points to a valid world skill
5. Assert: screenshot_path points to a valid PNG
6. Assert: entity_count > 0
7. Send `gen_list_experiments` and verify the experiment appears

### Acceptance Criteria

- [ ] External MCP client can queue, poll, and retrieve experiment results
- [ ] Full lifecycle works through the MCP protocol
