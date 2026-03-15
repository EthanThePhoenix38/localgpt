//! Global and per-entity gravity.
//!
//! Implements Spec 5.5: `gen_set_gravity` — Control gravity direction and strength.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Gravity presets.
pub const GRAVITY_EARTH: f32 = 9.81;
pub const GRAVITY_MOON: f32 = 1.62;
pub const GRAVITY_MARS: f32 = 3.72;
pub const GRAVITY_JUPITER: f32 = 24.79;
pub const GRAVITY_ZERO: f32 = 0.0;

/// Parameters for gravity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityParams {
    /// Target entity ID (optional, global if omitted).
    #[serde(default)]
    pub entity_id: Option<String>,
    /// Gravity direction (normalized).
    #[serde(default = "default_direction")]
    pub direction: Vec3,
    /// Gravity strength (m/s²).
    #[serde(default = "default_strength")]
    pub strength: f32,
    /// Create gravity zone at position.
    #[serde(default)]
    pub zone_position: Option<Vec3>,
    /// Zone radius.
    #[serde(default)]
    pub zone_radius: Option<f32>,
    /// Transition duration.
    #[serde(default = "default_transition")]
    pub transition_duration: f32,
}

fn default_direction() -> Vec3 {
    Vec3::new(0.0, -1.0, 0.0)
}
fn default_strength() -> f32 {
    9.81
}
fn default_transition() -> f32 {
    0.5
}

impl Default for GravityParams {
    fn default() -> Self {
        Self {
            entity_id: None,
            direction: default_direction(),
            strength: default_strength(),
            zone_position: None,
            zone_radius: None,
            transition_duration: default_transition(),
        }
    }
}

/// Resource for global gravity.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GlobalGravity {
    /// Current gravity vector.
    pub gravity: Vec3,
    /// Target gravity (for transition).
    pub target_gravity: Vec3,
    /// Transition progress (0-1).
    pub transition_progress: f32,
    /// Transition duration.
    pub transition_duration: f32,
}

impl Default for GlobalGravity {
    fn default() -> Self {
        Self {
            gravity: default_direction() * default_strength(),
            target_gravity: default_direction() * default_strength(),
            transition_progress: 1.0,
            transition_duration: 0.0,
        }
    }
}

/// Component for per-entity gravity override.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GravityOverride {
    /// Custom gravity direction.
    pub direction: Vec3,
    /// Custom strength multiplier.
    pub strength_scale: f32,
}

/// Component for gravity zone.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GravityZone {
    /// Zone center.
    pub center: Vec3,
    /// Zone radius.
    pub radius: f32,
    /// Gravity in zone.
    pub gravity: Vec3,
    /// Transition duration.
    pub transition_duration: f32,
}

/// System to update global gravity transitions.
pub fn gravity_transition_system(time: Res<Time>, mut global: ResMut<GlobalGravity>) {
    if global.transition_progress < 1.0 {
        let t = (time.delta_secs() / global.transition_duration).min(1.0);
        global.gravity = global.gravity.lerp(global.target_gravity, t);
        global.transition_progress = t;
    }
}

/// System to apply gravity zones to entities within their radius.
///
/// Adds/updates `GravityOverride` components on entities inside zones
/// and removes them when entities leave.
pub fn gravity_zone_system(
    mut commands: Commands,
    zone_query: Query<&GravityZone>,
    mut body_query: Query<(Entity, &Transform, Option<&GravityOverride>)>,
    global: Res<GlobalGravity>,
) {
    for (entity, transform, existing_override) in body_query.iter_mut() {
        let mut in_zone = false;
        let mut zone_gravity = Vec3::ZERO;

        for zone in zone_query.iter() {
            let distance = transform.translation.distance(zone.center);
            if distance <= zone.radius {
                in_zone = true;
                zone_gravity = zone.gravity;
                break;
            }
        }

        if in_zone {
            // Entity is in a gravity zone — apply override
            let global_mag = global.gravity.length();
            let strength_scale = if global_mag > 0.001 {
                zone_gravity.length() / global_mag
            } else {
                1.0
            };
            let direction = zone_gravity.normalize_or_zero();

            commands.entity(entity).insert(GravityOverride {
                direction,
                strength_scale,
            });
        } else if existing_override.is_some() {
            // Entity left all zones — remove override
            commands.entity(entity).remove::<GravityOverride>();
        }
    }
}

/// Plugin for gravity systems.
pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalGravity>()
            .add_systems(Update, gravity_transition_system)
            .add_systems(Update, gravity_zone_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_params() {
        let params = GravityParams {
            strength: GRAVITY_MOON,
            ..default()
        };
        assert!((params.strength - GRAVITY_MOON).abs() < 0.01);
    }

    #[test]
    fn test_global_gravity_default() {
        let gravity = GlobalGravity::default();
        assert!((gravity.gravity.y - (-GRAVITY_EARTH)).abs() < 0.01);
    }

    #[test]
    fn test_gravity_params_default() {
        let params = GravityParams::default();
        assert!(params.entity_id.is_none());
        assert_eq!(params.direction, Vec3::new(0.0, -1.0, 0.0));
        assert!((params.strength - 9.81).abs() < 0.01);
        assert!(params.zone_position.is_none());
        assert!(params.zone_radius.is_none());
        assert!((params.transition_duration - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gravity_presets() {
        assert!((GRAVITY_EARTH - 9.81).abs() < 0.01);
        assert!((GRAVITY_MOON - 1.62).abs() < 0.01);
        assert!((GRAVITY_MARS - 3.72).abs() < 0.01);
        assert!((GRAVITY_JUPITER - 24.79).abs() < 0.01);
        assert!((GRAVITY_ZERO).abs() < 0.01);
    }

    #[test]
    fn test_global_gravity_transition_starts_complete() {
        let gravity = GlobalGravity::default();
        // Transition progress starts at 1.0 (complete)
        assert!((gravity.transition_progress - 1.0).abs() < f32::EPSILON);
        assert!((gravity.transition_duration).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gravity_override_component() {
        let override_comp = GravityOverride {
            direction: Vec3::new(0.0, 1.0, 0.0),
            strength_scale: 0.5,
        };
        assert_eq!(override_comp.direction, Vec3::Y);
        assert_eq!(override_comp.strength_scale, 0.5);
    }

    #[test]
    fn test_gravity_zone_component() {
        let zone = GravityZone {
            center: Vec3::new(10.0, 0.0, 5.0),
            radius: 15.0,
            gravity: Vec3::new(0.0, -GRAVITY_MOON, 0.0),
            transition_duration: 1.0,
        };
        assert_eq!(zone.radius, 15.0);
        assert!((zone.gravity.y - (-GRAVITY_MOON)).abs() < 0.01);
    }

    #[test]
    fn test_gravity_params_with_zone() {
        let params = GravityParams {
            zone_position: Some(Vec3::new(5.0, 0.0, 5.0)),
            zone_radius: Some(10.0),
            strength: GRAVITY_MARS,
            ..default()
        };
        assert_eq!(params.zone_position, Some(Vec3::new(5.0, 0.0, 5.0)));
        assert_eq!(params.zone_radius, Some(10.0));
        assert!((params.strength - GRAVITY_MARS).abs() < 0.01);
    }
}
