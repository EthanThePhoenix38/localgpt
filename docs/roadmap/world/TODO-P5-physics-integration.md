# Priority 5: Physics Integration

These 5 specs add physical simulation to LocalGPT Gen. Collision and gravity ship with the avatar system (P1); these specs cover explicit physics control, joints, forces, and effects.

**Dependencies:** Avian v0.5 (ECS-native physics), Priority 1 (player collider), Priority 2 (interaction triggers)

---

## Spec 5.1: `gen_set_physics` — Enable Physics on Entities

**Goal:** Make any entity physically simulated with configurable mass, bounciness, friction, and gravity.

### MCP Tool Schema

```json
{
  "name": "gen_set_physics",
  "description": "Enable physics simulation on an entity",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "body_type": { "type": "enum", "values": ["dynamic", "static", "kinematic"], "default": "dynamic" },
    "mass": { "type": "f32", "optional": true, "description": "Mass in kg. Auto-calculated from collider volume if omitted" },
    "restitution": { "type": "f32", "default": 0.3, "description": "Bounciness (0=no bounce, 1=perfect bounce)" },
    "friction": { "type": "f32", "default": 0.5, "description": "Surface friction (0=ice, 1=rubber)" },
    "gravity_scale": { "type": "f32", "default": 1.0, "description": "Gravity multiplier (0=floating, 2=heavy)" },
    "linear_damping": { "type": "f32", "default": 0.1, "description": "Air resistance for linear movement" },
    "angular_damping": { "type": "f32", "default": 0.1, "description": "Air resistance for rotation" },
    "lock_rotation": { "type": "bool", "default": false, "description": "Prevent entity from rotating due to physics" }
  }
}
```

### Implementation

1. **Body types** (Avian components):
   - `dynamic`: `RigidBody::Dynamic` — fully simulated, affected by gravity and forces. Used for pushable crates, balls, debris.
   - `static`: `RigidBody::Static` — immovable, infinite mass. Used for walls, floors, terrain. Default for most scene objects.
   - `kinematic`: `RigidBody::Kinematic` — moved by code, not by physics. Affects dynamic bodies on contact. Used for moving platforms, elevators.

2. **Auto-collider:** If the entity doesn't already have a `Collider` component, auto-generate one:
   - For primitive shapes (box, sphere, cylinder): use the matching Avian collider
   - For meshes: use `Collider::convex_hull_from_mesh` for simple shapes, `Collider::trimesh_from_mesh` for complex geometry

3. **Material properties:** Attach `Restitution::new(restitution)` and `Friction::new(friction)` components.

4. **Gravity:** Set `GravityScale(gravity_scale)` component. Setting to 0 makes the entity float.

5. **Damping:** `LinearDamping(linear_damping)` and `AngularDamping(angular_damping)` for air resistance.

6. **Rotation lock:** If `lock_rotation: true`, set `LockedAxes::ROTATION_LOCKED` to prevent tumbling.

### Acceptance Criteria

- [ ] Dynamic entity falls with gravity and bounces on collision
- [ ] Static entity is immovable regardless of forces
- [ ] Kinematic entity can be moved by code and pushes dynamic entities
- [ ] Mass affects how easily entity is pushed
- [ ] Restitution controls bounce height
- [ ] Friction controls sliding behavior
- [ ] Gravity scale 0 makes entity float
- [ ] Lock rotation prevents tumbling
- [ ] Auto-collider generated when entity has no existing collider
- [ ] Multiple physics entities interact correctly

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/physics/mod.rs` — module root
- `localgpt/crates/localgpt-gen/src/physics/body.rs` — physics setup, auto-collider, material properties
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_physics.rs` — MCP tool handler

---

## Spec 5.2: `gen_add_collider` — Collision Shapes Without Full Physics

**Goal:** Add collision detection to an entity without making it physically simulated. Used for invisible walls, trigger zones, and walkable surfaces.

