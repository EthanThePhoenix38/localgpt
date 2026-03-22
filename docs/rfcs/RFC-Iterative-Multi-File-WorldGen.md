# RFC: Iterative Multi-File World Generation Pipeline

**Status:** Draft
**Author:** Yi + Claude
**Date:** 2026-03-21
**Scope:** `crates/gen/src/gen3d/world.rs`, `crates/gen/src/mcp/worldgen_tools.rs`, `localgpt-world-types`
**Dependencies:** Existing `gen_save_world`/`gen_load_world` in `world.rs`, `WorldManifest` in `localgpt-world-types`, `BlockoutSpec` from WorldGen pipeline (WG1–WG3)

---

## Problem Statement

### Current generation flow

Today, world creation in LocalGPT Gen follows a **flat, terminal-state model**:

1. The LLM calls MCP tools (`gen_spawn_primitive`, `gen_set_environment`, `gen_add_behavior`, etc.) in an ad-hoc sequence.
2. Optionally, the WorldGen pipeline is invoked: `gen_plan_layout` → `gen_apply_blockout` → `gen_populate_region`.
3. At the end, `gen_save_world` snapshots everything into a skill directory containing exactly two meaningful files:
   - **`SKILL.md`** — A thin metadata stub (name, description, `useWhen` triggers). Contains no architectural knowledge about the world.
   - **`world.ron`** — A monolithic `WorldManifest` with all entities, materials, behaviors, audio, avatar, and tours serialized inline.
   - **`history.jsonl`** — Undo/redo history (optional, not used for regeneration).

### What's missing

| Gap | Impact |
|-----|--------|
| **No generation plan** | The LLM has no structured recipe to follow; each generation is improvised. Regeneration requires replaying the entire conversation. |
| **No domain decomposition** | A 200-entity world produces a single 5000-line `world.ron`. The LLM cannot selectively load or reason about subsets (e.g., "just the audio" or "just the forest region"). |
| **SKILL.md is content-free** | It tells the LLM nothing about the world's architecture, regions, design intent, or how to reconstruct it. A human reading the skill directory learns almost nothing. |
| **No iterative refinement loop** | Generation is fire-and-forget. There's no structured cycle of plan → render → evaluate → revise at the file level. |
| **No reusable knowledge** | Behaviors, audio configurations, and entity groupings that could be shared across worlds are embedded inline and lost. |
| **WorldGen pipeline is disconnected** | `gen_plan_layout` produces a `BlockoutSpec` that lives only in a Bevy resource. It's never persisted as a file the LLM can re-read. |

### Proposed solution

A **multi-file, iterative generation architecture** where:

1. The LLM first plans and generates a rich **SKILL.md** that serves as the world's knowledge index and architectural blueprint.
2. The LLM then iteratively generates **`world.md`** (and domain-specific `.md` files) alongside corresponding **`.ron` data files**, with Bevy rendering happening between iterations.
3. The WorldGen blockout pipeline integrates naturally as a generation phase within this structure.

---

## Design Principles

1. **Markdown is for the LLM; RON is for the engine.** Every `.md` file describes intent, architecture, and constraints in natural language. Every `.ron` file is the machine-readable manifestation. They are paired 1:1 by naming convention.
2. **Progressive disclosure.** SKILL.md loads ~200 tokens for browsing. `world.md` loads ~1000 tokens for planning. Domain files load only when the LLM needs to work on that domain.
3. **Iterative over monolithic.** The pipeline explicitly supports plan → generate → render → evaluate → revise cycles at the domain level, not just the whole-world level.
4. **WorldGen is a strategy, not the only strategy.** The blockout pipeline (`plan_layout` → `apply_blockout` → `populate_region`) becomes one of several generation strategies that the SKILL.md can prescribe. Manual entity-by-entity construction remains valid.
5. **Backward compatible.** Existing worlds with flat `SKILL.md + world.ron` continue to load. The new multi-file structure is opt-in and detected by file presence.
6. **Naming convention.** UPPERCASE = AgentSkills spec file mandated by the skill loader (`SKILL.md` is the only file in this category). Lowercase = all content files, including engine-required ones like `world.ron`. The engine finds `world.ron` by naming convention, but it is a content file — not a skill spec — and follows lowercase naming accordingly.

---

## Proposed Directory Structure

