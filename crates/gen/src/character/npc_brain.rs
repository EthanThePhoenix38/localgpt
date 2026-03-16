//! AI-driven NPC brain system (AI2.1).
//!
//! Attaches a local SLM-driven brain to NPCs for autonomous behavior.
//! Brain runs as background tasks, issuing commands at configurable tick rates.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// NPC actions the brain can decide to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum NpcAction {
    MoveTo { position: [f32; 3] },
    LookAt { entity: String },
    Speak { text: String },
    Emote { emote_type: EmoteType },
    Interact { entity: String },
    Wait,
    Wander,
}

/// Emote types for NPC expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmoteType {
    Wave,
    Nod,
    Shrug,
    Point,
    Laugh,
}

/// An event the NPC perceived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcEvent {
    pub timestamp: f64,
    pub description: String,
}

/// Configuration for an NPC brain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcBrainConfig {
    pub personality: String,
    pub model: String,
    pub tick_rate: f32,
    pub perception_radius: f32,
    pub goals: Vec<String>,
    pub knowledge: Vec<String>,
}

impl Default for NpcBrainConfig {
    fn default() -> Self {
        Self {
            personality: "a friendly villager".to_string(),
            model: "llama3.2:3b".to_string(),
            tick_rate: 2.0,
            perception_radius: 15.0,
            goals: Vec::new(),
            knowledge: Vec::new(),
        }
    }
}

/// State of an NPC brain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcBrainState {
    pub config: NpcBrainConfig,
    pub active: bool,
    pub current_action: NpcAction,
    pub last_tick: f64,
    pub recent_events: VecDeque<NpcEvent>,
    pub tick_count: u64,
}

impl NpcBrainState {
    pub fn new(config: NpcBrainConfig) -> Self {
        Self {
            config,
            active: true,
            current_action: NpcAction::Wait,
            last_tick: 0.0,
            recent_events: VecDeque::with_capacity(20),
            tick_count: 0,
        }
    }

    /// Add an event to the recent events buffer (ring buffer, max 20).
    pub fn push_event(&mut self, description: String, timestamp: f64) {
        if self.recent_events.len() >= 20 {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(NpcEvent {
            timestamp,
            description,
        });
    }

    /// Check if enough time has passed for next tick.
    pub fn should_tick(&self, current_time: f64) -> bool {
        self.active && (current_time - self.last_tick) >= self.config.tick_rate as f64
    }
}

/// A nearby entity perceived by the NPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceivedEntity {
    pub name: String,
    pub entity_type: String,
    pub position: [f32; 3],
    pub distance: f32,
    pub direction: String,
}

/// Perception snapshot for building brain context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionSnapshot {
    pub npc_position: [f32; 3],
    pub current_action: NpcAction,
    pub nearby_entities: Vec<PerceivedEntity>,
    pub player_info: Option<PerceivedEntity>,
    pub recent_events: Vec<NpcEvent>,
}

