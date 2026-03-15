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

/// Action to play a sound.
#[derive(Component, Clone)]
pub struct PlaySoundAction {
    pub sound: String,
}

/// Action to spawn an entity.
#[derive(Component, Clone)]
pub struct SpawnAction {
    pub template: String,
}

/// Action to destroy the entity.
#[derive(Component, Clone)]
pub struct DestroyAction;

/// Action to enable the entity.
#[derive(Component, Clone)]
pub struct EnableAction;

/// Action to disable the entity.
#[derive(Component, Clone)]
pub struct DisableAction;

/// Marker for triggers that should fire only once.
#[derive(Component, Clone)]
pub struct OnceTrigger;

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

/// Message fired when score changes.
#[derive(Clone, Debug)]
pub struct ScoreChanged {
    pub category: String,
    pub new_value: i32,
    pub delta: i32,
}

/// Message fired when a trigger fires.
#[derive(Clone, Debug)]
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
    // Action-specific parameters
    /// Teleport destination [x, y, z].
    #[serde(default)]
    pub destination: Option<[f32; 3]>,
    /// Text for show_text action.
    #[serde(default)]
    pub text: Option<String>,
    /// Score amount for add_score action.
    #[serde(default)]
    pub amount: Option<i32>,
    /// State key for toggle_state action.
    #[serde(default)]
    pub state_key: Option<String>,
    /// Score category for add_score action.
    #[serde(default)]
    pub category: Option<String>,
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

/// Pickup visual effect for collectibles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "lowercase")]
pub enum PickupEffect {
    None,
    #[default]
    Sparkle,
    Dissolve,
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
    pub pickup_effect: PickupEffect,
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
            pickup_effect: PickupEffect::default(),
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
    pub pickup_effect: PickupEffect,
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

    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Runtime systems
// ---------------------------------------------------------------------------

/// System: check proximity triggers against the player each frame.
pub fn proximity_trigger_system(
    time: Res<Time>,
    mut player_query: Query<
        &mut Transform,
        (With<crate::character::Player>, Without<ProximityTrigger>),
    >,
    mut trigger_query: Query<(
        Entity,
        &Transform,
        &mut ProximityTrigger,
        Option<&TeleportAction>,
        Option<&AddScoreAction>,
        Option<&ToggleStateAction>,
        Option<&OnceTrigger>,
    )>,
    mut score_board: ResMut<ScoreBoard>,
    mut commands: Commands,
) {
    let player_pos = if let Ok(player_transform) = player_query.single() {
        player_transform.translation
    } else {
        return;
    };
    let now = time.elapsed_secs();

    for (entity, transform, mut trigger, teleport, score, toggle, once) in trigger_query.iter_mut()
    {
        let distance = player_pos.distance(transform.translation);
        if distance > trigger.radius {
            continue;
        }
        if now - trigger.last_triggered < trigger.cooldown {
            continue;
        }
        trigger.last_triggered = now;

        // Execute actions
        if let Some(teleport_action) = teleport
            && let Ok(mut pt) = player_query.single_mut()
        {
            pt.translation = teleport_action.destination;
        }
        if let Some(score_action) = score {
            let entry = score_board
                .scores
                .entry(score_action.category.clone())
                .or_insert(0);
            *entry += score_action.amount;
        }
        if let Some(_toggle) = toggle {
            // Toggle state handled via EntityState component
            if let Ok(mut estate) = commands.get_entity(entity) {
                estate.insert(EntityState::default());
            }
        }

        // Remove trigger if once
        if once.is_some() {
            commands.entity(entity).remove::<ProximityTrigger>();
            commands.entity(entity).remove::<OnceTrigger>();
        }
    }
}

/// System: handle click triggers (E key press within max_distance).
pub fn click_trigger_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        &mut Transform,
        (With<crate::character::Player>, Without<ClickTrigger>),
    >,
    click_query: Query<(
        Entity,
        &Transform,
        &ClickTrigger,
        Option<&TeleportAction>,
        Option<&AddScoreAction>,
        Option<&ToggleStateAction>,
        Option<&OnceTrigger>,
    )>,
    mut score_board: ResMut<ScoreBoard>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let player_pos = if let Ok(player_transform) = player_query.single() {
        player_transform.translation
    } else {
        return;
    };

    // Find the closest click-triggerable entity within range
    let mut closest: Option<(Entity, f32)> = None;
    for (entity, transform, trigger, _, _, _, _) in click_query.iter() {
        let distance = player_pos.distance(transform.translation);
        if distance <= trigger.max_distance && (closest.is_none() || distance < closest.unwrap().1)
        {
            closest = Some((entity, distance));
        }
    }

