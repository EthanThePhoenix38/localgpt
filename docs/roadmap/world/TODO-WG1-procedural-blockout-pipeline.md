# WorldGen 1: Procedural Blockout & LLM-Driven Parameters

**Highest leverage change.** Transforms LocalGPT Gen from "LLM places things one by one" to "LLM designs worlds through a structured interface." Combines WorldGen paper Ideas 1 and 6 — the procedural blockout stage and the principle that LLMs should generate parameters, not geometry.

**Source:** WorldGen paper Sections 3.1 (Procedural Generation) and the core architectural insight that LLMs should never generate geometry directly.

**Dependencies:** `noise` crate (already in P3 terrain spec), `ghx_proc_gen` (WFC), Bevy 0.18

**Priority within WorldGen series:** 1 of 7 (do this first)

---

## Spec WG1.1: `gen_plan_layout` — World Layout from Text Prompt

**Goal:** A single MCP tool that takes a text prompt and returns a structured JSON blockout specification. The LLM generates high-level parameters; a procedural system handles actual geometry.

### MCP Tool Schema

```json
{
  "name": "gen_plan_layout",
  "description": "Generate a structured world layout plan from a text description",
  "parameters": {
    "prompt": { "type": "string", "required": true, "description": "Natural language world description" },
    "size": { "type": "vec2", "default": [50, 50], "description": "World dimensions in meters (X, Z)" },
    "seed": { "type": "i64", "optional": true, "description": "Random seed for deterministic generation" }
  }
}
```

### Response Schema

The tool returns a `BlockoutSpec` JSON that drives the procedural generation:

```json
{
  "terrain": {
    "type": "hills",
    "verticality": 0.6,
    "roughness": 0.4,
    "base_height": 0.0,
    "noise_seed": 42
  },
  "layout": {
    "style": "organic",
    "density": 0.5,
    "regularity": 0.3
  },
  "regions": [
    {
      "id": "village_center",
      "bounds": { "center": [0, 0], "size": [20, 20] },
      "type": "structured",
      "density": 0.7,
      "walkable": true,
      "hero_slots": [
        { "position": [0, 0, 0], "size": [8, 6, 8], "role": "landmark", "hint": "town hall" }
      ],
      "medium_density": 0.5,
      "decorative_density": 0.3
    },
    {
      "id": "forest_east",
      "bounds": { "center": [30, 0], "size": [20, 50] },
      "type": "organic",
      "density": 0.8,
      "walkable": true,
      "hero_slots": [],
      "medium_density": 0.8,
      "decorative_density": 0.9
    }
  ],
  "paths": [
    { "from": "village_center", "to": "forest_east", "width": 3.0, "style": "dirt" }
  ],
  "palette": {
    "primary_biome": "temperate_forest",
    "accent_elements": ["stone", "flowers"],
    "time_of_day": 0.6
  }
}
```

### Implementation

1. **Two-step flow:** The LLM fills in the `BlockoutSpec` JSON from the user's text prompt. This spec is then passed to `gen_apply_blockout` (Spec WG1.2) which executes the procedural generation. The LLM can review and adjust the spec before applying.

2. **BlockoutSpec data structure:** Define as a Rust struct with `#[derive(Serialize, Deserialize)]`:
   ```
   BlockoutSpec {
     terrain: TerrainParams,
     layout: LayoutParams,
     regions: Vec<RegionDef>,
     paths: Vec<PathConnection>,
     palette: WorldPalette,
   }
   ```

3. **Terrain parameter mapping:**
   - `type` enum: `flat`, `hills`, `mountains`, `canyon`, `island`, `plateau`
   - Each type maps to noise parameters: `hills` → high frequency low amplitude, `mountains` → low frequency high amplitude, etc.
   - `verticality` (0-1): scales height_scale. 0 = flat, 1 = extreme elevation changes.
   - `roughness` (0-1): controls noise octaves. 0 = smooth rolling, 1 = jagged.

