//! Physics body configuration.
//!
//! Implements Spec 5.1: `gen_set_physics` — Enable physics on entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Body type for physics simulation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    #[default]
    Dynamic,
    Static,
    Kinematic,
}

/// Parameters for physics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsParams {
    /// Target entity ID.
    pub entity_id: String,
    /// Body type.
    #[serde(default)]
    pub body_type: BodyType,
    /// Mass in kg (optional, auto-calculated).
    #[serde(default)]
    pub mass: Option<f32>,
    /// Bounciness (0-1).
    #[serde(default = "default_restitution")]
    pub restitution: f32,
    /// Surface friction (0-1).
    #[serde(default = "default_friction")]
    pub friction: f32,
    /// Gravity multiplier.
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    /// Linear air resistance.
    #[serde(default = "default_linear_damping")]
    pub linear_damping: f32,
    /// Angular air resistance.
    #[serde(default = "default_angular_damping")]
    pub angular_damping: f32,
    /// Prevent rotation.
    #[serde(default)]
    pub lock_rotation: bool,
}

fn default_restitution() -> f32 {
    0.3
}
fn default_friction() -> f32 {
    0.5
}
fn default_gravity_scale() -> f32 {
    1.0
}
fn default_linear_damping() -> f32 {
    0.1
}
fn default_angular_damping() -> f32 {
    0.1
}

impl Default for PhysicsParams {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            body_type: BodyType::default(),
            mass: None,
            restitution: default_restitution(),
            friction: default_friction(),
            gravity_scale: default_gravity_scale(),
            linear_damping: default_linear_damping(),
            angular_damping: default_angular_damping(),
            lock_rotation: false,
        }
    }
}

/// Component marking entities with physics.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PhysicsBody {
    /// Body type.
    pub body_type: BodyType,
    /// Mass (computed or specified).
    pub mass: f32,
    /// Restitution.
    pub restitution: f32,
    /// Friction.
    pub friction: f32,
    /// Gravity scale.
    pub gravity_scale: f32,
}

/// Marker for rotation-locked bodies.
#[derive(Component, Default)]
pub struct RotationLocked;

/// Plugin for physics body systems.
pub struct PhysicsBodyPlugin;

impl Plugin for PhysicsBodyPlugin {
    fn build(&self, _app: &mut App) {
        // Physics body setup is handled by Avian integration
        // This module provides component definitions
    }
}