/// Build the context prompt for an NPC brain tick.
pub fn build_brain_context(
    brain: &NpcBrainState,
    perception: &PerceptionSnapshot,
    memory_context: Option<&str>,
) -> String {
    let mut ctx = format!("You are {}.\n\n", brain.config.personality);

    if !brain.config.goals.is_empty() {
        ctx.push_str("Your goals:\n");
        for goal in &brain.config.goals {
            ctx.push_str(&format!("- {}\n", goal));
        }
        ctx.push('\n');
    }

    if !brain.config.knowledge.is_empty() {
        ctx.push_str("You know:\n");
        for fact in &brain.config.knowledge {
            ctx.push_str(&format!("- {}\n", fact));
        }
        ctx.push('\n');
    }

    if let Some(mem) = memory_context {
        ctx.push_str(&format!("## Your memories\n{}\n\n", mem));
    }

    ctx.push_str(&format!(
        "## Current situation\nPosition: ({:.1}, {:.1}, {:.1})\nCurrently: {:?}\n\n",
        perception.npc_position[0], perception.npc_position[1], perception.npc_position[2],
        perception.current_action
    ));

    if !perception.nearby_entities.is_empty() {
        ctx.push_str(&format!(
            "## Nearby ({} entities within {}m)\n",
            perception.nearby_entities.len(),
            brain.config.perception_radius
        ));
        for entity in &perception.nearby_entities {
            ctx.push_str(&format!(
                "- \"{}\" ({}) at {:.1}m {}\n",
                entity.name, entity.entity_type, entity.distance, entity.direction
            ));
        }
        ctx.push('\n');
    }

    if let Some(player) = &perception.player_info {
        ctx.push_str(&format!(
            "Player at {:.1}m {}\n\n",
            player.distance, player.direction
        ));
    }

    if !perception.recent_events.is_empty() {
        ctx.push_str("## Recent events\n");
        for event in &perception.recent_events {
            ctx.push_str(&format!("- {}\n", event.description));
        }
        ctx.push('\n');
    }

    ctx.push_str("## Available actions (choose exactly ONE)\n");
    ctx.push_str("- move_to(x, y, z) — walk to position\n");
    ctx.push_str("- look_at(\"entity_name\") — turn toward entity\n");
    ctx.push_str("- speak(\"text\") — say aloud\n");
    ctx.push_str("- emote(wave|nod|shrug|point|laugh)\n");
    ctx.push_str("- interact(\"entity_name\") — interact with nearby entity\n");
    ctx.push_str("- wait — stay still, observe\n");
    ctx.push_str("- wander — walk to random nearby point\n\n");
    ctx.push_str("Respond with ONLY the action. Example: speak(\"Welcome, traveler!\")");

    ctx
}

/// Parse an NPC action from SLM response text.
pub fn parse_npc_action(response: &str) -> NpcAction {
    let trimmed = response.trim();

    // Try move_to(x, y, z)
    if let Some(inner) = extract_parens(trimmed, "move_to") {
        let parts: Vec<f32> = inner
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if parts.len() >= 3 {
            return NpcAction::MoveTo {
                position: [parts[0], parts[1], parts[2]],
            };
        }
    }

    // Try look_at("entity")
    if let Some(inner) = extract_parens(trimmed, "look_at") {
        let entity = inner.trim_matches('"').trim_matches('\'').to_string();
        if !entity.is_empty() {
            return NpcAction::LookAt { entity };
        }
    }

    // Try speak("text")
    if let Some(inner) = extract_parens(trimmed, "speak") {
        let text = inner.trim_matches('"').trim_matches('\'').to_string();
        if !text.is_empty() {
            return NpcAction::Speak { text };
        }
    }

    // Try emote(type)
    if let Some(inner) = extract_parens(trimmed, "emote") {
        let emote_str = inner.trim().to_lowercase();
        let emote_type = match emote_str.as_str() {
            "wave" => Some(EmoteType::Wave),
            "nod" => Some(EmoteType::Nod),
            "shrug" => Some(EmoteType::Shrug),
            "point" => Some(EmoteType::Point),
            "laugh" => Some(EmoteType::Laugh),
            _ => None,
        };
        if let Some(et) = emote_type {
            return NpcAction::Emote { emote_type: et };
        }
    }

    // Try interact("entity")
    if let Some(inner) = extract_parens(trimmed, "interact") {
        let entity = inner.trim_matches('"').trim_matches('\'').to_string();
        if !entity.is_empty() {
            return NpcAction::Interact { entity };
        }
    }

    // Simple keywords
    if trimmed.starts_with("wait") {
        return NpcAction::Wait;
    }
    if trimmed.starts_with("wander") {
        return NpcAction::Wander;
    }

    // Fallback
    NpcAction::Wait
}

