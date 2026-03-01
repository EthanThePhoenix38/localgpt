//! Bevy GenPlugin — command processing, default scene, screenshot capture, glTF loading.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use std::ffi::OsStr;
use std::path::PathBuf;

use localgpt_world_types as wt;

use super::GenChannels;
use super::audio::{self, SpatialAudioListener};
use super::behaviors::{self, BehaviorState, EntityBehaviors};
use super::commands::*;
use super::compat;
use super::registry::*;

/// Bevy resource holding the workspace path for default export locations.
#[derive(Resource, Clone)]
pub struct GenWorkspace {
    pub path: PathBuf,
}

/// Bevy resource wrapping the channel endpoints.
#[derive(Resource)]
pub struct GenChannelRes {
    channels: GenChannels,
}

impl GenChannelRes {
    pub fn new(channels: GenChannels) -> Self {
        Self { channels }
    }
}

/// Pending screenshot requests that need to wait N frames.
#[derive(Resource, Default)]
pub struct PendingScreenshots {
    queue: Vec<PendingScreenshot>,
}

#[allow(dead_code)]
struct PendingScreenshot {
    frames_remaining: u32,
    width: u32,
    height: u32,
    path: Option<String>,
}

/// Initial glTF scene to load at startup.
#[derive(Resource)]
pub struct GenInitialScene {
    pub path: Option<PathBuf>,
}

/// Undo/redo stack wrapping `EditHistory` from world-types.
///
/// Records `WorldEdit` operations (spawn, delete, modify) as they happen.
/// `gen_undo` / `gen_redo` commands apply inverse operations to restore state.
#[derive(Resource, Default)]
pub struct UndoStack {
    pub history: wt::EditHistory,
}

/// A glTF scene that is currently being loaded.
struct PendingGltfLoad {
    handle: Handle<Scene>,
    name: String,
    path: String,
    send_response: bool,
}

/// Queue of pending glTF loads waiting for asset server to finish loading.
#[derive(Resource, Default)]
struct PendingGltfLoads {
    queue: Vec<PendingGltfLoad>,
}

/// Deferred world setup — applied after a world's glTF scene finishes spawning.
///
/// When loading a world, the glTF scene is async. Entity names from the glTF
/// aren't available until Bevy's scene spawner creates them (1-2 frames after
/// the asset loads). This resource holds the behaviors and audio emitters
/// that need to be applied once the named entities appear.
#[derive(Resource, Default)]
struct PendingWorldSetup {
    active: Option<WorldSetupData>,
}

struct WorldSetupData {
    /// Entity-name → behavior definitions to apply.
    behaviors: Vec<(String, Vec<BehaviorDef>)>,
    /// Audio emitters that reference entities by name.
    emitters: Vec<AudioEmitterCmd>,
    /// How many frames we've been waiting (give up after a limit).
    frames_waited: u32,
}

/// Bevy resource storing the active avatar configuration for the current world.
#[derive(Resource, Default)]
pub struct AvatarConfig {
    pub active: Option<AvatarDef>,
}

/// Bevy resource storing guided tour definitions for the current world.
#[derive(Resource, Default)]
pub struct WorldTours {
    pub tours: Vec<TourDef>,
}

/// Marker component for the interactive fly camera.
#[derive(Component)]
struct FlyCam;

/// Configuration for the fly camera controller.
#[derive(Resource)]
struct FlyCamConfig {
    move_speed: f32,
    look_sensitivity: f32,
}

impl Default for FlyCamConfig {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            look_sensitivity: 0.003,
        }
    }
}

/// Plugin that sets up the Gen 3D environment.
#[allow(dead_code)]
pub struct GenPlugin {
    pub channels: GenChannels,
}

impl Plugin for GenPlugin {
    fn build(&self, _app: &mut App) {
        // We can't move channels out of &self in build(), so we use a
        // workaround: store channels in a temporary and take them in a
        // startup system. See `setup_channels` below.
    }
}

/// Initialize the Gen world: channels, default scene, systems.
///
/// Call this instead of using Plugin::build since we need to move the channels.
pub fn setup_gen_app(
    app: &mut App,
    channels: GenChannels,
    workspace: PathBuf,
    initial_scene: Option<PathBuf>,
) {
    app.insert_resource(GenChannelRes::new(channels))
        .insert_resource(GenWorkspace { path: workspace })
        .insert_resource(GenInitialScene {
            path: initial_scene,
        })
        .init_resource::<NameRegistry>()
        .init_resource::<NextEntityId>()
        .init_resource::<DirtyTracker>()
        .init_resource::<UndoStack>()
        .init_resource::<PendingScreenshots>()
        .init_resource::<PendingGltfLoads>()
        .init_resource::<PendingWorldSetup>()
        .init_resource::<AvatarConfig>()
        .init_resource::<WorldTours>()
        .init_resource::<FlyCamConfig>()
        .init_resource::<BehaviorState>()
        .add_systems(
            Startup,
            (
                setup_default_scene,
                load_initial_scene,
                audio::init_audio_engine,
            ),
        )
        .add_systems(Update, process_gen_commands)
        .add_systems(Update, process_pending_screenshots)
        .add_systems(Update, process_pending_gltf_loads)
        .add_systems(Update, process_pending_world_setup)
        .add_systems(Update, audio::spatial_audio_update)
        .add_systems(Update, audio::auto_infer_audio)
        .add_systems(Update, behaviors::behavior_tick)
        .add_systems(Update, fly_cam_movement)
        .add_systems(Update, fly_cam_look)
        .add_systems(Update, fly_cam_scroll_speed);
}

/// Default scene: ground plane, camera, directional light, ambient light.
fn setup_default_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut registry: ResMut<NameRegistry>,
    mut next_id: ResMut<NextEntityId>,
) {
    // Ground plane — 20×20 gray
    let ground_id = next_id.alloc();
    let ground = commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(10.0, 10.0)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.3, 0.3, 1.0),
                metallic: 0.0,
                perceptual_roughness: 0.8,
                ..default()
            })),
            Transform::from_translation(Vec3::ZERO),
            Name::new("ground_plane"),
            GenEntity {
                entity_type: GenEntityType::Primitive,
                world_id: ground_id,
            },
        ))
        .id();
    registry.insert_with_id("ground_plane".into(), ground, ground_id);

    // Camera at (5, 5, 5) looking at origin
    let cam_id = next_id.alloc();
    let camera = commands
        .spawn((
            Camera3d::default(),
            Transform::from_translation(Vec3::new(5.0, 5.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("main_camera"),
            FlyCam,
            SpatialAudioListener,
            GenEntity {
                entity_type: GenEntityType::Camera,
                world_id: cam_id,
            },
        ))
        .id();
    registry.insert_with_id("main_camera".into(), camera, cam_id);

    // Directional light — warm white, shadows
    let light_id = next_id.alloc();
    let light = commands
        .spawn((
            DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                color: Color::srgba(1.0, 0.95, 0.9, 1.0),
                ..default()
            },
            Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("main_light"),
            GenEntity {
                entity_type: GenEntityType::Light,
                world_id: light_id,
            },
        ))
        .id();
    registry.insert_with_id("main_light".into(), light, light_id);
}

/// Load the initial scene file if provided.
fn load_initial_scene(
    initial_scene: Res<GenInitialScene>,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingGltfLoads>,
) {
    let Some(ref path) = initial_scene.path else {
        return;
    };

    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "scene".to_string());

    let asset_path = path.to_string_lossy().trim_start_matches('/').to_string();
    let handle = asset_server.load::<Scene>(format!("{}#Scene0", asset_path));

    pending.queue.push(PendingGltfLoad {
        handle,
        name,
        path: path.to_string_lossy().into_owned(),
        send_response: false,
    });
}

/// Poll the command channel each frame and dispatch.
#[derive(SystemParam)]
struct GenCommandParams<'w, 's> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    registry: ResMut<'w, NameRegistry>,
    next_entity_id: ResMut<'w, NextEntityId>,
    dirty_tracker: ResMut<'w, DirtyTracker>,
    undo_stack: ResMut<'w, UndoStack>,
    pending_screenshots: ResMut<'w, PendingScreenshots>,
    pending_gltf: ResMut<'w, PendingGltfLoads>,
    audio_engine: ResMut<'w, audio::AudioEngine>,
    behavior_state: ResMut<'w, BehaviorState>,
    asset_server: Res<'w, AssetServer>,
    workspace: Res<'w, GenWorkspace>,
    transforms: Query<'w, 's, &'static Transform>,
    gen_entities: Query<'w, 's, &'static GenEntity>,
    names_query: Query<'w, 's, &'static Name>,
    children_query: Query<'w, 's, &'static Children>,
    parent_query: Query<'w, 's, &'static ChildOf>,
    visibility_query: Query<'w, 's, &'static Visibility>,
    material_handles: Query<'w, 's, &'static MeshMaterial3d<StandardMaterial>>,
    mesh_handles: Query<'w, 's, &'static Mesh3d>,
    behaviors_query: Query<'w, 's, &'static mut EntityBehaviors>,
    parametric_shapes: Query<'w, 's, &'static ParametricShape>,
    directional_lights: Query<'w, 's, &'static DirectionalLight>,
    point_lights: Query<'w, 's, &'static PointLight>,
    spot_lights: Query<'w, 's, &'static SpotLight>,
    audio_emitters: Query<'w, 's, &'static audio::AudioEmitter>,
    projections: Query<'w, 's, &'static Projection>,
    clear_color: Option<Res<'w, ClearColor>>,
    ambient_light: Option<Res<'w, GlobalAmbientLight>>,
    pending_world: ResMut<'w, PendingWorldSetup>,
    avatar_config: ResMut<'w, AvatarConfig>,
    world_tours: ResMut<'w, WorldTours>,
}

