# Priority 2: Interaction & Trigger System

These 5 specs add gameplay to LocalGPT Gen. The architecture follows the **Trigger → State Change → Behavior Response** pattern, mapping cleanly to Bevy's Observer system where each trigger type becomes an `EntityEvent`.

**Dependencies:** Priority 1 (avatar system for player entity), Avian v0.5 (collision detection)

---

## Spec 2.1: `gen_add_trigger` — Universal Interaction Primitive

**Goal:** A single tool that attaches any trigger-action pair to any entity. This is the foundational building block for all interactivity.

### MCP Tool Schema

```json
{
  "name": "gen_add_trigger",
  "description": "Add an interaction trigger and action to an entity",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "trigger_type": {
      "type": "enum",
      "values": ["proximity", "click", "area_enter", "area_exit", "collision", "timer"],
      "required": true
    },
    "action": {
      "type": "enum",
      "values": ["animate", "teleport", "play_sound", "show_text", "toggle_state", "spawn", "destroy", "add_score", "enable", "disable"],
      "required": true
    },
    "trigger_params": {
      "type": "object",
      "properties": {
        "radius": { "type": "f32" },
        "cooldown": { "type": "f32" },
        "interval": { "type": "f32" },
        "max_distance": { "type": "f32" },
        "prompt_text": { "type": "string" }
      }
    },
    "action_params": {
      "type": "object",
      "properties": {
        "property": { "type": "string" },
        "to": { "type": "any" },
        "duration": { "type": "f32" },
        "easing": { "type": "string" },
        "destination": { "type": "vec3" },
        "sound": { "type": "string" },
        "text": { "type": "string" },
        "state_key": { "type": "string" },
        "value": { "type": "any" },
        "amount": { "type": "i32" }
      }
    },
    "once": { "type": "bool", "default": false }
  }
}
```

### Implementation

1. **Trigger components** (one per trigger type, attached to entity):
   - `ProximityTrigger { radius, cooldown, last_triggered }` — checks distance to player each frame
   - `ClickTrigger { max_distance, prompt_text }` — responds to player E key or mouse click within range
   - `AreaTrigger { is_enter: bool }` — uses Avian sensor collider to detect player overlap
   - `CollisionTrigger` — uses Avian collision events
   - `TimerTrigger { interval, timer: Timer }` — fires on repeating Bevy `Timer`

2. **Action components** (one per action type, attached to same entity):
   - `AnimateAction { property, to, duration, easing }` — tweens a transform property
   - `TeleportAction { destination }` — moves player to destination
   - `PlaySoundAction { sound }` — plays audio asset
   - `ShowTextAction { text }` — displays floating text near entity
   - `ToggleStateAction { state_key, value }` — flips a bool/string in entity's `EntityState` map
   - `SpawnAction` / `DestroyAction` — create/remove entities
   - `AddScoreAction { amount }` — modifies the `Score` resource
   - `EnableAction` / `DisableAction` — toggle entity visibility and collision

3. **Event flow:** Each trigger system emits a `TriggerFired { entity, trigger_type }` event. A central `ActionExecutor` observer handles the event by reading the action component and executing it. This decouples triggers from actions.

4. **Cooldown:** After firing, trigger enters cooldown period. During cooldown, the trigger is inactive.

5. **Once flag:** If `once: true`, the trigger component is removed after first firing.

### Acceptance Criteria

- [ ] Proximity trigger fires when player enters radius
- [ ] Click trigger fires on E key press within max_distance
- [ ] Area enter/exit triggers fire on sensor collider overlap
- [ ] Timer trigger fires at specified interval
- [ ] Animate action tweens transform properties smoothly
- [ ] Teleport action moves player to destination
- [ ] Show text action displays floating text
- [ ] Toggle state action flips entity state
- [ ] Cooldown prevents rapid re-triggering
- [ ] Once flag removes trigger after first use

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/interaction/mod.rs` — module root
- `localgpt/crates/localgpt-gen/src/interaction/triggers.rs` — all trigger components and systems
- `localgpt/crates/localgpt-gen/src/interaction/actions.rs` — all action components and executor
- `localgpt/crates/localgpt-gen/src/interaction/state.rs` — EntityState map, Score resource
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_trigger.rs` — MCP tool handler

---

## Spec 2.2: `gen_add_teleporter` — Portal/Teleportation

**Goal:** Shorthand for creating a visible portal that teleports the player to a destination when they step into it.

### MCP Tool Schema

```json
{
  "name": "gen_add_teleporter",
  "description": "Create a portal that teleports the player to a destination",
  "parameters": {
    "position": { "type": "vec3", "required": true },
    "destination": { "type": "vec3", "required": true },
    "size": { "type": "vec3", "default": [2, 3, 2] },
    "effect": { "type": "enum", "values": ["none", "fade", "particles"], "default": "fade" },
    "sound": { "type": "string", "optional": true },
    "label": { "type": "string", "optional": true }
  }
}
```