```
medieval-village/
├── SKILL.md                     # World blueprint + knowledge index (THE entry point)
├── world.md                     # Master world spec: regions, phases, generation order
├── world.ron                    # Root manifest (metadata + references to domain .ron files)
│
├── layout/                      # Spatial planning domain
│   ├── blockout.md              # Layout design intent, spatial relationships
│   └── blockout.ron             # BlockoutSpec data (if WorldGen pipeline was used)
│
├── regions/
│   ├── village-center.md        # Design intent, entity grouping, placement rules
│   ├── village-center.ron       # Entity definitions for this region
│   ├── forest-edge.md
│   ├── forest-edge.ron
│   ├── lake-shore.md
│   └── lake-shore.ron
│
├── behaviors/
│   ├── water-effects.md         # Reusable behavior patterns (bob, ripple, flow)
│   ├── water-effects.ron        # Behavior definitions (can be referenced by entities)
│   ├── day-night-cycle.md
│   └── day-night-cycle.ron
│
├── audio/
│   ├── ambient-soundscape.md    # Audio design: layers, spatial zones, triggers
│   ├── ambient-soundscape.ron   # Audio emitter + ambience definitions
│   ├── music.md
│   └── music.ron
│
├── avatar/
│   ├── player.md                # Avatar design: spawn, movement, camera, interactions
│   └── player.ron               # Avatar + tour definitions
│
├── assets/
│   ├── meshes/                  # .glb files (copied mesh assets)
│   └── textures/                # Referenced textures
│
└── meta/                        # Operational files (not world content)
    ├── .sync.ron                # Drift tracking manifest (see drift RFC)
    ├── generation-log.jsonl     # Tool call log for deterministic reproduction
    └── history.jsonl            # Undo/redo history
```

---

## File Specifications

### 1. SKILL.md — The World Blueprint

The most important file. Transforms from a thin metadata stub into the world's **architectural knowledge base** that enables any LLM to understand, recreate, or modify the world.

```markdown
---
name: "Medieval Village"
description: "A peaceful medieval village nestled between forest and lake"
user-invocable: true
metadata:
  type: "world"
  version: "2.0"
  generation_strategy: "blockout"    # or "manual", "hybrid"
  estimated_entities: 150
  style: "low-poly stylized"
  themes: ["medieval", "fantasy", "village"]
useWhen:
  - contains: "medieval village"
  - contains: "medieval-village"
---

# Medieval Village

A peaceful medieval village nestled between a dense forest edge and a
tranquil lake shore. The village center features a market square, a
blacksmith, and a tavern, connected by cobblestone paths.

## Architecture Index

This world is organized into the following domain files:

| Domain | Spec | Data | Entities | Purpose |
|--------|------|------|----------|---------|
| World root | world.md | world.ron | — | Master manifest, environment, camera |
| Layout | layout/blockout.md | layout/blockout.ron | — | Spatial planning, region placement |
| Village center | regions/village-center.md | regions/village-center.ron | ~45 | Buildings, market stalls, NPCs |
| Forest edge | regions/forest-edge.md | regions/forest-edge.ron | ~60 | Trees, undergrowth, wildlife |
| Lake shore | regions/lake-shore.md | regions/lake-shore.ron | ~30 | Water, dock, boats, reeds |
| Behaviors | behaviors/water-effects.md | behaviors/water-effects.ron | — | Shared bob/ripple behaviors |
| Audio | audio/ambient-soundscape.md | audio/ambient-soundscape.ron | ~8 | Spatial audio zones |
| Avatar | avatar/player.md | avatar/player.ron | 1 | Player spawn + tours |

## Generation Strategy

This world uses the **blockout-first** strategy:
1. `gen_plan_layout` with prompt "medieval village with forest and lake"
2. `gen_apply_blockout` with terrain + paths enabled
3. Populate regions in connectivity order: village-center → forest-edge → lake-shore
4. Add behaviors (water effects, NPC patrol, day-night)
5. Add audio (ambient layers, spatial emitters)
6. Configure avatar + tours
7. Evaluate and refine

## Design Constraints

- **Y=0 is ground level.** All terrain uses `height_scale: 3.0` (avg surface ~1.5).
- **Color palette:** Warm earth tones (browns, greens, stone grays). No neon.
- **Scale:** Buildings 4-8m tall. Trees 6-12m. Village spans ~40x40m.
- **Style:** Low-poly stylized. No photorealism.

## Regeneration Notes

To recreate from scratch:
1. Read world.md for environment + camera setup
2. Read layout/blockout.md for spatial planning
3. Read each region .md in the order listed above
4. For each region, execute the MCP tool calls described in its .md
5. Apply shared behaviors from behaviors/
6. Apply audio from audio/
7. Configure avatar from avatar/

Load with `gen_load_world` to restore the 3D scene directly.
Export with `gen_export_world` for external viewers.
```

### 2. world.md — Master World Spec

The operational plan. Describes *what* the world contains at a high level and *how* regions relate to each other.