/// Build a `SnapshotQueries` from `GenCommandParams`. Used in many dispatch arms.
macro_rules! snap_queries {
    ($params:expr) => {
        SnapshotQueries {
            transforms: &$params.transforms,
            parametric_shapes: &$params.parametric_shapes,
            material_handles: &$params.material_handles,
            materials: &$params.materials,
            visibility_query: &$params.visibility_query,
            directional_lights: &$params.directional_lights,
            point_lights: &$params.point_lights,
            spot_lights: &$params.spot_lights,
            behaviors_query: &$params.behaviors_query,
            audio_emitters: &$params.audio_emitters,
            parent_query: &$params.parent_query,
            registry: &$params.registry,
        }
    };
}

fn process_gen_commands(
    mut channel_res: ResMut<GenChannelRes>,
    mut commands: Commands,
    mut params: GenCommandParams,
) {
    while let Ok(cmd) = channel_res.channels.cmd_rx.try_recv() {
        let response = match cmd {
            GenCommand::SceneInfo => handle_scene_info(
                &params.registry,
                &params.transforms,
                &params.gen_entities,
                &params.material_handles,
                &params.materials,
                &params.parametric_shapes,
            ),
            GenCommand::EntityInfo { name } => handle_entity_info(
                &name,
                &params.registry,
                &params.transforms,
                &params.gen_entities,
                &params.names_query,
                &params.children_query,
                &params.parent_query,
                &params.visibility_query,
                &params.material_handles,
                &params.materials,
                &params.behaviors_query,
                &params.parametric_shapes,
                &params.directional_lights,
                &params.point_lights,
                &params.spot_lights,
            ),
            GenCommand::Screenshot {
                width,
                height,
                wait_frames,
            } => {
                params.pending_screenshots.queue.push(PendingScreenshot {
                    frames_remaining: wait_frames,
                    width,
                    height,
                    path: None,
                });
                // Response will be sent by process_pending_screenshots
                continue;
            }
            GenCommand::SpawnPrimitive(cmd) => handle_spawn_primitive(
                cmd,
                &mut commands,
                &mut params.meshes,
                &mut params.materials,
                &mut params.registry,
                &mut params.next_entity_id,
            ),
            GenCommand::ModifyEntity(cmd) => {
                // Snapshot before modify so we can undo
                let pre_snapshot = params.registry.get_entity(&cmd.name).and_then(|e| {
                    params
                        .registry
                        .get_id(e)
                        .map(|id| snapshot_entity(&cmd.name, e, id, &snap_queries!(params)))
                });
                let resp = handle_modify_entity(
                    cmd.clone(),
                    &mut commands,
                    &params.registry,
                    &mut params.materials,
                    &params.material_handles,
                    &params.transforms,
                );
                if let GenResponse::Modified { .. } = &resp
                    && let Some(old_we) = pre_snapshot
                {
                    let id = old_we.id;
                    let mut new_we = old_we.clone();
                    apply_modify_to_snapshot(&mut new_we, &cmd);
                    params.dirty_tracker.mark_dirty(id);
                    params.undo_stack.history.push(
                        wt::EditOp::Batch {
                            ops: vec![wt::EditOp::delete(id), wt::EditOp::spawn(new_we)],
                        },
                        wt::EditOp::Batch {
                            ops: vec![wt::EditOp::delete(id), wt::EditOp::spawn(old_we)],
                        },
                        None,
                    );
                }
                resp
            }
            GenCommand::DeleteEntity { name } => {
                // Snapshot before delete so we can undo
                let pre_snapshot = params.registry.get_entity(&name).and_then(|e| {
                    params
                        .registry
                        .get_id(e)
                        .map(|id| snapshot_entity(&name, e, id, &snap_queries!(params)))
                });
                let resp = handle_delete_entity(&name, &mut commands, &mut params.registry);
                if let GenResponse::Deleted { .. } = &resp
                    && let Some(we) = pre_snapshot
                {
                    let id = we.id;
                    params.dirty_tracker.mark_dirty(id);
                    params.undo_stack.history.push(
                        wt::EditOp::delete(id),
                        wt::EditOp::spawn(we),
                        None,
                    );
                }
                resp
            }
            GenCommand::SetCamera(cmd) => {
                // Capture old camera state for undo
                let old_camera = params.registry.get_entity("main_camera").map(|cam_ent| {
                    let pos = params
                        .transforms
                        .get(cam_ent)
                        .map(|t| t.translation.to_array())
                        .unwrap_or([5.0, 5.0, 5.0]);
                    let fov = params
                        .projections
                        .get(cam_ent)
                        .ok()
                        .and_then(|p| match p {
                            Projection::Perspective(pp) => Some(pp.fov.to_degrees()),
                            _ => None,
                        })
                        .unwrap_or(45.0);
                    // Compute look_at from current forward direction
                    let forward = params
                        .transforms
                        .get(cam_ent)
                        .map(|t| t.forward().as_vec3())
                        .unwrap_or(Vec3::NEG_Z);
                    let look_at = params
                        .transforms
                        .get(cam_ent)
                        .map(|t| (t.translation + forward * 10.0).to_array())
                        .unwrap_or([0.0, 0.0, 0.0]);
                    wt::CameraDef {
                        position: pos,
                        look_at,
                        fov_degrees: fov,
                    }
                });
                let new_camera = wt::CameraDef {
                    position: cmd.position,
                    look_at: cmd.look_at,
                    fov_degrees: cmd.fov_degrees,
                };
                let resp = handle_set_camera(cmd, &mut commands, &params.registry);
                if let GenResponse::CameraSet = &resp {
                    if let Some(old_cam) = old_camera {
                        params.undo_stack.history.push(
                            wt::EditOp::SetCamera { camera: new_camera },
                            wt::EditOp::SetCamera { camera: old_cam },
                            None,
                        );
                    }
                }
                resp
            }
            GenCommand::SetLight(cmd) => {
                // Snapshot the old light before it gets despawned (for undo)
                let old_light_snapshot =
                    params.registry.get_entity(&cmd.name).and_then(|old_ent| {
                        params.registry.get_id(old_ent).map(|old_id| {
                            snapshot_entity(&cmd.name, old_ent, old_id, &snap_queries!(params))
                        })
                    });
                let resp = handle_set_light(
                    cmd,
                    &mut commands,
                    &mut params.registry,
                    &mut params.next_entity_id,
                );
                // Record undo: if we replaced an old light, use batch to restore it
                if let GenResponse::LightSet { ref name } = resp
                    && let Some(new_ent) = params.registry.get_entity(name)
                    && let Some(new_id) = params.registry.get_id(new_ent)
                {
                    let new_we = snapshot_entity(name, new_ent, new_id, &snap_queries!(params));
                    params.dirty_tracker.mark_dirty(new_id);
                    if let Some(old_we) = old_light_snapshot {
                        // Replacing existing light: undo restores old, redo re-applies new
                        let old_id = old_we.id;
                        params.undo_stack.history.push(
                            wt::EditOp::Batch {
                                ops: vec![wt::EditOp::delete(new_id), wt::EditOp::spawn(new_we)],
                            },
                            wt::EditOp::Batch {
                                ops: vec![wt::EditOp::delete(old_id), wt::EditOp::spawn(old_we)],
                            },
                            None,
                        );
                    } else {
                        // New light: undo is simply delete
                        params.undo_stack.history.push(
                            wt::EditOp::spawn(new_we),
                            wt::EditOp::delete(new_id),
                            None,
                        );
                    }
                }
                resp
            }
            GenCommand::SetEnvironment(cmd) => {
                // Capture current environment for undo
                let old_env = {
                    let bg = params.clear_color.as_ref().map(|cc| {
                        let c = cc.0.to_srgba();
                        [c.red, c.green, c.blue, c.alpha]
                    });
                    let (ambient_intensity, ambient_color) =
                        params.ambient_light.as_ref().map_or((None, None), |al| {
                            let c = al.color.to_srgba();
                            (Some(al.brightness), Some([c.red, c.green, c.blue, c.alpha]))
                        });
                    wt::EnvironmentDef {
                        background_color: bg,
                        ambient_intensity,
                        ambient_color,
                        fog_density: None,
                        fog_color: None,
                    }
                };
                // Build new env from the command (can't read resources after
                // deferred commands since they haven't been applied yet)
                let new_env = wt::EnvironmentDef {
                    background_color: cmd.background_color,
                    ambient_intensity: cmd.ambient_light,
                    ambient_color: cmd.ambient_color,
                    fog_density: None,
                    fog_color: None,
                };
                let resp = handle_set_environment(cmd, &mut commands);
                if let GenResponse::EnvironmentSet = &resp {
                    params.undo_stack.history.push(
                        wt::EditOp::SetEnvironment { env: new_env },
                        wt::EditOp::SetEnvironment { env: old_env },
                        None,
                    );
                }
                resp
            }
            GenCommand::SpawnMesh(cmd) => handle_spawn_mesh(
                cmd,
                &mut commands,
                &mut params.meshes,
                &mut params.materials,
                &mut params.registry,
                &mut params.next_entity_id,
            ),
            GenCommand::ExportScreenshot {
                path,
                width,
                height,
            } => {
                params.pending_screenshots.queue.push(PendingScreenshot {
                    frames_remaining: 3,
                    width,
                    height,
                    path: Some(path),
                });
                continue;
            }
            GenCommand::ExportGltf { path } => handle_export_gltf(
                path.as_deref(),
                &params.workspace,
                &params.registry,
                &params.transforms,
                &params.gen_entities,
                &params.parent_query,
                &params.material_handles,
                &params.materials,
                &params.mesh_handles,
                &params.meshes,
            ),
            GenCommand::LoadGltf { path } => {
                if let Some(resolved) = resolve_gltf_path(&path, &params.workspace.path) {
                    let name = resolved
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "imported_scene".to_string());

                    let asset_path = resolved
                        .to_string_lossy()
                        .trim_start_matches('/')
                        .to_string();
                    let handle = params
                        .asset_server
                        .load::<Scene>(format!("{}#Scene0", asset_path));

                    params.pending_gltf.queue.push(PendingGltfLoad {
                        handle,
                        name,
                        path: resolved.to_string_lossy().into_owned(),
                        send_response: true,
                    });
                } else {
                    let response = GenResponse::Error {
                        message: format!("Failed to resolve path: {}", path),
                    };
                    let _ = channel_res.channels.resp_tx.send(response);
                }
                // Response for successful loads is sent by process_pending_gltf_loads
                continue;
            }

            // Audio commands
            GenCommand::SetAmbience(cmd) => {
                audio::handle_set_ambience(cmd, &mut params.audio_engine)
            }
            GenCommand::SpawnAudioEmitter(cmd) => audio::handle_spawn_audio_emitter(
                cmd,
                &mut params.audio_engine,
                &mut commands,
                &mut params.registry,
                &mut params.next_entity_id,
            ),
            GenCommand::ModifyAudioEmitter(cmd) => {
                audio::handle_modify_audio_emitter(cmd, &mut params.audio_engine)
            }
            GenCommand::RemoveAudioEmitter { name } => {
                audio::handle_remove_audio_emitter(&name, &mut params.audio_engine)
            }
            GenCommand::AudioInfo => audio::handle_audio_info(&params.audio_engine),

            // Behavior commands
            GenCommand::AddBehavior(cmd) => {
                // Snapshot before adding behavior for undo
                let pre_snapshot = params.registry.get_entity(&cmd.entity).and_then(|e| {
                    params
                        .registry
                        .get_id(e)
                        .map(|id| snapshot_entity(&cmd.entity, e, id, &snap_queries!(params)))
                });
                let entity_name = cmd.entity.clone();
                let resp = behaviors::handle_add_behavior(
                    cmd,
                    &mut params.behavior_state,
                    &mut commands,
                    &params.registry,
                    &params.transforms,
                    &mut params.behaviors_query,
                );
                if let GenResponse::BehaviorAdded { .. } = &resp
                    && let Some(old_we) = pre_snapshot
                    && let Some(e) = params.registry.get_entity(&entity_name)
                    && let Some(id) = params.registry.get_id(e)
                {
                    let new_we = snapshot_entity(&entity_name, e, id, &snap_queries!(params));
                    params.undo_stack.history.push(
                        wt::EditOp::Batch {
                            ops: vec![wt::EditOp::delete(id), wt::EditOp::spawn(new_we)],
                        },
                        wt::EditOp::Batch {
                            ops: vec![wt::EditOp::delete(id), wt::EditOp::spawn(old_we)],
                        },
                        None,
                    );
                }
                resp
            }
            GenCommand::RemoveBehavior {
                entity,
                behavior_id,
            } => {
                // Snapshot before removing behavior for undo
                let pre_snapshot = params.registry.get_entity(&entity).and_then(|e| {
                    params
                        .registry
                        .get_id(e)
                        .map(|id| snapshot_entity(&entity, e, id, &snap_queries!(params)))
                });
                let entity_name = entity.clone();
                let resp = behaviors::handle_remove_behavior(
                    &entity,
                    behavior_id.as_deref(),
                    &params.registry,
                    &mut params.behaviors_query,
                );
                if let GenResponse::BehaviorRemoved { count, .. } = &resp
                    && *count > 0
                    && let Some(old_we) = pre_snapshot
                    && let Some(e) = params.registry.get_entity(&entity_name)
                    && let Some(id) = params.registry.get_id(e)
                {
                    let new_we = snapshot_entity(&entity_name, e, id, &snap_queries!(params));
                    params.undo_stack.history.push(
                        wt::EditOp::Batch {
                            ops: vec![wt::EditOp::delete(id), wt::EditOp::spawn(new_we)],
                        },
                        wt::EditOp::Batch {
                            ops: vec![wt::EditOp::delete(id), wt::EditOp::spawn(old_we)],
                        },
                        None,
                    );
                }
                resp
            }
            GenCommand::ListBehaviors { entity } => behaviors::handle_list_behaviors(
                entity.as_deref(),
                &params.behavior_state,
                &params.registry,
                &params.behaviors_query,
            ),
            GenCommand::SetBehaviorsPaused { paused } => {
                params.behavior_state.paused = paused;
                GenResponse::BehaviorsPaused { paused }
            }

            // World commands
            GenCommand::SaveWorld(cmd) => {
                let env_data = super::world::EnvironmentSnapshot {
                    background_color: params.clear_color.as_ref().map(|c| {
                        let srgba = c.0.to_srgba();
                        [srgba.red, srgba.green, srgba.blue, srgba.alpha]
                    }),
                    ambient_intensity: params.ambient_light.as_ref().map(|a| a.brightness),
                    ambient_color: params.ambient_light.as_ref().map(|a| {
                        let srgba = a.color.to_srgba();
                        [srgba.red, srgba.green, srgba.blue, srgba.alpha]
                    }),
                };
                super::world::handle_save_world(
                    cmd,
                    &params.workspace,
                    &params.registry,
                    &params.transforms,
                    &params.gen_entities,
                    &params.parent_query,
                    &params.material_handles,
                    &params.materials,
                    &params.mesh_handles,
                    &params.meshes,
                    &params.audio_engine,
                    &params.behaviors_query,
                    &params.parametric_shapes,
                    &params.visibility_query,
                    &params.directional_lights,
                    &params.point_lights,
                    &params.spot_lights,
                    &params.projections,
                    &env_data,
                    params.avatar_config.active.as_ref(),
                    &params.world_tours.tours,
                )
            }
            GenCommand::LoadWorld { path, clear } => {
                // Clear existing scene before loading if requested.
                if clear {
                    handle_clear_scene(
                        true, // keep camera
                        true, // keep lights
                        &mut commands,
                        &mut params.registry,
                        &params.gen_entities,
                        &mut params.audio_engine,
                        &mut params.behavior_state,
                        &mut params.pending_world,
                    );
                }

                let result = super::world::handle_load_world(
                    &path,
                    &params.workspace,
                    &mut params.behavior_state,
                );
                match result {
                    Ok(world_load) => {
                        if !world_load.world_entities.is_empty() {
                            // RON format — spawn entities directly from WorldEntity data
                            spawn_world_entities(
                                &world_load.world_entities,
                                &mut commands,
                                &mut params.meshes,
                                &mut params.materials,
                                &mut params.registry,
                                &mut params.next_entity_id,
                                &mut params.behavior_state,
                            );
                        } else if let Some(scene_path) = world_load.scene_path
                            && let Some(resolved) =
                                resolve_gltf_path(&scene_path, &params.workspace.path)
                        {
                            // Legacy format — queue glTF scene load (async)
                            let asset_path = resolved
                                .to_string_lossy()
                                .trim_start_matches('/')
                                .to_string();
                            let handle = params
                                .asset_server
                                .load::<Scene>(format!("{}#Scene0", asset_path));
                            params.pending_gltf.queue.push(PendingGltfLoad {
                                handle,
                                name: "world_scene".to_string(),
                                path: resolved.to_string_lossy().into_owned(),
                                send_response: false,
                            });
                        }

                        // Legacy: defer behaviors and emitters for glTF entities
                        if !world_load.behaviors.is_empty() || !world_load.emitters.is_empty() {
                            params.pending_world.active = Some(WorldSetupData {
                                behaviors: world_load.behaviors.clone(),
                                emitters: world_load.emitters.clone(),
                                frames_waited: 0,
                            });
                        }

                        // Ambience doesn't reference entities — apply immediately.
                        if let Some(ambience) = world_load.ambience {
                            audio::handle_set_ambience(ambience, &mut params.audio_engine);
                        }

                        // Audio emitters from RON format (already extracted by world.rs)
                        for emitter_cmd in &world_load.emitters {
                            audio::handle_spawn_audio_emitter(
                                emitter_cmd.clone(),
                                &mut params.audio_engine,
                                &mut commands,
                                &mut params.registry,
                                &mut params.next_entity_id,
                            );
                        }

                        // Environment and camera don't depend on scene entities.
                        if let Some(env) = world_load.environment {
                            handle_set_environment(env, &mut commands);
                        }
                        if let Some(cam) = world_load.camera {
                            handle_set_camera(cam, &mut commands, &params.registry);
                        }

                        // Store avatar and tour configuration as resources.
                        params.avatar_config.active = world_load.avatar;
                        params.world_tours.tours = world_load.tours;

                        GenResponse::WorldLoaded {
                            path: world_load.world_path,
                            entities: world_load.entity_count,
                            behaviors: world_load.behavior_count,
                        }
                    }
                    Err(e) => GenResponse::Error {
                        message: format!("Failed to load world: {}", e),
                    },
                }
            }

            GenCommand::ClearScene {
                keep_camera,
                keep_lights,
            } => {
                // Snapshot all entities before clearing for undo
                let mut pre_snapshots = Vec::new();
                let all_names: Vec<(String, bevy::ecs::entity::Entity)> = params
                    .registry
                    .all_names()
                    .map(|(n, e)| (n.to_string(), e))
                    .collect();
                for (name, ent) in &all_names {
                    if name == "main_camera" && keep_camera {
                        continue;
                    }
                    if let Some(id) = params.registry.get_id(*ent) {
                        // Check if this is a light entity (skip if keep_lights)
                        if keep_lights
                            && (params.directional_lights.get(*ent).is_ok()
                                || params.point_lights.get(*ent).is_ok()
                                || params.spot_lights.get(*ent).is_ok())
                        {
                            continue;
                        }
                        pre_snapshots.push(snapshot_entity(name, *ent, id, &snap_queries!(params)));
                    }
                }

                let resp = handle_clear_scene(
                    keep_camera,
                    keep_lights,
                    &mut commands,
                    &mut params.registry,
                    &params.gen_entities,
                    &mut params.audio_engine,
                    &mut params.behavior_state,
                    &mut params.pending_world,
                );

                if let GenResponse::SceneCleared { .. } = &resp
                    && !pre_snapshots.is_empty()
                {
                    // Forward: delete all entities; Inverse: re-spawn them all
                    let forward_ops: Vec<wt::EditOp> = pre_snapshots
                        .iter()
                        .map(|we| wt::EditOp::delete(we.id))
                        .collect();
                    let inverse_ops: Vec<wt::EditOp> =
                        pre_snapshots.into_iter().map(wt::EditOp::spawn).collect();
                    params.undo_stack.history.push(
                        wt::EditOp::Batch { ops: forward_ops },
                        wt::EditOp::Batch { ops: inverse_ops },
                        None,
                    );
                }

                // Avatar and tours are world-level metadata (not individual entities),
                // so they are always reset — a new world will provide its own.
                params.avatar_config.active = None;
                params.world_tours.tours.clear();
                resp
            }

            GenCommand::Undo => handle_undo(
                &mut params.undo_stack,
                &mut commands,
                &mut params.meshes,
                &mut params.materials,
                &mut params.registry,
                &mut params.next_entity_id,
                &mut params.behavior_state,
            ),
            GenCommand::Redo => handle_redo(
                &mut params.undo_stack,
                &mut commands,
                &mut params.meshes,
                &mut params.materials,
                &mut params.registry,
                &mut params.next_entity_id,
                &mut params.behavior_state,
            ),
            GenCommand::UndoInfo => GenResponse::UndoInfoResult {
                undo_count: params.undo_stack.history.undo_count(),
                redo_count: params.undo_stack.history.redo_count(),
            },
        };

        // Mark entities dirty and record undo history.
        match &response {
            GenResponse::Spawned { name, .. } => {
                if let Some(bevy_ent) = params.registry.get_entity(name)
                    && let Some(id) = params.registry.get_id(bevy_ent)
                {
                    params.dirty_tracker.mark_dirty(id);
                    // Record undo: inverse of spawn is delete
                    let we = snapshot_entity(name, bevy_ent, id, &snap_queries!(params));
                    params.undo_stack.history.push(
                        wt::EditOp::spawn(we),
                        wt::EditOp::delete(id),
                        None,
                    );
                }
            }
            GenResponse::Modified { name }
            | GenResponse::BehaviorAdded { entity: name, .. }
            | GenResponse::BehaviorRemoved { entity: name, .. }
            | GenResponse::AudioEmitterSpawned { name } => {
                if let Some(bevy_ent) = params.registry.get_entity(name)
                    && let Some(id) = params.registry.get_id(bevy_ent)
                {
                    params.dirty_tracker.mark_dirty(id);
                }
            }
            GenResponse::EnvironmentSet | GenResponse::CameraSet => {
                params.dirty_tracker.world_meta_dirty = true;
            }
            GenResponse::WorldSaved { .. } => {
                params.dirty_tracker.clear();
            }
            GenResponse::WorldLoaded { .. } | GenResponse::SceneCleared { .. } => {
                params.dirty_tracker.clear();
                params.undo_stack.history = wt::EditHistory::new();
            }
            _ => {}
        }

        let _ = channel_res.channels.resp_tx.send(response);
    }
}

