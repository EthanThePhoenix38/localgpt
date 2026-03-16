//! World Inspector Protocol — JSON message types for remote inspection.
//!
//! These types define the WebSocket protocol between the Bevy Gen process
//! and remote inspector clients (SwiftUI on iPad/macOS, Jetpack Compose on Android).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Client → Server messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to push updates for given topics.
    Subscribe { topics: Vec<String> },
    /// Request the full entity tree.
    RequestSceneTree,
    /// Request detailed info for a specific entity.
    RequestEntityDetail { entity_id: u64 },
    /// Request global world info.
    RequestWorldInfo,
    /// Select an entity (syncs across all clients).
    SelectEntity { entity_id: u64 },
    /// Clear the current selection.
    Deselect,
    /// Toggle visibility of an entity.
    ToggleVisibility { entity_id: u64 },
    /// Focus the Bevy camera on an entity.
    FocusEntity { entity_id: u64 },
    /// Request a GLB binary snapshot of the full scene.
    /// Server responds with a binary WebSocket frame containing GLB data.
    RequestSceneSnapshot,
}

// ---------------------------------------------------------------------------
// Server → Client messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Full entity tree (response to RequestSceneTree or on connect).
    SceneTree { entities: Vec<TreeEntity> },
    /// Detailed info for a single entity.
    EntityDetail {
        entity_id: u64,
        data: Box<EntityDetailData>,
    },
    /// Global world info.
    WorldInfo { data: WorldInfoData },
    /// Selection changed (broadcast to all clients).
    SelectionChanged { entity_id: u64 },
    /// Selection cleared (broadcast to all clients).
    SelectionCleared,
    /// Scene structure changed (entity added/removed/renamed).
    /// Client should re-request scene_tree.
    SceneChanged,
    /// Transform updated for the selected entity (throttled to 10 Hz).
    EntityTransformUpdated {
        entity_id: u64,
        position: [f32; 3],
        rotation_degrees: [f32; 3],
    },
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct TreeEntity {
    pub id: u64,
    pub name: String,
    pub entity_type: String,
    pub parent_id: Option<u64>,
    pub visible: bool,
    pub children: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityDetailData {
    pub identity: IdentitySection,
    pub transform: Option<TransformSection>,
    pub shape: Option<String>,
    pub material: Option<MaterialSection>,
    pub light: Option<LightSection>,
    pub behaviors: Vec<BehaviorSection>,
    pub audio: Option<AudioSection>,
    pub mesh_asset: Option<String>,
    pub hierarchy: HierarchySection,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentitySection {
    pub name: String,
    pub id: u64,
    pub entity_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransformSection {
    pub position: [f32; 3],
    pub rotation_degrees: [f32; 3],
    pub scale: [f32; 3],
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaterialSection {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub emissive: [f32; 4],
    pub alpha_mode: String,
    pub double_sided: bool,
    pub unlit: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LightSection {
    pub light_type: String,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: Option<f32>,
    pub shadows_enabled: bool,
    pub inner_angle: Option<f32>,
    pub outer_angle: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorSection {
    pub id: String,
    pub behavior_type: String,
    pub base_position: [f32; 3],
    pub base_scale: [f32; 3],
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioSection {
    pub sound_type: String,
    pub volume: f32,
    pub radius: f32,
    pub attached_to: Option<String>,
    pub position: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HierarchySection {
    pub parent: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldInfoData {
    pub name: Option<String>,
    pub entity_count: usize,
    pub behavior_state: BehaviorStateInfo,
    pub audio: Option<AudioStateInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorStateInfo {
    pub paused: bool,
    pub elapsed: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioStateInfo {
    pub active: bool,
    pub emitter_count: usize,
    pub ambience_layers: Vec<String>,
}