4. **Layout style mapping:**
   - `structured` → BSP tree partitioning (for villages, cities, dungeons)
   - `organic` → Voronoi diagram + Poisson disk sampling (for forests, archipelagos, natural landscapes)
   - `radial` → concentric rings from a center point (for arenas, temples)
   - `grid` → regular grid with optional rotation (for farms, warehouses)

5. **Region definition:** Each region defines a spatial area with placement parameters. Regions can overlap; when they do, the region with higher density wins for that area.

6. **Path connections:** Define walkable paths between named regions. The procedural system generates path geometry (reusing P3's `gen_add_path` infrastructure) and ensures navmesh connectivity.

### Acceptance Criteria

- [ ] `gen_plan_layout` accepts a text prompt and returns a valid BlockoutSpec JSON
- [ ] BlockoutSpec validates all required fields and ranges
- [ ] Terrain type enum maps to distinct noise configurations
- [ ] Layout style determines spatial partitioning algorithm
- [ ] Regions define bounded areas with per-region density parameters
- [ ] Path connections reference valid region IDs
- [ ] Seed parameter produces deterministic output
- [ ] BlockoutSpec is round-trip serializable (serde JSON)

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/mod.rs` — module root
- `localgpt/crates/localgpt-gen/src/worldgen/blockout.rs` — BlockoutSpec, TerrainParams, RegionDef, etc.
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_plan_layout.rs` — MCP tool handler

---

## Spec WG1.2: `gen_apply_blockout` — Execute Blockout into Scene

**Goal:** Takes a `BlockoutSpec` and generates the coarse 3D scene: terrain mesh, region volumes (simple boxes), path geometry, and walkable zones. This is the "blockout" — structural scaffolding, not final visuals.

### MCP Tool Schema

```json
{
  "name": "gen_apply_blockout",
  "description": "Generate a 3D blockout scene from a layout spec",
  "parameters": {
    "spec": { "type": "BlockoutSpec", "required": true },
    "show_debug_volumes": { "type": "bool", "default": true, "description": "Render translucent region volumes for visualization" },
    "generate_terrain": { "type": "bool", "default": true },
    "generate_paths": { "type": "bool", "default": true }
  }
}
```

### Implementation

1. **Terrain generation:** Read `spec.terrain` and call into the existing terrain generation pipeline (P3 `gen_add_terrain`), mapping terrain type + verticality + roughness to noise parameters. The blockout terrain is the actual terrain mesh, not a placeholder.

2. **Spatial partitioning per region:**
   - `structured` regions: Run BSP tree subdivision on the region bounds. Each leaf cell is a potential building footprint. Use `spec.regions[i].density` to decide how many cells get filled vs. left as open space.
   - `organic` regions: Run Poisson disk sampling within the region bounds. Sample spacing = `1.0 / density`. Each sample point is a potential object placement location.

3. **Hero slot volumes:** For each `hero_slot` in a region, spawn a translucent box entity at the specified position and size. Tag with `BlockoutVolume { role: "landmark", hint: "town hall" }` component. These are placeholders that tell the LLM (or a future generation pass) what to place there.

4. **Path generation:** For each path connection, find center points of source and target regions. Generate path geometry using the existing `gen_add_path` tool with appropriate waypoints. Paths auto-conform to terrain height.

5. **Debug volumes:** When `show_debug_volumes: true`, render each region as a translucent colored box (different color per region type). Hero slots get a distinct color (gold). This is the visual "blockout" — the structural preview before detailed generation.

6. **Blockout persistence:** Store the `BlockoutSpec` as a Bevy resource (`CurrentBlockout`) so it can be modified via WG5's editing tools and used by WG2's navmesh generation. Also store it in the WorldSpec for serialization.

7. **Entity tagging:** All entities spawned by the blockout get a `BlockoutGenerated { region_id, pass }` component, so they can be identified for incremental regeneration later (WG5).

### Acceptance Criteria

- [ ] Terrain generates matching the spec's terrain parameters
- [ ] Structured regions produce BSP-partitioned cells
- [ ] Organic regions produce Poisson disk sample points
- [ ] Hero slot volumes render as translucent boxes at correct positions
- [ ] Paths generate between connected regions, conforming to terrain
- [ ] Debug volumes visualize region boundaries
- [ ] BlockoutSpec persists as a Bevy resource
- [ ] All generated entities are tagged with BlockoutGenerated

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/apply.rs` — blockout execution, spatial partitioning
- `localgpt/crates/localgpt-gen/src/worldgen/partitioning.rs` — BSP tree, Voronoi, Poisson disk
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_apply_blockout.rs` — MCP tool handler

---

## Spec WG1.3: `gen_populate_region` — Fill Region with Content

**Goal:** After the blockout exists, populate a specific region with actual 3D content based on the region's parameters and palette. This is where hero/medium/decorative placement tiers (WG3) and navmesh constraints (WG2) are applied.

### MCP Tool Schema

```json
{
  "name": "gen_populate_region",
  "description": "Fill a blockout region with 3D content based on its parameters",
  "parameters": {
    "region_id": { "type": "string", "required": true },
    "style_hint": { "type": "string", "optional": true, "description": "Additional style guidance for content generation" },
    "replace_existing": { "type": "bool", "default": false, "description": "Clear existing content in region before populating" }
  }
}
```

### Implementation

1. **Region lookup:** Find the region definition in `CurrentBlockout` by `region_id`. Read its bounds, density, hero_slots, and type.

2. **Three-pass population** (maps to WG3 hierarchical placement):
   - **Pass 1 — Heroes:** For each `hero_slot`, the LLM generates the hero asset (building, landmark, etc.) via existing tools (`gen_spawn_primitive`, `gen_load_gltf`). The slot's `hint` field guides what to create.
   - **Pass 2 — Medium:** Based on `medium_density`, spawn medium-scale elements (trees, walls, fences) at BSP cell centers or Poisson sample points that aren't occupied by heroes.
   - **Pass 3 — Decorative:** Based on `decorative_density`, scatter small props (flowers, rocks, grass) in remaining open space using `gen_add_foliage` infrastructure from P3.

3. **Palette application:** Use `spec.palette.primary_biome` to select material and object presets. `temperate_forest` biome → green terrain, deciduous trees, stone paths. `desert` biome → sand terrain, cacti, sandstone.

4. **Navmesh respect:** If a navmesh exists (WG2), validate placements against it before finalizing. Skip placements that would block critical walkable paths.

5. **Content removal:** If `replace_existing: true`, despawn all entities tagged with `BlockoutGenerated { region_id }` before repopulating. This enables iterative refinement of individual regions.

### Acceptance Criteria

- [ ] Region lookup finds correct BlockoutSpec region by ID
- [ ] Hero slots populate with appropriate content
- [ ] Medium-density elements fill remaining space proportionally
- [ ] Decorative elements scatter at low density
- [ ] Palette biome influences material and object choices
- [ ] Existing content cleared when replace_existing is true
- [ ] Invalid region_id returns clear error message

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/populate.rs` — region population, three-pass logic
- `localgpt/crates/localgpt-gen/src/worldgen/palette.rs` — biome presets, material mapping
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_populate_region.rs` — MCP tool handler

---

## Summary

| Spec | Tool | What | Effort |
|------|------|------|--------|
| WG1.1 | `gen_plan_layout` | Text prompt → structured BlockoutSpec JSON | Medium |
| WG1.2 | `gen_apply_blockout` | BlockoutSpec → coarse 3D scene (terrain + volumes + paths) | High |
| WG1.3 | `gen_populate_region` | Fill a region with content using blockout parameters | Medium |

**Recommended build order:** WG1.1 → WG1.2 → WG1.3

**Net effect:** The default world-building workflow becomes:
1. User describes a world in text
2. `gen_plan_layout` produces a structured spec
3. LLM reviews/adjusts the spec
4. `gen_apply_blockout` generates the structural scaffold
5. `gen_populate_region` fills each region with appropriate content
6. User fine-tunes with existing direct-manipulation tools (`gen_spawn_primitive`, `gen_modify_entity`, etc.)

This replaces the current workflow where the LLM must decide every position, scale, and rotation for every object simultaneously.