    let Some((target_entity, _)) = closest else {
        return;
    };

    // Fire actions on the closest entity
    if let Ok((entity, _transform, _trigger, teleport, score, toggle, once)) =
        click_query.get(target_entity)
    {
        if let Some(teleport_action) = teleport
            && let Ok(mut pt) = player_query.single_mut()
        {
            pt.translation = teleport_action.destination;
        }
        if let Some(score_action) = score {
            let entry = score_board
                .scores
                .entry(score_action.category.clone())
                .or_insert(0);
            *entry += score_action.amount;
        }
        if toggle.is_some()
            && let Ok(mut estate) = commands.get_entity(entity)
        {
            estate.insert(EntityState::default());
        }

        if once.is_some() {
            commands.entity(entity).remove::<ClickTrigger>();
            commands.entity(entity).remove::<OnceTrigger>();
        }
    }
}

/// System: tick timer triggers and fire their actions.
pub fn timer_trigger_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimerTrigger, Option<&AddScoreAction>)>,
    mut score_board: ResMut<ScoreBoard>,
) {
    for (_entity, mut trigger, score) in query.iter_mut() {
        trigger.timer.tick(time.delta());
        if trigger.timer.just_finished()
            && let Some(score_action) = score
        {
            let entry = score_board
                .scores
                .entry(score_action.category.clone())
                .or_insert(0);
            *entry += score_action.amount;
        }
    }
}

/// System: animate doors through their state machine.
pub fn door_system(time: Res<Time>, mut query: Query<(&mut Door, &mut Transform)>) {
    let dt = time.delta_secs();

    for (mut door, mut transform) in query.iter_mut() {
        match door.state.clone() {
            DoorState::Opening { progress } => {
                let new_progress = (progress + dt / door.open_duration).min(1.0);
                let angle = door.open_angle.to_radians() * new_progress;
                transform.rotation = door.original_rotation * Quat::from_rotation_y(angle);

                if new_progress >= 1.0 {
                    if door.auto_close {
                        door.state = DoorState::Open {
                            close_timer: Timer::from_seconds(
                                door.auto_close_delay,
                                TimerMode::Once,
                            ),
                        };
                    } else {
                        door.state = DoorState::Open {
                            close_timer: Timer::from_seconds(f32::MAX, TimerMode::Once),
                        };
                    }
                } else {
                    door.state = DoorState::Opening {
                        progress: new_progress,
                    };
                }
            }
            DoorState::Open { mut close_timer } => {
                close_timer.tick(time.delta());
                if close_timer.just_finished() {
                    door.state = DoorState::Closing { progress: 1.0 };
                } else {
                    door.state = DoorState::Open { close_timer };
                }
            }
            DoorState::Closing { progress } => {
                let new_progress = (progress - dt / door.open_duration).max(0.0);
                let angle = door.open_angle.to_radians() * new_progress;
                transform.rotation = door.original_rotation * Quat::from_rotation_y(angle);

                if new_progress <= 0.0 {
                    door.state = DoorState::Closed;
                    transform.rotation = door.original_rotation;
                } else {
                    door.state = DoorState::Closing {
                        progress: new_progress,
                    };
                }
            }
            DoorState::Closed => {}
        }
    }
}

/// System: open doors when player is near (proximity-triggered doors).
pub fn door_proximity_system(
    player_query: Query<&Transform, With<crate::character::Player>>,
    mut door_query: Query<
        (&Transform, &mut Door, &ProximityTrigger),
        Without<crate::character::Player>,
    >,
    inventory: Res<PlayerInventory>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (transform, mut door, trigger) in door_query.iter_mut() {
        let distance = player_pos.distance(transform.translation);
        if distance <= trigger.radius && matches!(door.state, DoorState::Closed) {
            // Check key requirement
            if let Some(ref key) = door.requires_key
                && !inventory.has_key(key)
            {
                continue;
            }
            door.state = DoorState::Opening { progress: 0.0 };
        }
    }
}

/// System: handle collectible pickup.
pub fn collectible_system(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::character::Player>>,
    mut collectible_query: Query<(Entity, &Transform, &mut Collectible, &mut Visibility)>,
    mut score_board: ResMut<ScoreBoard>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, transform, mut collectible, mut visibility) in collectible_query.iter_mut() {
        // Handle respawn timer
        if let Some(ref mut timer) = collectible.respawn_timer {
            timer.tick(time.delta());
            if timer.just_finished() {
                collectible.respawn_timer = None;
                *visibility = Visibility::Inherited;
            }
            continue; // Skip pickup check while respawning
        }

        // Check if visible (already collected items are hidden)
        if *visibility == Visibility::Hidden {
            continue;
        }

        let distance = player_pos.distance(transform.translation);
        if distance > 2.0 {
            continue;
        }

        // Collect!
        let entry = score_board
            .scores
            .entry(collectible.category.clone())
            .or_insert(0);
        *entry += collectible.value;

        if let Some(respawn_time) = collectible.respawn_time {
            // Hide and start respawn timer
            *visibility = Visibility::Hidden;
            collectible.respawn_timer = Some(Timer::from_seconds(respawn_time, TimerMode::Once));
        } else {
            // Permanently remove
            commands.entity(entity).despawn();
        }
    }
}