### MCP Tool Schema

```json
{
  "name": "gen_add_collider",
  "description": "Add a collision shape to an entity",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "shape": { "type": "enum", "values": ["box", "sphere", "capsule", "cylinder", "mesh"], "default": "box" },
    "size": { "type": "vec3", "optional": true, "description": "Dimensions. Auto-fit to mesh if omitted" },
    "offset": { "type": "vec3", "default": [0, 0, 0], "description": "Offset from entity origin" },
    "is_trigger": { "type": "bool", "default": false, "description": "true = sensor only (no physics response, just event detection)" },
    "visible_in_debug": { "type": "bool", "default": true }
  }
}
```

### Implementation

1. **Collider shapes** (Avian):
   - `box`: `Collider::cuboid(half_x, half_y, half_z)` — from `size / 2`
   - `sphere`: `Collider::sphere(radius)` — radius from `size.x / 2`
   - `capsule`: `Collider::capsule(radius, height)` — radius from `size.x / 2`, height from `size.y`
   - `cylinder`: `Collider::cylinder(radius, height)` — radius from `size.x / 2`, height from `size.y`
   - `mesh`: `Collider::trimesh_from_mesh` — exact mesh collision (expensive, use sparingly)

2. **Auto-sizing:** If `size` is omitted, compute bounds from the entity's `Mesh` handle via `Aabb`. Use the AABB extents as the collider size.

3. **Sensor mode:** If `is_trigger: true`, add `Sensor` component. The collider detects overlaps but doesn't produce physical responses. Used for trigger zones (area_enter, area_exit events from P2 interaction system).

4. **Offset:** Spawn the collider as a child entity at the specified offset from the parent entity's origin. This allows placing collision shapes that don't match the visual mesh center.

5. **Debug visualization:** If `visible_in_debug: true`, render a wireframe of the collider shape using Avian's debug rendering (or Bevy gizmos). Hidden in play mode unless debug is enabled.

6. **Invisible walls:** Colliders can be added to entities with no mesh — creating invisible barriers. Document this as a pattern: spawn an empty entity at position, add a box collider sized as a wall.

### Acceptance Criteria

- [ ] Box collider blocks player movement with correct dimensions
- [ ] Sphere collider blocks with spherical shape
- [ ] Capsule and cylinder colliders work correctly
- [ ] Mesh collider matches entity geometry
- [ ] Auto-sizing fits collider to mesh bounds
- [ ] Sensor mode detects overlap without blocking
- [ ] Offset positions collider relative to entity
- [ ] Debug wireframe visible in debug mode
- [ ] Invisible wall pattern works (collider without mesh)

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/physics/collider.rs` — shape creation, auto-sizing, sensor setup
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_collider.rs` — MCP tool handler

---

## Spec 5.3: `gen_add_joint` — Constraints Between Entities

**Goal:** Connect two entities with a physical constraint (hinge, ball, fixed, spring) for doors, bridges, chains, and mechanical objects.

### MCP Tool Schema

```json
{
  "name": "gen_add_joint",
  "description": "Create a physical constraint between two entities",
  "parameters": {
    "entity_a": { "type": "string", "required": true },
    "entity_b": { "type": "string", "required": true },
    "joint_type": { "type": "enum", "values": ["fixed", "revolute", "spherical", "prismatic", "spring"], "required": true },
    "anchor_a": { "type": "vec3", "default": [0, 0, 0], "description": "Local-space attachment point on entity A" },
    "anchor_b": { "type": "vec3", "default": [0, 0, 0], "description": "Local-space attachment point on entity B" },
    "axis": { "type": "vec3", "default": [0, 1, 0], "description": "Rotation/slide axis (revolute/prismatic)" },
    "limits": { "type": "vec2", "optional": true, "description": "[min, max] angle in degrees or distance" },
    "stiffness": { "type": "f32", "optional": true, "description": "Spring stiffness (spring type only)" },
    "damping": { "type": "f32", "optional": true, "description": "Spring damping (spring type only)" }
  }
}
```

