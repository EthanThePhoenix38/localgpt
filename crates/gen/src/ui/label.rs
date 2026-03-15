//! Entity nameplates / floating labels.
//!
//! Implements Spec 4.3: `gen_add_label` — Attach floating name labels to entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Parameters for entity label creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelParams {
    /// Target entity ID.
    pub entity_id: String,
    /// Label text.
    pub text: String,
    /// Text color (hex).
    #[serde(default = "default_color")]
    pub color: String,
    /// Background color (hex with alpha).
    #[serde(default = "default_bg_color")]
    pub background_color: String,
    /// Height above entity top.
    #[serde(default = "default_offset_y")]
    pub offset_y: f32,
    /// Font size.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Maximum visible distance.
    #[serde(default = "default_visible_distance")]
    pub visible_distance: f32,
}

fn default_color() -> String {
    "#ffffff".to_string()
}
fn default_bg_color() -> String {
    "#00000088".to_string()
}
fn default_offset_y() -> f32 {
    0.5
}
fn default_font_size() -> f32 {
    16.0
}
fn default_visible_distance() -> f32 {
    15.0
}

impl Default for LabelParams {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            text: String::new(),
            color: default_color(),
            background_color: default_bg_color(),
            offset_y: default_offset_y(),
            font_size: default_font_size(),
            visible_distance: default_visible_distance(),
        }
    }
}

/// Component for floating labels.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EntityLabel {
    /// Label text.
    pub text: String,
    /// Height offset.
    pub offset_y: f32,
    /// Maximum visible distance.
    pub visible_distance: f32,
    /// Current alpha (for fading).
    pub current_alpha: f32,
    /// Parent entity.
    pub parent_entity: Entity,
}

/// Marker for label background.
#[derive(Component)]
pub struct LabelBackground;

/// System to update label positions and fade based on distance.
pub fn label_follow_system(
    camera_query: Query<&Transform, (With<Camera3d>, Without<EntityLabel>)>,
    parent_query: Query<&Transform, Without<EntityLabel>>,
    mut label_query: Query<(&EntityLabel, &mut Transform, &mut Visibility), Without<Camera3d>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (label, mut transform, mut visibility) in label_query.iter_mut() {
        // Get parent position
        let Ok(parent_transform) = parent_query.get(label.parent_entity) else {
            continue;
        };

        // Position label above parent
        let target_pos = parent_transform.translation + Vec3::Y * label.offset_y;
        transform.translation = target_pos;

        // Billboard to face camera
        let to_camera = camera_transform.translation - transform.translation;
        if to_camera != Vec3::ZERO {
            let forward = to_camera.normalize();
            let right = Vec3::Y.cross(forward).normalize_or_zero();
            let up = forward.cross(right).normalize_or_zero();
            if right != Vec3::ZERO && up != Vec3::ZERO {
                // Build rotation matrix from axes and convert to quaternion
                let rot_mat = Mat3::from_cols(right, up, forward);
                transform.rotation = Quat::from_mat3(&rot_mat);
            }
        }

        // Distance-based visibility
        let distance = to_camera.length();
        let _fade_start = label.visible_distance * 0.8;
        let fade_end = label.visible_distance;

        if distance > fade_end {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
            // Fade alpha would be applied to text material here
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_params_default() {
        let params = LabelParams::default();
        assert_eq!(params.color, "#ffffff");
        assert_eq!(params.background_color, "#00000088");
        assert_eq!(params.offset_y, 0.5);
        assert_eq!(params.font_size, 16.0);
        assert_eq!(params.visible_distance, 15.0);
    }

    #[test]
    fn test_label_params_custom() {
        let params = LabelParams {
            entity_id: "npc_1".to_string(),
            text: "Guard".to_string(),
            color: "#ff0000".to_string(),
            offset_y: 1.0,
            visible_distance: 25.0,
            ..Default::default()
        };
        assert_eq!(params.text, "Guard");
        assert_eq!(params.offset_y, 1.0);
        assert_eq!(params.visible_distance, 25.0);
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_color(), "#ffffff");
        assert_eq!(default_bg_color(), "#00000088");
        assert_eq!(default_offset_y(), 0.5);
        assert_eq!(default_font_size(), 16.0);
        assert_eq!(default_visible_distance(), 15.0);
    }
}

/// Plugin for label systems.
pub struct LabelPlugin;

impl Plugin for LabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, label_follow_system);
    }
}