```markdown
# Medieval Village — World Specification

## Environment

- Background: deep blue sky gradient `[0.4, 0.6, 0.9, 1.0]`
- Ambient light: warm white, intensity 0.4
- Sun: directional, warm yellow `[1.0, 0.95, 0.8]`, from `[-0.5, -1.0, -0.3]`
- Fog: light blue, density 0.005 (atmosphere at distance)

## Camera

- Default position: `[20, 15, 25]` looking at `[0, 0, 0]`
- FOV: 60°

## Region Layout

```text
        N
        ↑
  ┌─────────────┐
  │ Forest Edge  │  (z: -15 to -5)
  │   (dense)    │
  ├─────────────┤
  │   Village    │  (z: -5 to 10)
  │   Center     │
  ├─────────────┤
  │  Lake Shore  │  (z: 10 to 25)
  │   (water)    │
  └─────────────┘
        S
```

## Generation Phases

### Phase 1: Terrain + Blockout
- Use `gen_plan_layout` with `[50, 50]` world size
- Terrain type: `rolling_hills`, seed: `42`
- Generate paths connecting regions

### Phase 2: Village Center (see regions/village-center.md)
- Priority: hero structures first (tavern, blacksmith, market)
- Then medium props (barrels, crates, fences)
- Then decorative (flowers, cobblestones)

### Phase 3: Forest Edge (see regions/forest-edge.md)
- Use `gen_add_foliage` for tree scatter
- Manual hero trees at forest entrance

### Phase 4: Lake Shore (see regions/lake-shore.md)
- Water plane at Y=0.2
- Dock structure, boats as hero elements

### Phase 5: Behaviors + Audio + Avatar
- Apply shared behaviors from behaviors/
- Audio zones from audio/
- Avatar spawn at village gate
```

### 3. world.ron — Root Manifest (Refactored)

The root `world.ron` becomes a **thin manifest** that references domain-specific `.ron` files rather than inlining everything:

```ron
WorldManifest(
    meta: WorldMeta(
        name: "medieval-village",
        format_version: "2.0",
        description: Some("A peaceful medieval village"),
        tags: Some(["medieval", "fantasy", "village"]),
        source: Some("interactive"),
    ),
    environment: Some(EnvironmentDef(
        background_color: Some([0.4, 0.6, 0.9, 1.0]),
        ambient_intensity: Some(0.4),
        ambient_color: Some([1.0, 0.98, 0.95, 1.0]),
        fog_density: Some(0.005),
        fog_color: Some([0.7, 0.8, 0.95, 1.0]),
    )),
    camera: Some(CameraDef(
        position: [20.0, 15.0, 25.0],
        look_at: [0.0, 0.0, 0.0],
        fov: Some(60.0),
    )),
    sky: Some(SkyDef( /* ... */ )),

    // NEW: references to domain-specific .ron files
    // When present, these are loaded instead of inline entities
    layout_file: Some("layout/blockout.ron"),
    region_files: Some([
        "regions/village-center.ron",
        "regions/forest-edge.ron",
        "regions/lake-shore.ron",
    ]),
    behavior_files: Some([
        "behaviors/water-effects.ron",
    ]),
    audio_files: Some([
        "audio/ambient-soundscape.ron",
    ]),
    avatar_file: Some("avatar/player.ron"),

    // Inline entities still supported for backward compat
    // (loaded if region_files is None)
    entities: [],
    next_entity_id: 151,
)
```

### 4. Region .md + .ron Pairs

Each region has a paired `.md` (design intent for LLM) and `.ron` (entity data for engine):

**`regions/village-center.md`:**
```markdown
# Village Center Region

## Design Intent
The heart of the village. A cobblestone market square surrounded by
three hero buildings (tavern, blacksmith, general store) with market
stalls, barrels, and NPC characters populating the space.

## Entity Groups

### Hero Structures (tier: hero)
- **tavern** — 2-story building at `[3, 0, 0]`, 6x4x5m, warm brown wood
- **blacksmith** — Single story at `[-5, 0, 2]`, 4x3x3m, dark stone + chimney
- **general_store** — At `[0, 0, -4]`, 5x3x4m, light wood with awning

### Medium Props (tier: medium)
- 4x market stalls along market square edges
- Barrel clusters near tavern and blacksmith
- Wooden fence sections along paths

### Decorative (tier: decorative)
- Cobblestone ground plane
- Flower pots at building entrances
- Hanging lanterns (with pulse behavior)

### NPCs
- **innkeeper** — patrols between tavern entrance and market center
- **blacksmith_npc** — stationed at forge with idle animation

## Placement Rules
- Buildings face inward toward market square center `[0, 0, 0]`
- Market stalls 2m clearance from building walls
- NPCs patrol waypoints avoid building interiors
```

