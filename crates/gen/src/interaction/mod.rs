//! Interaction & Trigger System (P2)
//!
//! These specs add gameplay to LocalGPT Gen.
//! Following the **Trigger → State Change → Behavior Response** pattern.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker for entities that can be interacted via triggers.
#[derive(Component)]
pub struct InteractionEntity;

// ---------------------------------------------------------------------------
// Trigger components
// ---------------------------------------------------------------------------

/// Marker for proximity triggers.
#[derive(Component, Clone)]
pub struct ProximityTrigger {
    pub radius: f32,
    pub cooldown: f32,
    pub last_triggered: f32,
}

impl Default for ProximityTrigger {
    fn default() -> Self {
        Self {
            radius: 5.0,
            cooldown: 1.0,
            last_triggered: 0.0,
        }
    }
}

/// Marker for click triggers.
#[derive(Component, Clone)]
pub struct ClickTrigger {
    pub max_distance: f32,
    pub prompt_text: Option<String>,
}

impl Default for ClickTrigger {
    fn default() -> Self {
        Self {
            max_distance: 5.0,
            prompt_text: None,
        }
    }
}

/// Marker for area triggers.
#[derive(Component, Clone)]
pub struct AreaTrigger {
    pub is_enter: bool,
}

impl Default for AreaTrigger {
    fn default() -> Self {
        Self { is_enter: true }
    }
}

/// Marker for timer triggers.
#[derive(Component, Clone)]
pub struct TimerTrigger {
    pub interval: f32,
    pub timer: Timer,
}

impl TimerTrigger {
    pub fn new(interval: f32) -> Self {
        Self {
            interval,
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
        }
    }
}

// ---------------------------------------------------------------------------
// Action components
// ---------------------------------------------------------------------------

/// Action to animate a transform property.
#[derive(Component, Clone)]
pub struct AnimateAction {
    pub property: String,
    pub to: Vec<f32>,
    pub duration: f32,
    pub progress: f32,
}

/// Action to teleport the player.
#[derive(Component, Clone)]
pub struct TeleportAction {
    pub destination: Vec3,
}

/// Action to show floating text.
#[derive(Component, Clone)]
pub struct ShowTextAction {
    pub text: String,
    pub duration: Option<f32>,
}

/// Action to toggle entity state.
#[derive(Component, Clone)]
pub struct ToggleStateAction {
    pub state_key: String,
    pub value: Option<String>,
}

/// Action to add to score.
#[derive(Component, Clone)]
pub struct AddScoreAction {
    pub amount: i32,
    pub category: String,
}

// ---------------------------------------------------------------------------
// Entity linking
// ---------------------------------------------------------------------------

/// Links a trigger on this entity to an action on another entity.
#[derive(Component, Clone)]
pub struct EntityLink {
    pub source_event: String,
    pub target_entity: String,
    pub target_action: String,
    pub condition: Option<String>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Global score tracking.
#[derive(Resource, Clone, Default)]
pub struct ScoreBoard {
    pub scores: HashMap<String, i32>,
}

/// Event fired when score changes.
#[derive(Event, Clone, Debug)]
pub struct ScoreChanged {
    pub category: String,
    pub new_value: i32,
    pub delta: i32,
}

/// Event fired when a trigger fires.
#[derive(Event, Clone, Debug)]
pub struct TriggerFired {
    pub entity: Entity,
    pub trigger_type: String,
}

/// Entity state storage.
#[derive(Component, Clone, Default)]
pub struct EntityState {
    pub states: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// MCP Tool parameters
// ---------------------------------------------------------------------------

/// Parameters for adding a trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTriggerParams {
    pub entity_id: String,
    pub trigger_type: String,
    pub action: String,
    #[serde(default)]
    pub radius: Option<f32>,
    #[serde(default)]
    pub cooldown: Option<f32>,
    #[serde(default)]
    pub interval: Option<f32>,
    #[serde(default)]
    pub max_distance: Option<f32>,
    #[serde(default)]
    pub prompt_text: Option<String>,
    #[serde(default)]
    pub once: bool,
}

/// Parameters for teleporter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleporterParams {
    pub position: [f32; 3],
    pub destination: [f32; 3],
    #[serde(default = "default_teleporter_size")]
    pub size: [f32; 3],
    #[serde(default)]
    pub effect: String,
    #[serde(default)]
    pub sound: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

fn default_teleporter_size() -> [f32; 3] {
    [2.0, 3.0, 2.0]
}

/// Parameters for collectible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectibleParams {
    pub entity_id: String,
    #[serde(default)]
    pub value: i32,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub pickup_sound: Option<String>,
    #[serde(default)]
    pub pickup_effect: String,
    #[serde(default)]
    pub respawn_time: Option<f32>,
}

