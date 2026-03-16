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

/// System to enforce single default spawn point.
/// When a new default spawn point is added (or changed), unset others.
/// Collects changed default entity first, then iterates all spawn points to unset others.
pub fn enforce_single_default_system(mut query: Query<(Entity, &mut SpawnPoint)>) {
    // Find entity that was most recently set as default (heuristic: first one found)
    let mut default_entity: Option<Entity> = None;
    for (entity, sp) in query.iter() {
        if sp.is_default {
            if default_entity.is_some() {
                // Multiple defaults found — we'll fix this below
                break;
            }
            default_entity = Some(entity);
        }
    }

    // If we have a default entity, ensure no others are also default
    if let Some(keep_entity) = default_entity {
        for (entity, mut sp) in query.iter_mut() {
            if entity != keep_entity && sp.is_default {
                sp.is_default = false;
            }
        }
    }
}

/// System to visualize spawn points in debug mode (gizmos).
pub fn debug_spawn_point_system(query: Query<(&Transform, &SpawnPoint)>, mut gizmos: Gizmos) {
    for (transform, sp) in query.iter() {
        let color = if sp.is_default {
            Color::srgb(0.0, 1.0, 0.0)
        } else {
            Color::srgb(1.0, 1.0, 0.0)
        };

        // Rotation to orient circle in the XZ plane (normal pointing up along Y)
        let flat_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::Y);

        // Cylinder base
        gizmos.circle(
            Isometry3d::new(transform.translation, flat_rotation),
            1.0, // radius
            color,
        );

        // Cylinder top
        gizmos.circle(
            Isometry3d::new(
                transform.translation + Vec3::new(0.0, 2.0, 0.0),
                flat_rotation,
            ),
            1.0, // radius
            color,
        );

        // Vertical lines
        gizmos.line(
            transform.translation + Vec3::new(1.0, 0.0, 0.0),
            transform.translation + Vec3::new(1.0, 2.0, 0.0),
            color,
        );
        gizmos.line(
            transform.translation + Vec3::new(-1.0, 0.0, 0.0),
            transform.translation + Vec3::new(-1.0, 2.0, 0.0),
            color,
        );
        gizmos.line(
            transform.translation + Vec3::new(0.0, 0.0, 1.0),
            transform.translation + Vec3::new(0.0, 2.0, 1.0),
            color,
        );
        gizmos.line(
            transform.translation + Vec3::new(0.0, 0.0, -1.0),
            transform.translation + Vec3::new(0.0, 2.0, -1.0),
            color,
        );

        // Forward arrow
        let forward = transform.forward().as_vec3().xz().normalize_or_zero();
        gizmos.arrow(
            transform.translation + Vec3::new(0.0, 1.0, 0.0),
            transform.translation
                + Vec3::new(0.0, 1.0, 0.0)
                + Vec3::new(forward.x, 0.0, forward.y) * 1.5,
            color,
        );
    }
}

/// System to respawn the player when below the kill plane.
#[allow(clippy::type_complexity)]
pub fn respawn_player_system(
    kill_plane: Res<KillPlane>,
    mut player_query: Query<&mut Transform, With<Player>>,
    spawn_query: Query<(&Transform, &SpawnPoint), (With<SpawnPoint>, Without<Player>)>,
) {
    for mut transform in player_query.iter_mut() {
        if transform.translation.y < kill_plane.y_level {
            // Find default spawn point or use fallback
            let spawn_pos = spawn_query
                .iter()
                .find(|(_, sp)| sp.is_default)
                .map(|(t, _)| t.translation)
                .or_else(|| spawn_query.iter().next().map(|(t, _)| t.translation))
                .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

            // Teleport player
            transform.translation = spawn_pos;
        }
    }
}

// Import from player module
use super::player::Player;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_point_params_default() {
        let params = SpawnPointParams::default();
        assert_eq!(params.position, [0.0, 1.0, 0.0]);
        assert!(params.is_default);
        assert!(params.name.is_none());
    }

    #[test]
    fn test_kill_plane_default() {
        let kp = KillPlane::default();
        assert_eq!(kp.y_level, -50.0);
    }

    #[test]
    fn test_spawn_point_default() {
        let sp = SpawnPoint::default();
        assert!(sp.is_default);
        assert!(sp.name.is_none());
    }

    #[test]
    fn test_spawn_point_named() {
        let sp = SpawnPoint {
            name: Some("checkpoint_1".to_string()),
            is_default: false,
        };
        assert!(!sp.is_default);
        assert_eq!(sp.name.as_deref(), Some("checkpoint_1"));
    }
}

/// Plugin for spawn point systems.
pub struct SpawnPointPlugin;

impl Plugin for SpawnPointPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KillPlane::default()).add_systems(
            Update,
            (
                respawn_player_system,
                enforce_single_default_system,
                debug_spawn_point_system,
            ),
        );
    }
}