**`regions/village-center.ron`:**
```ron
RegionEntities(
    region_id: "village-center",
    bounds: RegionBounds(
        center: [0.0, 0.0, 2.5],
        size: [30.0, 10.0, 15.0],
    ),
    entities: [
        WorldEntity(
            id: 10,
            name: "tavern",
            entity_type: Structure,
            transform: Transform(
                position: [3.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            ),
            shape: Some(Cuboid(x: 6.0, y: 5.0, z: 4.0)),
            material: Some(MaterialDef(
                color: [0.55, 0.35, 0.2, 1.0],
                roughness: 0.85,
            )),
            role: Some("structure"),
            tier: Some("hero"),
            children: [ /* roof, door, windows */ ],
        ),
        // ... more entities
    ],
)
```

### 5. Behavior + Audio .md/.ron Pairs

**`behaviors/water-effects.md`:**
```markdown
# Water Effects Behaviors

Reusable behavior definitions for water-adjacent entities.

## Patterns
- **gentle_bob** — Y-axis oscillation, amplitude 0.15, frequency 0.3 Hz.
  Apply to: boats, floating debris, lily pads.
- **ripple_pulse** — Scale pulse 0.95-1.05 at 0.5 Hz.
  Apply to: water surface segments.
- **slow_spin** — Y-axis rotation at 5°/sec.
  Apply to: water wheels.

## Usage
Reference these by name in entity definitions:
`behavior_ref: "water-effects/gentle_bob"`
```

**`behaviors/water-effects.ron`:**
```ron
BehaviorLibrary(
    behaviors: {
        "gentle_bob": BehaviorDef(Bob(
            axis: [0.0, 1.0, 0.0],
            amplitude: 0.15,
            frequency: 0.3,
        )),
        "ripple_pulse": BehaviorDef(Pulse(
            min_scale: 0.95,
            max_scale: 1.05,
            frequency: 0.5,
        )),
        "slow_spin": BehaviorDef(Spin(
            axis: [0.0, 1.0, 0.0],
            speed: 5.0,
        )),
    },
)
```

### 6. layout/blockout.md + blockout.ron — Spatial Planning Domain

When WorldGen is used, the blockout gets a paired `.md`/`.ron` just like every other domain. The `.md` captures *why* the layout is arranged this way — spatial relationships, design rationale, region purpose. The `.ron` contains the `BlockoutSpec` data.

**`layout/blockout.md`:**
```markdown
# Blockout — Spatial Layout Plan

## Design Intent
The village occupies the center of a 50x50m world. Forest to the north
provides enclosure and mystery. The lake to the south opens the view
and provides a natural boundary. Paths radiate from the market square.

## Spatial Relationships
- Forest Edge screens the village from the north, creating a sense of arrival
- Village Center is the gravitational core — all paths lead here
- Lake Shore provides visual openness and a contrasting biome

## Generation Parameters
- World size: 50x50m
- Terrain: rolling hills, height_scale 3.0
- Seed: 42 (for deterministic reproduction)
- Layout style: organic (not grid)

## Region Connectivity
- village-center ↔ forest-edge: 2m wide cobblestone path
- village-center ↔ lake-shore: 2m wide dirt path
```

**`layout/blockout.ron`:**
```ron
BlockoutSpec(
    world_size: [50.0, 50.0],
    seed: Some(42),
    terrain: TerrainDef(
        terrain_type: RollingHills,
        height_scale: 3.0,
    ),
    layout_style: Organic,
    regions: [
        RegionDef(
            id: "village-center",
            center: [0.0, 2.5],
            size: [30.0, 15.0],
            biome: TemperateForest,
            density: 0.7,
            hero_slots: [
                HeroSlot(role: "landmark", hint: "tavern", position: [3.0, 0.0]),
                HeroSlot(role: "landmark", hint: "blacksmith", position: [-5.0, 2.0]),
            ],
        ),
        // ... more regions
    ],
    paths: [
        PathDef(from: "village-center", to: "forest-edge", width: 2.0),
        PathDef(from: "village-center", to: "lake-shore", width: 2.0),
    ],
    palette: WorldPalette(
        primary_biome: TemperateForest,
        time_of_day: 0.6,
    ),
)
```

### 7. meta/generation-log.jsonl — Deterministic Tool Call Log

Every MCP tool call during generation is recorded in `meta/generation-log.jsonl` for deterministic reproduction (inspired by ComfyUI workflow embedding):