/// Process pending screenshots that need frame delays.
fn process_pending_screenshots(
    channel_res: ResMut<GenChannelRes>,
    mut pending: ResMut<PendingScreenshots>,
) {
    let mut completed = Vec::new();

    for (i, screenshot) in pending.queue.iter_mut().enumerate() {
        if screenshot.frames_remaining > 0 {
            screenshot.frames_remaining -= 1;
        } else {
            completed.push(i);
        }
    }

    // Process completed screenshots in reverse order to preserve indices
    for i in completed.into_iter().rev() {
        let screenshot = pending.queue.remove(i);

        // Determine output path
        let path = screenshot.path.unwrap_or_else(|| {
            let tmp = std::env::temp_dir().join(format!(
                "localgpt_gen_screenshot_{}.png",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ));
            tmp.to_string_lossy().into_owned()
        });

        // TODO: Actual Bevy screenshot capture requires camera entity access
        // and render-to-texture. For now, we create a placeholder and report
        // the path. Full implementation needs Bevy's Screenshot observer or
        // render target approach.
        //
        // In a full implementation:
        //   commands.entity(camera).trigger(Screenshot::to_disk(path));
        let response = GenResponse::Screenshot {
            image_path: path.clone(),
        };
        let _ = channel_res.channels.resp_tx.send(response);
    }
}

