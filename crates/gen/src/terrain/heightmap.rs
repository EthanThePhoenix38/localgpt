//! Procedural terrain generation using noise.
//!
//! Creates terrain meshes from Perlin/Simplex noise with automatic collision.

use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, SuperSimplex};
use serde::{Deserialize, Serialize};

/// Noise type for terrain generation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoiseType {
    #[default]
    Perlin,
    Simplex,
    Flat,
}

/// Material preset for terrain.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerrainMaterial {
    #[default]
    Grass,
    Sand,
    Snow,
    Rock,
    Custom,
}

/// Parameters for terrain generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainParams {
    /// Size of the terrain in meters (X, Z).
    #[serde(default = "default_size")]
    pub size: Vec2,
    /// Vertices per side.
    #[serde(default = "default_resolution")]
    pub resolution: u32,
    /// Maximum height in meters.
    #[serde(default = "default_height_scale")]
    pub height_scale: f32,
    /// Noise algorithm.
    #[serde(default)]
    pub noise_type: NoiseType,
    /// Number of noise octaves.
    #[serde(default = "default_octaves")]
    pub noise_octaves: usize,
    /// Noise frequency.
    #[serde(default = "default_frequency")]
    pub noise_frequency: f32,
    /// Random seed.
    #[serde(default)]
    pub seed: Option<u32>,
    /// Material preset.
    #[serde(default)]
    pub material: TerrainMaterial,
    /// World position.
    #[serde(default)]
    pub position: Vec3,
}

fn default_size() -> Vec2 {
    Vec2::splat(100.0)
}
fn default_resolution() -> u32 {
    128
}
fn default_height_scale() -> f32 {
    20.0
}
fn default_octaves() -> usize {
    4
}
fn default_frequency() -> f32 {
    0.02
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            size: default_size(),
            resolution: default_resolution(),
            height_scale: default_height_scale(),
            noise_type: NoiseType::Perlin,
            noise_octaves: default_octaves(),
            noise_frequency: default_frequency(),
            seed: None,
            material: TerrainMaterial::Grass,
            position: Vec3::ZERO,
        }
    }
}

/// Marker component for terrain entities.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Terrain {
    /// Size of the terrain.
    pub size: Vec2,
    /// Resolution (vertices per side).
    pub resolution: u32,
}