```jsonl
{"seq":1,"tool":"gen_plan_layout","args":{"prompt":"medieval village with forest and lake","size":[50,50],"seed":42},"result_hash":"a1b2c3","phase":"layout"}
{"seq":2,"tool":"gen_apply_blockout","args":{"spec":"@layout/blockout.ron","generate_terrain":true},"result_hash":"d4e5f6","phase":"layout"}
{"seq":3,"tool":"gen_spawn_primitive","args":{"shape":"box","size":[6,5,4],"position":[3,0,0],"name":"tavern"},"result_hash":"g7h8i9","phase":"village-center"}
```

---

## Generation Flow: The Iterative Pipeline

### Overview

```
User prompt
    │
    ▼
┌──────────────────────────────────────────────────────┐
│  Phase 0: PLAN                                        │
│  LLM generates SKILL.md (blueprint + index)           │
│  LLM generates world.md (master spec + phase order)   │
│  LLM generates world.ron (root manifest + references)  │
│  Optionally: gen_plan_layout → layout/blockout.ron    │
└───────────────────┬──────────────────────────────────┘
                    │
        ┌───────────▼───────────┐
        │  For each region/domain │◄─────────────────┐
        │  in world.md phase order│                   │
        └───────────┬───────────┘                   │
                    │                                 │
                    ▼                                 │
        ┌───────────────────────┐                   │
        │  Phase N: GENERATE     │                   │
        │  LLM writes region .md │                   │
        │  LLM writes region .ron│                   │
        │  (or calls MCP tools   │                   │
        │   to populate scene)   │                   │
        └───────────┬───────────┘                   │
                    │                                 │
                    ▼                                 │
        ┌───────────────────────┐                   │
        │  Phase N+1: RENDER     │                   │
        │  Engine loads .ron      │                   │
        │  Bevy renders scene     │                   │
        │  gen_screenshot         │                   │
        └───────────┬───────────┘                   │
                    │                                 │
                    ▼                                 │
        ┌───────────────────────┐                   │
        │  Phase N+2: EVALUATE   │                   │
        │  gen_evaluate_scene    │                   │
        │  LLM reviews screenshot│                   │
        │  Score: style/spatial/ │                   │
        │         density        │                   │
        └───────────┬───────────┘                   │
                    │                                 │
              ┌─────┴─────┐                          │
              │ Score ≥ 0.7│                          │
              ├─YES────────┤                          │
              │  Next region├──────────────────────────┘
              ├─NO─────────┤
              │  Revise .md │
              │  Revise .ron│
              │  Re-render  ├──────────────────────────┘
              └─────────────┘

        ┌───────────────────────┐
        │  Final: SAVE           │
        │  gen_save_world writes │
        │  all files to disk     │
        │  meta/generation-log   │
        └───────────────────────┘
```

### Detailed Phase Descriptions

#### Phase 0 — Plan

**Input:** User's natural language prompt (e.g., "Create a medieval village with a forest and a lake")

**Actions:**
1. LLM generates `SKILL.md` with:
   - World description and themes
   - Architecture index (list of planned domain files)
   - Generation strategy (`blockout`, `manual`, or `hybrid`)
   - Design constraints (palette, scale, style)
   - Regeneration instructions

2. LLM generates `world.md` with:
   - Environment specification (sky, lighting, fog)
   - Camera defaults
   - Region layout map (spatial relationships)
   - Ordered generation phases with dependencies

3. LLM generates root `world.ron` with:
   - Metadata
   - Environment/camera/sky definitions
   - File references to planned region/behavior/audio/avatar .ron files

4. If using blockout strategy:
   - Call `gen_plan_layout` → get `BlockoutSpec`
   - Write to `layout/blockout.md` + `layout/blockout.ron`
   - Call `gen_apply_blockout` → terrain + paths rendered in Bevy

**New MCP tools needed:**
- `gen_write_world_plan` — Creates `SKILL.md`, `world.md`, and root `world.ron` from structured input. Returns the plan for LLM review.
- `gen_persist_blockout` — Saves current `BlockoutSpec` resource to `layout/blockout.md` + `layout/blockout.ron`.

#### Phase 1..N — Generate Region

For each region defined in `world.md`, in the specified order:

**Actions:**
1. LLM writes `regions/{name}.md`:
   - Design intent for the region
   - Entity groups by tier (hero → medium → decorative)
   - Placement rules and spatial constraints
   - References to shared behaviors

2. LLM writes `regions/{name}.ron`:
   - `RegionEntities` struct with all entity definitions
   - References to behavior library entries where applicable

3. Engine loads and renders the region:
   - New MCP tool `gen_load_region` reads the `.ron` and spawns entities
   - Alternatively, LLM calls individual `gen_spawn_*` tools (each logged)

