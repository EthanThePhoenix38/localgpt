//! Spawn point management for player respawn.
//!
//! Implements Spec 1.2: `gen_set_spawn_point` — Spawn/Respawn Locations

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for a spawn point location.
#[derive(Component, Clone)]
pub struct SpawnPoint {
    /// Optional name for referencing this spawn point.
    pub name: Option<String>,
    /// Whether this is the default spawn point.
    pub is_default: bool,
}

impl Default for SpawnPoint {
    fn default() -> Self {
        Self {
            name: None,
            is_default: true,
        }
    }
}

/// Parameters for creating a spawn point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPointParams {
    /// Spawn position [x, y, z].
    pub position: [f32; 3],
    /// Spawn rotation in degrees [pitch, yaw, roll].
    #[serde(default)]
    pub rotation: [f32; 3],
    /// Optional name for the spawn point.
    #[serde(default)]
    pub name: Option<String>,
    /// Whether this is the default spawn point (only one allowed).
    #[serde(default = "default_is_default")]
    pub is_default: bool,
}

fn default_is_default() -> bool {
    true
}

impl Default for SpawnPointParams {
    fn default() -> Self {
        Self {
            position: [0.0, 1.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            name: None,
            is_default: true,
        }
    }
}

/// Resource tracking the kill plane Y level.
#[derive(Resource, Clone)]
pub struct KillPlane {
    /// Y level below which the player respawns.
    pub y_level: f32,
}

impl Default for KillPlane {
    fn default() -> Self {
        Self { y_level: -50.0 }
    }
}

/// Spawn a spawn point entity.
pub fn spawn_spawn_point(commands: &mut Commands, params: &SpawnPointParams) -> Entity {
    let position = Vec3::from_array(params.position);
    // Y-axis rotation only
    let rotation = Quat::from_rotation_y(params.rotation[1].to_radians());

    commands
        .spawn((
            Name::new(
                params
                    .name
                    .clone()
                    .unwrap_or_else(|| "SpawnPoint".to_string()),
            ),
            SpawnPoint {
                name: params.name.clone(),
                is_default: params.is_default,
            },
            Transform::from_translation(position).with_rotation(rotation),
            Visibility::default(),
        ))
        .id()
}

/// Find the default spawn point position, or a fallback position.
pub fn get_default_spawn_position(query: Query<&Transform, With<SpawnPoint>>) -> Vec3 {
    query
        .iter()
        .find(|_| true)
        .map(|t| t.translation)
        .unwrap_or(Vec3::new(0.0, 1.0, 0.0))
}

/// Find a spawn point by name.
pub fn find_spawn_point_by_name(
    query: Query<(&SpawnPoint, &Transform)>,
    name: &str,
) -> Option<Vec3> {
    query.iter().find_map(|(sp, t)| {
        if sp.name.as_deref() == Some(name) {
            Some(t.translation)
        } else {
            None
        }
    })
}

/// System to respawn the player when below the kill plane.
pub fn respawn_player_system(
    kill_plane: Res<KillPlane>,
    mut player_query: Query<(&mut Transform, &mut PlayerVelocity), With<Player>>,
    spawn_query: Query<&Transform, (With<SpawnPoint>, Without<Player>)>,
) {
    for (mut transform, mut velocity) in player_query.iter_mut() {
        if transform.translation.y < kill_plane.y_level {
            // Find default spawn point or use fallback
            let spawn_pos = spawn_query
                .iter()
                .next()
                .map(|t| t.translation)
                .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

            // Teleport player
            transform.translation = spawn_pos;
            velocity.linear = Vec3::ZERO;
            velocity.is_grounded = true;
        }
    }
}

// Import from player module
use super::player::{Player, PlayerVelocity};

/// Plugin for spawn point systems.
pub struct SpawnPointPlugin;

impl Plugin for SpawnPointPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KillPlane::default())
            .add_systems(Update, respawn_player_system);
    }
}