### Implementation

1. **Portal entity:** Spawn a thin box collider (`size`) at `position` with `Sensor` flag (no physics response, just detection). Visual: translucent glowing plane with animated UV scrolling (swirling effect).

2. **Visual style:** Two concentric torus meshes with emissive material (blue-purple gradient). Inner torus rotates slowly. Particle system emits small glowing dots drifting inward.

3. **Teleport trigger:** On `CollisionStarted` between player and portal sensor:
   - If `effect == "fade"`: fade screen to black over 0.3s, teleport player, fade back over 0.3s
   - If `effect == "particles"`: burst particle effect at source and destination
   - If `effect == "none"`: instant teleport

4. **Label:** If provided, render `Text2d` billboard above the portal showing the label text.

5. **Bidirectional option:** If two teleporters reference each other's positions, they form a two-way link. Implement cooldown (2s) to prevent immediate re-teleportation.

### Acceptance Criteria

- [ ] Portal spawns with visible glowing effect at position
- [ ] Player stepping into portal is teleported to destination
- [ ] Fade effect shows black screen during teleport
- [ ] Particle effect bursts at source and destination
- [ ] Label text displayed above portal
- [ ] 2-second cooldown prevents teleport loops
- [ ] Portal does not block physics (sensor only)

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/interaction/teleporter.rs` — portal entity, effects, teleport logic
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_teleporter.rs` — MCP tool handler

---

## Spec 2.3: `gen_add_collectible` — Pickupable Items

**Goal:** Make any entity collectible with scoring, pickup effects, and optional respawning.

### MCP Tool Schema

```json
{
  "name": "gen_add_collectible",
  "description": "Make an entity collectible with score value and pickup effects",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "value": { "type": "i32", "default": 1 },
    "category": { "type": "string", "default": "points" },
    "pickup_sound": { "type": "string", "optional": true },
    "pickup_effect": { "type": "enum", "values": ["none", "sparkle", "dissolve"], "default": "sparkle" },
    "respawn_time": { "type": "f32", "optional": true, "description": "Seconds until respawn, null = no respawn" }
  }
}
```

### Implementation

1. **Collectible component:** `struct Collectible { value, category, pickup_effect, respawn_time }` attached to the target entity.

2. **Idle animation:** Collectible entities slowly bob up and down (sine wave, 0.2m amplitude, 2s period) and rotate around Y axis (45°/s). This universally signals "pickup" to players.

3. **Pickup detection:** Avian sensor collider on the collectible. On player overlap:
   - Add `value` to `ScoreBoard` resource under `category` key
   - Play `pickup_sound` if specified
   - Execute `pickup_effect`:
     - `sparkle`: burst of 20 gold particles flying upward
     - `dissolve`: scale entity to 0 over 0.3s with increasing transparency
   - Despawn the entity (or hide if `respawn_time` is set)

4. **Respawn:** If `respawn_time` is Some, after the timer elapses: re-show the entity at original position, replay the bobbing animation.

5. **ScoreBoard resource:** `HashMap<String, i32>` tracking scores per category. Emit `ScoreChanged { category, new_value }` event for HUD integration.

### Acceptance Criteria

- [ ] Collectible entity bobs and rotates as idle animation
- [ ] Player touching collectible adds value to score
- [ ] Sparkle effect shows particle burst on pickup
- [ ] Dissolve effect scales/fades entity on pickup
- [ ] Collectible despawns after pickup
- [ ] Collectible respawns after respawn_time if set
- [ ] ScoreBoard resource tracks scores per category
- [ ] ScoreChanged event fires for HUD updates

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/interaction/collectible.rs` — component, idle anim, pickup, respawn
- `localgpt/crates/localgpt-gen/src/interaction/scoreboard.rs` — ScoreBoard resource, events
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_collectible.rs` — MCP tool handler

---

## Spec 2.4: `gen_add_door` — Interactive Doors

**Goal:** Add open/close behavior to any entity that should act as a door, with proximity or click triggers and optional key requirements.

### MCP Tool Schema

```json
{
  "name": "gen_add_door",
  "description": "Add interactive door behavior to an entity",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "trigger": { "type": "enum", "values": ["proximity", "click"], "default": "proximity" },
    "open_angle": { "type": "f32", "default": 90.0 },
    "open_duration": { "type": "f32", "default": 1.5 },
    "auto_close": { "type": "bool", "default": true },
    "auto_close_delay": { "type": "f32", "default": 3.0 },
    "sound_open": { "type": "string", "optional": true },
    "sound_close": { "type": "string", "optional": true },
    "requires_key": { "type": "string", "optional": true }
  }
}
```

### Implementation