4. Screenshot + evaluate:
   - `gen_screenshot` → LLM reviews
   - If score < threshold → LLM revises the `.md` and `.ron`, re-renders

**New MCP tools needed:**
- `gen_write_region` — Writes a `.md` + `.ron` pair for a region. **Atomicity guarantee:** both files are written as a pair — if either fails, neither is persisted. Files are held in a `PendingWrites` Bevy resource in memory until `gen_save_world` flushes to disk. Pass `flush: true` to write immediately (useful during iterative editing).
- `gen_load_region` — Loads a region `.ron` file and spawns its entities into the scene
- `gen_unload_region` — Removes all entities belonging to a region (for re-generation)

#### Phase N+1 — Behaviors

After all regions are spatially populated:

1. LLM writes `behaviors/{name}.md` + `.ron` pairs for reusable behavior patterns
2. LLM writes `audio/{name}.md` + `.ron` pairs for soundscape
3. LLM writes `avatar/player.md` + `.ron` for player configuration
4. Engine applies behaviors, audio, avatar via new tools

**New MCP tools needed:**
- `gen_write_behaviors` — Writes behavior library `.md` + `.ron`
- `gen_apply_behavior_library` — Loads a behavior `.ron` and attaches behaviors to referenced entities
- `gen_write_audio` — Writes audio spec `.md` + `.ron`
- `gen_apply_audio_spec` — Loads audio `.ron` and creates emitters/ambience

#### Final — Save

`gen_save_world` is enhanced to:
1. Write all `.md` and `.ron` files to the skill directory
2. Write `meta/generation-log.jsonl` from accumulated tool call history
3. Write `layout/blockout.ron` if a blockout was used
4. Write `meta/history.jsonl` (undo/redo)
5. Copy mesh assets to `assets/`

---

## Implementation Plan

### Phase 1: Data Model Extensions (~2 weeks)

**`localgpt-world-types` crate changes:**

```rust
// New: WorldManifest v2 with file references
#[derive(Serialize, Deserialize)]
pub struct WorldManifest {
    pub meta: WorldMeta,
    pub environment: Option<EnvironmentDef>,
    pub camera: Option<CameraDef>,
    pub sky: Option<SkyDef>,

    // v2: file references (if present, loaded instead of inline entities)
    #[serde(default)]
    pub layout_file: Option<String>,
    #[serde(default)]
    pub region_files: Option<Vec<String>>,
    #[serde(default)]
    pub behavior_files: Option<Vec<String>>,
    #[serde(default)]
    pub audio_files: Option<Vec<String>>,
    #[serde(default)]
    pub avatar_file: Option<String>,

    // v1 compat: inline entities (used when region_files is None)
    #[serde(default)]
    pub entities: Vec<WorldEntity>,
    pub next_entity_id: u32,
}

// RegionBounds used by RegionEntities (defined below with ID range allocation)
#[derive(Serialize, Deserialize)]
pub struct RegionBounds {
    pub center: [f32; 3],
    pub size: [f32; 3],
}

// New: Behavior library
#[derive(Serialize, Deserialize)]
pub struct BehaviorLibrary {
    pub behaviors: HashMap<String, BehaviorDef>,
}

// New: Audio spec
#[derive(Serialize, Deserialize)]
pub struct AudioSpec {
    pub ambience: Option<AmbienceDef>,
    pub emitters: Vec<AudioEmitterDef>,
}

// New: Region entity ID range allocation
// Each region gets a non-overlapping ID range, allocated by gen_write_world_plan
// based on estimated_entities. Prevents ID collisions when regions are written
// in parallel or revised independently.
#[derive(Serialize, Deserialize)]
pub struct RegionEntities {
    pub region_id: String,
    pub bounds: Option<RegionBounds>,
    pub id_range: (u32, u32),  // Allocated range, e.g., (1, 999) for region 0
    pub entities: Vec<WorldEntity>,
}

// Allocation strategy:
// - gen_write_world_plan divides the ID space by estimated_entities per region
//   with 2x headroom (e.g., 50 estimated → range of 100).
// - If a region exhausts its range, gen_write_region auto-extends into
//   unallocated space and updates the root world.ron manifest.
// - next_entity_id in WorldManifest tracks the global high-water mark.

// New: Generation log entry
#[derive(Serialize, Deserialize)]
pub struct GenLogEntry {
    pub seq: u32,
    pub tool: String,
    pub args: serde_json::Value,
    pub result_hash: Option<String>,
    pub phase: Option<String>,
    pub timestamp: Option<String>,
}
```

