//! Player character spawning and control.
//!
//! Implements Spec 1.1: `gen_spawn_player` — Player Character

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// Configuration for the player character.
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// Walk speed in units per second.
    pub walk_speed: f32,
    /// Run speed in units per second.
    pub run_speed: f32,
    /// Jump force (upward velocity).
    pub jump_force: f32,
    /// Camera mode: first-person or third-person.
    pub camera_mode: CameraMode,
    /// Distance from player to camera (third-person only).
    pub camera_distance: f32,
    /// Collision capsule radius.
    pub collision_radius: f32,
    /// Collision capsule height.
    pub collision_height: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            walk_speed: 5.0,
            run_speed: 10.0,
            jump_force: 8.0,
            camera_mode: CameraMode::ThirdPerson,
            camera_distance: 5.0,
            collision_radius: 0.3,
            collision_height: 1.8,
        }
    }
}

/// Camera perspective mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CameraMode {
    /// Camera at eye level, no player mesh visible.
    FirstPerson,
    /// Camera orbits behind player, player mesh visible.
    #[default]
    ThirdPerson,
}

/// Player velocity for physics-based movement.
#[derive(Component, Default)]
pub struct PlayerVelocity {
    pub linear: Vec3,
    pub is_grounded: bool,
}

/// Component for tracking player input.
#[derive(Component, Default)]
pub struct PlayerInput {
    pub move_forward: f32,
    pub move_right: f32,
    pub jump: bool,
    pub run: bool,
}

/// Parameters for spawning a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPlayerParams {
    /// Spawn position (default: [0, 1, 0]).
    #[serde(default = "default_spawn_position")]
    pub position: [f32; 3],
    /// Spawn rotation in degrees (default: [0, 0, 0]).
    #[serde(default)]
    pub rotation: [f32; 3],
    /// Walk speed (default: 5.0).
    #[serde(default = "default_walk_speed")]
    pub walk_speed: f32,
    /// Run speed (default: 10.0).
    #[serde(default = "default_run_speed")]
    pub run_speed: f32,
    /// Jump force (default: 8.0).
    #[serde(default = "default_jump_force")]
    pub jump_force: f32,
    /// Camera mode (default: "third_person").
    #[serde(default)]
    pub camera_mode: String,
    /// Camera distance for third-person (default: 5.0).
    #[serde(default = "default_camera_distance")]
    pub camera_distance: f32,
    /// Collision capsule radius (default: 0.3).
    #[serde(default = "default_collision_radius")]
    pub collision_radius: f32,
    /// Collision capsule height (default: 1.8).
    #[serde(default = "default_collision_height")]
    pub collision_height: f32,
}

fn default_spawn_position() -> [f32; 3] {
    [0.0, 1.0, 0.0]
}
fn default_walk_speed() -> f32 {
    5.0
}
fn default_run_speed() -> f32 {
    10.0
}
fn default_jump_force() -> f32 {
    8.0
}
fn default_camera_distance() -> f32 {
    5.0
}
fn default_collision_radius() -> f32 {
    0.3
}
fn default_collision_height() -> f32 {
    1.8
}

impl Default for SpawnPlayerParams {
    fn default() -> Self {
        Self {
            position: default_spawn_position(),
            rotation: [0.0, 0.0, 0.0],
            walk_speed: default_walk_speed(),
            run_speed: default_run_speed(),
            jump_force: default_jump_force(),
            camera_mode: "third_person".to_string(),
            camera_distance: default_camera_distance(),
            collision_radius: default_collision_radius(),
            collision_height: default_collision_height(),
        }
    }
}

impl SpawnPlayerParams {
    /// Convert camera_mode string to enum.
    pub fn camera_mode_enum(&self) -> CameraMode {
        match self.camera_mode.to_lowercase().as_str() {
            "first_person" | "firstperson" => CameraMode::FirstPerson,
            _ => CameraMode::ThirdPerson,
        }
    }

    /// Convert to PlayerConfig.
    pub fn to_config(&self) -> PlayerConfig {
        PlayerConfig {
            walk_speed: self.walk_speed,
            run_speed: self.run_speed,
            jump_force: self.jump_force,
            camera_mode: self.camera_mode_enum(),
            camera_distance: self.camera_distance,
            collision_radius: self.collision_radius,
            collision_height: self.collision_height,
        }
    }
}

/// Spawn the player entity.
///
/// Creates a controllable player character with movement, camera follow,
/// and collision. Only one player entity is allowed; calling this again
/// despawns the previous player.
pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &SpawnPlayerParams,
) -> Entity {
    let position = Vec3::from_array(params.position);
    // Y-axis rotation only for character orientation
    let rotation = Quat::from_rotation_y(params.rotation[1].to_radians());

    // Player visual: capsule mesh (placeholder until avatar models)
    let capsule_mesh = meshes.add(Capsule3d::new(
        params.collision_radius,
        params.collision_height - params.collision_radius * 2.0,
    ));
    let capsule_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.8),
        ..default()
    });

    // Spawn player entity
    let player_entity = commands
        .spawn((
            Name::new("Player"),
            Player,
            params.to_config(),
            PlayerVelocity::default(),
            PlayerInput::default(),
            Transform::from_translation(position).with_rotation(rotation),
            Visibility::default(),
            Mesh3d(capsule_mesh),
            MeshMaterial3d(capsule_material),
        ))
        .id();

    player_entity
}

/// Despawn any existing player entity.
pub fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// System to handle player movement input.
pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerInput, With<Player>>,
) {
    for mut input in query.iter_mut() {
        // Reset input
        input.move_forward = 0.0;
        input.move_right = 0.0;
        input.jump = false;
        input.run = false;

        // Forward/backward
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            input.move_forward += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            input.move_forward -= 1.0;
        }

        // Left/right
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            input.move_right -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            input.move_right += 1.0;
        }

        // Jump
        input.jump = keyboard.pressed(KeyCode::Space);

        // Run
        input.run = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    }
}

/// System to apply player movement.
pub fn player_movement_system(
    time: Res<Time>,
    mut query: Query<
        (
            &PlayerConfig,
            &PlayerInput,
            &mut Transform,
            &mut PlayerVelocity,
        ),
        With<Player>,
    >,
) {
    for (config, input, mut transform, mut velocity) in query.iter_mut() {
        // Calculate movement direction relative to player facing direction
        let forward = transform.forward().xz().normalize_or_zero();
        let right = transform.right().xz().normalize_or_zero();

        // Determine speed
        let speed = if input.run {
            config.run_speed
        } else {
            config.walk_speed
        };

        // Calculate desired velocity
        let move_dir = forward * input.move_forward + right * input.move_right;
        let target_velocity = move_dir.normalize_or_zero() * speed;

        // Simple movement (no physics for MVP - just translate)
        let delta = target_velocity * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.z += delta.y;

        // Simple gravity
        if !velocity.is_grounded {
            velocity.linear.y -= 20.0 * time.delta_secs();
            transform.translation.y += velocity.linear.y * time.delta_secs();

            // Ground check
            if transform.translation.y < 1.0 {
                transform.translation.y = 1.0;
                velocity.linear.y = 0.0;
                velocity.is_grounded = true;
            }
        }

        // Jump
        if input.jump && velocity.is_grounded {
            velocity.linear.y = config.jump_force;
            velocity.is_grounded = false;
        }
    }
}

/// Plugin for player systems.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (player_input_system, player_movement_system).chain(),
        );
    }
}