/// System: sync ScoreBoard → HudScore so the HUD reflects interaction scores.
pub fn score_sync_system(score_board: Res<ScoreBoard>, mut hud_score: ResMut<crate::ui::HudScore>) {
    // Sum all score categories into the HUD score
    let total: i32 = score_board.scores.values().sum();
    hud_score.score = total as i64;
}

/// System: animate entities with AnimateAction (tweens transform properties).
pub fn animate_action_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimateAction, &mut Transform)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, mut action, mut transform) in query.iter_mut() {
        action.progress = (action.progress + dt / action.duration.max(0.01)).min(1.0);
        let t = action.progress;

        match action.property.as_str() {
            "position" | "translation" if action.to.len() >= 3 => {
                let target = Vec3::new(action.to[0], action.to[1], action.to[2]);
                transform.translation = transform.translation.lerp(target, t);
            }
            "scale" if action.to.len() >= 3 => {
                let target = Vec3::new(action.to[0], action.to[1], action.to[2]);
                transform.scale = transform.scale.lerp(target, t);
            }
            "scale" if action.to.len() == 1 => {
                let target = Vec3::splat(action.to[0]);
                transform.scale = transform.scale.lerp(target, t);
            }
            "rotation" if action.to.len() >= 3 => {
                let target = Quat::from_euler(
                    EulerRot::YXZ,
                    action.to[1].to_radians(),
                    action.to[0].to_radians(),
                    action.to[2].to_radians(),
                );
                transform.rotation = transform.rotation.slerp(target, t);
            }
            _ => {}
        }

        if action.progress >= 1.0 {
            commands.entity(entity).remove::<AnimateAction>();
        }
    }
}

/// System: handle enable/disable actions toggling visibility.
pub fn enable_disable_system(
    mut commands: Commands,
    enable_query: Query<Entity, Added<EnableAction>>,
    disable_query: Query<Entity, Added<DisableAction>>,
) {
    for entity in enable_query.iter() {
        commands
            .entity(entity)
            .insert(Visibility::Inherited)
            .remove::<EnableAction>();
    }
    for entity in disable_query.iter() {
        commands
            .entity(entity)
            .insert(Visibility::Hidden)
            .remove::<DisableAction>();
    }
}