/// Process pending glTF loads that are waiting for the asset server.
fn process_pending_gltf_loads(
    channel_res: Res<GenChannelRes>,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingGltfLoads>,
    mut commands: Commands,
    mut registry: ResMut<NameRegistry>,
) {
    let mut completed = Vec::new();

    for (i, load) in pending.queue.iter().enumerate() {
        if asset_server.is_loaded_with_dependencies(&load.handle) {
            completed.push(i);
        }
    }

    // Process completed loads in reverse order to preserve indices
    for i in completed.into_iter().rev() {
        let load = pending.queue.remove(i);

        // Spawn the scene
        let entity = commands.spawn(SceneRoot(load.handle.clone())).id();

        // Register in the name registry
        registry.insert(load.name.clone(), entity);

        // Send response if this was a tool request (not a startup load)
        if load.send_response {
            let response = GenResponse::GltfLoaded {
                name: load.name,
                path: load.path,
            };
            let _ = channel_res.channels.resp_tx.send(response);
        }
    }
}

/// After a world's glTF scene spawns, Bevy's scene spawner creates child
/// entities with `Name` components (from glTF node names). This system
/// scans for those named entities, registers them in `NameRegistry`, and
/// applies the deferred behaviors and audio emitters.
fn process_pending_world_setup(
    mut pending: ResMut<PendingWorldSetup>,
    mut registry: ResMut<NameRegistry>,
    mut next_entity_id: ResMut<NextEntityId>,
    mut commands: Commands,
    transforms: Query<&Transform>,
    mut behavior_state: ResMut<BehaviorState>,
    mut audio_engine: ResMut<audio::AudioEngine>,
    named_entities: Query<(Entity, &Name), Without<GenEntity>>,
) {
    let Some(ref mut setup) = pending.active else {
        return;
    };

    setup.frames_waited += 1;

    // Collect all entity names we need to find.
    let needed: std::collections::HashSet<String> = setup
        .behaviors
        .iter()
        .map(|(name, _)| name.clone())
        .chain(setup.emitters.iter().filter_map(|e| e.entity.clone()))
        .collect();

    if needed.is_empty() {
        pending.active = None;
        return;
    }

    // Scan for newly spawned named entities from the glTF scene.
    // These won't have GenEntity yet (Bevy scene spawner adds Name but not our marker).
    let mut found_count = 0;
    for (entity, name) in named_entities.iter() {
        let name_str = name.as_str();
        if needed.contains(name_str) && registry.get_entity(name_str).is_none() {
            let wid = next_entity_id.alloc();
            registry.insert_with_id(name_str.to_string(), entity, wid);
            commands.entity(entity).insert(GenEntity {
                entity_type: GenEntityType::Mesh,
                world_id: wid,
            });
            found_count += 1;
        }
    }

    // If no entities found yet and we haven't waited too long, try again next frame.
    if found_count == 0 && setup.frames_waited < 120 {
        return;
    }

    // Apply behaviors to now-registered entities.
    for (entity_name, behavior_defs) in &setup.behaviors {
        if let Some(entity) = registry.get_entity(entity_name) {
            let base_transform = transforms.get(entity).copied().unwrap_or_default();
            let instances: Vec<behaviors::BehaviorInstance> = behavior_defs
                .iter()
                .map(|def| behaviors::BehaviorInstance {
                    id: behavior_state.next_id(),
                    def: def.clone(),
                    base_position: base_transform.translation,
                    base_scale: base_transform.scale,
                })
                .collect();
            commands.entity(entity).insert(EntityBehaviors {
                behaviors: instances,
            });
        }
    }

    // Apply audio emitters (which reference entities by name for spatial attachment).
    for emitter_cmd in &setup.emitters {
        audio::handle_spawn_audio_emitter(
            emitter_cmd.clone(),
            &mut audio_engine,
            &mut commands,
            &mut registry,
            &mut next_entity_id,
        );
    }

    pending.active = None;
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

fn handle_scene_info(
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    parametric_shapes: &Query<&ParametricShape>,
) -> GenResponse {
    let mut entities = Vec::new();

    for (name, entity) in registry.all_names() {
        let position = transforms
            .get(entity)
            .map(|t| t.translation.to_array())
            .unwrap_or_default();
        let scale = transforms
            .get(entity)
            .map(|t| t.scale.to_array())
            .unwrap_or([1.0, 1.0, 1.0]);
        let entity_type = gen_entities
            .get(entity)
            .map(|g| g.entity_type.as_str().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let shape = parametric_shapes
            .get(entity)
            .ok()
            .map(|p| p.shape.kind().to_string());

        let color = material_handles
            .get(entity)
            .ok()
            .and_then(|h| material_assets.get(&h.0))
            .map(|mat| {
                let c = mat.base_color.to_srgba();
                [c.red, c.green, c.blue, c.alpha]
            });

        entities.push(EntitySummary {
            name: name.to_string(),
            entity_type,
            shape,
            position,
            scale,
            color,
        });
    }

    GenResponse::SceneInfo(SceneInfoData {
        entity_count: entities.len(),
        entities,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_entity_info(
    name: &str,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    names_query: &Query<&Name>,
    children_query: &Query<&Children>,
    parent_query: &Query<&ChildOf>,
    visibility_query: &Query<&Visibility>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    behaviors_query: &Query<&mut EntityBehaviors>,
    parametric_shapes: &Query<&ParametricShape>,
    directional_lights: &Query<&DirectionalLight>,
    point_lights: &Query<&PointLight>,
    spot_lights: &Query<&SpotLight>,
) -> GenResponse {
    let Some(entity) = registry.get_entity(name) else {
        return GenResponse::Error {
            message: format!("Entity '{}' not found", name),
        };
    };

    let transform = transforms.get(entity).copied().unwrap_or_default();
    let euler = transform.rotation.to_euler(EulerRot::XYZ);

    let entity_type = gen_entities
        .get(entity)
        .map(|g| g.entity_type.as_str().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let visible = visibility_query
        .get(entity)
        .map(|v| *v != Visibility::Hidden)
        .unwrap_or(true);

    let shape_name = parametric_shapes
        .get(entity)
        .ok()
        .map(|p| p.shape.kind().to_string());

    let (color, metallic, roughness, emissive) = material_handles
        .get(entity)
        .ok()
        .and_then(|h| material_assets.get(&h.0))
        .map(|mat| {
            let c = mat.base_color.to_srgba();
            let e = mat.emissive;
            let emissive_arr = [e.red, e.green, e.blue, e.alpha];
            let has_emissive = emissive_arr.iter().any(|&v| v > 0.0);
            (
                Some([c.red, c.green, c.blue, c.alpha]),
                Some(mat.metallic),
                Some(mat.perceptual_roughness),
                if has_emissive {
                    Some(emissive_arr)
                } else {
                    None
                },
            )
        })
        .unwrap_or((None, None, None, None));

    let light_info = if let Ok(dl) = directional_lights.get(entity) {
        let c = dl.color.to_srgba();
        Some(LightInfoData {
            light_type: "directional".to_string(),
            color: [c.red, c.green, c.blue, c.alpha],
            intensity: dl.illuminance,
            shadows: dl.shadows_enabled,
        })
    } else if let Ok(pl) = point_lights.get(entity) {
        let c = pl.color.to_srgba();
        Some(LightInfoData {
            light_type: "point".to_string(),
            color: [c.red, c.green, c.blue, c.alpha],
            intensity: pl.intensity,
            shadows: pl.shadows_enabled,
        })
    } else if let Ok(sl) = spot_lights.get(entity) {
        let c = sl.color.to_srgba();
        Some(LightInfoData {
            light_type: "spot".to_string(),
            color: [c.red, c.green, c.blue, c.alpha],
            intensity: sl.intensity,
            shadows: sl.shadows_enabled,
        })
    } else {
        None
    };

    let children: Vec<String> = children_query
        .get(entity)
        .map(|ch| {
            ch.iter()
                .filter_map(|c| {
                    registry
                        .get_name(c)
                        .map(|s| s.to_string())
                        .or_else(|| names_query.get(c).ok().map(|n| n.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    let parent = parent_query
        .get(entity)
        .ok()
        .and_then(|p| registry.get_name(p.parent()).map(|s| s.to_string()));

    let behavior_summaries: Vec<BehaviorSummary> = behaviors_query
        .get(entity)
        .ok()
        .map(|b| {
            b.behaviors
                .iter()
                .map(behaviors::behavior_to_summary)
                .collect()
        })
        .unwrap_or_default();

    GenResponse::EntityInfo(EntityInfoData {
        name: name.to_string(),
        entity_id: registry
            .get_id(entity)
            .map(|id| id.0)
            .unwrap_or(entity.to_bits()),
        entity_type,
        shape: shape_name,
        position: transform.translation.to_array(),
        rotation_degrees: [
            euler.0.to_degrees(),
            euler.1.to_degrees(),
            euler.2.to_degrees(),
        ],
        scale: transform.scale.to_array(),
        color,
        metallic,
        roughness,
        emissive,
        visible,
        light: light_info,
        children,
        parent,
        behaviors: behavior_summaries,
    })
}

fn handle_spawn_primitive(
    cmd: SpawnPrimitiveCmd,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<NameRegistry>,
    next_id: &mut ResMut<NextEntityId>,
) -> GenResponse {
    if registry.contains_name(&cmd.name) {
        return GenResponse::Error {
            message: format!("Entity '{}' already exists", cmd.name),
        };
    }

    let mesh = match cmd.shape {
        PrimitiveShape::Cuboid => {
            let x = cmd.dimensions.get("x").copied().unwrap_or(1.0);
            let y = cmd.dimensions.get("y").copied().unwrap_or(1.0);
            let z = cmd.dimensions.get("z").copied().unwrap_or(1.0);
            meshes.add(Cuboid::new(x, y, z))
        }
        PrimitiveShape::Sphere => {
            let radius = cmd.dimensions.get("radius").copied().unwrap_or(0.5);
            meshes.add(Sphere::new(radius).mesh().uv(32, 18))
        }
        PrimitiveShape::Cylinder => {
            let radius = cmd.dimensions.get("radius").copied().unwrap_or(0.5);
            let height = cmd.dimensions.get("height").copied().unwrap_or(1.0);
            meshes.add(Cylinder::new(radius, height))
        }
        PrimitiveShape::Cone => {
            let radius = cmd.dimensions.get("radius").copied().unwrap_or(0.5);
            let height = cmd.dimensions.get("height").copied().unwrap_or(1.0);
            meshes.add(Cone { radius, height })
        }
        PrimitiveShape::Capsule => {
            let radius = cmd.dimensions.get("radius").copied().unwrap_or(0.5);
            let half_length = cmd.dimensions.get("half_length").copied().unwrap_or(0.5);
            meshes.add(Capsule3d::new(radius, half_length * 2.0))
        }
        PrimitiveShape::Torus => {
            let major = cmd.dimensions.get("major_radius").copied().unwrap_or(1.0);
            let minor = cmd.dimensions.get("minor_radius").copied().unwrap_or(0.25);
            meshes.add(Torus::new(minor, major))
        }
        PrimitiveShape::Plane => {
            let x = cmd.dimensions.get("x").copied().unwrap_or(1.0);
            let z = cmd.dimensions.get("z").copied().unwrap_or(1.0);
            meshes.add(Plane3d::new(Vec3::Y, Vec2::new(x / 2.0, z / 2.0)))
        }
    };

    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(cmd.color[0], cmd.color[1], cmd.color[2], cmd.color[3]),
        metallic: cmd.metallic,
        perceptual_roughness: cmd.roughness,
        emissive: bevy::color::LinearRgba::new(
            cmd.emissive[0],
            cmd.emissive[1],
            cmd.emissive[2],
            cmd.emissive[3],
        ),
        ..default()
    });

    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        cmd.rotation_degrees[0].to_radians(),
        cmd.rotation_degrees[1].to_radians(),
        cmd.rotation_degrees[2].to_radians(),
    );

    let transform = Transform {
        translation: Vec3::from_array(cmd.position),
        rotation,
        scale: Vec3::from_array(cmd.scale),
    };

    // Store the parametric shape so it survives save/load cycles.
    let parametric = ParametricShape {
        shape: compat::shape_from_primitive(cmd.shape, &cmd.dimensions),
    };

    let wid = next_id.alloc();
    let entity = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            Name::new(cmd.name.clone()),
            GenEntity {
                entity_type: GenEntityType::Primitive,
                world_id: wid,
            },
            parametric,
        ))
        .id();

    // Handle parenting
    if let Some(ref parent_name) = cmd.parent
        && let Some(parent_entity) = registry.get_entity(parent_name)
    {
        commands.entity(entity).set_parent_in_place(parent_entity);
    }

    registry.insert_with_id(cmd.name.clone(), entity, wid);

    GenResponse::Spawned {
        name: cmd.name,
        entity_id: wid.0,
    }
}

fn handle_modify_entity(
    cmd: ModifyEntityCmd,
    commands: &mut Commands,
    registry: &NameRegistry,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    transforms: &Query<&Transform>,
) -> GenResponse {
    let Some(entity) = registry.get_entity(&cmd.name) else {
        return GenResponse::Error {
            message: format!("Entity '{}' not found", cmd.name),
        };
    };

    let mut entity_commands = commands.entity(entity);

    // Update transform
    if cmd.position.is_some() || cmd.rotation_degrees.is_some() || cmd.scale.is_some() {
        let current = transforms.get(entity).copied().unwrap_or_default();
        let new_transform = Transform {
            translation: cmd
                .position
                .map(Vec3::from_array)
                .unwrap_or(current.translation),
            rotation: cmd
                .rotation_degrees
                .map(|r| {
                    Quat::from_euler(
                        EulerRot::XYZ,
                        r[0].to_radians(),
                        r[1].to_radians(),
                        r[2].to_radians(),
                    )
                })
                .unwrap_or(current.rotation),
            scale: cmd.scale.map(Vec3::from_array).unwrap_or(current.scale),
        };
        entity_commands.insert(new_transform);
    }

    // Update material if any material properties changed
    if cmd.color.is_some()
        || cmd.metallic.is_some()
        || cmd.roughness.is_some()
        || cmd.emissive.is_some()
    {
        // Get current material properties as defaults
        let current_mat = material_handles
            .get(entity)
            .ok()
            .and_then(|h| materials.get(&h.0))
            .cloned();

        let base = current_mat.unwrap_or_default();

        let new_material = materials.add(StandardMaterial {
            base_color: cmd
                .color
                .map(|c| Color::srgba(c[0], c[1], c[2], c[3]))
                .unwrap_or(base.base_color),
            metallic: cmd.metallic.unwrap_or(base.metallic),
            perceptual_roughness: cmd.roughness.unwrap_or(base.perceptual_roughness),
            emissive: cmd
                .emissive
                .map(|e| bevy::color::LinearRgba::new(e[0], e[1], e[2], e[3]))
                .unwrap_or(base.emissive),
            ..base
        });
        entity_commands.insert(MeshMaterial3d(new_material));
    }

    // Update visibility
    if let Some(visible) = cmd.visible {
        entity_commands.insert(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }

    // Update parent
    if let Some(parent_opt) = cmd.parent {
        match parent_opt {
            Some(parent_name) => {
                if let Some(parent_entity) = registry.get_entity(&parent_name) {
                    commands.entity(entity).set_parent_in_place(parent_entity);
                }
            }
            None => {
                commands.entity(entity).remove_parent_in_place();
            }
        }
    }

    GenResponse::Modified { name: cmd.name }
}

fn handle_delete_entity(
    name: &str,
    commands: &mut Commands,
    registry: &mut ResMut<NameRegistry>,
) -> GenResponse {
    let Some(entity) = registry.remove_by_name(name) else {
        return GenResponse::Error {
            message: format!("Entity '{}' not found", name),
        };
    };

    commands.entity(entity).despawn();

    GenResponse::Deleted {
        name: name.to_string(),
    }
}

fn handle_set_camera(
    cmd: CameraCmd,
    commands: &mut Commands,
    registry: &NameRegistry,
) -> GenResponse {
    let Some(camera_entity) = registry.get_entity("main_camera") else {
        return GenResponse::Error {
            message: "main_camera not found in registry".to_string(),
        };
    };

    let transform = Transform::from_translation(Vec3::from_array(cmd.position))
        .looking_at(Vec3::from_array(cmd.look_at), Vec3::Y);

    commands.entity(camera_entity).insert(transform);

    // Update projection FOV
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: cmd.fov_degrees.to_radians(),
        ..default()
    });
    commands.entity(camera_entity).insert(projection);

    GenResponse::CameraSet
}

fn handle_set_light(
    cmd: SetLightCmd,
    commands: &mut Commands,
    registry: &mut ResMut<NameRegistry>,
    next_id: &mut ResMut<NextEntityId>,
) -> GenResponse {
    let color = Color::srgba(cmd.color[0], cmd.color[1], cmd.color[2], cmd.color[3]);

    // If light already exists, update it
    if let Some(entity) = registry.get_entity(&cmd.name) {
        commands.entity(entity).despawn();
        registry.remove_by_name(&cmd.name);
    }

    let wid = next_id.alloc();
    let entity = match cmd.light_type {
        LightType::Directional => {
            let dir = cmd.direction.unwrap_or([0.0, -1.0, -0.5]);
            let transform = Transform::from_translation(Vec3::new(0.0, 10.0, 0.0))
                .looking_at(Vec3::new(0.0, 10.0, 0.0) + Vec3::from_array(dir), Vec3::Y);
            commands
                .spawn((
                    DirectionalLight {
                        illuminance: cmd.intensity,
                        shadows_enabled: cmd.shadows,
                        color,
                        ..default()
                    },
                    transform,
                    Name::new(cmd.name.clone()),
                    GenEntity {
                        entity_type: GenEntityType::Light,
                        world_id: wid,
                    },
                ))
                .id()
        }
        LightType::Point => {
            let pos = cmd.position.unwrap_or([0.0, 5.0, 0.0]);
            commands
                .spawn((
                    PointLight {
                        intensity: cmd.intensity,
                        shadows_enabled: cmd.shadows,
                        color,
                        ..default()
                    },
                    Transform::from_translation(Vec3::from_array(pos)),
                    Name::new(cmd.name.clone()),
                    GenEntity {
                        entity_type: GenEntityType::Light,
                        world_id: wid,
                    },
                ))
                .id()
        }
        LightType::Spot => {
            let pos = cmd.position.unwrap_or([0.0, 5.0, 0.0]);
            let dir = cmd.direction.unwrap_or([0.0, -1.0, 0.0]);
            let transform = Transform::from_translation(Vec3::from_array(pos))
                .looking_at(Vec3::from_array(pos) + Vec3::from_array(dir), Vec3::Y);
            commands
                .spawn((
                    SpotLight {
                        intensity: cmd.intensity,
                        shadows_enabled: cmd.shadows,
                        color,
                        ..default()
                    },
                    transform,
                    Name::new(cmd.name.clone()),
                    GenEntity {
                        entity_type: GenEntityType::Light,
                        world_id: wid,
                    },
                ))
                .id()
        }
    };

    registry.insert_with_id(cmd.name.clone(), entity, wid);

    GenResponse::LightSet { name: cmd.name }
}

fn handle_set_environment(cmd: EnvironmentCmd, commands: &mut Commands) -> GenResponse {
    if let Some(color) = cmd.background_color {
        commands.insert_resource(ClearColor(Color::srgba(
            color[0], color[1], color[2], color[3],
        )));
    }

    if let Some(intensity) = cmd.ambient_light {
        let color = cmd
            .ambient_color
            .map(|c| Color::srgba(c[0], c[1], c[2], c[3]))
            .unwrap_or(Color::WHITE);
        commands.insert_resource(GlobalAmbientLight {
            color,
            brightness: intensity,
            affects_lightmapped_meshes: true,
        });
    }

    GenResponse::EnvironmentSet
}

fn handle_spawn_mesh(
    cmd: RawMeshCmd,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<NameRegistry>,
    next_id: &mut ResMut<NextEntityId>,
) -> GenResponse {
    if registry.contains_name(&cmd.name) {
        return GenResponse::Error {
            message: format!("Entity '{}' already exists", cmd.name),
        };
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Positions
    let positions: Vec<[f32; 3]> = cmd.vertices.clone();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    // Indices
    mesh.insert_indices(Indices::U32(cmd.indices));

    // Normals — use provided or compute flat normals
    if let Some(normals) = cmd.normals {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    } else {
        mesh.compute_flat_normals();
    }

    // UVs
    if let Some(uvs) = cmd.uvs {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }

    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(cmd.color[0], cmd.color[1], cmd.color[2], cmd.color[3]),
        metallic: cmd.metallic,
        perceptual_roughness: cmd.roughness,
        ..default()
    });

    let wid = next_id.alloc();
    let entity = commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::from_array(cmd.position)),
            Name::new(cmd.name.clone()),
            GenEntity {
                entity_type: GenEntityType::Mesh,
                world_id: wid,
            },
        ))
        .id();

    registry.insert_with_id(cmd.name.clone(), entity, wid);

    GenResponse::Spawned {
        name: cmd.name,
        entity_id: wid.0,
    }
}

// ---------------------------------------------------------------------------
// Spawn world entities from RON WorldManifest
// ---------------------------------------------------------------------------

/// Spawn all entities from a loaded RON `WorldManifest`.
///
/// Creates Bevy ECS entities from `WorldEntity` definitions, preserving:
/// - Parametric shapes (via `ParametricShape` component)
/// - Materials (via `StandardMaterial`)
/// - Lights (directional / point / spot)
/// - Behaviors (attached after spawn)
/// - Parent-child relationships (resolved after all entities are spawned)
fn spawn_world_entities(
    world_entities: &[wt::WorldEntity],
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<NameRegistry>,
    next_entity_id: &mut ResMut<NextEntityId>,
    behavior_state: &mut BehaviorState,
) {
    // First pass: collect world_id → entity name for parent resolution
    let id_to_name: std::collections::HashMap<u64, String> = world_entities
        .iter()
        .map(|we| (we.id.0, we.name.as_str().to_string()))
        .collect();

    // Deferred parent assignments (child_name, parent_name)
    let mut parent_assignments: Vec<(String, String)> = Vec::new();

    for we in world_entities {
        let name = we.name.as_str().to_string();

        // Skip if already exists
        if registry.contains_name(&name) {
            tracing::warn!("Entity '{}' already exists, skipping", name);
            continue;
        }

        let transform = Transform {
            translation: Vec3::from_array(we.transform.position),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                we.transform.rotation_degrees[0].to_radians(),
                we.transform.rotation_degrees[1].to_radians(),
                we.transform.rotation_degrees[2].to_radians(),
            ),
            scale: Vec3::from_array(we.transform.scale),
        };

        let world_id = wt::EntityId(we.id.0);
        next_entity_id.ensure_at_least(we.id.0 + 1);

        // Determine what kind of entity to spawn based on component slots
        let bevy_entity = if let Some(ref shape) = we.shape {
            // Entity with a parametric shape → spawn mesh
            let mesh_handle = shape_to_mesh(shape, meshes);
            let mat = we.material.as_ref().cloned().unwrap_or_default();
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgba(mat.color[0], mat.color[1], mat.color[2], mat.color[3]),
                metallic: mat.metallic,
                perceptual_roughness: mat.roughness,
                emissive: bevy::color::LinearRgba::new(
                    mat.emissive[0],
                    mat.emissive[1],
                    mat.emissive[2],
                    mat.emissive[3],
                ),
                ..default()
            });

            let parametric = ParametricShape {
                shape: shape.clone(),
            };

            let mut entity_cmd = commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                transform,
                Name::new(name.clone()),
                GenEntity {
                    entity_type: GenEntityType::Primitive,
                    world_id,
                },
                parametric,
            ));

            if !we.transform.visible {
                entity_cmd.insert(Visibility::Hidden);
            }

            // If entity also has a light, add it as a child or additional component
            if let Some(ref light) = we.light {
                insert_light_component(&mut entity_cmd, light);
            }

            entity_cmd.id()
        } else if let Some(ref light) = we.light {
            // Light-only entity (no shape)
            spawn_light_entity(light, &name, transform, world_id, commands)
        } else {
            // Empty entity (group, audio-only, etc.)
            commands
                .spawn((
                    transform,
                    Name::new(name.clone()),
                    GenEntity {
                        entity_type: GenEntityType::Group,
                        world_id,
                    },
                ))
                .id()
        };

        registry.insert_with_id(name.clone(), bevy_entity, world_id);

        // Attach behaviors
        if !we.behaviors.is_empty() {
            let mut behavior_instances: Vec<behaviors::BehaviorInstance> = Vec::new();
            for wt_behavior in &we.behaviors {
                let cmd_behavior: BehaviorDef = wt_behavior.into();
                behavior_instances.push(behaviors::BehaviorInstance {
                    id: behavior_state.next_id(),
                    def: cmd_behavior,
                    base_position: transform.translation,
                    base_scale: transform.scale,
                });
            }
            commands
                .entity(bevy_entity)
                .insert(behaviors::EntityBehaviors {
                    behaviors: behavior_instances,
                });
        }

        // Record deferred parent assignment
        if let Some(ref parent_id) = we.parent
            && let Some(parent_name) = id_to_name.get(&parent_id.0)
        {
            parent_assignments.push((name, parent_name.clone()));
        }
    }

    // Second pass: resolve parent-child relationships
    for (child_name, parent_name) in &parent_assignments {
        if let (Some(child), Some(parent)) = (
            registry.get_entity(child_name),
            registry.get_entity(parent_name),
        ) {
            commands.entity(child).set_parent_in_place(parent);
        }
    }
}

