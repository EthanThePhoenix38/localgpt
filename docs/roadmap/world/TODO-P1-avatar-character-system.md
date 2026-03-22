# Priority 1: Avatar & Character System

These 5 specs transform LocalGPT Gen from a static scene viewer into an explorable world. The avatar system is the single highest-impact feature — `gen_spawn_player` alone changes the entire user experience.

**Dependencies:** Bevy 0.18, bevy-tnua v0.28.0 (floating character controller), Avian v0.5 (ECS-native physics)

---

## Spec 1.1: `gen_spawn_player` — Player Character

**Goal:** Create a controllable player entity with movement, camera follow, and collision. This is the single most important missing tool in LocalGPT Gen.

### MCP Tool Schema

```json
{
  "name": "gen_spawn_player",
  "description": "Spawn a controllable player character with movement, camera, and collision",
  "parameters": {
    "position": { "type": "vec3", "default": [0, 1, 0] },
    "rotation": { "type": "vec3", "default": [0, 0, 0] },
    "walk_speed": { "type": "f32", "default": 5.0 },
    "run_speed": { "type": "f32", "default": 10.0 },
    "jump_force": { "type": "f32", "default": 8.0 },
    "camera_mode": { "type": "enum", "values": ["first_person", "third_person"], "default": "third_person" },
    "camera_distance": { "type": "f32", "default": 5.0 },
    "collision_radius": { "type": "f32", "default": 0.3 },
    "collision_height": { "type": "f32", "default": 1.8 }
  }
}
```

### Implementation

1. **Entity structure:** Spawn a Bevy entity with:
   - `Collider::capsule(collision_radius, collision_height)` (Avian)
   - `RigidBody::Dynamic` with locked rotation on X/Z axes
   - `TnuaBuiltinWalk` configured with `desired_velocity`, `float_height: 1.0`, `max_slope: 45°`
   - `TnuaBuiltinJump` with `height: jump_force`
   - Visual: capsule mesh with `StandardMaterial` (placeholder until avatar models)

2. **Camera system:**
   - Third-person: child entity with `Camera3d`, offset `(0, camera_distance * 0.6, camera_distance)`, look-at parent
   - First-person: child entity with `Camera3d` at eye height `(0, collision_height * 0.85, 0)`
   - Mouse look: yaw rotates player entity, pitch rotates camera entity (clamped ±89°)

3. **Input handling:** Register `PlayerInput` system reading keyboard (WASD/arrows + Space + Shift) and mouse. Movement is relative to camera forward direction projected onto XZ plane.

4. **Ground detection:** Use Avian's `ShapeCaster` pointing downward to detect ground. Feed into bevy-tnua's `TnuaController` for grounded/airborne state.

5. **Singleton enforcement:** Only one player entity allowed. If `gen_spawn_player` is called again, despawn the previous player and spawn a new one.

### Acceptance Criteria

- [ ] Player spawns at specified position with capsule collider
- [ ] WASD movement works relative to camera direction
- [ ] Jump with spacebar, run with shift
- [ ] Third-person camera follows with mouse-look orbiting
- [ ] First-person camera at eye height with mouse-look
- [ ] Player cannot walk through static geometry
- [ ] Player falls with gravity onto surfaces
- [ ] Calling gen_spawn_player twice replaces the previous player

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/avatar/mod.rs` — module root
- `localgpt/crates/localgpt-gen/src/avatar/player.rs` — spawn, movement, input
- `localgpt/crates/localgpt-gen/src/avatar/camera.rs` — first/third person camera
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_spawn_player.rs` — MCP tool handler
- `Cargo.toml` — add bevy-tnua, avian dependencies

---

## Spec 1.2: `gen_set_spawn_point` — Spawn/Respawn Locations

**Goal:** Define named locations where players appear initially or reappear after falling off the world.

### MCP Tool Schema

```json
{
  "name": "gen_set_spawn_point",
  "description": "Set a spawn/respawn location for the player",
  "parameters": {
    "position": { "type": "vec3", "required": true },
    "rotation": { "type": "vec3", "default": [0, 0, 0] },
    "name": { "type": "string", "optional": true },
    "is_default": { "type": "bool", "default": true }
  }
}
```

### Implementation

1. **SpawnPoint component:** `struct SpawnPoint { name: Option<String>, is_default: bool }` attached to an entity with `Transform`.

2. **Default management:** When `is_default: true`, clear `is_default` from all other spawn points (only one default at a time).

