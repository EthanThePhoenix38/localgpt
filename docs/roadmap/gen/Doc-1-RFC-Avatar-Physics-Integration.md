# RFC: Avatar, Physics & Player Control Integration

**Status:** Draft  
**Author:** Yi  
**Date:** 2026-03-21  
**Target crates:** `localgpt-gen` (feature: `physics`)  
**Depends on:** Gen mode MCP tools (existing), Avian 3D, bevy-tnua  
**Supersedes:** N/A (integrates existing scaffolding)

---

## 1. Summary

Wire up, test, and complete the player character system for LocalGPT Gen so that `gen_spawn_player` produces a controllable avatar with physics-based movement, collision, camera follow, and seamless integration with the existing FreeFly spectator camera. After this RFC is implemented, **a user can generate a world via MCP tools and immediately walk through it with WASD/mouse, jump, sprint, and toggle between exploration and spectator modes**.

### 1.1 Why This Is Not Greenfield

The codebase already contains substantial scaffolding:

- `crates/gen/src/character/player.rs` — `Player`, `PlayerConfig`, `PlayerInput`, `SpawnPlayerParams`, `spawn_player()` (physics + non-physics paths), `player_input_system`, `player_movement_system`
- `crates/gen/src/character/camera.rs` — `PlayerCamera`, `CameraPov` (ThirdPerson/FirstPerson/Fixed), `camera_follow_system` (with spring-arm collision avoidance via `SpatialQuery`), `camera_input_system`, `player_mesh_visibility_system`
- `crates/gen/src/gen3d/avatar.rs` — `AvatarEntity`, `CameraMode` (Attached/FreeFly), `PovState`, `AvatarMovementConfig`, `avatar_movement`, `avatar_look`, `camera_follow_avatar`
- `crates/gen/src/mcp/avatar_tools.rs` — `GenSpawnPlayerTool`, `GenSetSpawnPointTool`, `GenSpawnNpcTool`, `GenSetNpcDialogueTool`, `GenSetCameraModeTool`
- `crates/gen/src/physics/` — `PhysicsBodyPlugin`, `ColliderPlugin`, `ForceFieldPlugin`, `GravityPlugin`, `JointPlugin` all registered
- `crates/gen/src/gen3d/commands.rs` — `SpawnPlayer`, `SetSpawnPoint`, `SpawnNpc`, `SetNpcDialogue`, `SetPlayerCameraMode` commands defined

**This RFC's job is to verify, integrate, and fill gaps — not rebuild.**

---

## 2. Architecture Decision: Avatar ↔ Player Unification

### 2.1 The Problem

Two parallel systems exist for controlling an entity in the scene:

| System | File | Input | Camera | Physics | Purpose |
|--------|------|-------|--------|---------|---------|
| **Avatar** | `avatar.rs` | WASD + mouse + scroll | FreeFly ↔ Attached (Tab) | None (transform-based) | Scene spectator/editor camera |
| **Player** | `character/player.rs` + `camera.rs` | WASD + mouse + space + shift | ThirdPerson ↔ FirstPerson (V) | Avian + Tnua (behind feature flag) | Explorable world character |

Both register input systems. Both move entities. Both control the camera. Running both simultaneously will cause conflicts.

### 2.2 Recommended Design: Three-Mode Camera System

Merge into a single unified system with three modes:

```
┌─────────────────────────────────────────────────────┐
│                  CameraController                    │
│                                                     │
│  Mode A: FreeFly (default, no player spawned)       │
│  ├── WASD + Space/Shift for 6DOF movement           │
│  ├── Mouse look                                     │
│  ├── Scroll wheel speed                             │
│  └── No physics, no collision                       │
│                                                     │
│  Mode B: ThirdPerson (after gen_spawn_player)       │
│  ├── WASD moves player entity                       │
│  ├── Mouse orbits camera around player              │
│  ├── Space = jump, Shift = sprint                   │
│  ├── Avian physics + Tnua controller                │
│  └── Spring-arm collision avoidance                 │
│                                                     │
│  Mode C: FirstPerson (V key from ThirdPerson)       │
│  ├── WASD moves player entity                       │
│  ├── Mouse = player look direction                  │
│  ├── Camera at eye height, mesh hidden              │
│  ├── Same physics as ThirdPerson                    │
│  └── No spring-arm needed                           │
│                                                     │
│  Tab: FreeFly ↔ last player mode (B or C)           │
│  V: ThirdPerson ↔ FirstPerson (only when player)    │
│  N: Noclip toggle (disable collision in B/C)        │
└─────────────────────────────────────────────────────┘
```

