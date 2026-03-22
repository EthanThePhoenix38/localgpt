# WorldGen 6: Scene Decomposition & Semantic Ordering

**Improves generation logic and enables semantic bulk operations.** Formalizes generation order by connectivity (ground → structures → objects → decorations) and tags entities with semantic roles, enabling operations like "change all vegetation" or "make all buildings taller."

**Source:** WorldGen paper Stage III (Scene Decomposition) — generates parts in connectivity-degree order (ground first, then structures, then details) and decomposes monolithic meshes into editable objects.

**Dependencies:** WG1 (blockout), WG3 (tier tagging), existing `gen_load_gltf` tool

**Priority within WorldGen series:** 6 of 7

---

## Spec WG6.1: Semantic Role Tagging

**Goal:** Tag every entity with its semantic role in the scene: ground, structure, prop, vegetation, or decoration. This enables selective bulk operations and informs generation order.

### Implementation

1. **SemanticRole enum:**
   ```rust
   enum SemanticRole {
       Ground,      // Terrain, floors, ground planes, paths
       Structure,   // Buildings, walls, bridges, fences, stairs
       Prop,        // Furniture, crates, barrels, vehicles, functional objects
       Vegetation,  // Trees, bushes, grass, flowers
       Decoration,  // Small visual details, particles, ambient effects
       Character,   // NPCs, player
       Lighting,    // Light sources, ambient lighting entities
       Audio,       // Sound emitters
       Untagged,    // Default for manually placed entities
   }
   ```

2. **Auto-tagging rules:** When entities are created via blockout pipeline (WG1) or population tools (WG3), auto-assign semantic roles:
   - Terrain entities → `Ground`
   - Path entities → `Ground`
   - Entities in hero_slots with "building"/"wall"/"bridge" hints → `Structure`
   - Foliage entities (from `gen_add_foliage`) → `Vegetation`
   - Pass 3 decorative entities → `Decoration`
   - Entities from `gen_add_npc` → `Character`
   - Light entities → `Lighting`
   - Audio emitters → `Audio`

3. **Manual override:** `gen_set_role` MCP tool:
   ```json
   {
     "name": "gen_set_role",
     "description": "Set the semantic role of an entity",
     "parameters": {
       "entity_id": { "type": "string", "required": true },
       "role": { "type": "enum", "values": ["ground", "structure", "prop", "vegetation", "decoration", "character", "lighting", "audio"], "required": true }
     }
   }
   ```

4. **Bulk operations:** `gen_bulk_modify` tool for semantic role-based changes:
   ```json
   {
     "name": "gen_bulk_modify",
     "description": "Apply a modification to all entities matching a semantic role",
     "parameters": {
       "role": { "type": "enum", "values": ["ground", "structure", "prop", "vegetation", "decoration"], "required": true },
       "region_id": { "type": "string", "optional": true, "description": "Limit to entities in this blockout region" },
       "action": { "type": "enum", "values": ["scale", "recolor", "remove", "hide", "show", "set_material"], "required": true },
       "value": { "type": "any", "optional": true, "description": "Action parameter (scale factor, color hex, material preset)" }
     }
   }
   ```

5. **Query support:** Extend `gen_scene_info` to support filtering by semantic role:
   ```json
   { "role": "vegetation" }
   ```
   Returns only entities with that semantic role.

### Acceptance Criteria

- [ ] SemanticRole component can be attached to any entity
- [ ] Auto-tagging assigns correct roles during generation
- [ ] gen_set_role allows manual role override
- [ ] gen_bulk_modify applies changes to all entities with matching role
- [ ] gen_scene_info supports role filtering
- [ ] Role persists in WorldSpec serialization
- [ ] Bulk operations respect optional region_id scope

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/semantic.rs` — SemanticRole enum, auto-tagging rules
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_role.rs` — MCP tool handler
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_bulk_modify.rs` — bulk operations handler
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_scene_info.rs` — add role filter

---

## Spec WG6.2: Connectivity-Ordered Generation

**Goal:** Formalize generation order based on connectivity: ground first (highest connectivity — everything connects to it), then structures (connected to ground), then props (connected to structures), then decorations (connected to anything). This ensures each layer has stable foundations before the next is built.

### Implementation

1. **Generation order:**
   ```
   Layer 0: Ground     — terrain mesh, paths, water, floor planes
   Layer 1: Structure  — buildings, walls, bridges, stairs, fences
   Layer 2: Prop       — furniture, vehicles, crates, functional objects
   Layer 3: Vegetation — trees, bushes, grass (overlaps with decorative)
   Layer 4: Decoration — small visual props, particles, ambient details
   Layer 5: Character  — NPCs (placed last, on walkable surfaces)
   Layer 6: Lighting   — lights (positioned relative to structures/ground)
   Layer 7: Audio      — sound emitters (positioned relative to features)
   ```

