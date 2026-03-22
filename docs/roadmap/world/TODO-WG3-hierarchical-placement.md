# WorldGen 3: Hierarchical Asset Placement

**Low effort, immediate quality improvement.** Formalizes a three-tier placement order (hero → medium → decorative) that naturally produces more coherent scenes with clear focal points. Mostly a workflow change in how prompts are structured and tools are sequenced.

**Source:** WorldGen paper Section 3.1 — three-pass placement strategy with terrain smoothing.

**Dependencies:** WG1 (blockout spec with regions), WG2 (navmesh for collision-aware placement), P3 (terrain, foliage), Avian v0.5 (raycasts)

**Priority within WorldGen series:** 3 of 7

---

## Spec WG3.1: Entity Tier Tagging

**Goal:** Extend the WorldSpec and entity model to tag every entity with a placement tier: `hero`, `medium`, or `decorative`. This enables tier-based operations (e.g., "remove all decorative elements", "list hero landmarks").

### Implementation

1. **PlacementTier enum:**
   ```rust
   enum PlacementTier {
       Hero,       // Major landmarks, buildings, focal points
       Medium,     // Trees, walls, bridges, fences, medium structures
       Decorative, // Flowers, rocks, grass, small props, ground clutter
       Untiered,   // Manually placed entities not part of hierarchical generation
   }
   ```

2. **Component:** Add `PlacementTier` as a Bevy component on entities. Entities spawned by direct MCP tools (manual placement) default to `Untiered`. Entities spawned by the blockout pipeline (WG1) or population tools get appropriate tiers.

3. **WorldSpec serialization:** Include `tier` field in `WorldEntity`:
   ```json
   { "id": 42, "name": "town_hall", "tier": "hero", ... }
   ```

4. **Query support:** Add `gen_scene_info` filtering by tier:
   - Extend `gen_scene_info` response to include tier per entity
   - Add optional `tier` filter parameter: `gen_scene_info(tier: "hero")` returns only hero entities

5. **Tier assignment tool:** `gen_set_tier` MCP tool to manually override an entity's tier:
   ```json
   {
     "name": "gen_set_tier",
     "description": "Set an entity's placement tier",
     "parameters": {
       "entity_id": { "type": "string", "required": true },
       "tier": { "type": "enum", "values": ["hero", "medium", "decorative", "untiered"], "required": true }
     }
   }
   ```

### Acceptance Criteria

