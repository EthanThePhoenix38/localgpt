//! Physical joints between entities.
//!
//! Implements Spec 5.3: `gen_add_joint` — Constraints between entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "physics")]
use avian3d::prelude::*;

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
    /// Angular limits in degrees [min, max].
    pub limits: Option<Vec2>,
    /// Spring stiffness (Spring joint only).
    pub stiffness: Option<f32>,
    /// Spring damping (Spring joint only).
    pub damping: Option<f32>,
}

/// System to convert JointConfig into Avian joint constraints.
///
/// Spawns the appropriate joint type between entity_a and entity_b.
/// Spring joints use a DistanceJoint with compliance derived from stiffness.
#[cfg(feature = "physics")]
pub fn joint_setup_system(
    mut commands: Commands,
    query: Query<(Entity, &JointConfig), Added<JointConfig>>,
) {
    for (joint_entity, config) in query.iter() {
        match config.joint_type {
            JointType::Fixed => {
                commands.entity(joint_entity).insert(
                    FixedJoint::new(config.entity_a, config.entity_b)
                        .with_local_anchor1(config.anchor_a)
                        .with_local_anchor2(config.anchor_b),
                );
            }
            JointType::Revolute => {
                let mut joint = RevoluteJoint::new(config.entity_a, config.entity_b)
                    .with_hinge_axis(config.axis)
                    .with_local_anchor1(config.anchor_a)
                    .with_local_anchor2(config.anchor_b);
                if let Some(limits) = config.limits {
                    joint = joint.with_angle_limits(limits.x.to_radians(), limits.y.to_radians());
                }
                commands.entity(joint_entity).insert(joint);
            }
            JointType::Spherical => {
                commands.entity(joint_entity).insert(
                    SphericalJoint::new(config.entity_a, config.entity_b)
                        .with_local_anchor1(config.anchor_a)
                        .with_local_anchor2(config.anchor_b),
                );
            }
            JointType::Prismatic => {
                let mut joint = PrismaticJoint::new(config.entity_a, config.entity_b)
                    .with_slider_axis(config.axis)
                    .with_local_anchor1(config.anchor_a)
                    .with_local_anchor2(config.anchor_b);
                if let Some(limits) = config.limits {
                    joint = joint.with_limits(limits.x, limits.y);
                }
                commands.entity(joint_entity).insert(joint);
            }
            JointType::Spring => {
                let mut joint = DistanceJoint::new(config.entity_a, config.entity_b)
                    .with_local_anchor1(config.anchor_a)
                    .with_local_anchor2(config.anchor_b);
                if let Some(stiffness) = config.stiffness {
                    // Compliance = 1/stiffness
                    let compliance = if stiffness > 0.0 {
                        1.0 / stiffness
                    } else {
                        0.0
                    };
                    joint = joint.with_compliance(compliance);
                }
                commands.entity(joint_entity).insert(joint);
            }
        }
    }
}

/// Plugin for joint systems.
pub struct JointPlugin;

impl Plugin for JointPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "physics")]
        app.add_systems(Update, joint_setup_system);

        let _ = app;
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

    #[test]
    fn test_joint_type_all_variants() {
        assert_ne!(JointType::Fixed, JointType::Revolute);
        assert_ne!(JointType::Spherical, JointType::Prismatic);
        assert_ne!(JointType::Prismatic, JointType::Spring);
    }

    #[test]
    fn test_joint_params_with_limits() {
        let params = JointParams {
            entity_a: "hinge_a".to_string(),
            entity_b: "hinge_b".to_string(),
            joint_type: JointType::Revolute,
            axis: Vec3::X,
            limits: Some(Vec2::new(-90.0, 90.0)),
            ..default()
        };
        assert_eq!(params.axis, Vec3::X);
        let limits = params.limits.unwrap();
        assert_eq!(limits.x, -90.0);
        assert_eq!(limits.y, 90.0);
    }
}