**State transitions:**

```
App start → FreeFly (no player exists)
                │
    gen_spawn_player called
                │
                ▼
          ThirdPerson ◄──── V key ────► FirstPerson
                │                            │
            Tab key                      Tab key
                │                            │
                ▼                            ▼
            FreeFly ◄──── Tab key ────► (return to last player mode)
```

### 2.3 Implementation Strategy

Keep `avatar.rs` as the FreeFly mode. Promote `character/player.rs` + `character/camera.rs` as the Player modes. Unify via a single `CameraController` resource that tracks the active mode and gates which systems run.

```rust
/// Unified camera controller state.
#[derive(Resource)]
pub struct CameraController {
    /// Current active mode.
    pub mode: ControlMode,
    /// Last player mode before switching to FreeFly.
    pub last_player_mode: PlayerCameraMode,
    /// Whether noclip is active (disables collision in player modes).
    pub noclip: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    /// No player spawned, or Tab-detached. Free camera.
    FreeFly,
    /// Player spawned, camera follows. Uses Tnua physics.
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerCameraMode {
    #[default]
    ThirdPerson,
    FirstPerson,
}
```

**Run conditions replace the current dual-system approach:**

```rust
// Current (conflicting):
avatar::avatar_movement.run_if(avatar::in_attached_mode)
player_input_system  // always runs

// New (exclusive):
avatar::avatar_movement.run_if(in_freefly_mode)
player_input_system.run_if(in_player_mode)
camera_follow_system.run_if(in_player_mode)
camera_follow_avatar.run_if(in_freefly_mode)  // rename: this IS freefly
```

---

## 3. Cargo.toml Changes

### 3.1 Verify existing dependencies

The following should already be in `Cargo.toml` behind the `physics` feature:

```toml
[features]
default = ["physics"]  # ← CRITICAL: physics should be ON by default for gen mode
physics = ["dep:avian3d", "dep:bevy-tnua", "dep:bevy-tnua-avian3d"]

[dependencies]
avian3d = { version = "0.5", optional = true }  # Check: is this pinned? Need 0.5 or 0.6?
bevy-tnua = { version = "0.30", optional = true }
bevy-tnua-avian3d = { version = "0.30", optional = true }
```

### 3.2 Audit actions

```
□ Check: What Avian version is currently pinned?
□ Check: Is `physics` in the `default` features list?
□ Check: Does `cargo build --features physics` compile clean?
□ Check: bevy-tnua version compatibility with pinned Avian version
□ Check: bevy-tnua-avian3d version compatibility
□ Decision: Upgrade to Avian 0.6 (for move-and-slide) or stay on 0.5?
    - If Avian 0.6: check tnua compatibility, may need git dep
    - If Avian 0.5: skip move-and-slide, Tnua's floating controller is sufficient
```

### 3.3 Avian 0.6 evaluation

Avian 0.6 released March 16, 2026 with built-in `MoveAndSlide` system parameter — the fundamental kinematic character controller algorithm. Benefits:

- Native move-and-slide means less reliance on Tnua for basic movement
- Better wall-sliding behavior out of the box
- New `contact_impulse` and `contact_normal` queries for interaction triggers

Risks:

- Five days old — may have undiscovered bugs
- bevy-tnua-avian3d may not support 0.6 yet (check `bevy-tnua` releases)
- If tnua doesn't support 0.6, use bevy_ahoy instead (same author as Avian's move-and-slide)

**Recommendation:** Check tnua compatibility first. If compatible, use Avian 0.6. If not, stay on 0.5 — the existing Tnua floating controller is sufficient for an MVP.

---

## 4. Step-by-Step Implementation

### Step 0: Build Audit (Day 1, ~4 hours)

This is the most important step. Before writing any new code:

```bash
# 1. Enable physics feature and compile
cargo build --features physics 2>&1 | head -100

# 2. If clean, run gen mode
cargo run --features physics -- gen

# 3. In gen mode (or via MCP client), call:
gen_spawn_player(position=[0,2,0])

# 4. Observe:
#    □ Does a blue capsule appear at (0,2,0)?
#    □ Does it fall to the ground (gravity)?
#    □ Can you move with WASD?
#    □ Does the camera follow?
#    □ Can you jump with Space?
#    □ Does sprint (Shift) work?
#    □ Does V toggle first/third person?
#    □ Does Tab toggle to FreeFly?
#    □ If you walk into a gen_spawn_primitive box, do you collide?

# 5. Record results in Doc 0 session log
```