3. **Visual indicator:** In editor/debug mode, render a translucent cylinder (1m radius, 2m height) with a downward arrow particle effect at the spawn point location. Hidden during play mode.

4. **Respawn system:** Monitor player Y position. If below a configurable kill plane (default Y = -50), teleport player to the nearest spawn point (or default spawn point). Apply a brief fade-to-black transition.

5. **Auto-spawn integration:** If `gen_spawn_player` is called without a position and a default spawn point exists, use the spawn point's position instead of the default `[0, 1, 0]`.

### Acceptance Criteria

- [ ] Spawn points are created at specified positions
- [ ] Only one spawn point is marked as default at a time
- [ ] Player respawns at default spawn point after falling below kill plane
- [ ] Spawn points are visible in debug mode, hidden in play mode
- [ ] gen_spawn_player uses default spawn point when no position given

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/avatar/spawn_point.rs` — component, respawn system
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_spawn_point.rs` — MCP tool handler

---

## Spec 1.3: `gen_add_npc` — Non-Player Characters

**Goal:** Spawn AI-controlled characters that can idle, patrol waypoints, or wander randomly within an area.

### MCP Tool Schema

```json
{
  "name": "gen_add_npc",
  "description": "Create a non-player character with optional patrol or wander behavior",
  "parameters": {
    "position": { "type": "vec3", "required": true },
    "name": { "type": "string", "required": true },
    "model": { "type": "string", "default": "default_humanoid", "description": "\"default_humanoid\" or asset URL" },
    "behavior": { "type": "enum", "values": ["idle", "patrol", "wander"], "default": "idle" },
    "patrol_points": { "type": "vec3[]", "optional": true, "description": "Required if behavior is patrol" },
    "patrol_speed": { "type": "f32", "default": 3.0 },
    "dialogue_id": { "type": "string", "optional": true }
  }
}
```

### Implementation

1. **NPC entity:** Spawn with `Collider::capsule`, `RigidBody::Kinematic`, nameplate text above head, and visual mesh (default: colored capsule with simple face indicator).

2. **Behavior state machine** (component `NpcBehavior`):
   - `Idle`: Face toward nearest player if within 10m, play idle animation (gentle bob)
   - `Patrol`: Move through `patrol_points` in sequence at `patrol_speed`, pause 1s at each point, loop. Use linear interpolation between points.
   - `Wander`: Pick random point within 8m radius of spawn position, walk there at 60% of patrol_speed, pause 2–5s, repeat.

3. **Nameplate:** `Text2d` child entity positioned above the NPC capsule, billboard-facing (always toward camera). Displays `name` parameter.

4. **Dialogue integration:** If `dialogue_id` is set, attach `DialogueTarget { dialogue_id }` component. The NPC shows an interaction prompt ("Press E to talk") when the player is within range.

5. **Entity naming:** Store the `name` in Bevy's `Name` component for entity lookup by other tools (e.g., gen_link_entities).

### Acceptance Criteria

- [ ] NPC spawns at position with visible mesh and nameplate
- [ ] Idle NPC faces toward nearby player
- [ ] Patrol NPC moves between waypoints in a loop
- [ ] Wander NPC moves randomly within spawn radius
- [ ] NPC has collision — player cannot walk through them
- [ ] Dialogue prompt appears when player is near NPC with dialogue_id

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/avatar/npc.rs` — spawn, behavior state machine
- `localgpt/crates/localgpt-gen/src/avatar/nameplate.rs` — billboard text above entities
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_npc.rs` — MCP tool handler

---

## Spec 1.4: `gen_set_npc_dialogue` — Branching Dialogue Trees

**Goal:** Attach branching conversation trees to NPCs, triggered by proximity or click, rendered as in-world UI.

### MCP Tool Schema

```json
{
  "name": "gen_set_npc_dialogue",
  "description": "Attach a branching conversation to an NPC",
  "parameters": {
    "npc_id": { "type": "string", "required": true },
    "nodes": {
      "type": "array",
      "items": {
        "id": "string",
        "text": "string",
        "speaker": "string (optional, defaults to NPC name)",
        "choices": [{ "text": "string", "next_node_id": "string" }]
      }
    },
    "start_node": { "type": "string", "required": true },
    "trigger": { "type": "enum", "values": ["proximity", "click"], "default": "click" },
    "trigger_radius": { "type": "f32", "default": 3.0 }
  }
}
```

### Implementation

