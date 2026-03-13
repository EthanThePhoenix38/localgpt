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
///
/// Without the `physics` feature (avian3d), forces are applied directly as
/// translation offsets scaled by delta time. With avian3d, this would use
/// `ExternalForce` / `ExternalImpulse` components instead.
pub fn force_field_system(
    time: Res<Time>,
    force_fields: Query<(Entity, &Transform, &ForceField)>,
    mut body_query: Query<(Entity, &mut Transform), Without<ForceField>>,
    player_query: Query<Entity, With<Player>>,
) {
    let dt = time.delta_secs();

    for (_field_entity, field_transform, field) in force_fields.iter() {
        if !field.continuous {
            continue;
        }

        for (body_entity, mut body_transform) in body_query.iter_mut() {
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
                    // One-time impulse direction (handled outside continuous check)
                    continue;
                }
            };

            // Apply force as translation offset (simple integration without physics engine)
            let force = force_direction * force_magnitude;
            body_transform.translation += force * dt;
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

    #[test]
    fn test_force_params_default() {
        let params = ForceParams::default();
        assert_eq!(params.force_type, ForceType::Directional);
        assert!((params.strength - 10.0).abs() < f32::EPSILON);
        assert!((params.radius - 5.0).abs() < f32::EPSILON);
        assert!(params.direction.is_none());
        assert!(params.affects_player);
        assert!(params.continuous);
    }

    #[test]
    fn test_force_field_default() {
        let field = ForceField::default();
        assert_eq!(field.force_type, ForceType::Directional);
        assert_eq!(field.direction, Vec3::Z);
        assert!(field.affects_player);
        assert!(field.continuous);
    }

    #[test]
    fn test_falloff_type_default_is_none() {
        assert!(matches!(FalloffType::default(), FalloffType::None));
    }
}