impl Default for CollectibleParams {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            value: 1,
            category: "points".to_string(),
            pickup_sound: None,
            pickup_effect: "sparkle".to_string(),
            respawn_time: None,
        }
    }
}

/// Parameters for door.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorParams {
    pub entity_id: String,
    #[serde(default)]
    pub trigger: String,
    #[serde(default = "default_open_angle")]
    pub open_angle: f32,
    #[serde(default = "default_open_duration")]
    pub open_duration: f32,
    #[serde(default = "default_auto_close")]
    pub auto_close: bool,
    #[serde(default = "default_auto_close_delay")]
    pub auto_close_delay: f32,
    #[serde(default)]
    pub sound_open: Option<String>,
    #[serde(default)]
    pub sound_close: Option<String>,
    #[serde(default)]
    pub requires_key: Option<String>,
}

fn default_open_angle() -> f32 {
    90.0
}
fn default_open_duration() -> f32 {
    1.5
}
fn default_auto_close() -> bool {
    true
}
fn default_auto_close_delay() -> f32 {
    3.0
}

impl Default for DoorParams {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            trigger: "proximity".to_string(),
            open_angle: default_open_angle(),
            open_duration: default_open_duration(),
            auto_close: default_auto_close(),
            auto_close_delay: default_auto_close_delay(),
            sound_open: None,
            sound_close: None,
            requires_key: None,
        }
    }
}

/// Parameters for entity linking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntitiesParams {
    pub source_id: String,
    pub source_event: String,
    pub target_id: String,
    pub target_action: String,
    #[serde(default)]
    pub condition: Option<String>,
}

// ---------------------------------------------------------------------------
// Collectible component
// ---------------------------------------------------------------------------

/// Collectible component with idle animation.
#[derive(Component, Clone)]
pub struct Collectible {
    pub value: i32,
    pub category: String,
    pub pickup_effect: String,
    pub respawn_time: Option<f32>,
    pub original_position: Vec3,
    pub respawn_timer: Option<Timer>,
}

// ---------------------------------------------------------------------------
// Door component
// ---------------------------------------------------------------------------

/// Door state machine.
#[derive(Debug, Clone, Default)]
pub enum DoorState {
    #[default]
    Closed,
    Opening {
        progress: f32,
    },
    Open {
        close_timer: Timer,
    },
    Closing {
        progress: f32,
    },
}

/// Door component.
#[derive(Component, Clone)]
pub struct Door {
    pub state: DoorState,
    pub open_angle: f32,
    pub open_duration: f32,
    pub auto_close: bool,
    pub auto_close_delay: f32,
    pub requires_key: Option<String>,
    pub original_rotation: Quat,
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// Resource for tracking player inventory.
#[derive(Resource, Clone, Default)]
pub struct PlayerInventory {
    pub items: Vec<String>,
}

impl PlayerInventory {
    pub fn has_key(&self, key: &str) -> bool {
        self.items.contains(&key.to_string())
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for interaction systems.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreBoard::default())
            .insert_resource(PlayerInventory::default());
        // TODO: Add events when systems are implemented
        // .add_event::<TriggerFired>()
        // .add_event::<ScoreChanged>()
    }
}