/// Convert a `wt::Shape` to a Bevy `Mesh` handle.
fn shape_to_mesh(shape: &wt::Shape, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    match shape {
        wt::Shape::Cuboid { x, y, z } => meshes.add(Cuboid::new(*x, *y, *z)),
        wt::Shape::Sphere { radius } => meshes.add(Sphere::new(*radius).mesh().uv(32, 18)),
        wt::Shape::Cylinder { radius, height } => meshes.add(Cylinder::new(*radius, *height)),
        wt::Shape::Cone { radius, height } => meshes.add(Cone {
            radius: *radius,
            height: *height,
        }),
        wt::Shape::Capsule {
            radius,
            half_length,
        } => meshes.add(Capsule3d::new(*radius, *half_length * 2.0)),
        wt::Shape::Torus {
            major_radius,
            minor_radius,
        } => meshes.add(Torus::new(*minor_radius, *major_radius)),
        wt::Shape::Plane { x, z } => {
            meshes.add(Plane3d::new(Vec3::Y, Vec2::new(*x / 2.0, *z / 2.0)))
        }
    }
}

/// Insert a light component onto an existing entity command builder.
fn insert_light_component(
    entity_cmd: &mut bevy::ecs::system::EntityCommands,
    light: &wt::LightDef,
) {
    let color = Color::srgba(
        light.color[0],
        light.color[1],
        light.color[2],
        light.color[3],
    );
    match light.light_type {
        wt::LightType::Directional => {
            entity_cmd.insert(DirectionalLight {
                illuminance: light.intensity,
                shadows_enabled: light.shadows,
                color,
                ..default()
            });
        }
        wt::LightType::Point => {
            entity_cmd.insert(PointLight {
                intensity: light.intensity,
                shadows_enabled: light.shadows,
                color,
                ..default()
            });
        }
        wt::LightType::Spot => {
            entity_cmd.insert(SpotLight {
                intensity: light.intensity,
                shadows_enabled: light.shadows,
                color,
                ..default()
            });
        }
    }
}