**Version detection logic in `world.rs`:**

```rust
fn load_world(world_dir: &Path) -> Result<WorldLoadResult, String> {
    let ron_path = world_dir.join("world.ron");
    let manifest: WorldManifest = /* parse ron */;

    if manifest.region_files.is_some() || manifest.layout_file.is_some() {
        // v2: load from domain-specific files
        load_multi_file_world(world_dir, &manifest)
    } else {
        // v1 compat: load inline entities
        load_inline_world(&manifest)
    }
}

fn load_multi_file_world(
    world_dir: &Path,
    manifest: &WorldManifest,
) -> Result<WorldLoadResult, String> {
    let mut all_entities = Vec::new();

    if let Some(ref region_files) = manifest.region_files {
        for rel_path in region_files {
            let path = world_dir.join(rel_path);
            let ron_str = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", rel_path, e))?;
            let region: RegionEntities = ron::from_str(&ron_str)
                .map_err(|e| format!("Failed to parse {}: {}", rel_path, e))?;
            all_entities.extend(region.entities);
        }
    }
    // ... load behaviors, audio, avatar similarly
    Ok(WorldLoadResult { /* ... */ })
}
```

### Phase 2: New MCP Tools (~2 weeks)

| Tool | Purpose | Priority |
|------|---------|----------|
| `gen_write_world_plan` | Create SKILL.md + world.md + root world.ron from structured input | P0 |
| `gen_write_region` | Write region .md + .ron pair | P0 |
| `gen_load_region` | Load a region .ron into the live scene | P0 |
| `gen_unload_region` | Remove all entities from a specific region | P0 |
| `gen_persist_blockout` | Save BlockoutSpec to layout/blockout.md + blockout.ron | P1 |
| `gen_write_behaviors` | Write behavior library .md + .ron | P1 |
| `gen_apply_behavior_library` | Load and attach behaviors from .ron | P1 |
| `gen_write_audio` | Write audio spec .md + .ron | P1 |
| `gen_apply_audio_spec` | Load and create audio from .ron | P1 |
| `gen_write_avatar` | Write avatar .md + .ron | P1 |
| `gen_generation_status` | Report current generation phase + progress | P2 |

**Tool call logging infrastructure:**

```rust
// New resource in plugin.rs
#[derive(Resource, Default)]
pub struct GenerationLog {
    pub entries: Vec<GenLogEntry>,
    pub current_phase: Option<String>,
    seq_counter: u32,
}

impl GenerationLog {
    pub fn log(&mut self, tool: &str, args: &Value, result_hash: Option<String>) {
        self.seq_counter += 1;
        self.entries.push(GenLogEntry {
            seq: self.seq_counter,
            tool: tool.to_string(),
            args: args.clone(),
            result_hash,
            phase: self.current_phase.clone(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        });
    }
}
```

### Phase 3: Enhanced Save/Load (~1 week)

Modify `handle_save_world` in `world.rs`:

1. If `GenerationLog` has entries with phase annotations → write multi-file structure
2. Group entities by their `BlockoutGenerated.region_id` component (if present) or by proximity/role
3. Write `.md` files from entity metadata + generation log context
4. Write `.ron` files from entity data
5. Fall back to monolithic `world.ron` if no region decomposition exists

Modify `handle_load_world` in `world.rs`:

1. Check for `region_files` in manifest → multi-file path
2. Load each `.ron` file, merge entities
3. Load behavior libraries, audio specs, avatar
4. Apply all to scene

### Phase 4: System Prompt Overlay for Iterative Generation (~1 week)

A gen-specific system prompt that instructs the LLM to follow the iterative pipeline:

```markdown
## World Generation Protocol

When creating a new world, follow this iterative pipeline:

### Step 1: Plan
- Generate SKILL.md with architecture index and generation strategy
- Generate world.md with region layout and phase order
- Generate root world.ron with environment and file references
- If blockout strategy: call gen_plan_layout → gen_persist_blockout → gen_apply_blockout

### Step 2: Generate Regions (iterate)
For each region in world.md phase order:
  a. Call gen_write_region with design intent + entity definitions
  b. Call gen_load_region to spawn entities into scene
  c. Call gen_screenshot to capture current state
  d. Evaluate: style consistency, spatial coherence, density balance
  e. If score < 0.7: call gen_unload_region, revise, repeat from (a)
  f. If score ≥ 0.7: proceed to next region

### Step 3: Polish
- Write behavior libraries with gen_write_behaviors
- Write audio specs with gen_write_audio
- Configure avatar with gen_write_avatar
- Final evaluation pass with gen_evaluate_scene

### Step 4: Save
- Call gen_save_world (auto-writes all files + meta/generation-log.jsonl)
```

