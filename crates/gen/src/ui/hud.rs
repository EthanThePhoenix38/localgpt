//! Screen-space HUD elements.
//!
//! Implements Spec 4.2: `gen_add_hud` — Persistent UI overlays.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// HUD element types.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum HudElementType {
    #[default]
    Score,
    Health,
    Text,
    Timer,
}

/// HUD position presets.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum HudPosition {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    CenterTop,
    CenterBottom,
}

/// Parameters for HUD element creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudParams {
    /// Element type.
    #[serde(default)]
    pub element_type: HudElementType,
    /// Screen position.
    #[serde(default)]
    pub position: HudPosition,
    /// Label prefix.
    #[serde(default)]
    pub label: Option<String>,
    /// Initial value.
    #[serde(default = "default_initial_value")]
    pub initial_value: String,
    /// Font size.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Text color (hex).
    #[serde(default = "default_color")]
    pub color: String,
    /// Unique ID for updates.
    #[serde(default)]
    pub id: Option<String>,
}

fn default_initial_value() -> String {
    "0".to_string()
}
fn default_font_size() -> f32 {
    18.0
}
fn default_color() -> String {
    "#ffffff".to_string()
}

impl Default for HudParams {
    fn default() -> Self {
        Self {
            element_type: HudElementType::Score,
            position: HudPosition::TopLeft,
            label: None,
            initial_value: default_initial_value(),
            font_size: default_font_size(),
            color: default_color(),
            id: None,
        }
    }
}

/// Marker component for HUD elements.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HudElement {
    /// Element type.
    pub element_type: HudElementType,
    /// Unique ID for updates.
    pub id: Option<String>,
    /// Current value.
    pub value: String,
    /// Label prefix.
    pub label: Option<String>,
}

/// Resource for tracking score (shared with P2 interaction).
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct HudScore {
    pub score: i64,
}

/// Resource for tracking health.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HudHealth {
    pub current: f32,
    pub max: f32,
}

impl Default for HudHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

/// Resource for timer state.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HudTimer {
    /// Remaining seconds (for countdown) or elapsed (for countup).
    pub value: f32,
    /// Is this a countdown timer?
    pub is_countdown: bool,
    /// Is the timer running?
    pub running: bool,
}

impl Default for HudTimer {
    fn default() -> Self {
        Self {
            value: 0.0,
            is_countdown: false,
            running: true,
        }
    }
}

impl HudTimer {
    /// Format timer as MM:SS.
    pub fn formatted(&self) -> String {
        let total = self.value.max(0.0) as i32;
        let minutes = total / 60;
        let seconds = total % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// System to update timer.
pub fn hud_timer_system(time: Res<Time>, mut timer: ResMut<HudTimer>) {
    if timer.running {
        if timer.is_countdown {
            timer.value = (timer.value - time.delta_secs()).max(0.0);
            if timer.value <= 0.0 {
                timer.running = false;
                // TimerExpired event would be emitted here
            }
        } else {
            timer.value += time.delta_secs();
        }
    }
}

/// System to sync HUD text with values.
pub fn hud_sync_system(
    score: Res<HudScore>,
    health: Res<HudHealth>,
    timer: Res<HudTimer>,
    mut hud_query: Query<(&HudElement, &mut Text)>,
) {
    for (hud, mut text) in hud_query.iter_mut() {
        let value_str = match hud.element_type {
            HudElementType::Score => score.score.to_string(),
            HudElementType::Health => format!("{:.0}/{:.0}", health.current, health.max),
            HudElementType::Timer => timer.formatted(),
            HudElementType::Text => hud.value.clone(),
        };

        let display = if let Some(label) = &hud.label {
            format!("{}: {}", label, value_str)
        } else {
            value_str
        };

        **text = display.clone();
    }
}

/// Plugin for HUD systems.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudScore>()
            .init_resource::<HudHealth>()
            .init_resource::<HudTimer>()
            .add_systems(Update, (hud_timer_system, hud_sync_system));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_timer_format() {
        let timer = HudTimer {
            value: 125.0,
            is_countdown: true,
            running: true,
        };
        assert_eq!(timer.formatted(), "02:05");
    }

    #[test]
    fn test_hud_timer_zero() {
        let timer = HudTimer {
            value: 0.0,
            is_countdown: true,
            running: false,
        };
        assert_eq!(timer.formatted(), "00:00");
    }

    #[test]
    fn test_hud_health_default() {
        let health = HudHealth::default();
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
    }

    #[test]
    fn test_hud_params_default() {
        let params = HudParams::default();
        assert_eq!(params.font_size, 18.0);
        assert_eq!(params.initial_value, "0");
    }
}