### Implementation

1. **Joint types** (Avian):
   - `fixed`: `FixedJoint` — entities locked together rigidly. Used for attaching objects.
   - `revolute`: `RevoluteJoint` — rotation around a single axis. Used for door hinges, wheels, pendulums. Configure `axis` and optional angle `limits`.
   - `spherical`: `SphericalJoint` — rotation around any axis (ball socket). Used for ragdoll shoulders, chain links.
   - `prismatic`: `PrismaticJoint` — sliding along one axis. Used for pistons, drawers, elevators. Configure `axis` and optional distance `limits`.
   - `spring`: `DistanceJoint` with spring compliance. Used for bungee cords, suspension, bouncy connections.

2. **Anchor points:** Specify where the joint attaches in each entity's local space. Default `[0,0,0]` is the entity center. For a door hinge, anchor_a would be at the edge of the door.

3. **Limits:** For revolute joints, `limits` in degrees converted to radians. For prismatic joints, `limits` in world units. Avian's `JointLimit` component.

4. **Spring parameters:** For spring joints, set compliance from `stiffness` (higher = stiffer) and `damping` (higher = less oscillation).

5. **Physics requirement:** Both entities must have `RigidBody` components. If entity_a or entity_b lacks one, auto-add `RigidBody::Dynamic` with a warning log. At least one entity should be dynamic for the joint to have visible effect.

### Acceptance Criteria

- [ ] Fixed joint locks two entities together
- [ ] Revolute joint allows rotation around specified axis
- [ ] Spherical joint allows free rotation
- [ ] Prismatic joint allows sliding along axis
- [ ] Spring joint creates bouncy connection
- [ ] Angle limits constrain revolute joint range
- [ ] Distance limits constrain prismatic joint range
- [ ] Anchor points position joint attachment correctly
- [ ] Joint works between dynamic and static entities

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/physics/joint.rs` — joint creation, type mapping, limits
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_joint.rs` — MCP tool handler

---

## Spec 5.4: `gen_add_force` — Force Fields and Impulses

**Goal:** Apply persistent forces (wind, gravity wells, conveyor belts) or instant impulses (explosions, jump pads) to dynamic entities in an area.

### MCP Tool Schema

```json
{
  "name": "gen_add_force",
  "description": "Create a force field or apply an impulse in an area",
  "parameters": {
    "position": { "type": "vec3", "required": true },
    "force_type": { "type": "enum", "values": ["directional", "point_attract", "point_repel", "vortex", "impulse"], "required": true },
    "strength": { "type": "f32", "default": 10.0 },
    "radius": { "type": "f32", "default": 5.0, "description": "Area of effect radius" },
    "direction": { "type": "vec3", "optional": true, "description": "Force direction (directional type only)" },
    "falloff": { "type": "enum", "values": ["none", "linear", "quadratic"], "default": "linear" },
    "affects_player": { "type": "bool", "default": true },
    "continuous": { "type": "bool", "default": true, "description": "false = one-time impulse, true = persistent force" }
  }
}
```

### Implementation

1. **Force field entity:** Spawn at `position` with a sensor `Collider::sphere(radius)` and a `ForceField` component storing all parameters.

2. **Force types:**
   - `directional`: Constant force in `direction` (e.g., wind blowing east). Applied equally to all entities in radius.
   - `point_attract`: Force pulls entities toward `position` (gravity well). Magnitude increases closer to center.
   - `point_repel`: Force pushes entities away from `position` (explosion). Magnitude decreases with distance.
   - `vortex`: Force tangent to the radius circle, creating a spinning motion around `position`.
   - `impulse`: Single instantaneous velocity change on entities currently in radius. Not persistent.

3. **Falloff:**
   - `none`: full strength everywhere in radius
   - `linear`: strength × (1 - distance/radius)
   - `quadratic`: strength × (1 - distance/radius)²