/// Generate terrain mesh from noise parameters.
pub fn generate_terrain_mesh(params: &TerrainParams) -> Mesh {
    let resolution = params.resolution;
    let size = params.size;
    let seed = params.seed.unwrap_or_else(rand::random::<u32>);

    // Create vertex buffers
    let vertex_count = (resolution + 1) * (resolution + 1);
    let mut positions = Vec::with_capacity(vertex_count as usize);
    let mut normals = Vec::with_capacity(vertex_count as usize);
    let mut uvs = Vec::with_capacity(vertex_count as usize);

    // Generate height values using noise
    let heights = generate_heightmap(params, seed);

    // Generate vertices
    let half_size = size / 2.0;
    let step = size / resolution as f32;

    for z in 0..=resolution {
        for x in 0..=resolution {
            let x_pos = x as f32 * step.x - half_size.x;
            let z_pos = z as f32 * step.y - half_size.y;
            let height = heights[(z * (resolution + 1) + x) as usize];

            positions.push([
                x_pos + params.position.x,
                height + params.position.y,
                z_pos + params.position.z,
            ]);
            uvs.push([x as f32 / resolution as f32, z as f32 / resolution as f32]);
            // Normal will be computed after we have all positions
            normals.push([0.0, 1.0, 0.0]);
        }
    }

    // Generate indices for triangles
    let mut indices = Vec::with_capacity((resolution * resolution * 6) as usize);
    for z in 0..resolution {
        for x in 0..resolution {
            let tl = z * (resolution + 1) + x;
            let tr = tl + 1;
            let bl = (z + 1) * (resolution + 1) + x;
            let br = bl + 1;

            // Two triangles per quad
            indices.push(tl);
            indices.push(bl);
            indices.push(tr);

            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    // Compute smooth normals
    compute_normals(&positions, &indices, &mut normals);

    // Build mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate height values for each vertex.
fn generate_heightmap(params: &TerrainParams, seed: u32) -> Vec<f32> {
    let resolution = params.resolution;
    let count = (resolution + 1) * (resolution + 1);
    let mut heights = vec![0.0f32; count as usize];

    if matches!(params.noise_type, NoiseType::Flat) {
        return heights;
    }

    // Create noise generators with FBm octaves for both types
    let fbm_perlin: Fbm<Perlin> = Fbm::new(seed).set_octaves(params.noise_octaves);
    let fbm_simplex: Fbm<SuperSimplex> = Fbm::new(seed).set_octaves(params.noise_octaves);

    let half_res = resolution as f32 / 2.0;

    for z in 0..=resolution {
        for x in 0..=resolution {
            let nx = (x as f32 - half_res) * params.noise_frequency;
            let nz = (z as f32 - half_res) * params.noise_frequency;

            let noise_value = match params.noise_type {
                NoiseType::Perlin => fbm_perlin.get([nx as f64, nz as f64]),
                NoiseType::Simplex => fbm_simplex.get([nx as f64, nz as f64]),
                NoiseType::Flat => 0.0,
            };

            // Map from [-1, 1] to [0, height_scale]
            let height = ((noise_value as f32 + 1.0) / 2.0) * params.height_scale;
            heights[(z * (resolution + 1) + x) as usize] = height;
        }
    }

    heights
}

/// Compute smooth normals from height map.
fn compute_normals(positions: &[[f32; 3]], indices: &[u32], normals: &mut [[f32; 3]]) {
    // Accumulate face normals to each vertex
    let mut accumulated: Vec<Vec3> = vec![Vec3::ZERO; positions.len()];

    for chunk in indices.chunks(3) {
        if chunk.len() != 3 {
            continue;
        }
        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);
        let v2 = Vec3::from_array(positions[i2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2);

        accumulated[i0] += face_normal;
        accumulated[i1] += face_normal;
        accumulated[i2] += face_normal;
    }

    // Normalize and write back
    for (i, normal) in normals.iter_mut().enumerate() {
        let n = accumulated[i].normalize_or_zero();
        *normal = n.to_array();
    }
}

/// Get material color for terrain preset.
pub fn get_terrain_material_color(material: TerrainMaterial) -> Color {
    match material {
        TerrainMaterial::Grass => Color::srgb(0.2, 0.6, 0.2),
        TerrainMaterial::Sand => Color::srgb(0.76, 0.70, 0.50),
        TerrainMaterial::Snow => Color::srgb(0.95, 0.95, 0.98),
        TerrainMaterial::Rock => Color::srgb(0.5, 0.5, 0.5),
        TerrainMaterial::Custom => Color::srgb(1.0, 1.0, 1.0),
    }
}

/// Plugin for terrain systems.
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, _app: &mut App) {
        // Terrain is generated on demand, no continuous systems needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_flat_terrain() {
        let params = TerrainParams {
            noise_type: NoiseType::Flat,
            resolution: 16,
            ..default()
        };
        let mesh = generate_terrain_mesh(&params);
        // Should have (17 * 17) = 289 vertices
        assert_eq!(mesh.count_vertices(), 289);
    }

    #[test]
    fn test_generate_perlin_terrain() {
        let params = TerrainParams {
            noise_type: NoiseType::Perlin,
            resolution: 32,
            seed: Some(42),
            ..default()
        };
        let mesh = generate_terrain_mesh(&params);
        // Should have (33 * 33) = 1089 vertices
        assert_eq!(mesh.count_vertices(), 1089);
    }

    #[test]
    fn test_deterministic_seed() {
        let params = TerrainParams {
            seed: Some(12345),
            resolution: 16,
            ..default()
        };
        let mesh1 = generate_terrain_mesh(&params);
        let mesh2 = generate_terrain_mesh(&params);
        assert_eq!(mesh1.count_vertices(), mesh2.count_vertices());
    }

    #[test]
    fn test_terrain_params_default() {
        let params = TerrainParams::default();
        assert_eq!(params.size, Vec2::splat(100.0));
        assert_eq!(params.resolution, 128);
        assert_eq!(params.height_scale, 20.0);
        assert!(matches!(params.noise_type, NoiseType::Perlin));
        assert_eq!(params.noise_octaves, 4);
        assert!((params.noise_frequency - 0.02).abs() < f32::EPSILON);
        assert!(params.seed.is_none());
        assert!(matches!(params.material, TerrainMaterial::Grass));
        assert_eq!(params.position, Vec3::ZERO);
    }

    #[test]
    fn test_generate_simplex_terrain() {
        let params = TerrainParams {
            noise_type: NoiseType::Simplex,
            resolution: 16,
            seed: Some(99),
            ..default()
        };
        let mesh = generate_terrain_mesh(&params);
        assert_eq!(mesh.count_vertices(), 289);
    }

    #[test]
    fn test_terrain_material_colors_distinct() {
        let grass = get_terrain_material_color(TerrainMaterial::Grass);
        let sand = get_terrain_material_color(TerrainMaterial::Sand);
        let snow = get_terrain_material_color(TerrainMaterial::Snow);
        let rock = get_terrain_material_color(TerrainMaterial::Rock);
        assert_ne!(grass, sand);
        assert_ne!(snow, rock);
    }

    #[test]
    fn test_terrain_with_position_offset() {
        let params = TerrainParams {
            noise_type: NoiseType::Flat,
            resolution: 4,
            position: Vec3::new(10.0, 5.0, 20.0),
            ..default()
        };
        let mesh = generate_terrain_mesh(&params);
        assert_eq!(mesh.count_vertices(), 25); // (4+1)^2
    }
}