**Expected outcome:** Some things work, some don't. The audit tells you exactly what to fix vs what's already wired up.

### Step 1: Resolve Compile Errors (Day 1, ~2 hours)

Common issues to expect:

| Error | Likely Cause | Fix |
|-------|-------------|-----|
| `unresolved import avian3d` | Feature not enabled or version mismatch | Check Cargo.toml `[features]` |
| `TnuaController not found` | bevy-tnua version mismatch | Pin compatible version |
| `SpatialQuery not found` | Avian API changed between versions | Check migration guide |
| `LockedAxes::new()` deprecated | Avian 0.6 changed API | Use `LockedAxes::ALL_LOCKED.unlock_rotation_y()` or equivalent |
| Duplicate system registration | Both avatar.rs and player.rs register conflicting systems | Resolve with CameraController gating |

### Step 2: Unify Avatar and Player Systems (Day 2, ~6 hours)

Following the design from Section 2:

**2a.** Create `CameraController` resource in a new file `crates/gen/src/gen3d/camera_controller.rs`:

```rust
use bevy::prelude::*;

/// Unified camera mode controller.
#[derive(Resource)]
pub struct CameraController {
    pub mode: ControlMode,
    pub last_player_mode: PlayerCameraMode,
    pub noclip: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            mode: ControlMode::FreeFly,
            last_player_mode: PlayerCameraMode::ThirdPerson,
            noclip: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlMode {
    #[default]
    FreeFly,
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerCameraMode {
    #[default]
    ThirdPerson,
    FirstPerson,
}

/// Run condition: camera is in FreeFly mode.
pub fn in_freefly_mode(controller: Res<CameraController>) -> bool {
    controller.mode == ControlMode::FreeFly
}

/// Run condition: camera is in Player mode (third or first person).
pub fn in_player_mode(controller: Res<CameraController>) -> bool {
    controller.mode == ControlMode::Player
}
```

**2b.** Modify `plugin.rs` system registration to use new run conditions:

```rust
// Replace current avatar/freefly conditional systems with:
.add_systems(
    Update,
    (
        // FreeFly systems (spectator/editor camera)
        fly_cam_movement,
        fly_cam_look,
        fly_cam_scroll_speed,
    )
        .run_if(in_freefly_mode.and(not_ui_hovered)),
)
.add_systems(
    Update,
    (
        // Player systems (explorable world character)
        player_input_system,
        player_movement_system,
    )
        .chain()
        .run_if(in_player_mode.and(not_ui_hovered)),
)
.add_systems(
    Update,
    camera_follow_system
        .run_if(in_player_mode)
        .after(player_movement_system),
)
.add_systems(
    Update,
    (
        toggle_mode_system,      // Tab key
        toggle_pov_system,       // V key
        toggle_noclip_system,    // N key
    ),
)
```

**2c.** Implement mode toggle systems:

```rust
/// Tab key: toggle between FreeFly and Player mode.
fn toggle_mode_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut controller: ResMut<CameraController>,
    player_query: Query<Entity, With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    match controller.mode {
        ControlMode::FreeFly => {
            // Only switch to Player if a player entity exists
            if player_query.iter().next().is_some() {
                controller.mode = ControlMode::Player;
            }
        }
        ControlMode::Player => {
            controller.mode = ControlMode::FreeFly;
        }
    }
}

/// V key: toggle ThirdPerson ↔ FirstPerson (only in Player mode).
fn toggle_pov_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut controller: ResMut<CameraController>,
) {
    if controller.mode != ControlMode::Player {
        return;
    }
    if !keys.just_pressed(KeyCode::KeyV) {
        return;
    }

    controller.last_player_mode = match controller.last_player_mode {
        PlayerCameraMode::ThirdPerson => PlayerCameraMode::FirstPerson,
        PlayerCameraMode::FirstPerson => PlayerCameraMode::ThirdPerson,
    };
}

/// N key: toggle noclip (disable collision, enable flight in Player mode).
fn toggle_noclip_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut controller: ResMut<CameraController>,
) {
    if !keys.just_pressed(KeyCode::KeyN) {
        return;
    }
    controller.noclip = !controller.noclip;
    // TODO: When noclip is on, disable Avian collision on player entity
    // and switch to transform-based movement
}
```

**2d.** Hook `gen_spawn_player` command to switch mode:

In the command handler for `GenCommand::SpawnPlayer`, after spawning the entity:

