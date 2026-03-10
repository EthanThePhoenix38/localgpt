//! Collision shapes.
//!
//! Implements Spec 5.2: `gen_add_collider` — Add collision shapes.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Collider shape types.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum ColliderShape {
    #[default]
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Mesh,
}

/// Parameters for collider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColliderParams {
    /// Target entity ID.
    pub entity_id: String,
    /// Shape type.
    #[serde(default)]
    pub shape: ColliderShape,
    /// Dimensions (optional, auto-fit to mesh).
    #[serde(default)]
    pub size: Option<Vec3>,
    /// Offset from entity origin.
    #[serde(default)]
    pub offset: Vec3,
    /// Is this a sensor (trigger only).
    #[serde(default)]
    pub is_trigger: bool,
    /// Show in debug view.
    #[serde(default = "default_visible_in_debug")]
    pub visible_in_debug: bool,
}

fn default_visible_in_debug() -> bool {
    true
}

impl Default for ColliderParams {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            shape: ColliderShape::default(),
            size: None,
            offset: Vec3::ZERO,
            is_trigger: false,
            visible_in_debug: true,
        }
    }
}

/// Component for collider configuration.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ColliderConfig {
    /// Shape type.
    pub shape: ColliderShape,
    /// Is sensor.
    pub is_trigger: bool,
    /// Debug visibility.
    pub visible_in_debug: bool,
}

/// Component for sensor colliders.
#[derive(Component, Default)]
pub struct SensorCollider;

/// Plugin for collider systems.
pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, _app: &mut App) {
        // Collider setup handled by Avian integration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collider_params() {
        let params = ColliderParams {
            entity_id: "test".to_string(),
            shape: ColliderShape::Sphere,
            is_trigger: true,
            ..default()
        };
        assert!(params.is_trigger);
    }
}