/// Spawn a standalone light entity (no shape).
fn spawn_light_entity(
    light: &wt::LightDef,
    name: &str,
    transform: Transform,
    world_id: wt::EntityId,
    commands: &mut Commands,
) -> bevy::ecs::entity::Entity {
    let color = Color::srgba(
        light.color[0],
        light.color[1],
        light.color[2],
        light.color[3],
    );
    match light.light_type {
        wt::LightType::Directional => {
            let dir = light.direction.unwrap_or([0.0, -1.0, -0.5]);
            let light_transform = Transform::from_translation(transform.translation)
                .looking_at(transform.translation + Vec3::from_array(dir), Vec3::Y);
            commands
                .spawn((
                    DirectionalLight {
                        illuminance: light.intensity,
                        shadows_enabled: light.shadows,
                        color,
                        ..default()
                    },
                    light_transform,
                    Name::new(name.to_string()),
                    GenEntity {
                        entity_type: GenEntityType::Light,
                        world_id,
                    },
                ))
                .id()
        }
        wt::LightType::Point => commands
            .spawn((
                PointLight {
                    intensity: light.intensity,
                    shadows_enabled: light.shadows,
                    color,
                    ..default()
                },
                transform,
                Name::new(name.to_string()),
                GenEntity {
                    entity_type: GenEntityType::Light,
                    world_id,
                },
            ))
            .id(),
        wt::LightType::Spot => {
            let dir = light.direction.unwrap_or([0.0, -1.0, 0.0]);
            let light_transform = Transform::from_translation(transform.translation)
                .looking_at(transform.translation + Vec3::from_array(dir), Vec3::Y);
            commands
                .spawn((
                    SpotLight {
                        intensity: light.intensity,
                        shadows_enabled: light.shadows,
                        color,
                        ..default()
                    },
                    light_transform,
                    Name::new(name.to_string()),
                    GenEntity {
                        entity_type: GenEntityType::Light,
                        world_id,
                    },
                ))
                .id()
        }
    }
}

