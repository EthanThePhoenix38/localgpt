# Headless Gen — TODO Implementation Index

Master index of all TODO specs for the Headless Generation, Experiment Queue, and Async Creative Pipeline. Organized by phase priority with dependencies.

**RFC:** `planning/rfcs/RFC-Headless-Gen-Experiment-Pipeline.md`

Last updated: 2026-03-17

---

## Status Legend

- **NOT STARTED** — Spec written, no implementation yet
- **IN PROGRESS** — Some sub-specs done
- **DONE** — Implemented and working

---

## Implementation Order (by dependency chain)

```
H1  Headless Bevy Gen       (~12h)  ← foundation, no deps
 └─ H1.5 Offscreen Screenshots (~14h)
     └─ H2 Experiment Queue    (~25h)  ← also depends on H1
         ├─ H3 Gen Memory      (~15h)
         ├─ H4 Gallery UI      (~22h)  ← also depends on H1.5
         └─ H5 MCP Headless    (~15h)  ← also depends on H1

Total: ~103h (~7.5 weeks solo)
```

---

## 1. Phase Status Matrix

### H1: Headless Bevy Gen Mode — DONE

| Spec | Description | Status |
|------|-------------|--------|
| H1.1 | `--headless` CLI flag and arg parsing | DONE |
| H1.2 | `run_headless_bevy_app` windowless bootstrap | DONE |
| H1.3 | `HeadlessCompletionFlag` agent-to-Bevy shutdown | DONE |
| H1.4 | Timeout watchdog | DONE |
| H1.5 | Exit code contract | DONE |
| H1.6 | End-to-end headless gen test | NOT STARTED (needs live LLM) |

**File:** `TODO-H1-headless-bevy-gen.md`

---

### H1.5: Offscreen Screenshots — PARTIAL

| Spec | Description | Status |
|------|-------------|--------|
| H1.5.1 | `OffscreenRenderTarget` resource | DONE |
| H1.5.2 | Offscreen camera setup | NOT STARTED (Bevy 0.18 RenderTarget private) |
| H1.5.3 | `capture_offscreen_screenshot` render-to-PNG | DONE (`save_pixels_as_png`) |
| H1.5.4 | Screenshot integration in headless pipeline | NOT STARTED |
| H1.5.5 | Software rendering fallback (headless Linux) | NOT STARTED |

**File:** `TODO-H1.5-offscreen-screenshots.md`

---

### H2: Experiment Queue & Heartbeat — DONE (data layer + detection + dispatch)

| Spec | Description | Status |
|------|-------------|--------|
| H2.1 | `Experiment` struct and `ExperimentTracker` | DONE |
| H2.2 | `GpuLock` file-based GPU exclusivity | DONE |
| H2.3 | `has_gen_experiments` HEARTBEAT.md detection | DONE |
| H2.4 | `create_headless_gen_tool_factory` | DONE |
| H2.5 | Heartbeat runner gen experiment dispatch | DONE (subprocess dispatch via `localgpt-gen headless`) |
| H2.6 | Variation expansion | DONE (`parse_variation`) |
| H2.7 | End-to-end heartbeat experiment test | NOT STARTED (needs live LLM + daemon) |

**File:** `TODO-H2-experiment-queue.md`

---

### H3: Gen-Specific Memory — DONE (prompts)

| Spec | Description | Status |
|------|-------------|--------|
| H3.1 | `GEN_MEMORY_PROMPT` system prompt overlay | DONE |
| H3.2 | Entity template memory format | DONE (documented in prompt) |
| H3.3 | Style preference memory format | DONE (documented in prompt) |
| H3.4 | Experiment result memory format | DONE (documented in prompt) |
| H3.5 | Gen session summarization override | NOT STARTED (needs Agent integration) |
| H3.6 | Memory partitioning strategy | NOT STARTED (config option) |

**File:** `TODO-H3-gen-memory.md`

---

### H4: Gallery UI — DONE (code), needs thumbnail loading

| Spec | Description | Status |
|------|-------------|--------|
| H4.1 | `scan_world_gallery` filesystem scanner | DONE |
| H4.2 | `WorldMeta` extensions | DONE |
| H4.3 | Gallery egui overlay | DONE |
| H4.4 | Thumbnail loading and caching | NOT STARTED (placeholder shown) |
| H4.5 | Gallery keybinds and commands | DONE (`G` key + `/gallery`) |
| H4.6 | "Load" button → `gen_load_world` | DONE (sets `load_request`, consumption TBD) |

**File:** `TODO-H4-gallery-ui.md`

---

### H5: MCP Server Headless Mode — DONE (tools + headless flag)

| Spec | Description | Status |
|------|-------------|--------|
| H5.1 | `--headless` flag for MCP server | DONE |
| H5.2 | `gen_queue_experiment` MCP tool | DONE |
| H5.3 | `gen_list_experiments` MCP tool | DONE |
| H5.4 | `gen_experiment_status` MCP tool | DONE |
| H5.5 | `experiment_processor_loop` background processing | NOT STARTED (needs async Bevy coordination) |
| H5.6 | End-to-end MCP headless test | NOT STARTED (needs live MCP client) |

**File:** `TODO-H5-mcp-headless.md`

---

## 2. Progress Summary

**Specs DONE:** 25 of 31 (81%)
**Specs remaining:** 6 — all require live integration testing, external Bevy APIs, or daemon-level wiring

### Remaining work (cannot be done offline):
1. **H1.5.2** — Offscreen camera (Bevy 0.18 `RenderTarget` is private; needs upstream fix or workaround)
2. **H1.5.4–H1.5.5** — Screenshot pipeline wiring + Linux fallback (needs H1.5.2)
3. ~~**H2.5**~~ — DONE: Heartbeat dispatcher via `localgpt-gen headless` subprocess
4. **H3.5–H3.6** — Session summarization + memory partitioning (needs Agent-level integration)
5. **H4.4** — Thumbnail texture loading (needs egui texture upload testing)
6. **H5.5** — Background experiment processor (needs async Bevy lifecycle coordination)