```rust
// After spawn_player() returns the entity:
controller.mode = ControlMode::Player;
controller.last_player_mode = match params.camera_mode_enum() {
    CameraMode::FirstPerson => PlayerCameraMode::FirstPerson,
    CameraMode::ThirdPerson => PlayerCameraMode::ThirdPerson,
};
```

### Step 3: Ensure World Geometry Has Colliders (Day 3, ~4 hours)

**This is the silent killer — if spawned primitives don't have colliders, the player falls through everything.**

Audit every spawn command and ensure collider insertion:

| Command | Spawns | Needs Collider | Implementation |
|---------|--------|---------------|----------------|
| `gen_spawn_primitive` (box) | `Mesh3d` box | Yes | `Collider::cuboid(hx, hy, hz)` |
| `gen_spawn_primitive` (sphere) | `Mesh3d` sphere | Yes | `Collider::sphere(radius)` |
| `gen_spawn_primitive` (cylinder) | `Mesh3d` cylinder | Yes | `Collider::cylinder(radius, half_height)` |
| `gen_spawn_primitive` (plane) | `Mesh3d` plane | Yes | `Collider::cuboid(hx, 0.01, hz)` (thin box) |
| `gen_spawn_primitive` (capsule) | `Mesh3d` capsule | Yes | `Collider::capsule(radius, half_height)` |
| `gen_load_gltf` | glTF model | Yes | `Collider::trimesh_from_mesh()` on load |
| `gen_spawn_mesh` | Custom mesh | Yes | `Collider::trimesh_from_mesh()` |
| `gen_add_terrain` | Heightmap mesh | Yes | `Collider::trimesh_from_mesh()` on terrain mesh |
| Ground plane (default scene) | Plane mesh | Yes | `Collider::cuboid(50.0, 0.01, 50.0)` |

**Implementation pattern** (add to each spawn function):

```rust
#[cfg(feature = "physics")]
{
    commands.entity(entity).insert((
        RigidBody::Static,  // World geometry is static
        collider,           // Computed per shape type
    ));
}
```

**For glTF models** (most complex case):

```rust
// After loading glTF, traverse children to find meshes and add trimesh colliders
fn add_gltf_colliders(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<(Entity, &Mesh3d), Added<Mesh3d>>,
) {
    for (entity, mesh_handle) in query.iter() {
        if let Some(mesh) = meshes.get(&mesh_handle.0) {
            if let Some(collider) = Collider::trimesh_from_mesh(mesh) {
                commands.entity(entity).insert((
                    RigidBody::Static,
                    collider,
                ));
            }
        }
    }
}
```

### Step 4: Spawn Point and Respawn (Day 3, ~2 hours)

Verify the existing `SpawnPointPlugin` and add respawn-on-fall:

```rust
/// System to respawn player when they fall below the world.
fn respawn_on_fall_system(
    mut player_query: Query<&mut Transform, With<Player>>,
    spawn_points: Query<(&Transform, &SpawnPoint), Without<Player>>,
) {
    for mut player_transform in player_query.iter_mut() {
        if player_transform.translation.y < -50.0 {
            // Find default spawn point
            let respawn_pos = spawn_points
                .iter()
                .find(|(_, sp)| sp.is_default)
                .or_else(|| spawn_points.iter().next())
                .map(|(t, _)| t.translation)
                .unwrap_or(Vec3::new(0.0, 2.0, 0.0));

            player_transform.translation = respawn_pos;

            // Reset velocity if physics enabled
            // (handled by Tnua controller reset)
        }
    }
}
```

### Step 5: Polish Movement Feel (Day 4, ~4 hours)

The difference between "technically works" and "feels good":

**5a. Tnua configuration for snappy movement:**

```rust
TnuaBuiltinWalk {
    desired_velocity,
    float_height: 1.0,
    // Snappy acceleration (reach full speed quickly)
    acceleration: 50.0,
    // Instant deceleration (stop when keys released)
    air_acceleration: 20.0,
    // Coyote time: can still jump briefly after walking off edge
    coyote_time: 0.15,
    ..Default::default()
}

TnuaBuiltinJump {
    height: config.jump_force,
    // Allow jump input slightly before landing
    input_buffer_time: 0.1,
    // Shorter jump on tap, full height on hold
    shorten_extra_gravity: 40.0,
    ..Default::default()
}
```

**5b. Camera smoothing:**

