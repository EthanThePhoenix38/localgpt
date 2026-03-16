//! GLTF Mesh Segmentation — decompose imported meshes into editable sub-objects.
//!
//! When importing external 3D assets, connected-component analysis splits the mesh
//! into individually editable parts. Each part becomes a child entity with its own
//! Transform, Mesh, and Material.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::tier::SemanticRole;

/// Result of mesh segmentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationResult {
    pub components: Vec<MeshComponent>,
    pub total_vertices: usize,
    pub total_triangles: usize,
}

/// A single connected component from mesh segmentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshComponent {
    /// Name of this component (from GLTF node or auto-generated).
    pub name: String,
    /// Indices of the original mesh's triangles that belong to this component.
    pub triangle_indices: Vec<u32>,
    /// Vertex positions (world-space).
    pub vertices: Vec<[f32; 3]>,
    /// Triangle indices into `vertices`.
    pub indices: Vec<u32>,
    /// Normals (if available).
    pub normals: Option<Vec<[f32; 3]>>,
    /// UV coordinates (if available).
    pub uvs: Option<Vec<[f32; 2]>>,
    /// Inferred semantic role for this component.
    pub inferred_role: SemanticRole,
    /// Bounding box min.
    pub aabb_min: [f32; 3],
    /// Bounding box max.
    pub aabb_max: [f32; 3],
}

/// Segment a mesh into connected components using union-find.
///
/// `vertices` — vertex positions (3 floats per vertex).
/// `indices` — triangle indices (3 per triangle).
/// `parent_name` — name of the parent entity for naming components.
/// `node_names` — optional GLTF node names for sub-meshes.
pub fn segment_mesh(
    vertices: &[[f32; 3]],
    indices: &[u32],
    parent_name: &str,
    node_names: &[String],
) -> SegmentationResult {
    let vertex_count = vertices.len();
    let triangle_count = indices.len() / 3;

    if vertex_count == 0 || triangle_count == 0 {
        return SegmentationResult {
            components: Vec::new(),
            total_vertices: 0,
            total_triangles: 0,
        };
    }

    // Union-Find data structure
    let mut parent: Vec<usize> = (0..vertex_count).collect();
    let mut rank: Vec<u32> = vec![0; vertex_count];

    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]]; // path compression
            x = parent[x];
        }
        x
    };

    let union = |parent: &mut Vec<usize>, rank: &mut Vec<u32>, x: usize, y: usize| {
        let rx = find(parent, x);
        let ry = find(parent, y);
        if rx == ry {
            return;
        }
        if rank[rx] < rank[ry] {
            parent[rx] = ry;
        } else if rank[rx] > rank[ry] {
            parent[ry] = rx;
        } else {
            parent[ry] = rx;
            rank[rx] += 1;
        }
    };

    // Build connectivity from triangle edges
    for tri in 0..triangle_count {
        let i0 = indices[tri * 3] as usize;
        let i1 = indices[tri * 3 + 1] as usize;
        let i2 = indices[tri * 3 + 2] as usize;

        if i0 < vertex_count && i1 < vertex_count && i2 < vertex_count {
            union(&mut parent, &mut rank, i0, i1);
            union(&mut parent, &mut rank, i1, i2);
            union(&mut parent, &mut rank, i0, i2);
        }
    }

    // Group vertices by component root
    let mut component_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for v in 0..vertex_count {
        let root = find(&mut parent, v);
        component_map.entry(root).or_default().push(v);
    }

    // Sort components by size (largest first) and limit to 50
    let mut component_roots: Vec<_> = component_map.keys().copied().collect();
    component_roots.sort_by(|a, b| component_map[b].len().cmp(&component_map[a].len()));
    component_roots.truncate(50);

    // Build component meshes
    let mut components = Vec::new();
    for (comp_idx, root) in component_roots.iter().enumerate() {
        let vert_set: std::collections::HashSet<usize> =
            component_map[root].iter().copied().collect();

        // Remap vertices
        let mut old_to_new: HashMap<usize, u32> = HashMap::new();
        let mut new_vertices = Vec::new();

        for &v in &component_map[root] {
            let new_idx = new_vertices.len() as u32;
            old_to_new.insert(v, new_idx);
            new_vertices.push(vertices[v]);
        }

        // Remap triangles
        let mut new_indices = Vec::new();
        let mut tri_indices = Vec::new();
        for tri in 0..triangle_count {
            let i0 = indices[tri * 3] as usize;
            let i1 = indices[tri * 3 + 1] as usize;
            let i2 = indices[tri * 3 + 2] as usize;

            if vert_set.contains(&i0) && vert_set.contains(&i1) && vert_set.contains(&i2) {
                new_indices.push(old_to_new[&i0]);
                new_indices.push(old_to_new[&i1]);
                new_indices.push(old_to_new[&i2]);
                tri_indices.push(tri as u32);
            }
        }

        if new_vertices.is_empty() {
            continue;
        }

        // Compute AABB
        let mut aabb_min = [f32::MAX; 3];
        let mut aabb_max = [f32::MIN; 3];
        for v in &new_vertices {
            for i in 0..3 {
                aabb_min[i] = aabb_min[i].min(v[i]);
                aabb_max[i] = aabb_max[i].max(v[i]);
            }
        }

        // Name component
        let name = if comp_idx < node_names.len() {
            node_names[comp_idx].clone()
        } else {
            format!("{}_part_{}", parent_name, comp_idx)
        };

        // Infer semantic role from geometry
        let role = infer_role_from_geometry(&new_vertices, &aabb_min, &aabb_max);

        components.push(MeshComponent {
            name,
            triangle_indices: tri_indices,
            vertices: new_vertices,
            indices: new_indices,
            normals: None,
            uvs: None,
            inferred_role: role,
            aabb_min,
            aabb_max,
        });
    }

    SegmentationResult {
        components,
        total_vertices: vertex_count,
        total_triangles: triangle_count,
    }
}

