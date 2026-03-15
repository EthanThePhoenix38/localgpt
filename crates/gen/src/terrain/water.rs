//! Water plane with animated waves.
//!
//! Creates transparent water surfaces at specified heights.

use bevy::mesh::{Mesh, PrimitiveTopology};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Parameters for water generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterParams {
    /// Height of the water plane.
    #[serde(default)]
    pub height: f32,
    /// Size of the water plane (X, Z).
    #[serde(default = "default_size")]
    pub size: Vec2,
    /// Water color (hex string).
    #[serde(default = "default_color")]
    pub color: String,
    /// Transparency (0.0 = invisible, 1.0 = opaque).
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    /// Wave animation speed multiplier.
    #[serde(default = "default_wave_speed")]
    pub wave_speed: f32,
    /// Wave height amplitude.
    #[serde(default = "default_wave_height")]
    pub wave_height: f32,
    /// World position (XZ center, Y from height param).
    #[serde(default)]
    pub position: Option<Vec3>,
}

fn default_size() -> Vec2 {
    Vec2::splat(100.0)
}
fn default_color() -> String {
    "#2389da".to_string()
}
fn default_opacity() -> f32 {
    0.7
}
fn default_wave_speed() -> f32 {
    1.0
}
fn default_wave_height() -> f32 {
    0.3
}

impl Default for WaterParams {
    fn default() -> Self {
        Self {
            height: 0.0,
            size: default_size(),
            color: default_color(),
            opacity: default_opacity(),
            wave_speed: default_wave_speed(),
            wave_height: default_wave_height(),
            position: None,
        }
    }
}

/// Marker component for water entities.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Water {
    /// Wave animation speed.
    pub wave_speed: f32,
    /// Wave height amplitude.
    pub wave_height: f32,
    /// Base Y position.
    pub base_height: f32,
}

/// Generate water plane mesh.
pub fn generate_water_mesh(params: &WaterParams) -> Mesh {
    let segments = 32u32;
    let vertex_count = (segments + 1) * (segments + 1);

    let mut positions = Vec::with_capacity(vertex_count as usize);
    let mut normals = Vec::with_capacity(vertex_count as usize);
    let mut uvs = Vec::with_capacity(vertex_count as usize);

    let half_size = params.size / 2.0;
    let step = params.size / segments as f32;
    let center = params.position.unwrap_or(Vec3::ZERO);

    for z in 0..=segments {
        for x in 0..=segments {
            let x_pos = x as f32 * step.x - half_size.x + center.x;
            let z_pos = z as f32 * step.y - half_size.y + center.z;

            positions.push([x_pos, params.height + center.y, z_pos]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([x as f32 / segments as f32, z as f32 / segments as f32]);
        }
    }

    let mut indices = Vec::with_capacity((segments * segments * 6) as usize);
    for z in 0..segments {
        for x in 0..segments {
            let tl = z * (segments + 1) + x;
            let tr = tl + 1;
            let bl = (z + 1) * (segments + 1) + x;
            let br = bl + 1;

            indices.push(tl);
            indices.push(bl);
            indices.push(tr);

            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));

    mesh
}

/// Parse hex color string to Color.
pub fn parse_water_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color::srgb(r, g, b))
    } else {
        None
    }
}

/// System to animate water waves.
pub fn water_wave_system(time: Res<Time>, mut water_query: Query<(&Water, &mut Transform)>) {
    let t = time.elapsed_secs();

    for (water, mut transform) in water_query.iter_mut() {
        // Simple wave animation using sine waves
        // Wave A: direction (1, 0), frequency 0.5, amplitude 0.6
        // Wave B: direction (0.7, 0.7), frequency 0.8, amplitude 0.4
        let wave_a = (t * water.wave_speed * 0.5).sin() * water.wave_height * 0.6;
        let wave_b = (t * water.wave_speed * 0.8).sin() * water.wave_height * 0.4;

        let wave_offset = wave_a + wave_b;
        transform.translation.y = water.base_height + wave_offset;
    }
}

/// Plugin for water systems.
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, water_wave_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_water_mesh() {
        let params = WaterParams::default();
        let mesh = generate_water_mesh(&params);
        // (33 * 33) = 1089 vertices
        assert_eq!(mesh.count_vertices(), 1089);
    }

    #[test]
    fn test_parse_water_color() {
        let color = parse_water_color("#2389da");
        assert!(color.is_some());

        let color = parse_water_color("invalid");
        assert!(color.is_none());
    }

    #[test]
    fn test_water_params_default() {
        let params = WaterParams::default();
        assert_eq!(params.height, 0.0);
        assert_eq!(params.size, Vec2::splat(100.0));
        assert_eq!(params.color, "#2389da");
        assert!((params.opacity - 0.7).abs() < f32::EPSILON);
        assert!((params.wave_speed - 1.0).abs() < f32::EPSILON);
        assert!((params.wave_height - 0.3).abs() < f32::EPSILON);
        assert!(params.position.is_none());
    }

    #[test]
    fn test_water_component() {
        let water = Water {
            wave_speed: 2.0,
            wave_height: 0.5,
            base_height: 3.0,
        };
        assert_eq!(water.wave_speed, 2.0);
        assert_eq!(water.base_height, 3.0);
    }
}
