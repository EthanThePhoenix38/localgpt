# WorldGen 5: Blockout Editing & Incremental Regeneration

**Enables the "WorldSpec is editable source code" vision.** Instead of editing final geometry, users (or the LLM) edit the blockout/navmesh, and the system regenerates only affected regions. This maps to the Terraform-style plan/apply workflow described in the architecture docs.

**Source:** WorldGen paper Figure 7 — users edit the procedural blockout directly (delete structures, resize buildings, lower terrain), and the system regenerates appropriate 3D content that respects the modified layout.

**Dependencies:** WG1 (blockout infrastructure), WG2 (navmesh), WG3 (hierarchical placement for regeneration)

**Priority within WorldGen series:** 5 of 7

---

## Spec WG5.1: `gen_modify_blockout` — Edit Blockout Regions

**Goal:** Modify the structural blockout without manually editing individual entities. Changes to the blockout drive selective regeneration of affected areas.

### MCP Tool Schema

```json
{
  "name": "gen_modify_blockout",
  "description": "Edit the world blockout layout — add, remove, or resize regions",
  "parameters": {
    "action": { "type": "enum", "values": ["add_region", "remove_region", "resize_region", "move_region", "merge_regions", "set_density"], "required": true },
    "region_id": { "type": "string", "required": true },
    "new_bounds": { "type": "object", "optional": true, "properties": {
      "center": { "type": "vec2" },
      "size": { "type": "vec2" }
    }},
    "new_region": { "type": "RegionDef", "optional": true, "description": "Full region definition for add_region action" },
    "density": { "type": "f32", "optional": true, "description": "New density value for set_density action" },
    "merge_target": { "type": "string", "optional": true, "description": "Target region ID for merge_regions action" },
    "auto_regenerate": { "type": "bool", "default": false, "description": "Automatically regenerate affected content" }
  }
}
```

### Implementation

1. **Actions:**
   - `add_region`: Insert a new RegionDef into the BlockoutSpec. Spawn debug volume.
   - `remove_region`: Remove region from BlockoutSpec. Despawn all entities tagged with that region_id.
   - `resize_region`: Update bounds. Entities outside new bounds are despawned; new area is unpopulated until regeneration.
   - `move_region`: Update center position. All entities in the region move by the same delta.
   - `merge_regions`: Combine two adjacent regions into one. Union of bounds, average of densities.
   - `set_density`: Change density parameters for a region. Does not immediately affect placed entities until regeneration.

2. **Dirty tracking:** When a blockout region is modified, mark it as `BlockoutDirty { region_id, change_type }`. This drives incremental regeneration (WG5.3).

3. **Debug volume update:** After any modification, update the translucent debug volume to reflect new region bounds.

4. **Auto-regenerate:** If `auto_regenerate: true`, immediately run `gen_populate_region(region_id, replace_existing: true)` after the blockout change. Otherwise, the user/LLM can review the blockout change before triggering regeneration.

5. **Undo support:** Before modifying, snapshot the current BlockoutSpec. Store the last 10 snapshots in a `BlockoutHistory` resource. Expose `gen_undo_blockout` to revert.

### Acceptance Criteria

- [ ] add_region inserts new region with debug volume
- [ ] remove_region despawns all entities in that region
- [ ] resize_region updates bounds and removes out-of-bounds entities
- [ ] move_region translates all region entities by the position delta
- [ ] merge_regions combines two regions into one
- [ ] set_density updates density parameters
- [ ] Modified regions are marked dirty for regeneration
- [ ] auto_regenerate triggers immediate repopulation
- [ ] Undo reverts to previous blockout state

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/edit.rs` — blockout editing actions, dirty tracking
- `localgpt/crates/localgpt-gen/src/worldgen/history.rs` — BlockoutHistory, undo support
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_modify_blockout.rs` — MCP tool handler

---

## Spec WG5.2: `gen_edit_navmesh` — Manual Navmesh Overrides

**Goal:** Allow manual marking of areas as walkable or blocked, overriding the auto-generated navmesh. This enables creating intentional barriers, secret passages, and custom navigation constraints.

### MCP Tool Schema

```json
{
  "name": "gen_edit_navmesh",
  "description": "Manually mark areas as walkable or blocked on the navmesh",
  "parameters": {
    "action": { "type": "enum", "values": ["block_area", "allow_area", "add_connection", "remove_connection"], "required": true },
    "position": { "type": "vec3", "required": true },
    "radius": { "type": "f32", "default": 2.0, "description": "Area of effect radius" },
    "shape": { "type": "enum", "values": ["circle", "rectangle"], "default": "circle" },
    "size": { "type": "vec2", "optional": true, "description": "Rectangle dimensions for shape=rectangle" },
    "connection_target": { "type": "vec3", "optional": true, "description": "Target position for add/remove_connection" }
  }
}
```

### Implementation

1. **Navmesh overrides:** Store a list of `NavMeshOverride` entries in a Bevy resource:
   ```rust
   struct NavMeshOverride {
       action: OverrideAction, // Block or Allow
       position: Vec3,
       shape: OverrideShape,   // Circle(radius) or Rect(size)
   }
   ```

