//! Contextual tooltips for entities.
//!
//! Implements Spec 4.4: `gen_add_tooltip` — Interaction prompts.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Trigger type for tooltip.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum TooltipTrigger {
    #[default]
    Proximity,
    LookAt,
}

/// Display style for tooltip.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum TooltipStyle {
    #[default]
    Floating,
    ScreenCenter,
    ScreenBottom,
}

/// Parameters for tooltip creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipParams {
    /// Target entity ID.
    pub entity_id: String,
    /// Tooltip text.
    pub text: String,
    /// Trigger type.
    #[serde(default)]
    pub trigger: TooltipTrigger,
    /// Trigger range.
    #[serde(default = "default_range")]
    pub range: f32,
    /// Display style.
    #[serde(default)]
    pub style: TooltipStyle,
    /// Text color (hex).
    #[serde(default = "default_tooltip_color")]
    pub color: String,
    /// Auto-dismiss after seconds.
    #[serde(default)]
    pub duration: Option<f32>,
}

fn default_range() -> f32 {
    3.0
}
fn default_tooltip_color() -> String {
    "#ffffff".to_string()
}

impl Default for TooltipParams {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            text: String::new(),
            trigger: TooltipTrigger::default(),
            range: default_range(),
            style: TooltipStyle::default(),
            color: default_tooltip_color(),
            duration: None,
        }
    }
}

/// Component for tooltip configuration.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Tooltip {
    /// Target entity.
    pub entity: Entity,
    /// Tooltip text.
    pub text: String,
    /// Trigger type.
    pub trigger: TooltipTrigger,
    /// Trigger range.
    pub range: f32,
    /// Display style.
    pub style: TooltipStyle,
    /// Current visibility.
    pub visible: bool,
    /// Fade alpha.
    pub fade_alpha: f32,
    /// Display timer.
    pub display_timer: Timer,
    /// Cooldown timer.
    pub cooldown_timer: Timer,
}

/// Resource tracking active tooltips.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ActiveTooltips {
    /// Currently visible screen-center tooltip.
    pub active_center: Option<Entity>,
    /// Currently visible screen-bottom tooltip.
    pub active_bottom: Option<Entity>,
}

/// System to check proximity triggers.
pub fn tooltip_proximity_system(
    player_query: Query<&Transform, With<Player>>,
    mut tooltip_query: Query<(&Transform, &mut Tooltip)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (transform, mut tooltip) in tooltip_query.iter_mut() {
        if tooltip.trigger != TooltipTrigger::Proximity {
            continue;
        }

        let distance = player_transform.translation.distance(transform.translation);

        let fade_start = tooltip.range * 0.8;
        if distance > tooltip.range {
            // Out of range
            tooltip.visible = false;
        } else if distance > fade_start {
            // Fade out
            tooltip.visible = true;
            tooltip.fade_alpha = 1.0 - (distance - fade_start) / (tooltip.range - fade_start);
        } else {
            // In range
            tooltip.visible = true;
            tooltip.fade_alpha = 1.0;
        }
    }
}

/// System to check look-at triggers.
pub fn tooltip_look_at_system(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut tooltip_query: Query<(Entity, &Transform, &mut Tooltip)>,
    mut active: ResMut<ActiveTooltips>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (entity, transform, mut tooltip) in tooltip_query.iter_mut() {
        if tooltip.trigger != TooltipTrigger::LookAt {
            continue;
        }

        // Get camera position and forward direction
        let camera_pos = camera_transform.translation;
        let camera_forward = camera_transform.forward();

        // Check if looking at this entity (simplified sphere check)
        let to_entity = transform.translation - camera_pos;
        let distance = to_entity.length();

        if distance <= tooltip.range {
            // Check if entity is in front of camera
            let dot = to_entity.normalize().dot(*camera_forward);
            if dot > 0.7 {
                // Looking at entity
                tooltip.visible = true;
                tooltip.fade_alpha = 1.0;
                match tooltip.style {
                    TooltipStyle::ScreenCenter => {
                        active.active_center = Some(entity);
                    }
                    TooltipStyle::ScreenBottom => {
                        active.active_bottom = Some(entity);
                    }
                    TooltipStyle::Floating => {}
                }
            } else {
                tooltip.visible = false;
            }
        } else {
            tooltip.visible = false;
        }
    }
}

/// System to update tooltip fade.
pub fn tooltip_fade_system(time: Res<Time>, mut tooltip_query: Query<&mut Tooltip>) {
    for mut tooltip in tooltip_query.iter_mut() {
        if tooltip.visible {
            // Fade in
            if tooltip.fade_alpha < 1.0 {
                tooltip.fade_alpha = (tooltip.fade_alpha + time.delta_secs() * 5.0).min(1.0);
            }
        } else {
            // Fade out
            if tooltip.fade_alpha > 0.0 {
                tooltip.fade_alpha = (tooltip.fade_alpha - time.delta_secs() * 5.0).max(0.0);
            }
        }
    }
}

/// Plugin for tooltip systems.
pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTooltips>()
            .add_systems(Update, tooltip_proximity_system)
            .add_systems(Update, tooltip_look_at_system)
            .add_systems(Update, tooltip_fade_system);
    }
}

// Import marker for player (from character module)
use crate::character::Player;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_params_default() {
        let params = TooltipParams::default();
        assert_eq!(params.trigger, TooltipTrigger::Proximity);
        assert_eq!(params.style, TooltipStyle::Floating);
        assert_eq!(params.range, 3.0);
        assert_eq!(params.color, "#ffffff");
        assert!(params.duration.is_none());
    }

    #[test]
    fn test_tooltip_trigger_variants() {
        assert_eq!(TooltipTrigger::default(), TooltipTrigger::Proximity);
        let look = TooltipTrigger::LookAt;
        assert_ne!(look, TooltipTrigger::Proximity);
    }

    #[test]
    fn test_tooltip_style_variants() {
        assert_eq!(TooltipStyle::default(), TooltipStyle::Floating);
        let _center = TooltipStyle::ScreenCenter;
        let _bottom = TooltipStyle::ScreenBottom;
    }

    #[test]
    fn test_active_tooltips_default() {
        let active = ActiveTooltips::default();
        assert!(active.active_center.is_none());
        assert!(active.active_bottom.is_none());
    }
}