1. **Data model:** `DialogueTree` component stores a `HashMap<String, DialogueNode>`. Each `DialogueNode` has `text: String`, `speaker: Option<String>`, `choices: Vec<DialogueChoice>`. Each `DialogueChoice` has `text: String`, `next_node_id: Option<String>` (None = end conversation).

2. **Trigger system:**
   - `click`: When player presses E within `trigger_radius` of an NPC with a dialogue tree, enter dialogue mode.
   - `proximity`: Automatically enter dialogue when player enters `trigger_radius` (with a cooldown to prevent re-triggering immediately).

3. **Dialogue UI:** Screen-space overlay panel at bottom of screen (RPG-style):
   - Speaker name in bold at top
   - Dialogue text with typewriter effect (30 chars/sec)
   - Numbered choice buttons below (1, 2, 3... or click)
   - Semi-transparent dark background

4. **Dialogue state:** `DialogueState` resource tracks: active NPC entity, current node ID, whether text is still typing. Player movement is disabled during dialogue (but camera look is allowed).

5. **Conversation flow:** Selecting a choice transitions to `next_node_id`. If `next_node_id` is null or choices array is empty, dialogue ends and player regains movement.

### Acceptance Criteria

- [ ] Dialogue tree data is stored on NPC entity
- [ ] Click trigger: pressing E near NPC opens dialogue
- [ ] Proximity trigger: entering radius auto-starts dialogue
- [ ] Dialogue UI shows speaker name, text, and choices
- [ ] Selecting a choice advances to the next node
- [ ] Dialogue with no remaining choices ends the conversation
- [ ] Player movement is paused during dialogue
- [ ] Proximity trigger has cooldown to prevent re-triggering

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/avatar/dialogue.rs` — data model, state machine, UI
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_npc_dialogue.rs` — MCP tool handler

---

## Spec 1.5: `gen_set_camera_mode` — Camera Mode Switching

**Goal:** Allow switching between first-person and third-person camera at runtime, with smooth transitions and configurable parameters.

### MCP Tool Schema

```json
{
  "name": "gen_set_camera_mode",
  "description": "Switch or configure the player camera mode",
  "parameters": {
    "mode": { "type": "enum", "values": ["first_person", "third_person", "top_down", "fixed"], "required": true },
    "distance": { "type": "f32", "default": 5.0, "description": "Distance from player (third_person/top_down only)" },
    "pitch": { "type": "f32", "default": -20.0, "description": "Initial pitch in degrees (top_down: -60 recommended)" },
    "fov": { "type": "f32", "default": 60.0, "description": "Field of view in degrees" },
    "transition_duration": { "type": "f32", "default": 0.5, "description": "Seconds to blend between modes" },
    "fixed_position": { "type": "vec3", "optional": true, "description": "Camera position (fixed mode only)" },
    "fixed_look_at": { "type": "vec3", "optional": true, "description": "Look-at target (fixed mode only)" }
  }
}
```

### Implementation

1. **Camera modes:**
   - `first_person`: Camera at player eye height, no player mesh visible, direct mouse look
   - `third_person`: Camera orbits behind player at `distance`, mouse controls orbit angle, spring arm collision avoidance
   - `top_down`: Camera above and behind player at fixed pitch angle, WASD moves player in screen-relative directions
   - `fixed`: Camera at `fixed_position` looking at `fixed_look_at`, player moves with WASD in world space (for cutscenes or puzzle rooms)

2. **Spring arm (third-person):** Raycast from player to desired camera position. If geometry is hit, move camera to hit point minus small offset. This prevents camera from clipping through walls.

3. **Smooth transition:** When switching modes, lerp camera position and rotation over `transition_duration` seconds. During transition, input is temporarily paused to prevent disorientation.

4. **FOV control:** Set `Projection::Perspective { fov }` on the camera entity. Changes animate smoothly over 0.3s.

5. **State tracking:** `CameraMode` resource stores current mode and parameters. Other systems (e.g., dialogue) can temporarily override the camera and restore it afterward.

### Acceptance Criteria

- [ ] First-person mode: camera at eye height, no player mesh visible
- [ ] Third-person mode: camera orbits behind player with spring arm
- [ ] Top-down mode: camera above player at fixed pitch
- [ ] Fixed mode: camera at specified position looking at target
- [ ] Smooth animated transition between modes
- [ ] Spring arm prevents camera clipping through walls
- [ ] FOV changes animate smoothly
- [ ] Camera mode can be called via MCP tool at any time

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/avatar/camera.rs` — extend with all 4 modes, spring arm, transitions
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_camera_mode.rs` — MCP tool handler