2. **Layer dependency validation:** Before generating layer N, validate that layer N-1 is complete:
   - Structures require ground: check that terrain exists at structure footprint
   - Props require structures or ground: check that a surface exists below the prop
   - Vegetation requires ground: check terrain exists in scatter area
   - Characters require walkable navmesh: check navmesh covers placement area

3. **Integration with WG1.3 / WG3:** The three-pass population workflow (hero → medium → decorative) operates within this ordering. Heroes are typically layer 1-2, mediums are layer 2-3, decoratives are layer 3-4.

4. **Regeneration order:** When a lower layer changes (e.g., terrain height modification), all higher layers in the affected area must be re-validated:
   - Terrain change → snap structures to new height → snap props to structures → re-scatter vegetation → rebuild navmesh → re-validate character placement

5. **Layer state tracking:** `GenerationState` resource tracks which layers are complete per region:
   ```rust
   struct GenerationState {
       layers: HashMap<String, Vec<LayerStatus>>,
       // region_id → [Ground: Complete, Structure: InProgress, ...]
   }
   ```

### Acceptance Criteria

- [ ] Generation follows correct layer order (ground → structures → props → decorations)
- [ ] Layer dependency validation prevents out-of-order generation
- [ ] Lower layer changes trigger re-validation of higher layers
- [ ] GenerationState tracks per-region layer completion
- [ ] Population tools respect generation order constraints
- [ ] Error message when attempting to generate without dependencies met

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/ordering.rs` — layer definitions, dependency validation, state tracking

---

## Spec WG6.3: GLTF Mesh Segmentation

**Goal:** When importing external 3D assets via `gen_load_gltf`, decompose them into individually editable sub-objects using connected-component analysis. This enables selective operations on parts of imported meshes.

### Implementation

1. **Connected component analysis:** When a GLTF file is loaded:
   - Extract the mesh's vertex positions and triangle indices
   - Build an adjacency graph (vertices sharing an edge are connected)
   - Find connected components using union-find or BFS
   - Each connected component becomes a separate child entity

2. **Component naming:** Auto-name each component based on:
   - GLTF node names if available (many GLTF exporters include names like "roof", "wall_left", "door")
   - Fallback: `{parent_name}_part_{index}` (e.g., "house_part_0", "house_part_1")

3. **Semantic role inference:** Attempt to infer semantic role from component properties:
   - Lowest Y extent → likely ground/foundation → `Ground`
   - Vertical surfaces → likely walls → `Structure`
   - Small isolated pieces → likely props → `Prop`
   - Components named "leaf", "branch", "trunk" → `Vegetation`

4. **Entity hierarchy:** Create a parent `Group` entity containing all component child entities. The parent stores the original GLTF reference. Each child has its own `Transform` (local to parent), `Mesh`, and `StandardMaterial`.

5. **Selective editing:** After segmentation, each component can be individually:
   - Recolored (`gen_set_material` on the component entity)
   - Moved/scaled (`gen_modify_entity` on the component)
   - Hidden (`gen_modify_entity` visibility)
   - Deleted (removes component, parent group stays)

6. **Opt-in behavior:** Segmentation runs only when `gen_load_gltf` is called with `segment: true`:
   ```json
   {
     "name": "gen_load_gltf",
     "parameters": {
       "... existing parameters ...": {},
       "segment": { "type": "bool", "default": false, "description": "Decompose mesh into editable sub-objects" }
     }
   }
   ```

7. **Performance guard:** If a mesh has >10,000 triangles, warn that segmentation may take several seconds. If >100,000 triangles, require explicit confirmation. Maximum 50 components per mesh to prevent over-segmentation.

### Acceptance Criteria

- [ ] Connected component analysis splits mesh into separate entities
- [ ] GLTF node names are used for component naming when available
- [ ] Fallback naming produces readable component names
- [ ] Semantic role inference assigns reasonable roles
- [ ] Parent-child entity hierarchy is correct
- [ ] Individual components can be modified independently
- [ ] Segmentation is opt-in via segment parameter
- [ ] Performance guard warns on large meshes

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/segment.rs` — connected-component analysis, naming, role inference
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_load_gltf.rs` — add segment parameter, call segmentation

---

## Summary

| Spec | Tool/Feature | What | Effort |
|------|-------------|------|--------|
| WG6.1 | `gen_set_role` + `gen_bulk_modify` | Semantic role tagging with bulk operations | Low-Medium |
| WG6.2 | Generation ordering | Connectivity-ordered layer generation | Low |
| WG6.3 | GLTF segmentation | Decompose imported meshes into editable parts | Medium-High |

**Net effect:** Scenes become semantically structured — not just a flat list of entities, but a layered composition where each entity has a role and can be addressed as part of a group. "Make all buildings stone" or "remove all vegetation" becomes a single command instead of manual per-entity editing.