2. **Block area:** Mark a circular or rectangular area as non-walkable. After navmesh regeneration, this area is carved out. Use case: creating invisible walls, blocking shortcuts, defining maze walls.

3. **Allow area:** Mark an area as forcibly walkable, overriding slope/obstacle checks. Use case: creating walkable bridges over gaps, secret passages through walls.

4. **Add connection:** Create a navmesh link (off-mesh connection) between two points. Use case: jump pads, teleporter paths, climbable surfaces that the auto-generator can't detect. The pathfinding system treats these as valid edges.

5. **Remove connection:** Remove a navmesh link between two points. Use case: breaking a path that the auto-generator creates but the designer doesn't want.

6. **Override application:** When `gen_build_navmesh` runs, overrides are applied after the initial generation:
   - Block overrides: carve polygons out of the navmesh
   - Allow overrides: add polygons to the navmesh
   - Connections: add off-mesh links to the navmesh graph

7. **Debug visualization:** Blocked areas show as red circles/rectangles on the navmesh overlay. Allowed areas show as blue. Connections show as dashed yellow lines between points.

### Acceptance Criteria

- [ ] block_area removes walkable area from navmesh
- [ ] allow_area forces area to be walkable
- [ ] add_connection creates navigable link between two points
- [ ] remove_connection breaks a navmesh edge
- [ ] Overrides persist across navmesh regeneration
- [ ] Debug visualization shows overrides distinctly
- [ ] Overrides serialize to WorldSpec

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/navmesh_edit.rs` — override storage, application, visualization
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_edit_navmesh.rs` — MCP tool handler

---

## Spec WG5.3: Incremental Regeneration

**Goal:** When the blockout changes, only regenerate content in affected regions. Unchanged regions keep their entities. This is critical for interactive editing — full regeneration of a large world is too slow.

### Implementation

1. **Change detection:** `BlockoutDirty` flags from WG5.1 identify which regions need regeneration. Changes are categorized:
   - `BoundsChanged` — region resized or moved. Regenerate content at new positions.
   - `DensityChanged` — density parameters modified. Add or remove entities to match new density.
   - `Removed` — region deleted. Despawn all entities.
   - `TerrainChanged` — terrain parameters changed. Regenerate terrain mesh, then snap all entities to new height.

2. **Selective despawn:** Before regenerating a region, despawn only entities tagged with `BlockoutGenerated { region_id: dirty_region }`. Entities manually placed by the user (tagged `Untiered`) are preserved.

3. **Content-addressable cache:** Hash each region's parameters (bounds, density, seed, palette) to create a cache key. If a region's parameters haven't actually changed (e.g., user resized then resized back), skip regeneration. Store previously generated entity lists keyed by parameter hash.

4. **Regeneration order:** Process dirty regions in dependency order:
   1. Terrain changes (affects all entities via height snapping)
   2. Removed regions (free up space)
   3. Resized/moved regions (relocate entities)
   4. Density changes (add/remove entities)
   5. Rebuild navmesh (WG2) after all regions are updated

5. **Plan/apply workflow:** Before executing regeneration, show the user what will change:
   ```
   gen_regenerate_preview:
     - Region "village_center": 12 entities will be removed, ~15 new entities generated
     - Region "forest_east": density change, ~30 new trees will be added
     - Navmesh will be rebuilt
     Apply? [y/n]
   ```
   This maps to the Terraform-style `plan/apply` model.

6. **`gen_regenerate` tool:**
   ```json
   {
     "name": "gen_regenerate",
     "description": "Regenerate content in dirty blockout regions",
     "parameters": {
       "region_ids": { "type": "string[]", "optional": true, "description": "Specific regions to regenerate. Null = all dirty regions" },
       "preview_only": { "type": "bool", "default": false, "description": "Show what would change without executing" },
       "preserve_manual": { "type": "bool", "default": true, "description": "Keep manually placed (untiered) entities" }
     }
   }
   ```

### Acceptance Criteria

- [ ] Only dirty regions are regenerated
- [ ] Manually placed entities are preserved during regeneration
- [ ] Content-addressable cache prevents unnecessary regeneration
- [ ] Regeneration follows correct dependency order
- [ ] Preview mode shows planned changes without executing
- [ ] Navmesh rebuilds after regeneration completes
- [ ] Performance: regenerating one region takes < 2 seconds

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/regenerate.rs` — dirty processing, selective despawn, caching
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_regenerate.rs` — MCP tool handler

---

## Summary

| Spec | Tool | What | Effort |
|------|------|------|--------|
| WG5.1 | `gen_modify_blockout` | Edit regions (add/remove/resize/merge) with dirty tracking | Medium |
| WG5.2 | `gen_edit_navmesh` | Manual walkable/blocked area overrides | Medium |
| WG5.3 | `gen_regenerate` | Incremental regeneration of dirty regions with plan/apply | High |

**Recommended build order:** WG5.1 → WG5.3 → WG5.2

**Net effect:** World editing becomes non-destructive. Instead of deleting and recreating entities manually, the user edits the blockout (the "source code") and the system regenerates the affected output. This is the fundamental workflow shift that makes procedural world-building practical for iterative creative work.
