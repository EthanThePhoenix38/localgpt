# WorldGen 2: Navmesh-First Generation

**Ensures traversability** — the single most important quality for explorable worlds. WorldGen paper's core innovation: the navmesh acts as a hard constraint on generation, not a post-hoc analysis. Generated worlds that can't be traversed are useless.

**Source:** WorldGen paper Section 3.2 (Navmesh-Conditioned Reconstruction) — achieves 40-50% lower Chamfer distance to ground-truth navmeshes vs. baselines.

**Dependencies:** `oxidized_navigation` crate (Recast-based navmesh, pure Rust), WG1 (blockout), P3 (terrain mesh), Avian v0.5 (physics colliders)

**Priority within WorldGen series:** 2 of 7

---

## Spec WG2.1: `gen_build_navmesh` — Navigation Mesh Generation

**Goal:** Generate a navigation mesh from the current scene geometry. The navmesh defines all walkable surfaces and is the authority for whether a scene is traversable.

### MCP Tool Schema

```json
{
  "name": "gen_build_navmesh",
  "description": "Generate a navigation mesh from current scene geometry",
  "parameters": {
    "agent_radius": { "type": "f32", "default": 0.3, "description": "Agent collision radius in meters" },
    "agent_height": { "type": "f32", "default": 1.8, "description": "Agent height in meters" },
    "max_slope": { "type": "f32", "default": 45.0, "description": "Maximum walkable slope in degrees" },
    "step_height": { "type": "f32", "default": 0.4, "description": "Maximum step-up height in meters" },
    "cell_size": { "type": "f32", "default": 0.3, "description": "Voxel cell size (smaller = more detail, slower)" },
    "include_entities": { "type": "string[]", "optional": true, "description": "Entity IDs to include as obstacles. Null = all static entities" }
  }
}
```

### Implementation

1. **Geometry collection:** Query all entities with `RigidBody::Static` (or no RigidBody) and a `Mesh` handle. Extract vertex positions and triangle indices from each mesh, transformed to world space. If `include_entities` is specified, only include those entities; otherwise include all static geometry.

2. **Navmesh generation:** Feed collected geometry into `oxidized_navigation`:
   - `NavMeshSettings { cell_width: cell_size, cell_height: cell_size * 0.5, .. }`
   - `AgentSettings { radius: agent_radius, height: agent_height, max_slope: max_slope.to_radians(), max_step_height: step_height }`
   - Generate via `NavMeshGenerator::generate()`

3. **Navmesh storage:** Store the generated navmesh as:
   - `NavMeshResource` — Bevy resource holding the `oxidized_navigation::NavMesh` for runtime queries
   - `NavMeshDebugMesh` — a visual mesh representation for debug rendering

4. **Debug visualization:** Render the navmesh as a translucent green overlay on top of the scene geometry. Walkable polygons are green; steep slopes are red; edges are shown as yellow wireframe lines. Toggle visibility with a `gen_toggle_navmesh_debug` tool or key binding.

5. **Regeneration trigger:** The navmesh should regenerate when terrain changes or when static entities are added/removed. Add a `NavMeshDirty` flag that gets set by terrain/entity modification systems. A system watches this flag and triggers async regeneration (navmesh generation can be expensive for large scenes).

6. **WorldSpec persistence:** Serialize navmesh agent parameters (not the mesh data itself) into the WorldSpec. On world load, regenerate the navmesh from the scene geometry.

### Acceptance Criteria

- [ ] Navmesh generates from terrain and static entity geometry
- [ ] Agent radius/height/slope parameters correctly filter walkable areas
- [ ] Debug visualization shows walkable vs. non-walkable areas
- [ ] NavMeshResource is queryable for pathfinding
- [ ] Navmesh regenerates when scene geometry changes
- [ ] Step height allows traversing stairs and small obstacles
- [ ] Cell size controls navmesh resolution vs. performance
- [ ] Large scenes (500+ entities) generate navmesh within 5 seconds

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/navmesh.rs` — generation, storage, debug rendering
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_build_navmesh.rs` — MCP tool handler
- `Cargo.toml` — add `oxidized_navigation` dependency

---

## Spec WG2.2: `gen_validate_navigability` — Traversability Check

**Goal:** Check whether the current scene is traversable between specified points (or generally). Returns a report of walkable coverage, blocked areas, and disconnected regions.

### MCP Tool Schema

```json
{
  "name": "gen_validate_navigability",
  "description": "Check if the scene is traversable between points or overall",
  "parameters": {
    "from": { "type": "vec3", "optional": true, "description": "Start point. If omitted, uses player spawn point" },
    "to": { "type": "vec3", "optional": true, "description": "End point. If omitted, checks general connectivity" },
    "check_all_regions": { "type": "bool", "default": false, "description": "Check connectivity between all blockout regions" },
    "show_overlay": { "type": "bool", "default": true, "description": "Render visual overlay showing walkable/blocked areas" }
  }
}
```

### Response Schema

