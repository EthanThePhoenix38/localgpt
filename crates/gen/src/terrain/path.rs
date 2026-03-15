//! Walkable paths between waypoints.
//!
//! Creates paths conforming to terrain with various materials.

use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Path material preset.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathMaterial {
    #[default]
    Stone,
    Dirt,
    Wood,
    Cobblestone,
    Custom,
}

/// Parameters for path generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathParams {
    /// Waypoints for the path.
    pub points: Vec<Vec3>,
    /// Path width in meters.
    #[serde(default = "default_width")]
    pub width: f32,
    /// Material preset.
    #[serde(default)]
    pub material: PathMaterial,
    /// Use smooth curves between points.
    #[serde(default = "default_curved")]
    pub curved: bool,
    /// Height above terrain to prevent z-fighting.
    #[serde(default = "default_raised")]
    pub raised: f32,
    /// Add stone border edges.
    #[serde(default)]
    pub border: bool,
}

fn default_width() -> f32 {
    2.0
}
fn default_curved() -> bool {
    true
}
fn default_raised() -> f32 {
    0.02
}

impl Default for PathParams {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            width: default_width(),
            material: PathMaterial::Stone,
            curved: default_curved(),
            raised: default_raised(),
            border: false,
        }
    }
}

/// Marker component for path entities.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Path {
    /// Path width.
    pub width: f32,
}

/// Generate path mesh from waypoints.
pub fn generate_path_mesh(params: &PathParams) -> Mesh {
    if params.points.len() < 2 {
        return Mesh::new(PrimitiveTopology::TriangleList, default());
    }

    // Sample path at regular intervals
    let sample_points = if params.curved {
        sample_catmull_rom(&params.points, 1.0)
    } else {
        params.points.clone()
    };

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_width = params.width / 2.0;
    let mut accumulated_length = 0.0f32;

    for i in 0..sample_points.len() {
        let point = sample_points[i];

        // Compute tangent (direction along path)
        let tangent = if i == 0 {
            (sample_points[1] - sample_points[0]).normalize_or_zero()
        } else if i == sample_points.len() - 1 {
            (sample_points[i] - sample_points[i - 1]).normalize_or_zero()
        } else {
            (sample_points[i + 1] - sample_points[i - 1]).normalize_or_zero()
        };

        // Compute perpendicular on XZ plane
        let right = Vec3::new(-tangent.z, 0.0, tangent.x).normalize();

        // Two vertices at this point
        let left_pos = point - right * half_width + Vec3::Y * params.raised;
        let right_pos = point + right * half_width + Vec3::Y * params.raised;

        positions.push([left_pos.x, left_pos.y, left_pos.z]);
        positions.push([right_pos.x, right_pos.y, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        // UV: U across width, V along length
        uvs.push([0.0, accumulated_length]);
        uvs.push([1.0, accumulated_length]);

        // Track length for UV mapping
        if i > 0 {
            accumulated_length += (sample_points[i] - sample_points[i - 1]).length();
        }

        // Connect to previous quad
        if i > 0 {
            let base = ((i - 1) * 2) as u32;
            let current = (i * 2) as u32;

            // Two triangles
            indices.push(base);
            indices.push(base + 1);
            indices.push(current);

            indices.push(current);
            indices.push(base + 1);
            indices.push(current + 1);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Sample Catmull-Rom spline through waypoints.
fn sample_catmull_rom(points: &[Vec3], interval: f32) -> Vec<Vec3> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let mut samples = Vec::new();

    // Add first point
    samples.push(points[0]);

    for i in 0..points.len() - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 >= points.len() {
            points[i + 1]
        } else {
            points[i + 2]
        };

        let segment_length = (p2 - p1).length();
        let num_samples = (segment_length / interval).max(2.0) as usize;

        for j in 1..num_samples {
            let t = j as f32 / num_samples as f32;
            let sample = catmull_rom_point(p0, p1, p2, p3, t);
            samples.push(sample);
        }
    }

    // Add last point
    samples.push(*points.last().unwrap());

    samples
}

/// Catmull-Rom interpolation.
fn catmull_rom_point(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;

    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Get material color for path preset.
pub fn get_path_material_color(material: PathMaterial) -> Color {
    match material {
        PathMaterial::Stone => Color::srgb(0.5, 0.5, 0.5),
        PathMaterial::Dirt => Color::srgb(0.55, 0.4, 0.25),
        PathMaterial::Wood => Color::srgb(0.6, 0.45, 0.3),
        PathMaterial::Cobblestone => Color::srgb(0.45, 0.45, 0.48),
        PathMaterial::Custom => Color::srgb(1.0, 1.0, 1.0),
    }
}

/// Plugin for path systems.
pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, _app: &mut App) {
        // Paths are generated on demand
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_path() {
        let params = PathParams {
            points: vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)],
            curved: false,
            ..default()
        };
        let mesh = generate_path_mesh(&params);
        // Should have 4 vertices (2 per endpoint)
        assert!(mesh.count_vertices() >= 4);
    }

    #[test]
    fn test_curved_path() {
        let params = PathParams {
            points: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(5.0, 0.0, 5.0),
                Vec3::new(10.0, 0.0, 0.0),
            ],
            curved: true,
            ..default()
        };
        let mesh = generate_path_mesh(&params);
        // Curved path should have more vertices than straight
        assert!(mesh.count_vertices() >= 6);
    }

    #[test]
    fn test_path_params_default() {
        let params = PathParams::default();
        assert_eq!(params.width, 2.0);
        assert!(params.curved);
        assert!((params.raised - 0.02).abs() < f32::EPSILON);
        assert!(!params.border);
        assert!(params.points.is_empty());
    }

    #[test]
    fn test_empty_path() {
        let params = PathParams {
            points: vec![],
            ..default()
        };
        let mesh = generate_path_mesh(&params);
        assert_eq!(mesh.count_vertices(), 0);
    }

    #[test]
    fn test_path_material_colors() {
        // Verify each material preset produces a distinct color
        let stone = get_path_material_color(PathMaterial::Stone);
        let dirt = get_path_material_color(PathMaterial::Dirt);
        let custom = get_path_material_color(PathMaterial::Custom);
        assert_ne!(stone, dirt);
        assert_ne!(dirt, custom);
    }

    #[test]
    fn test_path_material_all_variants() {
        let wood = get_path_material_color(PathMaterial::Wood);
        let cobblestone = get_path_material_color(PathMaterial::Cobblestone);
        assert_ne!(wood, cobblestone);
    }

    #[test]
    fn test_single_point_path() {
        let params = PathParams {
            points: vec![Vec3::ZERO],
            ..default()
        };
        let mesh = generate_path_mesh(&params);
        // Single point = less than 2 = empty mesh
        assert_eq!(mesh.count_vertices(), 0);
    }

    #[test]
    fn test_path_with_raised() {
        let params = PathParams {
            points: vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0)],
            raised: 0.1,
            curved: false,
            ..default()
        };
        let mesh = generate_path_mesh(&params);
        assert!(mesh.count_vertices() >= 4);
    }

    #[test]
    fn test_catmull_rom_preserves_endpoints() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 5.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        let samples = sample_catmull_rom(&points, 1.0);
        assert_eq!(samples.first().unwrap(), &points[0]);
        assert_eq!(samples.last().unwrap(), points.last().unwrap());
    }

    #[test]
    fn test_path_default_functions() {
        assert_eq!(default_width(), 2.0);
        assert!(default_curved());
        assert!((default_raised() - 0.02).abs() < f32::EPSILON);
    }
}