```rust
// In camera_follow_system, use smooth interpolation:
let smoothing_factor = 1.0 - (-15.0 * time.delta_secs()).exp();
camera_transform.translation = camera_transform
    .translation
    .lerp(target_position, smoothing_factor);
```

**5c. Mouse sensitivity and inversion:**

```rust
#[derive(Resource)]
pub struct MouseSensitivity {
    pub sensitivity: f32,
    pub invert_y: bool,
}

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self {
            sensitivity: 0.003,
            invert_y: false,
        }
    }
}
```

---

## 5. MCP Tool Behavior Specifications

### gen_spawn_player

**When called:**
1. If a player entity already exists, despawn it and its camera
2. Spawn new player entity with physics components
3. Spawn companion camera entity parented to scene (not to player)
4. Set `CameraController.mode = ControlMode::Player`
5. Return `{ entity_id, position }`

**Physics components spawned:**
- `RigidBody::Dynamic`
- `Collider::capsule(collision_radius, collision_height - 2 * collision_radius)`
- `LockedAxes` — lock X and Z rotation (prevent toppling)
- `TnuaController::default()`
- `TnuaAvian3dSensorShape(Collider::cylinder(collision_radius * 0.95, 0.0))`

**Visual:**
- Blue capsule mesh (placeholder until custom avatar meshes)
- Hidden in FirstPerson mode via `player_mesh_visibility_system` (already exists)

### gen_set_spawn_point

**When called:**
1. If `is_default: true`, remove `is_default` from any existing default spawn point
2. Spawn or update a spawn point marker entity
3. If no player exists, this is informational only
4. Return `{ entity_id, position }`

### gen_set_camera_mode

**When called:**
1. Accept modes: "first_person", "third_person", "freefly"
2. Update `CameraController` accordingly
3. If switching to freefly, detach camera from player
4. If switching to player mode and no player exists, return error

---

## 6. Testing Checklist

Execute these tests **in order** after implementation. Each test builds on the previous.

### 6.1 Foundation Tests (no MCP, just Bevy)

```
□ cargo build --features physics — compiles without errors
□ cargo test --features physics — all existing tests pass
□ cargo run --features physics -- gen — app launches, default scene renders
□ FreeFly camera works: WASD movement, mouse look, scroll speed
```

### 6.2 Player Spawn Tests (via MCP)

```
□ gen_spawn_player() — blue capsule appears at (0,1,0)
□ Capsule falls to ground plane (gravity works)
□ Capsule stops on ground (collision works)
□ WASD moves player on ground plane
□ Mouse rotates view around player (third person)
□ Space bar makes player jump
□ Player lands back on ground after jump
□ Shift key increases movement speed (sprint)
□ gen_spawn_player(position=[10,5,10]) — spawns at specified position
□ gen_spawn_player(camera_mode="first_person") — camera at eye level, capsule hidden
```

### 6.3 Camera Tests

```
□ V key toggles between third-person and first-person
□ Third-person: camera orbits, doesn't clip through walls (spring arm)
□ First-person: camera at eye height (1.8m), capsule mesh hidden
□ Tab key switches to FreeFly (camera detaches, player stays in place)
□ Tab again returns to player view (camera snaps back to follow)
□ Camera smooth interpolation (no jitter, no hard snapping)
```

### 6.4 Collision Tests

```
□ gen_spawn_primitive(shape="box", position=[3,0.5,0], size=[1,1,1])
□ Walk player into box — player stops, doesn't pass through
□ Jump onto box — player stands on top
□ gen_spawn_primitive(shape="sphere", position=[5,1,0], size=[1,1,1])
□ Walk into sphere — collision works
□ gen_add_terrain() — terrain generates with collision mesh
□ Walk player over terrain — follows height, doesn't fall through
□ N key (noclip) — player passes through geometry, floats
□ N key again — collision re-enables, player falls to ground
```

### 6.5 Spawn Point Tests

```
□ gen_set_spawn_point(position=[0,2,0])
□ Walk player off a high edge → falls below y=-50 → respawns at spawn point
□ gen_set_spawn_point(position=[10,2,10], is_default=true) — new default
□ Fall again → respawns at new position
```

### 6.6 Integration Tests (full workflow)

```
□ Full Willowmere Village workflow:
  1. Load Willowmere (gen_load_world or rebuild)
  2. gen_spawn_player(position=[0,2,0])
  3. Walk through village — all buildings have collision
  4. Jump onto walls/roofs — collision at all heights
  5. Walk along paths — no falling through terrain
  6. Tab → FreeFly → inspect from above → Tab → back to player
  7. V → first person → walk through doorways → V → back to third

□ MCP roundtrip:
  1. External MCP client sends gen_spawn_player
  2. Client sends gen_spawn_primitive (several objects)
  3. Client sends gen_set_spawn_point
  4. User explores in-engine — all objects solid
  5. Fall off → respawn works
```

