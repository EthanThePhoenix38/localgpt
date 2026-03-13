//! Physical joints between entities.
//!
//! Implements Spec 5.3: `gen_add_joint` — Constraints between entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Joint type enumeration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum JointType {
    #[default]
    Fixed,
    Revolute,
    Spherical,
    Prismatic,
    Spring,
}

/// Parameters for joint creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointParams {
    /// First entity ID.
    pub entity_a: String,
    /// Second entity ID.
    pub entity_b: String,
    /// Joint type.
    #[serde(default)]
    pub joint_type: JointType,
    /// Anchor point on entity A (local space).
    #[serde(default)]
    pub anchor_a: Vec3,
    /// Anchor point on entity B (local space).
    #[serde(default)]
    pub anchor_b: Vec3,
    /// Rotation/slide axis.
    #[serde(default = "default_axis")]
    pub axis: Vec3,
    /// Angle limits [min, max] in degrees.
    #[serde(default)]
    pub limits: Option<Vec2>,
    /// Spring stiffness.
    #[serde(default)]
    pub stiffness: Option<f32>,
    /// Spring damping.
    #[serde(default)]
    pub damping: Option<f32>,
}

fn default_axis() -> Vec3 {
    Vec3::Y
}

impl Default for JointParams {
    fn default() -> Self {
        Self {
            entity_a: String::new(),
            entity_b: String::new(),
            joint_type: JointType::default(),
            anchor_a: Vec3::ZERO,
            anchor_b: Vec3::ZERO,
            axis: default_axis(),
            limits: None,
            stiffness: None,
            damping: None,
        }
    }
}

/// Component for joint configuration.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct JointConfig {
    /// Joint type.
    pub joint_type: JointType,
    /// Entity A.
    pub entity_a: Entity,
    /// Entity B.
    pub entity_b: Entity,
    /// Anchor A.
    pub anchor_a: Vec3,
    /// Anchor B.
    pub anchor_b: Vec3,
    /// Axis.
    pub axis: Vec3,
}

/// Plugin for joint systems.
pub struct JointPlugin;

impl Plugin for JointPlugin {
    fn build(&self, _app: &mut App) {
        // Joint creation handled by Avian integration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_params() {
        let params = JointParams {
            entity_a: "a".to_string(),
            entity_b: "b".to_string(),
            joint_type: JointType::Revolute,
            limits: Some(Vec2::new(-45.0, 45.0)),
            ..default()
        };
        assert_eq!(params.joint_type, JointType::Revolute);
    }

    #[test]
    fn test_joint_params_default() {
        let params = JointParams::default();
        assert_eq!(params.joint_type, JointType::Fixed);
        assert_eq!(params.axis, Vec3::Y);
        assert_eq!(params.anchor_a, Vec3::ZERO);
        assert_eq!(params.anchor_b, Vec3::ZERO);
        assert!(params.limits.is_none());
        assert!(params.stiffness.is_none());
        assert!(params.damping.is_none());
    }

    #[test]
    fn test_joint_type_default_is_fixed() {
        assert_eq!(JointType::default(), JointType::Fixed);
    }

    #[test]
    fn test_joint_params_spring() {
        let params = JointParams {
            entity_a: "spring_a".to_string(),
            entity_b: "spring_b".to_string(),
            joint_type: JointType::Spring,
            stiffness: Some(100.0),
            damping: Some(5.0),
            ..default()
        };
        assert_eq!(params.joint_type, JointType::Spring);
        assert_eq!(params.stiffness, Some(100.0));
        assert_eq!(params.damping, Some(5.0));
    }
}
