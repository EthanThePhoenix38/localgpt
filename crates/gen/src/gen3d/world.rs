//! World skill save/load — serialize scenes as complete skill directories.
//!
//! A world skill directory contains:
//! - `SKILL.md`       — Skill metadata + description
//! - `world.toml`     — Manifest tying everything together
//! - `scene.glb`      — glTF geometry & materials
//! - `behaviors.toml` — Declarative behavior definitions
//! - `audio.toml`     — Ambience + spatial audio emitters

use std::path::{Path, PathBuf};

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::audio::AudioEngine;
use super::behaviors::{self, BehaviorState, EntityBehaviors};
use super::commands::*;
use super::plugin::GenWorkspace;
use super::registry::*;

// ---------------------------------------------------------------------------
// TOML data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct WorldManifest {
    world: WorldMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    environment: Option<EnvironmentDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    camera: Option<CameraDef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorldMeta {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default = "default_scene_file")]
    scene: String,
    #[serde(default = "default_behaviors_file")]
    behaviors: String,
    #[serde(default = "default_audio_file")]
    audio: String,
}

fn default_scene_file() -> String {
    "scene.glb".to_string()
}
fn default_behaviors_file() -> String {
    "behaviors.toml".to_string()
}
fn default_audio_file() -> String {
    "audio.toml".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentDef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    background_color: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ambient_intensity: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ambient_color: Option<[f32; 4]>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CameraDef {
    position: [f32; 3],
    look_at: [f32; 3],
    #[serde(default = "default_fov")]
    fov_degrees: f32,
}

fn default_fov() -> f32 {
    45.0
}

#[derive(Debug, Serialize, Deserialize)]
struct BehaviorsFile {
    #[serde(default)]
    behaviors: Vec<BehaviorEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BehaviorEntry {
    entity: String,
    #[serde(flatten)]
    behavior: BehaviorDef,
}

#[derive(Debug, Serialize, Deserialize)]
struct AudioFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ambience: Option<AmbienceDef>,
    #[serde(default)]
    emitters: Vec<AudioEmitterCmd>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AmbienceDef {
    layers: Vec<AmbienceLayerDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    master_volume: Option<f32>,
}

// ---------------------------------------------------------------------------
// Save world
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn handle_save_world(
    cmd: SaveWorldCmd,
    workspace: &GenWorkspace,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    parent_query: &Query<&ChildOf>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    mesh_handles: &Query<&Mesh3d>,
    mesh_assets: &Assets<Mesh>,
    audio_engine: &AudioEngine,
    behaviors_query: &Query<&mut EntityBehaviors>,
) -> GenResponse {
    // Resolve output directory
    let skill_dir = if let Some(ref path) = cmd.path {
        PathBuf::from(shellexpand::tilde(path).as_ref())
    } else {
        workspace.path.join("skills").join(&cmd.name)
    };

    if let Err(e) = std::fs::create_dir_all(&skill_dir) {
        return GenResponse::Error {
            message: format!("Failed to create skill directory: {}", e),
        };
    }

    // 1. Export scene.glb
    let scene_path = skill_dir.join("scene.glb");
    let glb_result = export_scene_glb(
        &scene_path,
        registry,
        transforms,
        gen_entities,
        parent_query,
        material_handles,
        material_assets,
        mesh_handles,
        mesh_assets,
    );
    if let Err(e) = glb_result {
        return GenResponse::Error {
            message: format!("Failed to export scene: {}", e),
        };
    }

    // 2. Write behaviors.toml
    let all_behaviors = behaviors::collect_all_behaviors(registry, behaviors_query);
    let behaviors_file = BehaviorsFile {
        behaviors: all_behaviors
            .iter()
            .flat_map(|(entity, defs)| {
                defs.iter().map(move |def| BehaviorEntry {
                    entity: entity.clone(),
                    behavior: def.clone(),
                })
            })
            .collect(),
    };
    let behaviors_toml = toml::to_string_pretty(&behaviors_file).unwrap_or_else(|_| String::new());
    if let Err(e) = std::fs::write(skill_dir.join("behaviors.toml"), &behaviors_toml) {
        return GenResponse::Error {
            message: format!("Failed to write behaviors.toml: {}", e),
        };
    }

    // 3. Write audio.toml
    let audio_file = collect_audio_state(audio_engine);
    let audio_toml = toml::to_string_pretty(&audio_file).unwrap_or_else(|_| String::new());
    if let Err(e) = std::fs::write(skill_dir.join("audio.toml"), &audio_toml) {
        return GenResponse::Error {
            message: format!("Failed to write audio.toml: {}", e),
        };
    }

    // 4. Gather camera info
    let camera_def = registry.get_entity("main_camera").and_then(|e| {
        transforms.get(e).ok().map(|t| {
            let forward = t.forward().as_vec3();
            let look_at = t.translation + forward * 10.0;
            CameraDef {
                position: t.translation.to_array(),
                look_at: look_at.to_array(),
                fov_degrees: 45.0,
            }
        })
    });

    // 5. Write world.toml
    let manifest = WorldManifest {
        world: WorldMeta {
            name: cmd.name.clone(),
            description: cmd.description.clone(),
            scene: "scene.glb".to_string(),
            behaviors: "behaviors.toml".to_string(),
            audio: "audio.toml".to_string(),
        },
        environment: None, // TODO: capture from current ClearColor/AmbientLight resources
        camera: camera_def,
    };
    let manifest_toml = toml::to_string_pretty(&manifest).unwrap_or_else(|_| String::new());
    if let Err(e) = std::fs::write(skill_dir.join("world.toml"), &manifest_toml) {
        return GenResponse::Error {
            message: format!("Failed to write world.toml: {}", e),
        };
    }

    // 6. Write SKILL.md
    let description = cmd.description.as_deref().unwrap_or("A generated 3D world");
    let skill_md = format!(
        r#"---
name: "{name}"
description: "{description}"
user-invocable: true
metadata:
  type: "world"
useWhen:
  - contains: "{name}"
---
# {name}

{description}

This is a gen world skill. Load it with `gen_load_world` to restore the 3D scene,
behaviors, and audio.
"#,
        name = cmd.name,
        description = description,
    );
    if let Err(e) = std::fs::write(skill_dir.join("SKILL.md"), &skill_md) {
        return GenResponse::Error {
            message: format!("Failed to write SKILL.md: {}", e),
        };
    }

    GenResponse::WorldSaved {
        path: skill_dir.to_string_lossy().into_owned(),
        skill_name: cmd.name,
    }
}

// ---------------------------------------------------------------------------
// Load world
// ---------------------------------------------------------------------------

/// Result of parsing a world skill directory (returned to plugin.rs for ECS application).
pub struct WorldLoadResult {
    pub world_path: String,
    pub scene_path: Option<String>,
    pub behaviors: Vec<(String, Vec<BehaviorDef>)>,
    pub ambience: Option<AmbienceCmd>,
    pub emitters: Vec<AudioEmitterCmd>,
    pub environment: Option<EnvironmentCmd>,
    pub camera: Option<CameraCmd>,
    pub entity_count: usize,
    pub behavior_count: usize,
}

pub fn handle_load_world(
    path: &str,
    workspace: &GenWorkspace,
    _behavior_state: &mut BehaviorState,
) -> Result<WorldLoadResult, String> {
    let world_dir = resolve_world_path(path, &workspace.path)
        .ok_or_else(|| format!("World skill not found: {}", path))?;

    // Read world.toml
    let manifest_path = world_dir.join("world.toml");
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read world.toml: {}", e))?;
    let manifest: WorldManifest =
        toml::from_str(&manifest_str).map_err(|e| format!("Failed to parse world.toml: {}", e))?;

    // Resolve scene.glb path
    let scene_path = {
        let p = world_dir.join(&manifest.world.scene);
        if p.exists() {
            Some(p.to_string_lossy().into_owned())
        } else {
            None
        }
    };

    // Read behaviors.toml
    let behaviors_path = world_dir.join(&manifest.world.behaviors);
    let mut behaviors: Vec<(String, Vec<BehaviorDef>)> = Vec::new();
    if behaviors_path.exists() {
        let s = std::fs::read_to_string(&behaviors_path)
            .map_err(|e| format!("Failed to read behaviors.toml: {}", e))?;
        let file: BehaviorsFile =
            toml::from_str(&s).map_err(|e| format!("Failed to parse behaviors.toml: {}", e))?;

        // Group by entity
        let mut map: std::collections::HashMap<String, Vec<BehaviorDef>> =
            std::collections::HashMap::new();
        for entry in file.behaviors {
            map.entry(entry.entity).or_default().push(entry.behavior);
        }
        behaviors = map.into_iter().collect();
    }

    // Read audio.toml
    let audio_path = world_dir.join(&manifest.world.audio);
    let mut ambience: Option<AmbienceCmd> = None;
    let mut emitters: Vec<AudioEmitterCmd> = Vec::new();
    if audio_path.exists() {
        let s = std::fs::read_to_string(&audio_path)
            .map_err(|e| format!("Failed to read audio.toml: {}", e))?;
        let audio_file: AudioFile =
            toml::from_str(&s).map_err(|e| format!("Failed to parse audio.toml: {}", e))?;

        if let Some(amb) = audio_file.ambience {
            ambience = Some(AmbienceCmd {
                layers: amb.layers,
                master_volume: amb.master_volume,
                reverb: None,
            });
        }
        emitters = audio_file.emitters;
    }

    // Environment
    let environment = manifest.environment.map(|env| EnvironmentCmd {
        background_color: env.background_color,
        ambient_light: env.ambient_intensity,
        ambient_color: env.ambient_color,
    });

    // Camera
    let camera = manifest.camera.map(|cam| CameraCmd {
        position: cam.position,
        look_at: cam.look_at,
        fov_degrees: cam.fov_degrees,
    });

    let behavior_count: usize = behaviors.iter().map(|(_, v)| v.len()).sum();

    Ok(WorldLoadResult {
        world_path: world_dir.to_string_lossy().into_owned(),
        scene_path,
        entity_count: 0, // Will be counted after scene loads
        behavior_count,
        behaviors,
        ambience,
        emitters,
        environment,
        camera,
    })
}

/// Resolve a world skill path:
/// 1. As-is (absolute or relative)
/// 2. {workspace}/skills/{name}
/// 3. {workspace}/skills/{name}/ (with trailing slash)
fn resolve_world_path(path: &str, workspace: &Path) -> Option<PathBuf> {
    let expanded = shellexpand::tilde(path).into_owned();

    // 1. As-is
    let p = PathBuf::from(&expanded);
    if p.is_dir() && p.join("world.toml").exists() {
        return Some(p);
    }

    // 2. {workspace}/skills/{name}
    let skill_path = workspace.join("skills").join(&expanded);
    if skill_path.is_dir() && skill_path.join("world.toml").exists() {
        return Some(skill_path);
    }

    None
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_audio_state(engine: &AudioEngine) -> AudioFile {
    // We can't directly read back the ambience layers from AudioEngine,
    // so we store the layer names. For full round-trip, the ambience
    // would need to be stored in a resource. For now, emit what we can.
    AudioFile {
        ambience: if engine.active && !engine.ambience_layer_names.is_empty() {
            // We don't have full layer data in AudioEngine (only names),
            // so we output a placeholder. Full fidelity would require
            // storing the AmbienceCmd in the engine.
            None
        } else {
            None
        },
        emitters: engine
            .emitter_meta
            .iter()
            .map(|(name, meta)| {
                // Reconstruct emitter commands from stored metadata
                AudioEmitterCmd {
                    name: name.clone(),
                    entity: meta.attached_to.clone(),
                    position: meta.position,
                    sound: infer_emitter_sound_from_type(&meta.sound_type),
                    radius: meta.radius,
                    volume: meta.base_volume,
                }
            })
            .collect(),
    }
}

fn infer_emitter_sound_from_type(sound_type: &str) -> EmitterSound {
    match sound_type {
        "water" => EmitterSound::Water { turbulence: 0.5 },
        "fire" => EmitterSound::Fire {
            intensity: 0.5,
            crackle: 0.4,
        },
        "hum" => EmitterSound::Hum {
            frequency: 120.0,
            warmth: 0.5,
        },
        "wind" => EmitterSound::Wind { pitch: 400.0 },
        _ => EmitterSound::Hum {
            frequency: 120.0,
            warmth: 0.5,
        },
    }
}

/// Export scene geometry to GLB. This is a simplified version of plugin.rs's
/// handle_export_gltf that writes to a specific path.
#[allow(clippy::too_many_arguments)]
fn export_scene_glb(
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

        // Positions
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

        // Normals
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

        // Indices
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

        // Material
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

        // Mesh primitive
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

        // Node
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

    if root_nodes.is_empty() {
        // No mesh entities — still create an empty scene
        root_nodes.clear();
    }

    root.scenes.push(gltf_json::Scene {
        name: Some("Scene".to_string()),
        nodes: root_nodes,
        extensions: Default::default(),
        extras: Default::default(),
    });
    root.scene = Some(gltf_json::Index::new(0));

    if !bin_data.is_empty() {
        root.buffers.push(gltf_json::Buffer {
            byte_length: USize64(bin_data.len() as u64),
            uri: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
    }

    // Serialize
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

    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(&json_bytes);

    glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
    glb.extend_from_slice(&bin_data);

    std::fs::write(output_path, &glb).map_err(|e| format!("Failed to write GLB: {}", e))
}