---

## 7. Known Gotchas & Bevy-Specific Pitfalls

### 7.1 Behavior Anchor Baking

`gen_add_behavior` anchors animation reference to position at attachment time. If you `gen_modify_entity` to reposition after attaching a behavior, the entity drifts back. Correct pattern: `gen_remove_behavior` → reposition → `gen_add_behavior`. For patrol NPCs, full delete + respawn with corrected waypoints.

**Impact on player:** If any behaviors are ever attached to the player entity, beware this gotcha. Keep player behaviors minimal.

### 7.2 Terrain Height Baseline

Average terrain surface height ≈ half the `height_scale` parameter. No `query_terrain_height(x,z)` tool exists yet (it's in `commands.rs` as `QueryTerrainHeight` but implementation status unknown). All placement requires manual Y offset estimation.

**Impact on player:** When spawning player on terrain, either (a) spawn high and let gravity drop them, or (b) implement `QueryTerrainHeight` as part of this RFC.

**Recommendation:** Spawn player at `requested_y + 5.0` and let Avian gravity bring them down. Simple, reliable, no terrain query needed.

### 7.3 Avian Capsule Orientation

Avian's `Collider::capsule(radius, half_length)` creates a capsule along the **Y axis** by default (correct for an upright character). The `half_length` parameter is half the cylindrical middle section, NOT total height. Total height = `2 * half_length + 2 * radius`.

For a 1.8m tall character with 0.3m radius:
```
half_length = (1.8 - 2 * 0.3) / 2 = 0.6
Collider::capsule(0.3, 0.6)  // radius, half_length
```

The existing code uses `Collider::capsule(radius, height - radius * 2.0)` — verify this produces the correct capsule dimensions. It should be `(height - 2 * radius) / 2` for half_length.

### 7.4 Tnua Float Height

`TnuaBuiltinWalk.float_height` is the distance from the **bottom of the character** to the ground. Not the character's center. Set it to roughly `collision_height / 2` for the character to appear to stand on the ground rather than hover.

### 7.5 Input Conflicts with egui

When the inspector panel (F1) or gallery (G) is open, input should not reach the player. The existing `not_ui_hovered` run condition handles this, but verify it works with the unified system.

### 7.6 Physics Timestep

Avian runs on a fixed timestep (default 60Hz). Tnua must also run on this timestep. Ensure `TnuaController` is updated in `FixedUpdate`, not `Update`:

```rust
// If player_movement_system currently runs in Update, it should move to:
app.add_systems(
    FixedUpdate,
    player_movement_system.run_if(in_player_mode),
);
```

Check the existing registration in `plugin.rs` — if it's in `Update`, this is a bug that will cause jittery movement at variable framerates.

---

## 8. Acceptance Criteria

This RFC is **complete** when:

1. `cargo build --features physics` compiles with zero warnings
2. `gen_spawn_player` via MCP produces a controllable character
3. Player collides with all spawned geometry (primitives, terrain, glTF)
4. Camera follows in third-person with spring-arm collision avoidance
5. V key toggles first/third person
6. Tab key toggles FreeFly/Player mode
7. N key toggles noclip
8. Space = jump with coyote time, Shift = sprint
9. Respawn on fall below y < -50
10. All tests in Section 6 pass
11. Movement feels responsive (high acceleration, instant deceleration, no input lag)

---

## 9. Estimated Effort

| Task | Estimate | Depends On |
|------|----------|------------|
| Build audit (Step 0) | 4 hours | — |
| Resolve compile errors (Step 1) | 2 hours | Step 0 |
| Unify avatar/player (Step 2) | 6 hours | Step 1 |
| World geometry colliders (Step 3) | 4 hours | Step 1 |
| Spawn point + respawn (Step 4) | 2 hours | Step 2 |
| Movement polish (Step 5) | 4 hours | Step 2 |
| Testing + fixing (Section 6) | 4 hours | Steps 2-5 |
| **Total** | **~26 hours / 3-4 days** | |

This assumes the existing scaffolding is roughly correct and needs wiring + testing rather than rewriting. If the build audit reveals fundamental issues (wrong Avian version, broken Tnua API, etc.), add 1-2 days for migration.
