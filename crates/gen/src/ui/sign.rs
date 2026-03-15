//! World-space text signs.
//!
//! Implements Spec 4.1: `gen_add_sign` — Place readable text in 3D world.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Parameters for sign creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignParams {
    /// World position.
    pub position: Vec3,
    /// Text content.
    pub text: String,
    /// Font size in world units.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Text color (hex).
    #[serde(default = "default_color")]
    pub color: String,
    /// Background color (hex, optional).
    #[serde(default)]
    pub background_color: Option<String>,
    /// Always face camera.
    #[serde(default = "default_billboard")]
    pub billboard: bool,
    /// Word wrap width in world units.
    #[serde(default)]
    pub max_width: Option<f32>,
    /// Rotation (only when billboard=false).
    #[serde(default)]
    pub rotation: Vec3,
}

fn default_font_size() -> f32 {
    24.0
}
fn default_color() -> String {
    "#ffffff".to_string()
}
fn default_billboard() -> bool {
    true
}

impl Default for SignParams {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            text: String::new(),
            font_size: default_font_size(),
            color: default_color(),
            background_color: None,
            billboard: default_billboard(),
            max_width: None,
            rotation: Vec3::ZERO,
        }
    }
}

/// Marker component for sign entities.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Sign {
    /// Whether to always face camera.
    pub billboard: bool,
    /// Text for reference.
    pub text: String,
}

/// Marker for sign background panel.
#[derive(Component)]
pub struct SignBackground;

/// Parse hex color to Color.
pub fn parse_sign_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color::srgb(r, g, b))
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
        Some(Color::srgba(r, g, b, a))
    } else {
        None
    }
}

/// System to billboard signs to face camera.
pub fn sign_billboard_system(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Sign>)>,
    mut sign_query: Query<(&Sign, &mut Transform), (With<Sign>, Without<Camera3d>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (sign, mut transform) in sign_query.iter_mut() {
        if sign.billboard {
            // Face the camera
            let to_camera = camera_transform.translation - transform.translation;
            if to_camera != Vec3::ZERO {
                let forward = to_camera.normalize();
                // Billboard: face camera while keeping up vector as close to Y as possible
                let right = Vec3::Y.cross(forward).normalize_or_zero();
                let up = forward.cross(right).normalize_or_zero();
                if right != Vec3::ZERO && up != Vec3::ZERO {
                    // Build rotation matrix from axes and convert to quaternion
                    let rot_mat = Mat3::from_cols(right, up, forward);
                    transform.rotation = Quat::from_mat3(&rot_mat);
                }
            }
        }
    }
}

/// Plugin for sign systems.
pub struct SignPlugin;

impl Plugin for SignPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sign_billboard_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sign_color_rgb() {
        let color = parse_sign_color("#ff0000");
        assert!(color.is_some());
    }

    #[test]
    fn test_parse_sign_color_rgba() {
        let color = parse_sign_color("#ff000080");
        assert!(color.is_some());
    }

    #[test]
    fn test_parse_sign_color_invalid() {
        assert!(parse_sign_color("invalid").is_none());
        assert!(parse_sign_color("#abc").is_none());
    }

    #[test]
    fn test_parse_sign_color_no_hash() {
        let color = parse_sign_color("00ff00");
        assert!(color.is_some());
    }

    #[test]
    fn test_sign_params_default() {
        let params = SignParams::default();
        assert!(params.billboard);
        assert_eq!(params.font_size, 24.0);
        assert_eq!(params.color, "#ffffff");
        assert!(params.background_color.is_none());
        assert!(params.max_width.is_none());
        assert_eq!(params.rotation, Vec3::ZERO);
        assert_eq!(params.position, Vec3::ZERO);
        assert!(params.text.is_empty());
    }

    #[test]
    fn test_sign_component() {
        let sign = Sign {
            billboard: false,
            text: "Hello world".to_string(),
        };
        assert!(!sign.billboard);
        assert_eq!(sign.text, "Hello world");
    }

    #[test]
    fn test_parse_sign_color_values() {
        // Test actual RGB values
        let color = parse_sign_color("#ff8000").unwrap();
        // orange-ish color
        assert_ne!(color, Color::BLACK);

        // RGBA with half alpha
        let color = parse_sign_color("#00ff0080").unwrap();
        assert_ne!(color, Color::BLACK);
    }
}