4. **Application system:** Each physics frame, query all `ForceField` entities. For each dynamic `RigidBody` within the sensor radius, compute and apply `ExternalForce` (continuous) or `ExternalImpulse` (one-shot).

5. **Player interaction:** If `affects_player: true`, apply force to the player entity as well. This enables wind zones, jump pads, and conveyor belts affecting the player.

6. **Visual indicator:** Optional debug visualization — translucent sphere showing the radius, with animated arrows showing force direction.

### Acceptance Criteria

- [ ] Directional force pushes entities in specified direction
- [ ] Point attract pulls entities toward center
- [ ] Point repel pushes entities away from center
- [ ] Vortex creates spinning motion around center
- [ ] Impulse applies one-time velocity change
- [ ] Falloff reduces force with distance
- [ ] Force affects player when affects_player is true
- [ ] Force does not affect static entities
- [ ] Multiple force fields can overlap and combine

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/physics/force.rs` — ForceField component, force application system
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_force.rs` — MCP tool handler

---

## Spec 5.5: `gen_set_gravity` — Global and Per-Entity Gravity

**Goal:** Control gravity direction and strength globally or per-entity, enabling low-gravity zones, zero-G areas, and inverted gravity puzzles.

### MCP Tool Schema

```json
{
  "name": "gen_set_gravity",
  "description": "Set global or per-entity gravity",
  "parameters": {
    "entity_id": { "type": "string", "optional": true, "description": "If omitted, sets global gravity" },
    "direction": { "type": "vec3", "default": [0, -1, 0] },
    "strength": { "type": "f32", "default": 9.81 },
    "zone_radius": { "type": "f32", "optional": true, "description": "Create a gravity zone affecting entities within radius" },
    "zone_position": { "type": "vec3", "optional": true, "description": "Center of gravity zone (required if zone_radius set)" },
    "transition_duration": { "type": "f32", "default": 0.5, "description": "Seconds to blend to new gravity" }
  }
}
```

### Implementation

1. **Global gravity:** Set Avian's `Gravity` resource to `direction.normalize() * strength`. This affects all dynamic entities without a `GravityScale` override.

2. **Per-entity gravity:** Set `GravityScale` component on the specified entity. Compute the scale as `strength / 9.81` relative to the global gravity direction. For direction override, use `ExternalForce` to cancel global gravity and apply custom gravity direction.

3. **Gravity zones:** Spawn a sensor `Collider::sphere(zone_radius)` at `zone_position`. System monitors entities entering/exiting. On enter: store original gravity, apply zone gravity. On exit: restore original gravity. Smooth transition over `transition_duration`.

4. **Transition blending:** When gravity changes, lerp from current gravity vector to target over `transition_duration`. This prevents jarring instant gravity switches. Use a `GravityTransition { from, to, progress, duration }` component.

5. **Player gravity:** Player character respects gravity changes. The bevy-tnua controller handles arbitrary gravity directions (its `up` vector is configurable). Update the controller's `up` to match the gravity zone's direction.

6. **Presets via strength values:**
   - Earth: 9.81
   - Moon: 1.62
   - Mars: 3.72
   - Zero-G: 0.0
   - Jupiter: 24.79

### Acceptance Criteria

- [ ] Global gravity change affects all dynamic entities
- [ ] Per-entity gravity overrides global for specific entity
- [ ] Gravity zone affects entities entering the zone
- [ ] Entities leaving gravity zone return to normal gravity
- [ ] Transition blending smoothly interpolates gravity change
- [ ] Player movement adapts to gravity direction changes
- [ ] Zero gravity (strength=0) makes entities float
- [ ] Inverted gravity (direction [0,1,0]) makes entities fall upward
- [ ] Multiple gravity zones can coexist

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/physics/gravity.rs` — global/per-entity/zone gravity, transitions
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_gravity.rs` — MCP tool handler
