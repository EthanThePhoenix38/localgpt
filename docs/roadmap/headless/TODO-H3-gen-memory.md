# Headless 3: Gen-Specific Memory Integration

**Highest ROI for generation quality.** Redesigns how memory works in gen mode so the AI accumulates world-building knowledge: style preferences, entity templates, and experiment outcomes. Mostly prompt engineering — minimal new code.

**Source:** RFC-Headless-Gen-Experiment-Pipeline.md, Phase 3 (Section 8)

**Dependencies:** H2 (Experiment Queue — for experiment result logging)

**Priority:** 4 of 6 (~15h)

---

## Spec H3.1: `GEN_MEMORY_PROMPT` System Prompt Overlay

**Goal:** Replace generic memory guidance with world-building-specific instructions when the agent is in gen mode.

### Prompt Content

Instructs the LLM to:
- **Before building:** search memory for style preferences, entity templates, past outcomes
- **After approval:** save style preferences, entity templates, experiment outcomes
- **On reference:** search memory for past work ("like the forest from last week")
- **Daily log:** structured summary (`World: <name> | Entities: <N> | Style: <brief>`)

### Implementation

1. Define `GEN_MEMORY_PROMPT` constant in `crates/gen/src/system_prompt.rs`
2. Wire into agent creation: when `mode == gen`, append gen memory prompt to system prompt
3. This replaces the generic memory section, not the entire system prompt

### Acceptance Criteria

- [ ] Gen mode agent receives the gen-specific memory prompt
- [ ] CLI mode agent is unaffected (still gets generic memory prompt)
- [ ] LLM follows the prompt and searches memory before building

---

## Spec H3.2: Entity Template Memory Format

**Goal:** Define a markdown convention for saving reusable entity designs to memory.

### Format

```markdown
## Entity: Glowing Lantern
- Created: 2026-03-15
- Shape: box 0.15x0.4x0.15
- Color: #FFD700, emissive: 2.0
- Light: point, color #FFB347, intensity 800, radius 8
- Behavior: bob amplitude 0.1, speed 0.5
- Tags: lighting, decoration, medieval, warm
- Notes: User iterated 3x on color before settling on warm gold
```

### Implementation

1. Document format in the gen memory prompt (prompt engineering only)
2. LLM saves templates via existing `memory_save` tool when user is satisfied with an entity
3. `memory_search("entity lantern")` retrieves the template
4. LLM uses retrieved parameters when spawning the entity in future sessions

### Acceptance Criteria

- [ ] LLM saves entity templates when user approves a design
- [ ] `memory_search` retrieves entity templates by name or tag
- [ ] LLM applies saved parameters when recreating entities

---

## Spec H3.3: Style Preference Memory Format

**Goal:** Define a markdown convention for saving visual style preferences.

### Format

```markdown
## Style: Ghibli Forest
- Created: 2026-03-14
- Palette: earth tones (#8B7355, #D4A574, #556B2F, #87CEEB)
- Lighting: warm amber point lights, soft directional from 45 deg
- Sky: sunset preset, fog distance 80-120
- Atmosphere: magical realism, cozy, slightly oversaturated
- Camera: third-person, offset [0, 4, -8]
- Audio: forest ambience (birds, wind), volume 0.3
- Tags: nature, warm, ghibli, fantasy
```

### Implementation

1. Document format in the gen memory prompt
2. LLM saves styles via `memory_save` when user praises or selects a particular aesthetic
3. `memory_search("style ghibli")` retrieves the style
4. Support `(style: from memory)` syntax in experiment prompts → agent searches for named style

### Acceptance Criteria

- [ ] LLM saves style preferences when user indicates satisfaction
- [ ] Styles are searchable by name and tags
- [ ] Experiment prompts with `(style: from memory)` trigger memory lookup

---

## Spec H3.4: Experiment Result Memory Format

**Goal:** Define a markdown convention for logging experiment outcomes as structured memory.

### Format

```markdown
## Experiment: 2026-03-16 Enchanted Forest Variations
- Concept: Three density variations of an enchanted forest
- Variations:
  - sparse-forest (skills/sparse-forest/): 8 trees, 3 bushes, open clearings
  - medium-forest (skills/medium-forest/): 20 trees, 12 bushes, winding paths -- WINNER
  - dense-forest (skills/dense-forest/): 45 trees, 25 bushes, too dark
- Findings: Medium density with dappled light felt best. Dense needs stronger ambient.
- Style used: Ghibli Forest (from memory)
```

### Implementation

1. Document format in the gen memory prompt
2. After each experiment completes (especially variation sets), LLM logs the outcome
3. When user reviews and selects a winner, LLM updates the experiment entry
4. `memory_search("forest scene")` returns experiment results with dates

### Acceptance Criteria

- [ ] Experiment results are logged in structured format after completion
- [ ] Variation sets record all variants with a winner marker
- [ ] Past experiments are searchable by topic and date

---

## Spec H3.5: Gen Session Summarization Override

**Goal:** Override the default session-end memory dump to produce structured summaries instead of raw transcripts.

### Implementation

1. `save_gen_session_summary()` replaces default `save_session_to_memory` for gen sessions
2. Gets current scene state (entity count) from the GenBridge
3. Asks LLM to summarize in format: `World: <name> | Entities: <N> | Style: <brief>`
4. LLM uses `memory_log` tool to save the structured summary
5. Raw conversation text is NOT saved to daily logs

### Acceptance Criteria

- [ ] Gen sessions produce structured summaries, not raw transcripts
- [ ] Summaries include world name, entity count, style description
- [ ] Daily logs contain structured entries, not conversation dumps

---

## Spec H3.6: Memory Partitioning Strategy

**Goal:** Gen memory and CLI memory coexist in shared MEMORY.md using distinct section headers.

### Section Layout

```markdown
# MEMORY.md
## User Info
## Gen Styles
## Gen Entity Templates
## Gen Experiment Log
```

### Implementation

1. Gen memory entries use `## Gen Styles`, `## Gen Entity Templates`, `## Gen Experiment Log` headers
2. FTS5 search naturally ranks section-level results
3. Config option `[gen.memory] separate_file = false` for v1 (shared MEMORY.md)
4. Future: if combined file grows past embedding chunk window, split to `GEN_MEMORY.md`

### Acceptance Criteria

- [ ] Gen and CLI memory entries are organized under distinct sections
- [ ] `memory_search` returns relevant entries regardless of section
- [ ] No cross-contamination between gen and CLI memory topics