```json
{
  "navigable": true,
  "coverage_percent": 78.5,
  "path_found": true,
  "path_length": 42.3,
  "disconnected_regions": [],
  "blocked_areas": [
    { "center": [15, 0, 20], "radius": 3.0, "reason": "object_blocking_path" }
  ],
  "warnings": [
    "Region 'forest_east' has only one narrow entry point (2.1m wide)"
  ]
}
```

### Implementation

1. **Point-to-point check:** If both `from` and `to` are specified, run A* pathfinding on the navmesh. Return whether a path exists, its length, and any bottleneck points (narrow passages).

2. **General connectivity:** If no points specified, analyze the navmesh for:
   - **Coverage:** Percentage of the world bounds that is walkable
   - **Connected components:** Identify disconnected navmesh islands. Each island that doesn't connect to the player spawn region is flagged.
   - **Bottlenecks:** Find narrow passages (width < 2× agent radius) that could trap the player.

3. **Region connectivity** (when `check_all_regions: true`): For each pair of blockout regions (from WG1), check if a path exists between their center points. Flag disconnected region pairs.

4. **Visual overlay:** When `show_overlay: true`:
   - Green: walkable areas with good connectivity
   - Yellow: narrow passages or bottleneck areas
   - Red: blocked areas within regions that should be walkable
   - Gray: areas outside world bounds or intentionally non-walkable
   - Blue lines: pathfinding routes between regions

5. **Auto-fix suggestions:** For each blocked area, suggest a fix:
   - "Move entity 'large_rock_03' 2m east to clear the path"
   - "Reduce terrain height at [15, 20] to create a passable slope"
   - "Add a bridge between regions 'village' and 'island'"

### Acceptance Criteria

- [ ] Point-to-point pathfinding returns correct navigability result
- [ ] General connectivity identifies disconnected navmesh islands
- [ ] Coverage percentage accurately reflects walkable area ratio
- [ ] Bottleneck detection finds narrow passages
- [ ] Region connectivity checks all blockout region pairs
- [ ] Visual overlay renders with correct color coding
- [ ] Blocked areas include position and reason
- [ ] Auto-fix suggestions are actionable

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/validate.rs` — pathfinding, connectivity analysis, bottleneck detection
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_validate_navigability.rs` — MCP tool handler

---

## Spec WG2.3: Navmesh-Constrained Placement

**Goal:** When spawning objects via any placement tool, validate that the placement doesn't block critical navigation paths. This is not a separate MCP tool — it's a validation layer integrated into existing placement tools.

### Implementation

1. **Placement guard system:** Before finalizing any entity placement (via `gen_spawn_primitive`, `gen_spawn_mesh`, `gen_add_foliage`, or WG1.3's `gen_populate_region`), check the proposed placement against the navmesh:
   - Temporarily add the new entity's collision shape to the navmesh input geometry
   - Regenerate the navmesh in the affected local area (not full scene)
   - Check if any previously connected regions become disconnected
   - If connectivity is broken, reject the placement with an error message suggesting alternative positions

2. **Severity levels:**
   - **Block** — placement would completely disconnect regions: reject with error
   - **Warn** — placement narrows a passage below 2× agent width: allow with warning
   - **Allow** — placement doesn't affect navigation: proceed silently

3. **Bypass flag:** Add an optional `bypass_navmesh_check: bool` parameter to placement tools for cases where the user intentionally wants to block a path (e.g., placing a wall to create a maze).

4. **Performance:** Local navmesh regeneration (only the affected area) should be fast enough for interactive use. For bulk placement (e.g., `gen_add_foliage` with 200+ instances), batch all placements and do a single navmesh check at the end.

5. **Integration points:** Modify these existing tools to include navmesh checking:
   - `gen_spawn_primitive` — check before spawning static primitives
   - `gen_spawn_mesh` — check before spawning mesh assets
   - `gen_add_foliage` — batch check after all instances placed
   - `gen_populate_region` — check after each hero/medium placement pass

### Acceptance Criteria

- [ ] Placement that disconnects regions is rejected with error
- [ ] Placement that narrows passages produces a warning
- [ ] Placement in open areas proceeds without delay
- [ ] bypass_navmesh_check allows intentional path blocking
- [ ] Bulk placements are batch-checked efficiently
- [ ] Error message suggests alternative placement positions
- [ ] Guard system can be globally disabled for performance

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/placement_guard.rs` — navmesh validation, severity levels
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_spawn_primitive.rs` — add navmesh check call
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_spawn_mesh.rs` — add navmesh check call

---

## Summary

| Spec | Tool/Feature | What | Effort |
|------|-------------|------|--------|
| WG2.1 | `gen_build_navmesh` | Generate navmesh from scene geometry via oxidized_navigation | High |
| WG2.2 | `gen_validate_navigability` | Check traversability between points, find disconnected areas | Medium |
| WG2.3 | Placement guard | Validate placements don't break navigation paths | Medium |

**Recommended build order:** WG2.1 → WG2.2 → WG2.3

**Net effect:** Every generated world is guaranteed to be walkable. The navmesh is computed first (after terrain), and all subsequent object placements respect it. The `gen_validate_navigability` tool gives the LLM a way to verify its work before presenting the world to the user.
