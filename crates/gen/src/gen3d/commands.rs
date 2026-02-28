//! GenCommand / GenResponse protocol between agent and Bevy.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Commands (agent → Bevy)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum GenCommand {
    // Tier 1: Perceive
    SceneInfo,
    Screenshot {
        width: u32,
        height: u32,
        wait_frames: u32,
    },
    EntityInfo {
        name: String,
    },

    // Tier 2: Mutate
    SpawnPrimitive(SpawnPrimitiveCmd),
    ModifyEntity(ModifyEntityCmd),
    DeleteEntity {
        name: String,
    },
    SetCamera(CameraCmd),
    SetLight(SetLightCmd),
    SetEnvironment(EnvironmentCmd),

    // Tier 3: Advanced
    SpawnMesh(RawMeshCmd),

    // Tier 4: Export
    ExportScreenshot {
        path: String,
        width: u32,
        height: u32,
    },
    ExportGltf {
        path: Option<String>,
    },

    // Tier 3b: Import
    LoadGltf {
        path: String,
    },

    // Tier 5: Audio
    SetAmbience(AmbienceCmd),
    SpawnAudioEmitter(AudioEmitterCmd),
    ModifyAudioEmitter(ModifyAudioEmitterCmd),
    RemoveAudioEmitter {
        name: String,
    },
    AudioInfo,

    // Tier 6: Behaviors
    AddBehavior(AddBehaviorCmd),
    RemoveBehavior {
        entity: String,
        behavior_id: Option<String>,
    },
    ListBehaviors {
        entity: Option<String>,
    },
    SetBehaviorsPaused {
        paused: bool,
    },

    // Tier 7: World skills
    SaveWorld(SaveWorldCmd),
    LoadWorld {
        path: String,
    },

    // Tier 8: Scene management
    ClearScene {
        keep_camera: bool,
        keep_lights: bool,
    },
}