/// Infer semantic role from component geometry.
fn infer_role_from_geometry(
    vertices: &[[f32; 3]],
    aabb_min: &[f32; 3],
    aabb_max: &[f32; 3],
) -> SemanticRole {
    let height = aabb_max[1] - aabb_min[1];
    let width = aabb_max[0] - aabb_min[0];
    let depth = aabb_max[2] - aabb_min[2];
    let horizontal_extent = width.max(depth);
    let volume = height * width * depth;

    // Flat and wide → ground (check before small piece check)
    if height < 0.5 && horizontal_extent > 5.0 {
        return SemanticRole::Ground;
    }

    // Small isolated pieces are likely props
    if vertices.len() < 20 && volume < 1.0 {
        return SemanticRole::Decoration;
    }

    // Tall and narrow → vegetation (tree trunk) or structure
    if height > 3.0 && horizontal_extent < 2.0 {
        return SemanticRole::Vegetation;
    }

    // Large volume → structure
    if volume > 50.0 {
        return SemanticRole::Structure;
    }

    // Medium → prop
    if volume > 1.0 {
        return SemanticRole::Prop;
    }

    SemanticRole::Decoration
}

/// Infer semantic role from a component name (GLTF node name).
pub fn infer_role_from_name(name: &str) -> Option<SemanticRole> {
    let lower = name.to_lowercase();

    // Ground indicators
    if lower.contains("ground")
        || lower.contains("floor")
        || lower.contains("terrain")
        || lower.contains("path")
    {
        return Some(SemanticRole::Ground);
    }

    // Structure indicators
    if lower.contains("wall")
        || lower.contains("roof")
        || lower.contains("building")
        || lower.contains("house")
        || lower.contains("bridge")
        || lower.contains("fence")
        || lower.contains("stair")
        || lower.contains("door")
        || lower.contains("window")
    {
        return Some(SemanticRole::Structure);
    }

    // Vegetation indicators
    if lower.contains("tree")
        || lower.contains("bush")
        || lower.contains("grass")
        || lower.contains("leaf")
        || lower.contains("branch")
        || lower.contains("trunk")
        || lower.contains("flower")
        || lower.contains("plant")
    {
        return Some(SemanticRole::Vegetation);
    }

    // Character indicators
    if lower.contains("npc")
        || lower.contains("character")
        || lower.contains("person")
        || lower.contains("player")
    {
        return Some(SemanticRole::Character);
    }

    // Light indicators
    if lower.contains("light")
        || lower.contains("lamp")
        || lower.contains("torch")
        || lower.contains("candle")
    {
        return Some(SemanticRole::Lighting);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cube_vertices() -> Vec<[f32; 3]> {
        vec![
            // Front face
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            // Back face
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
        ]
    }

    fn make_cube_indices() -> Vec<u32> {
        vec![
            0, 1, 2, 0, 2, 3, // front
            4, 6, 5, 4, 7, 6, // back
            0, 3, 7, 0, 7, 4, // left
            1, 5, 6, 1, 6, 2, // right
            3, 2, 6, 3, 6, 7, // top
            0, 4, 5, 0, 5, 1, // bottom
        ]
    }

    #[test]
    fn test_single_mesh_one_component() {
        let vertices = make_cube_vertices();
        let indices = make_cube_indices();
        let result = segment_mesh(&vertices, &indices, "cube", &[]);
        assert_eq!(result.components.len(), 1);
        assert_eq!(result.total_vertices, 8);
        assert_eq!(result.total_triangles, 12);
    }

    #[test]
    fn test_two_disconnected_cubes() {
        let mut vertices = make_cube_vertices();
        // Add a second cube offset by 10 units
        let offset_verts: Vec<[f32; 3]> = make_cube_vertices()
            .iter()
            .map(|v| [v[0] + 10.0, v[1], v[2]])
            .collect();
        vertices.extend_from_slice(&offset_verts);

        let mut indices = make_cube_indices();
        // Offset indices for second cube
        let offset_indices: Vec<u32> = make_cube_indices().iter().map(|i| i + 8).collect();
        indices.extend_from_slice(&offset_indices);

        let result = segment_mesh(&vertices, &indices, "two_cubes", &[]);
        assert_eq!(result.components.len(), 2);
    }

    #[test]
    fn test_empty_mesh() {
        let result = segment_mesh(&[], &[], "empty", &[]);
        assert_eq!(result.components.len(), 0);
    }

    #[test]
    fn test_node_names_used() {
        let vertices = make_cube_vertices();
        let indices = make_cube_indices();
        let names = vec!["my_wall".to_string()];
        let result = segment_mesh(&vertices, &indices, "house", &names);
        assert_eq!(result.components[0].name, "my_wall");
    }

    #[test]
    fn test_fallback_naming() {
        let vertices = make_cube_vertices();
        let indices = make_cube_indices();
        let result = segment_mesh(&vertices, &indices, "house", &[]);
        assert_eq!(result.components[0].name, "house_part_0");
    }

    #[test]
    fn test_role_inference_from_name() {
        assert_eq!(
            infer_role_from_name("wall_left"),
            Some(SemanticRole::Structure)
        );
        assert_eq!(
            infer_role_from_name("tree_oak"),
            Some(SemanticRole::Vegetation)
        );
        assert_eq!(
            infer_role_from_name("ground_plane"),
            Some(SemanticRole::Ground)
        );
        assert_eq!(infer_role_from_name("random_thing"), None);
    }

    #[test]
    fn test_role_inference_from_geometry_flat() {
        // Flat ground plane
        let vertices = vec![
            [-10.0, 0.0, -10.0],
            [10.0, 0.0, -10.0],
            [10.0, 0.0, 10.0],
            [-10.0, 0.0, 10.0],
        ];
        let role = infer_role_from_geometry(&vertices, &[-10.0, 0.0, -10.0], &[10.0, 0.0, 10.0]);
        assert_eq!(role, SemanticRole::Ground);
    }

    #[test]
    fn test_aabb_computed() {
        let vertices = make_cube_vertices();
        let indices = make_cube_indices();
        let result = segment_mesh(&vertices, &indices, "cube", &[]);
        let comp = &result.components[0];
        assert_eq!(comp.aabb_min, [-1.0, -1.0, -1.0]);
        assert_eq!(comp.aabb_max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_max_50_components() {
        // Create 60 disconnected triangles
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for i in 0..60 {
            let offset = i as f32 * 100.0;
            let base = (i * 3) as u32;
            vertices.push([offset, 0.0, 0.0]);
            vertices.push([offset + 1.0, 0.0, 0.0]);
            vertices.push([offset + 0.5, 1.0, 0.0]);
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
        }
        let result = segment_mesh(&vertices, &indices, "many", &[]);
        assert!(result.components.len() <= 50);
    }
}
