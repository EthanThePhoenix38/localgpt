//! MCP tool handlers for P2: Interaction & Trigger System.

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::gen3d::GenBridge;
use crate::gen3d::commands::*;
use crate::interaction::*;
use localgpt_core::agent::ToolSchema;
use localgpt_core::agent::tools::Tool;

// ---------------------------------------------------------------------------
// gen_add_trigger
// ---------------------------------------------------------------------------

/// Tool: Add an interaction trigger and action.
pub struct GenAddTriggerTool {
    bridge: Arc<GenBridge>,
}

impl GenAddTriggerTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenAddTriggerTool {
    fn name(&self) -> &str {
        "gen_add_trigger"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_add_trigger".to_string(),
            description: "Add an interaction trigger and action to an entity. Combines triggers (proximity, click, area, collision, timer) with actions (animate, teleport, play sound, show text, toggle state, spawn, destroy, add score, enable, disable).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Target entity name"
                    },
                    "trigger_type": {
                        "type": "string",
                        "enum": ["proximity", "click", "area_enter", "area_exit", "collision", "timer"],
                        "description": "Type of trigger"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["animate", "teleport", "play_sound", "show_text", "toggle_state", "spawn", "destroy", "add_score", "enable", "disable"],
                        "description": "Action to perform"
                    },
                    "trigger_params": {
                        "type": "object",
                        "properties": {
                            "radius": { "type": "number", "default": 3.0 },
                            "cooldown": { "type": "number", "default": 1.0 },
                            "interval": { "type": "number" },
                            "max_distance": { "type": "number", "default": 5.0 },
                            "prompt_text": { "type": "string" }
                        }
                    },
                    "action_params": {
                        "type": "object",
                        "properties": {
                            "property": { "type": "string" },
                            "to": { "type": "array", "items": { "type": "number" } },
                            "duration": { "type": "number", "default": 1.0 },
                            "easing": { "type": "string", "default": "ease_in_out" },
                            "destination": { "type": "array", "items": { "type": "number" } },
                            "sound": { "type": "string" },
                            "text": { "type": "string" },
                            "state_key": { "type": "string" },
                            "value": { "type": "string" },
                            "amount": { "type": "integer" }
                        }
                    },
                    "once": { "type": "boolean", "default": false }
                },
                "required": ["entity_id", "trigger_type", "action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let params = AddTriggerParams::from_json(args)?;
        let cmd = GenCommand::AddTrigger(params);
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::TriggerAdded { entity } => {
                Ok(format!("Trigger added to entity '{}'", entity))
            }
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Trigger added successfully".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// gen_add_teleporter
// ---------------------------------------------------------------------------

/// Tool: Create a portal that teleports the player.
pub struct GenAddTeleporterTool {
    bridge: Arc<GenBridge>,
}

impl GenAddTeleporterTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenAddTeleporterTool {
    fn name(&self) -> &str {
        "gen_add_teleporter"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_add_teleporter".to_string(),
            description: "Create a portal that teleports the player to a destination when they step into it.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "position": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "Portal position [x, y, z]"
                    },
                    "destination": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "Teleport destination [x, y, z]"
                    },
                    "size": {
                        "type": "array",
                        "items": { "type": "number" },
                        "default": [2, 3, 2],
                        "description": "Portal trigger size [x, y, z]"
                    },
                    "effect": {
                        "type": "string",
                        "enum": ["none", "fade", "particles"],
                        "default": "fade",
                        "description": "Teleport effect"
                    },
                    "sound": {
                        "type": "string",
                        "description": "Optional sound effect"
                    },
                    "label": {
                        "type": "string",
                        "description": "Optional label text above portal"
                    }
                },
                "required": ["position", "destination"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let params = AddTeleporterParams::from_json(args)?;
        let cmd = GenCommand::AddTeleporter(params);
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::Spawned { name, .. } => {
                Ok(format!("Teleporter '{}' created at destination {:?}", name, params.destination))
            }
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Teleporter created successfully".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// gen_add_collectible
// ---------------------------------------------------------------------------

/// Tool: Make an entity collectible.
pub struct GenAddCollectibleTool {
    bridge: Arc<GenBridge>,
}

impl GenAddCollectibleTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenAddCollectibleTool {
    fn name(&self) -> &str {
        "gen_add_collectible"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_add_collectible".to_string(),
            description: "Make an entity collectible with score value, pickup effects, and optional respawning.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Target entity name"
                    },
                    "value": {
                        "type": "integer",
                        "default": 1,
                        "description": "Score value"
                    },
                    "category": {
                        "type": "string",
                        "default": "points",
                        "description": "Score category"
                    },
                    "pickup_sound": {
                        "type": "string",
                        "description": "Optional pickup sound"
                    },
                    "pickup_effect": {
                        "type": "string",
                        "enum": ["none", "sparkle", "dissolve"],
                        "default": "sparkle",
                        "description": "Pickup visual effect"
                    },
                    "respawn_time": {
                        "type": "number",
                        "description": "Seconds until respawn (null = no respawn)"
                    }
                },
                "required": ["entity_id"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let params = AddCollectibleParams::from_json(args)?;
        let cmd = GenCommand::AddCollectible(params);
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::Modified { name } => {
                Ok(format!("Entity '{}' is now collectible (value: {})", name, params.value))
            }
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Collectible added successfully".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// gen_add_door
// ---------------------------------------------------------------------------

/// Tool: Add interactive door behavior.
pub struct GenAddDoorTool {
    bridge: Arc<GenBridge>,
}

impl GenAddDoorTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenAddDoorTool {
    fn name(&self) -> &str {
        "gen_add_door"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_add_door".to_string(),
            description: "Add interactive door behavior to an entity with open/close logic and optional key requirement.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Target entity (should have hinge at origin)"
                    },
                    "trigger": {
                        "type": "string",
                        "enum": ["proximity", "click"],
                        "default": "proximity",
                        "description": "How door is triggered"
                    },
                    "open_angle": {
                        "type": "number",
                        "default": 90.0,
                        "description": "Door opening angle in degrees"
                    },
                    "open_duration": {
                        "type": "number",
                        "default": 1.5,
                        "description": "Animation duration"
                    },
                    "auto_close": {
                        "type": "boolean",
                        "default": true,
                        "description": "Auto-close after delay"
                    },
                    "auto_close_delay": {
                        "type": "number",
                        "default": 3.0,
                        "description": "Seconds before auto-close"
                    },
                    "sound_open": {
                        "type": "string",
                        "description": "Opening sound"
                    },
                    "sound_close": {
                        "type": "string",
                        "description": "Closing sound"
                    },
                    "requires_key": {
                        "type": "string",
                        "description": "Required key item name"
                    }
                },
                "required": ["entity_id"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let params = AddDoorParams::from_json(args)?;
        let cmd = GenCommand::AddDoor(params);
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::Modified { name } => {
                Ok(format!("Door behavior added to '{}'", name))
            }
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Door added successfully".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// gen_link_entities
// ---------------------------------------------------------------------------

/// Tool: Wire one entity's event to another's action.
pub struct GenLinkEntitiesTool {
    bridge: Arc<GenBridge>,
}

impl GenLinkEntitiesTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenLinkEntitiesTool {
    fn name(&self) -> &str {
        "gen_link_entities"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_link_entities".to_string(),
            description: "Wire one entity's event to trigger another entity's action. Enables chain reactions and puzzle logic.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "source_id": {
                        "type": "string",
                        "description": "Source entity name"
                    },
                    "source_event": {
                        "type": "string",
                        "description": "Event type: 'clicked', 'state_changed:is_active', 'proximity_entered', etc."
                    },
                    "target_id": {
                        "type": "string",
                        "description": "Target entity name"
                    },
                    "target_action": {
                        "type": "string",
                        "description": "Action: perform: 'toggle_state:is_open', 'play_animation:open', 'enable', 'disable', 'destroy'"
                    },
                    "condition": {
                        "type": "string",
                        "description": "Optional boolean expression: 'source.is_active AND other.is_active'"
                    }
                },
                "required": ["source_id", "source_event", "target_id", "target_action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let params = LinkEntitiesParams::from_json(args)?;
        let cmd = GenCommand::LinkEntities(params);
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::EntitiesLinked { source, target } => {
                Ok(format!("Linked '{}' -> '{}' (event: {})", source, target, params.source_event))
            }
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Entities linked successfully".to_string()),
        }
    }
}

/// Create all P2 interaction tools.
pub fn create_interaction_tools(bridge: Arc<GenBridge>) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(GenAddTriggerTool::new(bridge.clone())),
        Box::new(GenAddTeleporterTool::new(bridge.clone())),
        Box::new(GenAddCollectibleTool::new(bridge.clone())),
        Box::new(GenAddDoorTool::new(bridge.clone())),
        Box::new(GenLinkEntitiesTool::new(bridge)),
    ]
}