/// Helper to extract content inside parentheses after a function name.
fn extract_parens<'a>(text: &'a str, func_name: &str) -> Option<&'a str> {
    let lower = text.to_lowercase();
    if let Some(start) = lower.find(&format!("{}(", func_name)) {
        let after_paren = start + func_name.len() + 1;
        if after_paren < text.len() {
            // Find matching closing paren
            if let Some(end) = text[after_paren..].rfind(')') {
                return Some(&text[after_paren..after_paren + end]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_state_new() {
        let config = NpcBrainConfig::default();
        let state = NpcBrainState::new(config);
        assert!(state.active);
        assert!(matches!(state.current_action, NpcAction::Wait));
        assert_eq!(state.last_tick, 0.0);
        assert_eq!(state.tick_count, 0);
        assert!(state.recent_events.is_empty());
    }

    #[test]
    fn test_push_event_ring_buffer() {
        let config = NpcBrainConfig::default();
        let mut state = NpcBrainState::new(config);

        // Fill beyond capacity
        for i in 0..25 {
            state.push_event(format!("event_{}", i), i as f64);
        }

        // Should only keep last 20
        assert_eq!(state.recent_events.len(), 20);
        assert_eq!(state.recent_events.front().unwrap().description, "event_5");
        assert_eq!(
            state.recent_events.back().unwrap().description,
            "event_24"
        );
    }

    #[test]
    fn test_should_tick() {
        let config = NpcBrainConfig {
            tick_rate: 2.0,
            ..Default::default()
        };
        let mut state = NpcBrainState::new(config);
        state.last_tick = 10.0;

        // Not enough time
        assert!(!state.should_tick(11.0));
        // Enough time
        assert!(state.should_tick(12.0));
        assert!(state.should_tick(13.0));

        // Inactive brain should not tick
        state.active = false;
        assert!(!state.should_tick(13.0));
    }

    #[test]
    fn test_parse_speak() {
        let action = parse_npc_action("speak(\"Welcome, traveler!\")");
        match action {
            NpcAction::Speak { text } => assert_eq!(text, "Welcome, traveler!"),
            other => panic!("Expected Speak, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_move_to() {
        let action = parse_npc_action("move_to(1.0, 2.5, 3.0)");
        match action {
            NpcAction::MoveTo { position } => {
                assert_eq!(position, [1.0, 2.5, 3.0]);
            }
            other => panic!("Expected MoveTo, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_emote() {
        let action = parse_npc_action("emote(wave)");
        match action {
            NpcAction::Emote { emote_type } => assert_eq!(emote_type, EmoteType::Wave),
            other => panic!("Expected Emote, got {:?}", other),
        }

        let action = parse_npc_action("emote(laugh)");
        match action {
            NpcAction::Emote { emote_type } => assert_eq!(emote_type, EmoteType::Laugh),
            other => panic!("Expected Emote, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_fallback() {
        let action = parse_npc_action("some garbage text");
        assert!(matches!(action, NpcAction::Wait));

        let action = parse_npc_action("wait");
        assert!(matches!(action, NpcAction::Wait));

        let action = parse_npc_action("wander around");
        assert!(matches!(action, NpcAction::Wander));
    }

    #[test]
    fn test_build_context() {
        let config = NpcBrainConfig {
            personality: "a wise elder".to_string(),
            goals: vec!["protect the village".to_string()],
            knowledge: vec!["dragons roam the north".to_string()],
            ..Default::default()
        };
        let brain = NpcBrainState::new(config);
        let perception = PerceptionSnapshot {
            npc_position: [1.0, 0.0, 2.0],
            current_action: NpcAction::Wait,
            nearby_entities: vec![PerceivedEntity {
                name: "oak_tree".to_string(),
                entity_type: "prop".to_string(),
                position: [5.0, 0.0, 2.0],
                distance: 4.0,
                direction: "east".to_string(),
            }],
            player_info: Some(PerceivedEntity {
                name: "player".to_string(),
                entity_type: "player".to_string(),
                position: [3.0, 0.0, 2.0],
                distance: 2.0,
                direction: "east".to_string(),
            }),
            recent_events: vec![],
        };

        let ctx = build_brain_context(&brain, &perception, Some("Met a traveler yesterday"));

        assert!(ctx.contains("You are a wise elder."));
        assert!(ctx.contains("protect the village"));
        assert!(ctx.contains("dragons roam the north"));
        assert!(ctx.contains("Met a traveler yesterday"));
        assert!(ctx.contains("oak_tree"));
        assert!(ctx.contains("Player at 2.0m east"));
        assert!(ctx.contains("Available actions"));
    }
}