// ---------------------------------------------------------------------------
// Command data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPrimitiveCmd {
    pub name: String,
    pub shape: PrimitiveShape,
    #[serde(default)]
    pub dimensions: HashMap<String, f32>,
    #[serde(default = "default_position")]
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation_degrees: [f32; 3],
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    #[serde(default)]
    pub metallic: f32,
    #[serde(default = "default_roughness")]
    pub roughness: f32,
    #[serde(default)]
    pub emissive: [f32; 4],
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PrimitiveShape {
    Cuboid,
    Sphere,
    Cylinder,
    Cone,
    Capsule,
    Torus,
    Plane,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyEntityCmd {
    pub name: String,
    pub position: Option<[f32; 3]>,
    pub rotation_degrees: Option<[f32; 3]>,
    pub scale: Option<[f32; 3]>,
    pub color: Option<[f32; 4]>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,
    pub emissive: Option<[f32; 4]>,
    pub visible: Option<bool>,
    pub parent: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraCmd {
    #[serde(default = "default_camera_pos")]
    pub position: [f32; 3],
    #[serde(default)]
    pub look_at: [f32; 3],
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLightCmd {
    pub name: String,
    #[serde(default = "default_light_type")]
    pub light_type: LightType,
    #[serde(default = "default_white")]
    pub color: [f32; 4],
    #[serde(default = "default_intensity")]
    pub intensity: f32,
    pub position: Option<[f32; 3]>,
    pub direction: Option<[f32; 3]>,
    #[serde(default = "default_true")]
    pub shadows: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentCmd {
    pub background_color: Option<[f32; 4]>,
    pub ambient_light: Option<f32>,
    pub ambient_color: Option<[f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMeshCmd {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    #[serde(default)]
    pub metallic: f32,
    #[serde(default = "default_roughness")]
    pub roughness: f32,
    #[serde(default)]
    pub position: [f32; 3],
}

// ---------------------------------------------------------------------------
// Audio command data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbienceCmd {
    pub layers: Vec<AmbienceLayerDef>,
    pub master_volume: Option<f32>,
    pub reverb: Option<ReverbParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbienceLayerDef {
    pub name: String,
    pub sound: AmbientSound,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AmbientSound {
    Wind { speed: f32, gustiness: f32 },
    Rain { intensity: f32 },
    Forest { bird_density: f32, wind: f32 },
    Ocean { wave_size: f32 },
    Cave { drip_rate: f32, resonance: f32 },
    Stream { flow_rate: f32 },
    Silence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEmitterCmd {
    pub name: String,
    pub entity: Option<String>,
    pub position: Option<[f32; 3]>,
    pub sound: EmitterSound,
    pub radius: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmitterSound {
    Water {
        turbulence: f32,
    },
    Fire {
        intensity: f32,
        crackle: f32,
    },
    Hum {
        frequency: f32,
        warmth: f32,
    },
    Wind {
        pitch: f32,
    },
    Custom {
        waveform: WaveformType,
        filter_cutoff: f32,
        filter_type: FilterType,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaveformType {
    Sine,
    Saw,
    Square,
    WhiteNoise,
    PinkNoise,
    BrownNoise,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbParams {
    pub room_size: f32,
    pub damping: f32,
    pub wet: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyAudioEmitterCmd {
    pub name: String,
    pub volume: Option<f32>,
    pub radius: Option<f32>,
    pub sound: Option<EmitterSound>,
}

// ---------------------------------------------------------------------------
// Behavior command data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBehaviorCmd {
    pub entity: String,
    #[serde(default)]
    pub behavior_id: Option<String>,
    pub behavior: BehaviorDef,
}

/// Declarative behavior definition — data, not code.
/// Each variant fully describes a continuous animation that the tick system evaluates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BehaviorDef {
    /// Orbit around a center entity or point.
    Orbit {
        /// Name of entity to orbit around (mutually exclusive with `center_point`).
        #[serde(default)]
        center: Option<String>,
        /// Fixed point to orbit around [x,y,z] (used if `center` is None).
        #[serde(default)]
        center_point: Option<[f32; 3]>,
        /// Orbit radius.
        #[serde(default = "default_orbit_radius")]
        radius: f32,
        /// Orbital speed in degrees per second.
        #[serde(default = "default_orbit_speed")]
        speed: f32,
        /// Orbit axis (normalized). Default: Y-up.
        #[serde(default = "default_y_axis")]
        axis: [f32; 3],
        /// Initial phase angle in degrees.
        #[serde(default)]
        phase: f32,
        /// Orbit tilt in degrees (inclination from the axis plane).
        #[serde(default)]
        tilt: f32,
    },
    /// Spin (rotate) around a local axis.
    Spin {
        /// Local axis to spin around.
        #[serde(default = "default_y_axis")]
        axis: [f32; 3],
        /// Rotation speed in degrees per second.
        #[serde(default = "default_spin_speed")]
        speed: f32,
    },
    /// Bob up and down (sinusoidal oscillation along an axis).
    Bob {
        /// Axis of oscillation.
        #[serde(default = "default_y_axis")]
        axis: [f32; 3],
        /// Amplitude (distance from center in each direction).
        #[serde(default = "default_bob_amplitude")]
        amplitude: f32,
        /// Oscillation frequency in Hz.
        #[serde(default = "default_bob_frequency")]
        frequency: f32,
        /// Phase offset in degrees.
        #[serde(default)]
        phase: f32,
    },
    /// Continuously look at / follow another entity.
    LookAt {
        /// Name of entity to look at.
        target: String,
    },
    /// Scale pulsation (breathing effect).
    Pulse {
        /// Minimum scale multiplier.
        #[serde(default = "default_pulse_min")]
        min_scale: f32,
        /// Maximum scale multiplier.
        #[serde(default = "default_pulse_max")]
        max_scale: f32,
        /// Pulse frequency in Hz.
        #[serde(default = "default_bob_frequency")]
        frequency: f32,
    },
}

fn default_orbit_radius() -> f32 {
    5.0
}
fn default_orbit_speed() -> f32 {
    36.0
}
fn default_y_axis() -> [f32; 3] {
    [0.0, 1.0, 0.0]
}
fn default_spin_speed() -> f32 {
    90.0
}
fn default_bob_amplitude() -> f32 {
    0.5
}
fn default_bob_frequency() -> f32 {
    0.5
}
fn default_pulse_min() -> f32 {
    0.9
}
fn default_pulse_max() -> f32 {
    1.1
}

// ---------------------------------------------------------------------------
// World save/load command data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWorldCmd {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

// ---------------------------------------------------------------------------
// Responses (Bevy → agent)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum GenResponse {
    SceneInfo(SceneInfoData),
    Screenshot {
        image_path: String,
    },
    EntityInfo(EntityInfoData),
    Spawned {
        name: String,
        entity_id: u64,
    },
    Modified {
        name: String,
    },
    Deleted {
        name: String,
    },
    CameraSet,
    LightSet {
        name: String,
    },
    EnvironmentSet,
    Exported {
        path: String,
    },
    GltfLoaded {
        name: String,
        path: String,
    },

    // Audio responses
    AmbienceSet,
    AudioEmitterSpawned {
        name: String,
    },
    AudioEmitterModified {
        name: String,
    },
    AudioEmitterRemoved {
        name: String,
    },
    AudioInfoData(AudioInfoResponse),

    // Behavior responses
    BehaviorAdded {
        entity: String,
        behavior_id: String,
    },
    BehaviorRemoved {
        entity: String,
        count: usize,
    },
    BehaviorList(BehaviorListResponse),
    BehaviorsPaused {
        paused: bool,
    },

    // World responses
    WorldSaved {
        path: String,
        skill_name: String,
    },
    WorldLoaded {
        path: String,
        entities: usize,
        behaviors: usize,
    },

    // Scene management
    SceneCleared {
        removed_count: usize,
    },

    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfoData {
    pub entity_count: usize,
    pub entities: Vec<EntitySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySummary {
    pub name: String,
    pub entity_type: String,
    pub position: [f32; 3],
    pub scale: [f32; 3],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<[f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfoData {
    pub name: String,
    pub entity_id: u64,
    pub entity_type: String,
    pub position: [f32; 3],
    pub rotation_degrees: [f32; 3],
    pub scale: [f32; 3],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,
    pub visible: bool,
    pub children: Vec<String>,
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub behaviors: Vec<BehaviorSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfoResponse {
    pub active: bool,
    pub ambience_layers: Vec<String>,
    pub emitters: Vec<AudioEmitterSummary>,
    pub master_volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEmitterSummary {
    pub name: String,
    pub sound_type: String,
    pub volume: f32,
    pub radius: f32,
    pub position: Option<[f32; 3]>,
    pub attached_to: Option<String>,
}

// ---------------------------------------------------------------------------
// Behavior response data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorListResponse {
    pub paused: bool,
    pub entities: Vec<EntityBehaviorsSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBehaviorsSummary {
    pub entity: String,
    pub behaviors: Vec<BehaviorSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSummary {
    pub id: String,
    pub behavior_type: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// Default helpers
// ---------------------------------------------------------------------------

fn default_position() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}
fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}
fn default_color() -> [f32; 4] {
    [0.8, 0.8, 0.8, 1.0]
}
fn default_roughness() -> f32 {
    0.5
}
fn default_camera_pos() -> [f32; 3] {
    [5.0, 5.0, 5.0]
}
fn default_fov() -> f32 {
    45.0
}
fn default_light_type() -> LightType {
    LightType::Directional
}
fn default_white() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_intensity() -> f32 {
    1000.0
}
fn default_true() -> bool {
    true
}