- [ ] PlacementTier component can be attached to any entity
- [ ] Blockout-generated entities get appropriate tier tags
- [ ] Manual placements default to Untiered
- [ ] gen_scene_info includes tier in response and supports tier filtering
- [ ] gen_set_tier allows manual tier override
- [ ] Tier persists in WorldSpec serialization

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/tier.rs` — PlacementTier enum, component
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_tier.rs` — MCP tool handler
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_scene_info.rs` — add tier to response, add filter

---

## Spec WG3.2: Three-Pass Generation Workflow

**Goal:** Formalize the generation order so that hero assets are placed first (establishing structure and focal points), then medium elements (relative to heroes), then decorative fill. Each pass is a separate LLM call with the scene state passed between them.

### Implementation

1. **Pass 1 — Heroes (structure & focal points):**
   - Input: BlockoutSpec regions with `hero_slots`
   - For each hero_slot, the LLM generates one major entity (building, monument, large tree, bridge)
   - Heroes establish the visual anchors and compositional structure of the scene
   - After placing all heroes, run terrain smoothing around hero footprints (flatten terrain under buildings)
   - Tag all spawned entities with `PlacementTier::Hero`

2. **Pass 2 — Medium (relative to heroes):**
   - Input: current scene state (including all heroes)
   - For each region, the LLM places medium-scale elements at density controlled by `medium_density`
   - Medium elements are placed relative to heroes: walls connect to buildings, trees surround clearings, paths lead to landmarks
   - Placement positions come from BSP cells or Poisson samples (WG1) that aren't occupied by heroes
   - Run collision check via Avian raycasts before finalizing each placement
   - Tag all spawned entities with `PlacementTier::Medium`

3. **Pass 3 — Decorative (fill residual space):**
   - Input: current scene state (heroes + medium)
   - Scatter small props at density controlled by `decorative_density`
   - Use `gen_add_foliage` infrastructure for vegetation (P3)
   - Decorative elements fill gaps: flowers along paths, rocks at terrain breaks, grass in open areas
   - No collision checking needed (decorative elements are non-collidable)
   - Tag all spawned entities with `PlacementTier::Decorative`

4. **Terrain smoothing (between passes):**
   - After hero placement, for each hero entity with a ground footprint:
     - Sample terrain height at the entity's position
     - Flatten terrain in a radius equal to the entity's XZ bounding box + 2m margin
     - Smooth the transition from flat to natural terrain over a 3m blend zone
   - This prevents buildings from floating above terrain or sinking into hills

5. **Density parameters:**
   Each region in the BlockoutSpec defines three density values (0.0 - 1.0):
   - `hero_slots`: explicit count and positions (not density-based)
   - `medium_density`: controls items per area in pass 2. 0.3 = sparse, 0.7 = dense
   - `decorative_density`: controls fill in pass 3. 0.5 = moderate ground cover, 1.0 = lush

6. **Orchestration tool:** `gen_populate_region` (from WG1.3) runs all three passes in sequence. The LLM can also run passes individually by calling placement tools directly with tier tags.

### Acceptance Criteria

- [ ] Heroes are placed first and establish scene structure
- [ ] Medium elements are placed relative to hero positions
- [ ] Decorative elements fill remaining space
- [ ] Terrain smooths around hero footprints
- [ ] Medium placements respect collision (no overlapping entities)
- [ ] Density parameters control element count per region
- [ ] Each pass tags entities with the correct PlacementTier
- [ ] Passes can be run individually or as a sequence

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/populate.rs` — three-pass orchestration (extends WG1.3)
- `localgpt/crates/localgpt-gen/src/worldgen/terrain_smooth.rs` — terrain flattening around hero footprints

---

## Spec WG3.3: Collision-Aware Placement

**Goal:** Before finalizing any medium-tier placement, check that it doesn't intersect existing geometry. Uses Avian physics raycasts and AABB overlap tests.

### Implementation

1. **AABB overlap test:** Before spawning a medium or hero entity at position P with bounding box B:
   - Query all existing entities within B's radius
   - Check for AABB intersection with each
   - If overlap detected, try up to 3 alternative positions (offset by entity width in random directions)
   - If all alternatives overlap, skip placement and log warning

2. **Ground snap:** After collision check passes:
   - Raycast downward from placement position to find terrain height
   - Snap entity to terrain surface (Y = terrain height + entity half-height)
   - If no terrain below (gap/cliff), skip placement

3. **Clearance check:** Ensure minimum clearance between placed entities:
   - Heroes: 2m clearance from other heroes
   - Medium: 0.5m clearance from heroes and other mediums
   - Decorative: no clearance requirement

4. **Integration:** This system is called by:
   - `gen_populate_region` during pass 2 (medium placement)
   - `gen_spawn_primitive` and `gen_spawn_mesh` when `PlacementTier` is set
   - Can be disabled via `skip_collision_check: true` parameter for manual overrides

### Acceptance Criteria

- [ ] Overlapping placements are detected and repositioned
- [ ] Ground snap places entities on terrain surface
- [ ] Minimum clearance respected between tiers
- [ ] Alternative positions attempted before skipping
- [ ] Skipped placements logged with reason
- [ ] Collision check can be disabled per-call

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/collision_check.rs` — AABB overlap, ground snap, clearance

---

## Summary

| Spec | What | Effort |
|------|------|--------|
| WG3.1 | Entity tier tagging (hero/medium/decorative) + query support | Low |
| WG3.2 | Three-pass generation workflow with terrain smoothing | Medium |
| WG3.3 | Collision-aware placement with ground snap | Low-Medium |

**Net effect:** Scenes have clear visual hierarchy. Heroes create focal points, medium elements build structure around them, and decorative elements provide richness. The current approach of treating all objects equally produces flat, unstructured scenes — this fixes that.
