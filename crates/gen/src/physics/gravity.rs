//! Global and per-entity gravity.
//!
//! Implements Spec 5.5: `gen_set_gravity` — Control gravity direction and strength.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Import player from character module
use crate::character::Player;

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

/// System to apply gravity zones.
pub fn gravity_zone_system(
    player_query: Query<&Transform, With<Player>>,
    zone_query: Query<&GravityZone>,
    _body_query: Query<(Entity, &Transform, Option<&mut GravityOverride>)>,
    _global: Res<GlobalGravity>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for zone in zone_query.iter() {
        let distance = player_transform.translation.distance(zone.center);

        if distance <= zone.radius {
            // Player is in zone - apply zone gravity
            // Would add GravityOverride component to player
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
}
