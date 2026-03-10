//! Force fields and impulses.
//!
//! Implements Spec 5.4: `gen_add_force` — Force fields and impulses.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Force type enumeration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum ForceType {
    #[default]
    Directional,
    PointAttract,
    PointRepel,
    Vortex,
    Impulse,
}

/// Falloff type enumeration.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum FalloffType {
    #[default]
    None,
    Linear,
    Quadratic,
}

/// Parameters for force field creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceParams {
    /// World position.
    pub position: Vec3,
    /// Force type.
    #[serde(default)]
    pub force_type: ForceType,
    /// Force strength.
    #[serde(default = "default_strength")]
    pub strength: f32,
    /// Area of effect radius.
    #[serde(default = "default_radius")]
    pub radius: f32,
    /// Direction (directional only).
    #[serde(default)]
    pub direction: Option<Vec3>,
    /// Falloff type.
    #[serde(default)]
    pub falloff: FalloffType,
    /// Affects player.
    #[serde(default = "default_affects_player")]
    pub affects_player: bool,
    /// Continuous force.
    #[serde(default = "default_continuous")]
    pub continuous: bool,
}

fn default_strength() -> f32 {
    10.0
}
fn default_radius() -> f32 {
    5.0
}
fn default_affects_player() -> bool {
    true
}
fn default_continuous() -> bool {
    true
}

impl Default for ForceParams {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            force_type: ForceType::default(),
            strength: default_strength(),
            radius: default_radius(),
            direction: None,
            falloff: FalloffType::default(),
            affects_player: default_affects_player(),
            continuous: default_continuous(),
        }
    }
}

/// Component for force field.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ForceField {
    /// Force type.
    pub force_type: ForceType,
    /// Strength.
    pub strength: f32,
    /// Radius.
    pub radius: f32,
    /// Direction (for directional).
    pub direction: Vec3,
    /// Falloff type.
    pub falloff: FalloffType,
    /// Affects player.
    pub affects_player: bool,
    /// Continuous force.
    pub continuous: bool,
}

impl Default for ForceField {
    fn default() -> Self {
        Self {
            force_type: ForceType::default(),
            strength: default_strength(),
            radius: default_radius(),
            direction: Vec3::Z,
            falloff: FalloffType::default(),
            affects_player: true,
            continuous: true,
        }
    }
}

/// Marker for player entity.
use crate::character::Player;

/// System to apply force fields.
pub fn force_field_system(
    _time: Res<Time>,
    force_fields: Query<(Entity, &Transform, &ForceField)>,
    mut body_query: Query<(Entity, &Transform), Without<ForceField>>,
    player_query: Query<Entity, With<Player>>,
) {
    for (_field_entity, field_transform, field) in force_fields.iter() {
        if !field.continuous {
            continue;
        }

        for (body_entity, body_transform) in body_query.iter_mut() {
            // Skip player if not affecting player
            if !field.affects_player && player_query.contains(body_entity) {
                continue;
            }

            let distance = field_transform
                .translation
                .distance(body_transform.translation);

            if distance > field.radius || distance < 0.001 {
                continue;
            }

            // Calculate force magnitude with falloff
            let force_magnitude = match field.falloff {
                FalloffType::None => field.strength,
                FalloffType::Linear => field.strength * (1.0 - distance / field.radius),
                FalloffType::Quadratic => field.strength * (1.0 - distance / field.radius).powi(2),
            };

            // Calculate force direction
            let force_direction = match field.force_type {
                ForceType::Directional => field.direction,
                ForceType::PointAttract => {
                    (field_transform.translation - body_transform.translation).normalize_or_zero()
                }
                ForceType::PointRepel => {
                    (body_transform.translation - field_transform.translation).normalize_or_zero()
                }
                ForceType::Vortex => {
                    // Tangent to radius
                    let to_field = field_transform.translation - body_transform.translation;
                    Vec3::new(-to_field.z, 0.0, to_field.x).normalize_or_zero()
                }
                ForceType::Impulse => {
                    // One-time impulse in deterministic direction based on entity id
                    // Use entity's bits as seed for deterministic pseudo-random direction
                    let idx = body_entity.to_bits();
                    let seed = (idx.wrapping_mul(2654435761) as f32) / 4294967295.0;
                    Vec3::new((seed - 0.5) * 2.0, 0.5, ((seed * 1.618) % 1.0 - 0.5) * 2.0)
                        .normalize_or_zero()
                }
            };

            // Apply force (would use Avian's ExternalForce in real implementation)
            let _force = force_direction * force_magnitude;
            // In real implementation: external_force.force = force;
        }
    }
}

/// Plugin for force field systems.
pub struct ForceFieldPlugin;

impl Plugin for ForceFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, force_field_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_params() {
        let params = ForceParams {
            force_type: ForceType::Vortex,
            strength: 20.0,
            radius: 8.0,
            ..default()
        };
        assert_eq!(params.force_type, ForceType::Vortex);
    }
}