/// Plugin for interaction systems.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreBoard::default())
            .insert_resource(PlayerInventory::default())
            .add_systems(
                Update,
                (
                    proximity_trigger_system,
                    click_trigger_system,
                    timer_trigger_system,
                    door_system,
                    door_proximity_system,
                    collectible_system,
                    score_sync_system,
                    animate_action_system,
                    enable_disable_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoreboard() {
        let mut board = ScoreBoard::default();
        assert!(board.scores.is_empty());
        board.scores.insert("points".to_string(), 10);
        assert_eq!(board.scores["points"], 10);
    }

    #[test]
    fn test_player_inventory() {
        let mut inv = PlayerInventory::default();
        assert!(!inv.has_key("gold_key"));
        inv.add_item("gold_key".to_string());
        assert!(inv.has_key("gold_key"));
    }

    #[test]
    fn test_door_params_defaults() {
        let params = DoorParams::default();
        assert_eq!(params.open_angle, 90.0);
        assert!(params.auto_close);
        assert_eq!(params.auto_close_delay, 3.0);
        assert!(params.requires_key.is_none());
    }

    #[test]
    fn test_timer_trigger() {
        let trigger = TimerTrigger::new(2.0);
        assert_eq!(trigger.interval, 2.0);
    }

    #[test]
    fn test_click_trigger_default() {
        let trigger = ClickTrigger::default();
        assert_eq!(trigger.max_distance, 5.0);
        assert!(trigger.prompt_text.is_none());
    }

    #[test]
    fn test_area_trigger_default() {
        let trigger = AreaTrigger::default();
        assert!(trigger.is_enter);
    }

    #[test]
    fn test_proximity_trigger_default() {
        let trigger = ProximityTrigger::default();
        assert_eq!(trigger.radius, 5.0);
        assert_eq!(trigger.cooldown, 1.0);
        assert_eq!(trigger.last_triggered, 0.0);
    }

    #[test]
    fn test_collectible_params_default() {
        let params = CollectibleParams::default();
        assert_eq!(params.value, 1);
        assert_eq!(params.category, "points");
        assert!(params.pickup_sound.is_none());
        assert_eq!(params.pickup_effect, PickupEffect::Sparkle);
        assert!(params.respawn_time.is_none());
    }

    #[test]
    fn test_teleporter_params_size_default() {
        let size = default_teleporter_size();
        assert_eq!(size, [2.0, 3.0, 2.0]);
    }

    #[test]
    fn test_entity_link_fields() {
        let link = EntityLink {
            source_event: "clicked".to_string(),
            target_entity: "door_1".to_string(),
            target_action: "toggle_state:is_open".to_string(),
            condition: Some("source.is_active".to_string()),
        };
        assert_eq!(link.source_event, "clicked");
        assert!(link.condition.is_some());
    }

    #[test]
    fn test_add_trigger_params_action_fields() {
        let params = AddTriggerParams {
            entity_id: "box".to_string(),
            trigger_type: "proximity".to_string(),
            action: "add_score".to_string(),
            radius: Some(5.0),
            cooldown: None,
            interval: None,
            max_distance: None,
            prompt_text: None,
            once: false,
            destination: None,
            text: None,
            amount: Some(10),
            state_key: None,
            category: Some("coins".to_string()),
        };
        assert_eq!(params.amount, Some(10));
        assert_eq!(params.category.as_deref(), Some("coins"));
    }

    #[test]
    fn test_door_state_default() {
        assert!(matches!(DoorState::default(), DoorState::Closed));
    }

    #[test]
    fn test_collectible_component() {
        let c = Collectible {
            value: 5,
            category: "gems".to_string(),
            pickup_effect: PickupEffect::Sparkle,
            respawn_time: Some(10.0),
            original_position: Vec3::ZERO,
            respawn_timer: None,
        };
        assert_eq!(c.value, 5);
        assert_eq!(c.category, "gems");
        assert_eq!(c.respawn_time, Some(10.0));
    }

    #[test]
    fn test_scoreboard_add_multiple() {
        let mut board = ScoreBoard::default();
        board.scores.insert("coins".to_string(), 10);
        board.scores.insert("gems".to_string(), 3);
        assert_eq!(board.scores.len(), 2);
        assert_eq!(board.scores["coins"], 10);
        assert_eq!(board.scores["gems"], 3);
    }

    #[test]
    fn test_player_inventory_multiple() {
        let mut inv = PlayerInventory::default();
        inv.add_item("key_a".to_string());
        inv.add_item("key_b".to_string());
        assert!(inv.has_key("key_a"));
        assert!(inv.has_key("key_b"));
        assert!(!inv.has_key("key_c"));
    }

    #[test]
    fn test_teleporter_params_custom() {
        let params = TeleporterParams {
            position: [1.0, 0.0, 2.0],
            destination: [10.0, 0.0, 20.0],
            size: [3.0, 4.0, 3.0],
            effect: "beam".to_string(),
            sound: Some("whoosh".to_string()),
            label: Some("Portal".to_string()),
        };
        assert_eq!(params.label.as_deref(), Some("Portal"));
        assert_eq!(params.size, [3.0, 4.0, 3.0]);
    }

    #[test]
    fn test_link_entities_params() {
        let params = LinkEntitiesParams {
            source_id: "button".to_string(),
            source_event: "pressed".to_string(),
            target_id: "door".to_string(),
            target_action: "open".to_string(),
            condition: None,
        };
        assert_eq!(params.source_id, "button");
        assert!(params.condition.is_none());
    }

    #[test]
    fn test_pickup_effect_default() {
        assert_eq!(PickupEffect::default(), PickupEffect::Sparkle);
    }

    #[test]
    fn test_pickup_effect_variants() {
        assert_ne!(PickupEffect::None, PickupEffect::Sparkle);
        assert_ne!(PickupEffect::Sparkle, PickupEffect::Dissolve);
        assert_ne!(PickupEffect::Dissolve, PickupEffect::None);
    }

    #[test]
    fn test_pickup_effect_serde() {
        let json = serde_json::to_string(&PickupEffect::Dissolve).unwrap();
        assert_eq!(json, "\"dissolve\"");
        let parsed: PickupEffect = serde_json::from_str("\"sparkle\"").unwrap();
        assert_eq!(parsed, PickupEffect::Sparkle);
        let parsed_none: PickupEffect = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(parsed_none, PickupEffect::None);
    }
}