### Phase 5: Integration Testing + Migration (~1 week)

- Test backward compat: existing v1 worlds load unchanged
- Test round-trip: generate multi-file → save → load → verify identical scene
- Test iterative flow: plan → generate region → unload → revise → reload
- Test generation-log.jsonl replay: read log → re-execute tools → compare
- Migrate Willowmere Village template to new format as proof-of-concept

---

## Relation to Existing WorldGen Pipeline

The WorldGen blockout pipeline (`gen_plan_layout` → `gen_apply_blockout` → `gen_populate_region`) integrates as **one generation strategy** within this architecture:

| Phase | WorldGen Role | Multi-File Role |
|-------|--------------|-----------------|
| Plan | `gen_plan_layout` → `BlockoutSpec` | `gen_write_world_plan` → SKILL.md + world.md + world.ron + layout/blockout.ron |
| Blockout | `gen_apply_blockout` → terrain + volumes | Same, plus results are captured in region .ron files |
| Populate | `gen_populate_region` per region | `gen_write_region` + `gen_load_region` per region, using populate as an internal strategy |
| Evaluate | `gen_evaluate_scene` / `gen_auto_refine` | Same tools, but scoped per-region within the iterative loop |
| Save | `gen_save_world` → flat files | `gen_save_world` → multi-file structure |

The `BlockoutSpec` is no longer ephemeral — it persists as `layout/blockout.ron` with design rationale in `layout/blockout.md`, and can be re-loaded, modified, and re-applied.

---

## Estimated Effort

| Phase | Duration | Dependencies |
|-------|----------|-------------|
| 1. Data model extensions | 2 weeks | None |
| 2. New MCP tools | 2 weeks | Phase 1 |
| 3. Enhanced save/load | 1 week | Phase 1, 2 |
| 4. System prompt overlay | 1 week | Phase 2 |
| 5. Integration testing | 1 week | Phase 3, 4 |
| **Total** | **~7 weeks** | |

---

## Acceptance Criteria

1. **Backward compatibility:** Existing v1 worlds (flat `SKILL.md + world.ron`) load unchanged with no migration step required.
2. **Multi-file round-trip:** Generate a multi-file world → `gen_save_world` → `gen_load_world` → scene is visually and structurally identical (entity count, positions, materials match within float epsilon).
3. **Iterative flow:** `gen_write_world_plan` → `gen_write_region` → `gen_load_region` → `gen_unload_region` → revise `.md`/`.ron` → `gen_load_region` produces the revised scene without leftover entities from the previous iteration.
4. **Generation log replay:** Reading `meta/generation-log.jsonl` and re-executing the logged tool calls on an empty scene produces an equivalent world (same entity count and approximate positions).
5. **Architecture index consistency:** `SKILL.md`'s Architecture Index table matches the actual file tree on disk after `gen_save_world`. Missing or extra files are flagged as errors during save.
6. **Atomic writes:** If `gen_write_region` fails mid-generation (e.g., invalid RON serialization), neither the `.md` nor `.ron` file is written. The `PendingWrites` resource remains clean.
7. **Entity ID isolation:** Two regions written independently never produce overlapping entity IDs.

---

## Open Questions

1. **Auto-decomposition vs explicit regions:** Should `gen_save_world` automatically split a flat entity list into regions based on `BlockoutGenerated` tags, or should the LLM always explicitly manage regions?
   - **Recommendation:** Auto-decompose when `BlockoutGenerated` components exist; otherwise keep monolithic for backward compat.

2. **Behavior reference resolution:** Should entity `.ron` files reference behaviors by library name (`behavior_ref: "water-effects/gentle_bob"`) or inline the behavior definition?
   - **Recommendation:** Support both. Reference when a library exists; inline when entity has a unique behavior.

3. **world.md generation:** Should the LLM write world.md as free-form markdown, or should we provide a structured JSON schema that gets rendered to markdown?
   - **Recommendation:** Structured input via `gen_write_world_plan` tool (JSON params), rendered to both `.md` and `.ron` by the engine. LLM can also write raw `.md` for maximum flexibility.

4. **Generation log scope:** Should `meta/generation-log.jsonl` record only world-creation tool calls, or all MCP calls including evaluation/screenshot?
   - **Recommendation:** Record all tool calls, but tag generation calls vs evaluation calls via the `phase` field. Replay only processes generation calls.

5. **Maximum region file count:** At what point does file decomposition become counterproductive?
   - **Recommendation:** Cap at ~20 region files. Worlds larger than that should use nested regions or LOD-based decomposition (future work).