// ---------------------------------------------------------------------------
// Undo/Redo support
// ---------------------------------------------------------------------------

/// Borrows all the queries needed to snapshot entity state.
/// Avoids passing 12+ individual query parameters to `snapshot_entity`.
struct SnapshotQueries<'a, 'w, 's> {
    transforms: &'a Query<'w, 's, &'static Transform>,
    parametric_shapes: &'a Query<'w, 's, &'static ParametricShape>,
    material_handles: &'a Query<'w, 's, &'static MeshMaterial3d<StandardMaterial>>,
    materials: &'a Assets<StandardMaterial>,
    visibility_query: &'a Query<'w, 's, &'static Visibility>,
    directional_lights: &'a Query<'w, 's, &'static DirectionalLight>,
    point_lights: &'a Query<'w, 's, &'static PointLight>,
    spot_lights: &'a Query<'w, 's, &'static SpotLight>,
    behaviors_query: &'a Query<'w, 's, &'static mut EntityBehaviors>,
    audio_emitters: &'a Query<'w, 's, &'static audio::AudioEmitter>,
    parent_query: &'a Query<'w, 's, &'static ChildOf>,
    registry: &'a NameRegistry,
}

/// Capture the current ECS state of an entity as a `wt::WorldEntity`.
///
/// Used to record the entity state before/after modifications for undo history.
fn snapshot_entity(
    name: &str,
    entity: bevy::ecs::entity::Entity,
    id: wt::EntityId,
    sq: &SnapshotQueries,
) -> wt::WorldEntity {
    let mut we = wt::WorldEntity::new(id.0, name);

    if let Ok(transform) = sq.transforms.get(entity) {
        let euler = transform.rotation.to_euler(EulerRot::XYZ);
        we.transform = wt::WorldTransform {
            position: transform.translation.to_array(),
            rotation_degrees: [
                euler.0.to_degrees(),
                euler.1.to_degrees(),
                euler.2.to_degrees(),
            ],
            scale: transform.scale.to_array(),
            visible: sq
                .visibility_query
                .get(entity)
                .map(|v| *v != Visibility::Hidden)
                .unwrap_or(true),
        };
    }

    if let Ok(param) = sq.parametric_shapes.get(entity) {
        we.shape = Some(param.shape.clone());
    }

    if let Ok(mat_handle) = sq.material_handles.get(entity)
        && let Some(mat) = sq.materials.get(&mat_handle.0)
    {
        let c = mat.base_color.to_srgba();
        let e = mat.emissive;
        we.material = Some(wt::MaterialDef {
            color: [c.red, c.green, c.blue, c.alpha],
            metallic: mat.metallic,
            roughness: mat.perceptual_roughness,
            emissive: [e.red, e.green, e.blue, e.alpha],
        });
    }

    // Light
    if let Ok(dl) = sq.directional_lights.get(entity) {
        let c = dl.color.to_srgba();
        let dir = sq
            .transforms
            .get(entity)
            .ok()
            .map(|t| t.forward().as_vec3().to_array());
        we.light = Some(wt::LightDef {
            light_type: wt::LightType::Directional,
            color: [c.red, c.green, c.blue, c.alpha],
            intensity: dl.illuminance,
            direction: dir,
            shadows: dl.shadows_enabled,
        });
    } else if let Ok(pl) = sq.point_lights.get(entity) {
        let c = pl.color.to_srgba();
        we.light = Some(wt::LightDef {
            light_type: wt::LightType::Point,
            color: [c.red, c.green, c.blue, c.alpha],
            intensity: pl.intensity,
            direction: None,
            shadows: pl.shadows_enabled,
        });
    } else if let Ok(sl) = sq.spot_lights.get(entity) {
        let c = sl.color.to_srgba();
        let dir = sq
            .transforms
            .get(entity)
            .ok()
            .map(|t| t.forward().as_vec3().to_array());
        we.light = Some(wt::LightDef {
            light_type: wt::LightType::Spot,
            color: [c.red, c.green, c.blue, c.alpha],
            intensity: sl.intensity,
            direction: dir,
            shadows: sl.shadows_enabled,
        });
    }

    // Behaviors
    if let Ok(eb) = sq.behaviors_query.get(entity) {
        we.behaviors = eb
            .behaviors
            .iter()
            .map(|bi| wt::BehaviorDef::from(&bi.def))
            .collect();
    }

    // Audio
    if let Ok(ae) = sq.audio_emitters.get(entity) {
        we.audio = Some(wt::AudioDef {
            kind: wt::AudioKind::Sfx,
            source: wt::AudioSource::from(&ae.sound),
            volume: ae.volume,
            radius: Some(ae.radius),
            rolloff: wt::Rolloff::InverseSquare,
        });
    }

    // Parent
    if let Ok(child_of) = sq.parent_query.get(entity) {
        if let Some(parent_id) = sq.registry.get_id(child_of.0) {
            we.parent = Some(parent_id);
        }
    }

    we
}

/// Construct the expected post-modify state by applying a `ModifyEntityCmd`
/// to a pre-modify snapshot.  Used for undo/redo of modify operations.
fn apply_modify_to_snapshot(we: &mut wt::WorldEntity, cmd: &ModifyEntityCmd) {
    if let Some(pos) = cmd.position {
        we.transform.position = pos;
    }
    if let Some(rot) = cmd.rotation_degrees {
        we.transform.rotation_degrees = rot;
    }
    if let Some(scale) = cmd.scale {
        we.transform.scale = scale;
    }
    if let Some(visible) = cmd.visible {
        we.transform.visible = visible;
    }
    if cmd.color.is_some()
        || cmd.metallic.is_some()
        || cmd.roughness.is_some()
        || cmd.emissive.is_some()
    {
        let mut mat = we.material.clone().unwrap_or_default();
        if let Some(color) = cmd.color {
            mat.color = color;
        }
        if let Some(metallic) = cmd.metallic {
            mat.metallic = metallic;
        }
        if let Some(roughness) = cmd.roughness {
            mat.roughness = roughness;
        }
        if let Some(emissive) = cmd.emissive {
            mat.emissive = emissive;
        }
        we.material = Some(mat);
    }
}