1. **Door component:** `struct Door { state: DoorState, open_angle, open_duration, auto_close, auto_close_delay, requires_key, original_rotation }`. `DoorState`: `Closed | Opening(progress) | Open(timer) | Closing(progress)`.

2. **Pivot point:** Doors rotate around their local Y axis at their transform origin. The MCP tool assumes the entity's origin is already at the hinge point. Document that door meshes should be authored with the hinge at the origin.

3. **Trigger:**
   - `proximity`: When player enters 3m radius, attempt to open. When player exits 3m radius and `auto_close` is true, start close timer.
   - `click`: Show "Press E to open/close" prompt when player is within 3m. Toggle on press.

4. **Key requirement:** If `requires_key` is set, check if the player's inventory (`PlayerInventory` resource) contains the key string. If not, show "Locked — requires [key name]" floating text. Keys are collected via `gen_add_collectible` with a matching category.

5. **Animation:** Smooth rotation from `original_rotation` to `original_rotation + open_angle` around Y axis using ease-in-out cubic. Play `sound_open` at start of opening, `sound_close` at start of closing.

6. **Auto-close:** When `auto_close` is true and door is in `Open` state, a timer counts down from `auto_close_delay`. On expiry, transition to `Closing`.

### Acceptance Criteria

- [ ] Door opens by rotating around Y axis to open_angle
- [ ] Proximity trigger opens door when player approaches
- [ ] Click trigger opens door on E key press
- [ ] Door closes automatically after auto_close_delay
- [ ] Key requirement blocks opening without matching collectible
- [ ] "Locked" message shown when key is missing
- [ ] Open/close sounds play at animation start
- [ ] Smooth ease-in-out rotation animation

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/interaction/door.rs` — component, state machine, animation
- `localgpt/crates/localgpt-gen/src/interaction/inventory.rs` — PlayerInventory resource
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_door.rs` — MCP tool handler

---

## Spec 2.5: `gen_link_entities` — Entity Event Wiring

**Goal:** Connect one entity's event to another entity's action, enabling chain reactions and puzzle logic without code. Inspired by UEFN's device-wiring model.

### MCP Tool Schema

```json
{
  "name": "gen_link_entities",
  "description": "Wire one entity's event to trigger another entity's action",
  "parameters": {
    "source_id": { "type": "string", "required": true },
    "source_event": { "type": "string", "required": true, "description": "e.g. 'clicked', 'state_changed:is_active', 'proximity_entered'" },
    "target_id": { "type": "string", "required": true },
    "target_action": { "type": "string", "required": true, "description": "e.g. 'toggle_state:is_open', 'play_animation:open', 'enable', 'disable'" },
    "condition": { "type": "string", "optional": true, "description": "Boolean expression, e.g. 'source.is_active AND other_entity.is_active'" }
  }
}
```

### Implementation

1. **EntityLink component:** `struct EntityLink { source_event, target_entity, target_action, condition }`. Stored on the source entity. One entity can have multiple links (Vec<EntityLink>).

2. **Event types** (parsed from `source_event` string):
   - `"clicked"` — entity was click-interacted
   - `"proximity_entered"` / `"proximity_exited"` — player entered/left trigger radius
   - `"state_changed:<key>"` — entity state value changed
   - `"collected"` — entity was picked up
   - `"timer_fired"` — timer trigger elapsed
   - `"destroyed"` — entity was removed

3. **Action types** (parsed from `target_action` string):
   - `"toggle_state:<key>"` — flip boolean state on target
   - `"set_state:<key>:<value>"` — set specific state value
   - `"play_animation:<name>"` — trigger named animation
   - `"enable"` / `"disable"` — show/hide target
   - `"destroy"` — despawn target
   - `"teleport_player"` — move player to target's position

4. **Link resolution system:** When `TriggerFired` event occurs, query all `EntityLink` components on the source entity. For each link matching the event, evaluate the optional condition, then emit a `LinkedActionRequest { target, action }` event.

5. **Condition evaluation:** Simple boolean expressions with entity state lookups. Support `AND`, `OR`, `NOT`, and `entity_name.state_key` references. Parse at link creation time into an AST for efficient runtime evaluation.

6. **Debug visualization:** In debug mode, draw colored lines between linked entities. Green = active link, gray = condition not met, red = target not found.

### Acceptance Criteria

- [ ] Link created between source event and target action
- [ ] Clicking source entity triggers action on target entity
- [ ] State change on source triggers linked actions
- [ ] Conditional links only fire when condition is met
- [ ] Multiple links from one source all fire independently
- [ ] Chain reactions work (A → B → C)
- [ ] Debug mode shows link connections as colored lines
- [ ] Invalid entity IDs produce clear error messages

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/interaction/link.rs` — EntityLink, resolution system, condition evaluator
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_link_entities.rs` — MCP tool handler
