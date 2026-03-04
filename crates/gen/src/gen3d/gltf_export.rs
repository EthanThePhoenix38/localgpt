//! Shared glTF/GLB export logic used by both `gen_export_gltf` and `gen_save_world`.

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use std::path::Path;

use super::registry::*;

/// Build glTF JSON root and binary buffer data from scene entities.
/// Returns (root, bin_data, entity_to_node map).
#[allow(clippy::too_many_arguments)]
fn build_gltf_data(
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    parent_query: &Query<&ChildOf>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    mesh_handles: &Query<&Mesh3d>,
    mesh_assets: &Assets<Mesh>,
) -> (
    gltf_json::Root,
    Vec<u8>,
    std::collections::HashMap<Entity, u32>,
) {
    use gltf_json::validation::Checked::Valid;
    use gltf_json::validation::USize64;

    let mut root = gltf_json::Root::default();
    let mut bin_data: Vec<u8> = Vec::new();
    let mut entity_to_node: std::collections::HashMap<Entity, u32> =
        std::collections::HashMap::new();

    for (name, entity) in registry.all_names() {
        let Ok(gen_ent) = gen_entities.get(entity) else {
            continue;
        };
        match gen_ent.entity_type {
            GenEntityType::Primitive | GenEntityType::Mesh => {}
            _ => continue,
        }

        let Ok(mesh3d) = mesh_handles.get(entity) else {
            continue;
        };
        let Some(mesh) = mesh_assets.get(&mesh3d.0) else {
            continue;
        };

        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v.clone(),
            _ => continue,
        };
        if positions.is_empty() {
            continue;
        }

        let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(v)) => Some(v.clone()),
            _ => None,
        };

        let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(v)) => Some(v.clone()),
            _ => None,
        };

        let indices: Option<Vec<u32>> = mesh.indices().map(|idx| match idx {
            Indices::U16(v) => v.iter().map(|i| *i as u32).collect(),
            Indices::U32(v) => v.clone(),
        });

        // Bounding box
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for p in &positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }

        // --- Positions ---
        let pos_offset = bin_data.len();
        for p in &positions {
            for &v in p {
                bin_data.extend_from_slice(&v.to_le_bytes());
            }
        }
        let pos_length = bin_data.len() - pos_offset;
        while !bin_data.len().is_multiple_of(4) {
            bin_data.push(0);
        }

        let pos_view_idx = root.buffer_views.len() as u32;
        root.buffer_views.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_offset: Some(USize64(pos_offset as u64)),
            byte_length: USize64(pos_length as u64),
            byte_stride: None,
            target: Some(Valid(gltf_json::buffer::Target::ArrayBuffer)),
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let pos_accessor_idx = root.accessors.len() as u32;
        root.accessors.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(pos_view_idx)),
            byte_offset: None,
            count: USize64(positions.len() as u64),
            component_type: Valid(gltf_json::accessor::GenericComponentType(
                gltf_json::accessor::ComponentType::F32,
            )),
            type_: Valid(gltf_json::accessor::Type::Vec3),
            min: Some(gltf_json::Value::from(vec![
                serde_json::Number::from_f64(min[0] as f64).unwrap_or(serde_json::Number::from(0)),
                serde_json::Number::from_f64(min[1] as f64).unwrap_or(serde_json::Number::from(0)),
                serde_json::Number::from_f64(min[2] as f64).unwrap_or(serde_json::Number::from(0)),
            ])),
            max: Some(gltf_json::Value::from(vec![
                serde_json::Number::from_f64(max[0] as f64).unwrap_or(serde_json::Number::from(0)),
                serde_json::Number::from_f64(max[1] as f64).unwrap_or(serde_json::Number::from(0)),
                serde_json::Number::from_f64(max[2] as f64).unwrap_or(serde_json::Number::from(0)),
            ])),
            name: None,
            normalized: false,
            sparse: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // --- Normals ---
        let normal_accessor_idx = if let Some(ref normals) = normals {
            let offset = bin_data.len();
            for n in normals {
                for &v in n {
                    bin_data.extend_from_slice(&v.to_le_bytes());
                }
            }
            let length = bin_data.len() - offset;
            while !bin_data.len().is_multiple_of(4) {
                bin_data.push(0);
            }

            let view_idx = root.buffer_views.len() as u32;
            root.buffer_views.push(gltf_json::buffer::View {
                buffer: gltf_json::Index::new(0),
                byte_offset: Some(USize64(offset as u64)),
                byte_length: USize64(length as u64),
                byte_stride: None,
                target: Some(Valid(gltf_json::buffer::Target::ArrayBuffer)),
                name: None,
                extensions: Default::default(),
                extras: Default::default(),
            });

            let acc_idx = root.accessors.len() as u32;
            root.accessors.push(gltf_json::Accessor {
                buffer_view: Some(gltf_json::Index::new(view_idx)),
                byte_offset: None,
                count: USize64(normals.len() as u64),
                component_type: Valid(gltf_json::accessor::GenericComponentType(
                    gltf_json::accessor::ComponentType::F32,
                )),
                type_: Valid(gltf_json::accessor::Type::Vec3),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
            Some(acc_idx)
        } else {
            None
        };

        // --- UVs ---
        let uv_accessor_idx = if let Some(ref uvs) = uvs {
            let offset = bin_data.len();
            for uv in uvs {
                for &v in uv {
                    bin_data.extend_from_slice(&v.to_le_bytes());
                }
            }
            let length = bin_data.len() - offset;
            while !bin_data.len().is_multiple_of(4) {
                bin_data.push(0);
            }

            let view_idx = root.buffer_views.len() as u32;
            root.buffer_views.push(gltf_json::buffer::View {
                buffer: gltf_json::Index::new(0),
                byte_offset: Some(USize64(offset as u64)),
                byte_length: USize64(length as u64),
                byte_stride: None,
                target: Some(Valid(gltf_json::buffer::Target::ArrayBuffer)),
                name: None,
                extensions: Default::default(),
                extras: Default::default(),
            });

            let acc_idx = root.accessors.len() as u32;
            root.accessors.push(gltf_json::Accessor {
                buffer_view: Some(gltf_json::Index::new(view_idx)),
                byte_offset: None,
                count: USize64(uvs.len() as u64),
                component_type: Valid(gltf_json::accessor::GenericComponentType(
                    gltf_json::accessor::ComponentType::F32,
                )),
                type_: Valid(gltf_json::accessor::Type::Vec2),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
            Some(acc_idx)
        } else {
            None
        };

        // --- Indices ---
        let index_accessor_idx = if let Some(ref indices) = indices {
            let offset = bin_data.len();
            for &idx in indices {
                bin_data.extend_from_slice(&idx.to_le_bytes());
            }
            let length = bin_data.len() - offset;
            while !bin_data.len().is_multiple_of(4) {
                bin_data.push(0);
            }

            let view_idx = root.buffer_views.len() as u32;
            root.buffer_views.push(gltf_json::buffer::View {
                buffer: gltf_json::Index::new(0),
                byte_offset: Some(USize64(offset as u64)),
                byte_length: USize64(length as u64),
                byte_stride: None,
                target: Some(Valid(gltf_json::buffer::Target::ElementArrayBuffer)),
                name: None,
                extensions: Default::default(),
                extras: Default::default(),
            });

            let acc_idx = root.accessors.len() as u32;
            root.accessors.push(gltf_json::Accessor {
                buffer_view: Some(gltf_json::Index::new(view_idx)),
                byte_offset: None,
                count: USize64(indices.len() as u64),
                component_type: Valid(gltf_json::accessor::GenericComponentType(
                    gltf_json::accessor::ComponentType::U32,
                )),
                type_: Valid(gltf_json::accessor::Type::Scalar),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
            Some(acc_idx)
        } else {
            None
        };

        // --- Material ---
        let material_idx = {
            let (base_color, metallic, roughness) = material_handles
                .get(entity)
                .ok()
                .and_then(|h| material_assets.get(&h.0))
                .map(|mat| {
                    let c = mat.base_color.to_srgba();
                    (
                        [c.red, c.green, c.blue, c.alpha],
                        mat.metallic,
                        mat.perceptual_roughness,
                    )
                })
                .unwrap_or(([0.8, 0.8, 0.8, 1.0], 0.0, 0.5));

            let mat_idx = root.materials.len() as u32;
            root.materials.push(gltf_json::Material {
                name: Some(format!("{}_material", name)),
                pbr_metallic_roughness: gltf_json::material::PbrMetallicRoughness {
                    base_color_factor: gltf_json::material::PbrBaseColorFactor(base_color),
                    metallic_factor: gltf_json::material::StrengthFactor(metallic),
                    roughness_factor: gltf_json::material::StrengthFactor(roughness),
                    base_color_texture: None,
                    metallic_roughness_texture: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                alpha_cutoff: None,
                alpha_mode: Valid(gltf_json::material::AlphaMode::Opaque),
                double_sided: false,
                normal_texture: None,
                occlusion_texture: None,
                emissive_texture: None,
                emissive_factor: gltf_json::material::EmissiveFactor([0.0, 0.0, 0.0]),
                extensions: Default::default(),
                extras: Default::default(),
            });
            mat_idx
        };

        // --- Mesh primitive ---
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(
            Valid(gltf_json::mesh::Semantic::Positions),
            gltf_json::Index::new(pos_accessor_idx),
        );
        if let Some(idx) = normal_accessor_idx {
            attributes.insert(
                Valid(gltf_json::mesh::Semantic::Normals),
                gltf_json::Index::new(idx),
            );
        }
        if let Some(idx) = uv_accessor_idx {
            attributes.insert(
                Valid(gltf_json::mesh::Semantic::TexCoords(0)),
                gltf_json::Index::new(idx),
            );
        }

        let mesh_idx = root.meshes.len() as u32;
        root.meshes.push(gltf_json::Mesh {
            name: Some(format!("{}_mesh", name)),
            primitives: vec![gltf_json::mesh::Primitive {
                attributes,
                indices: index_accessor_idx.map(gltf_json::Index::new),
                material: Some(gltf_json::Index::new(material_idx)),
                mode: Valid(gltf_json::mesh::Mode::Triangles),
                targets: None,
                extensions: Default::default(),
                extras: Default::default(),
            }],
            weights: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // --- Node ---
        let transform = transforms.get(entity).copied().unwrap_or_default();
        let (axis, angle) = transform.rotation.to_axis_angle();
        let quat = if angle.abs() < f32::EPSILON {
            gltf_json::scene::UnitQuaternion([0.0, 0.0, 0.0, 1.0])
        } else {
            let q = Quat::from_axis_angle(axis, angle);
            gltf_json::scene::UnitQuaternion([q.x, q.y, q.z, q.w])
        };

        let node_idx = root.nodes.len() as u32;
        root.nodes.push(gltf_json::Node {
            name: Some(name.to_string()),
            mesh: Some(gltf_json::Index::new(mesh_idx)),
            translation: Some(transform.translation.to_array()),
            rotation: Some(quat),
            scale: Some(transform.scale.to_array()),
            camera: None,
            children: None,
            skin: None,
            matrix: None,
            weights: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        entity_to_node.insert(entity, node_idx);
    }

    // Parent-child hierarchy
    let mut root_nodes = Vec::new();
    for (_name, entity) in registry.all_names() {
        let Some(&node_idx) = entity_to_node.get(&entity) else {
            continue;
        };

        let parent_entity = parent_query.get(entity).ok().map(|p| p.parent());
        let parent_is_gen = parent_entity
            .and_then(|pe| entity_to_node.get(&pe))
            .copied();

        if let Some(parent_node_idx) = parent_is_gen {
            let parent_node = &mut root.nodes[parent_node_idx as usize];
            let children = parent_node.children.get_or_insert_with(Vec::new);
            children.push(gltf_json::Index::new(node_idx));
        } else {
            root_nodes.push(gltf_json::Index::new(node_idx));
        }
    }

    root.scenes.push(gltf_json::Scene {
        name: Some("Scene".to_string()),
        nodes: root_nodes,
        extensions: Default::default(),
        extras: Default::default(),
    });
    root.scene = Some(gltf_json::Index::new(0));

    (root, bin_data, entity_to_node)
}

/// Export all mesh entities from the scene to a GLB file at the given path.
/// Returns Ok(()) on success, Err(message) on failure.
#[allow(clippy::too_many_arguments)]
pub fn export_glb(
    output_path: &Path,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    parent_query: &Query<&ChildOf>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    mesh_handles: &Query<&Mesh3d>,
    mesh_assets: &Assets<Mesh>,
) -> Result<(), String> {
    use gltf_json::validation::USize64;

    let (mut root, mut bin_data, _) = build_gltf_data(
        registry,
        transforms,
        gen_entities,
        parent_query,
        material_handles,
        material_assets,
        mesh_handles,
        mesh_assets,
    );

    if !bin_data.is_empty() {
        root.buffers.push(gltf_json::Buffer {
            byte_length: USize64(bin_data.len() as u64),
            uri: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
    }

    // Serialize to GLB
    let json_string = serde_json::to_string(&root).map_err(|e| format!("JSON error: {}", e))?;
    let mut json_bytes = json_string.into_bytes();
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0);
    }

    let total_length = 12 + 8 + json_bytes.len() + 8 + bin_data.len();
    let mut glb = Vec::with_capacity(total_length);

    // Header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(&json_bytes);

    // BIN chunk
    glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
    glb.extend_from_slice(&bin_data);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    std::fs::write(output_path, &glb).map_err(|e| format!("Failed to write GLB: {}", e))
}

/// Export to separate glTF JSON + BIN files for human readability.
/// Creates `export/scene.gltf` and `export/scene.bin` under the output_dir.
#[allow(clippy::too_many_arguments)]
pub fn export_gltf(
    output_dir: &Path,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    parent_query: &Query<&ChildOf>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    mesh_handles: &Query<&Mesh3d>,
    mesh_assets: &Assets<Mesh>,
) -> Result<(), String> {
    use gltf_json::validation::USize64;

    let (mut root, bin_data, _) = build_gltf_data(
        registry,
        transforms,
        gen_entities,
        parent_query,
        material_handles,
        material_assets,
        mesh_handles,
        mesh_assets,
    );

    // Set buffer URI to reference external .bin file
    if !bin_data.is_empty() {
        root.buffers.push(gltf_json::Buffer {
            byte_length: USize64(bin_data.len() as u64),
            uri: Some("scene.bin".to_string()),
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
    }

    // Create export directory
    let export_dir = output_dir.join("export");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("Failed to create export directory: {}", e))?;

    // Write JSON (pretty-printed for readability)
    let json = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    std::fs::write(export_dir.join("scene.gltf"), json)
        .map_err(|e| format!("Failed to write scene.gltf: {}", e))?;

    // Write binary buffer
    std::fs::write(export_dir.join("scene.bin"), bin_data)
        .map_err(|e| format!("Failed to write scene.bin: {}", e))?;

    Ok(())
}