/// Apply a single `EditOp` to the scene. Returns a human-readable description.
fn apply_edit_op(
    op: &wt::EditOp,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<NameRegistry>,
    next_entity_id: &mut ResMut<NextEntityId>,
    behavior_state: &mut BehaviorState,
) -> String {
    match op {
        wt::EditOp::DeleteEntity { id } => {
            if let Some(entity) = registry.get_entity_by_id(id) {
                let name = registry.get_name(entity).unwrap_or("unknown").to_string();
                registry.remove_by_entity(entity);
                commands.entity(entity).despawn();
                format!("deleted '{}'", name)
            } else {
                format!("entity id {} not found", id.0)
            }
        }
        wt::EditOp::SpawnEntity { entity } => {
            let name = entity.name.as_str().to_string();
            spawn_world_entities(
                std::slice::from_ref(entity),
                commands,
                meshes,
                materials,
                registry,
                next_entity_id,
                behavior_state,
            );
            format!("re-spawned '{}'", name)
        }
        wt::EditOp::ModifyEntity { id, patch } => {
            if let Some(entity) = registry.get_entity_by_id(id) {
                let name = registry.get_name(entity).unwrap_or("unknown").to_string();
                // Build a minimal WorldEntity from what we know, apply the patch,
                // then delete-and-respawn to apply all changes atomically.
                let mut we = wt::WorldEntity::new(id.0, &name);
                patch.apply(&mut we);
                // Delete old
                registry.remove_by_entity(entity);
                commands.entity(entity).despawn();
                // Spawn patched version
                spawn_world_entities(
                    std::slice::from_ref(&we),
                    commands,
                    meshes,
                    materials,
                    registry,
                    next_entity_id,
                    behavior_state,
                );
                format!("modified '{}'", name)
            } else {
                format!("entity id {} not found for modify", id.0)
            }
        }
        wt::EditOp::SetEnvironment { env } => {
            if let Some(color) = env.background_color {
                commands.insert_resource(ClearColor(Color::srgba(
                    color[0], color[1], color[2], color[3],
                )));
            }
            if let Some(intensity) = env.ambient_intensity {
                let color = env
                    .ambient_color
                    .map(|c| Color::srgba(c[0], c[1], c[2], c[3]))
                    .unwrap_or(Color::WHITE);
                commands.insert_resource(GlobalAmbientLight {
                    color,
                    brightness: intensity,
                    affects_lightmapped_meshes: true,
                });
            }
            "restored environment".to_string()
        }
        wt::EditOp::SetCamera { camera } => {
            if let Some(cam_entity) = registry.get_entity("main_camera") {
                let transform = Transform::from_translation(Vec3::from_array(camera.position))
                    .looking_at(Vec3::from_array(camera.look_at), Vec3::Y);
                commands.entity(cam_entity).insert(transform);
                commands.entity(cam_entity).insert(Projection::Perspective(
                    PerspectiveProjection {
                        fov: camera.fov_degrees.to_radians(),
                        ..default()
                    },
                ));
                "restored camera".to_string()
            } else {
                "main_camera not found".to_string()
            }
        }
        wt::EditOp::Batch { ops } => {
            let descriptions: Vec<String> = ops
                .iter()
                .map(|o| {
                    apply_edit_op(
                        o,
                        commands,
                        meshes,
                        materials,
                        registry,
                        next_entity_id,
                        behavior_state,
                    )
                })
                .collect();
            descriptions.join("; ")
        }
    }
}

fn handle_undo(
    undo_stack: &mut ResMut<UndoStack>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<NameRegistry>,
    next_entity_id: &mut ResMut<NextEntityId>,
    behavior_state: &mut BehaviorState,
) -> GenResponse {
    let op = match undo_stack.history.undo() {
        Some(op) => op.clone(),
        None => return GenResponse::NothingToUndo,
    };

    let description = apply_edit_op(
        &op,
        commands,
        meshes,
        materials,
        registry,
        next_entity_id,
        behavior_state,
    );
    GenResponse::Undone { description }
}

fn handle_redo(
    undo_stack: &mut ResMut<UndoStack>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    registry: &mut ResMut<NameRegistry>,
    next_entity_id: &mut ResMut<NextEntityId>,
    behavior_state: &mut BehaviorState,
) -> GenResponse {
    let op = match undo_stack.history.redo() {
        Some(op) => op.clone(),
        None => return GenResponse::NothingToRedo,
    };

    let description = apply_edit_op(
        &op,
        commands,
        meshes,
        materials,
        registry,
        next_entity_id,
        behavior_state,
    );
    GenResponse::Redone { description }
}

// ---------------------------------------------------------------------------
// Scene management
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn handle_clear_scene(
    keep_camera: bool,
    keep_lights: bool,
    commands: &mut Commands,
    registry: &mut NameRegistry,
    gen_entities: &Query<&GenEntity>,
    audio_engine: &mut audio::AudioEngine,
    behavior_state: &mut BehaviorState,
    pending_world: &mut PendingWorldSetup,
) -> GenResponse {
    let mut removed = 0;
    let all_names: Vec<(String, bevy::ecs::entity::Entity)> = registry
        .all_names()
        .map(|(n, e)| (n.to_string(), e))
        .collect();

    for (name, entity) in &all_names {
        // Optionally keep cameras and lights
        if let Ok(gen_ent) = gen_entities.get(*entity) {
            if keep_camera && gen_ent.entity_type == GenEntityType::Camera {
                continue;
            }
            if keep_lights && gen_ent.entity_type == GenEntityType::Light {
                continue;
            }
        }

        commands.entity(*entity).despawn();
        registry.remove_by_name(name);
        removed += 1;
    }

    // Stop all audio
    audio_engine.stop_all();

    // Reset behavior state
    behavior_state.elapsed = 0.0;
    behavior_state.paused = false;

    // Clear any pending world setup
    pending_world.active = None;

    GenResponse::SceneCleared {
        removed_count: removed,
    }
}

// ---------------------------------------------------------------------------
// glTF/GLB export
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn handle_export_gltf(
    path: Option<&str>,
    workspace: &GenWorkspace,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    gen_entities: &Query<&GenEntity>,
    parent_query: &Query<&ChildOf>,
    material_handles: &Query<&MeshMaterial3d<StandardMaterial>>,
    material_assets: &Assets<StandardMaterial>,
    mesh_handles: &Query<&Mesh3d>,
    mesh_assets: &Assets<Mesh>,
) -> GenResponse {
    // Resolve output path: use provided path or default to {workspace}/exports/{timestamp}.glb
    let output_path = match path {
        Some(p) if !p.is_empty() => {
            if std::path::Path::new(p).extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("glb") || ext.eq_ignore_ascii_case("gltf")
            }) {
                p.to_string()
            } else {
                format!("{}.glb", p)
            }
        }
        _ => {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let exports_dir = workspace.path.join("exports");
            exports_dir
                .join(format!("{}.glb", timestamp))
                .to_string_lossy()
                .into_owned()
        }
    };

    match super::gltf_export::export_glb(
        std::path::Path::new(&output_path),
        registry,
        transforms,
        gen_entities,
        parent_query,
        material_handles,
        material_assets,
        mesh_handles,
        mesh_assets,
    ) {
        Ok(()) => GenResponse::Exported { path: output_path },
        Err(e) => GenResponse::Error { message: e },
    }
}

// ---------------------------------------------------------------------------
// glTF path resolution
// ---------------------------------------------------------------------------

/// Resolve a glTF file path with the following fallback logic:
/// 1. Expand `~` and try as-is
/// 2. Try `{workspace}/{path}`
/// 3. Try `{workspace}/exports/{path}`
/// 4. Walk workspace directory tree looking for a file whose name matches the basename
/// 5. Return None if nothing found
pub fn resolve_gltf_path(path: &str, workspace: &PathBuf) -> Option<PathBuf> {
    // 1. Expand ~ and try as-is
    let expanded = shellexpand::tilde(path).into_owned();
    let p = std::path::Path::new(&expanded);
    if p.exists() {
        return p.canonicalize().ok();
    }

    // 2. {workspace}/{path}
    let wp = workspace.join(&expanded);
    if wp.exists() {
        return wp.canonicalize().ok();
    }

    // 3. {workspace}/exports/{path}
    let ep = workspace.join("exports").join(&expanded);
    if ep.exists() {
        return ep.canonicalize().ok();
    }

    // 4. Walk workspace for matching basename
    let needle = std::path::Path::new(&expanded).file_name()?;
    find_in_dir(workspace, needle)
}

fn find_in_dir(dir: &PathBuf, needle: &OsStr) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_in_dir(&path, needle) {
                return Some(found);
            }
        } else if path.file_name() == Some(needle) {
            return path.canonicalize().ok();
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Fly camera systems
// ---------------------------------------------------------------------------

/// WASD + Space/Shift movement relative to camera orientation.
fn fly_cam_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<FlyCamConfig>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();

    let mut velocity = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        velocity += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        velocity -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        velocity -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        velocity += right;
    }
    if keys.pressed(KeyCode::Space) {
        velocity += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        velocity -= Vec3::Y;
    }

    if velocity != Vec3::ZERO {
        transform.translation += velocity.normalize() * config.move_speed * time.delta_secs();
    }
}

/// Right-click + mouse drag to rotate the camera (yaw and pitch).
fn fly_cam_look(
    mouse: Res<ButtonInput<MouseButton>>,
    config: Res<FlyCamConfig>,
    mut motion_reader: MessageReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    let delta: Vec2 = motion_reader.read().map(|e| e.delta).sum();
    if delta == Vec2::ZERO || !mouse.pressed(MouseButton::Right) {
        return;
    }

    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let yaw = -delta.x * config.look_sensitivity;
    let pitch = -delta.y * config.look_sensitivity;

    // Apply yaw (rotate around global Y axis)
    transform.rotate_y(yaw);

    // Apply pitch (rotate around local X axis) with clamping
    let right = transform.right().as_vec3();
    let new_rotation = Quat::from_axis_angle(right, pitch) * transform.rotation;

    // Clamp pitch: check the angle between the camera's forward and the horizontal plane
    let new_forward = new_rotation * Vec3::NEG_Z;
    let pitch_angle = new_forward.y.asin();
    let max_pitch = 89.0_f32.to_radians();

    if pitch_angle.abs() < max_pitch {
        transform.rotation = new_rotation;
    }
}

/// Scroll wheel adjusts movement speed.
fn fly_cam_scroll_speed(
    mut scroll_reader: MessageReader<MouseWheel>,
    mut config: ResMut<FlyCamConfig>,
) {
    for event in scroll_reader.read() {
        config.move_speed = (config.move_speed * (1.0 + event.y * 0.1)).clamp(0.5, 100.0);
    }
}
