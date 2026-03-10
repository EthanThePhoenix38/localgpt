//! Transient notification messages.
//!
//! Implements Spec 4.5: `gen_add_notification` — Temporary messages with animation.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Notification display style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStyle {
    #[default]
    Toast,
    Banner,
    Achievement,
}

/// Notification position on screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPosition {
    #[default]
    Top,
    Center,
    Bottom,
}

/// Built-in icon types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum NotificationIcon {
    #[default]
    None,
    Star,
    Coin,
    Key,
    Heart,
    Warning,
}

/// Parameters for notification creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationParams {
    /// Notification text.
    pub text: String,
    /// Display style.
    #[serde(default)]
    pub style: NotificationStyle,
    /// Position on screen.
    #[serde(default)]
    pub position: NotificationPosition,
    /// Display duration in seconds.
    #[serde(default = "default_duration")]
    pub duration: f32,
    /// Text color (hex).
    #[serde(default = "default_notification_color")]
    pub color: String,
    /// Icon type.
    #[serde(default)]
    pub icon: NotificationIcon,
    /// Sound to play.
    #[serde(default)]
    pub sound: Option<String>,
}

fn default_duration() -> f32 {
    3.0
}
fn default_notification_color() -> String {
    "#ffffff".to_string()
}

impl Default for NotificationParams {
    fn default() -> Self {
        Self {
            text: String::new(),
            style: NotificationStyle::default(),
            position: NotificationPosition::default(),
            duration: default_duration(),
            color: default_notification_color(),
            icon: NotificationIcon::default(),
            sound: None,
        }
    }
}

/// Component for notification.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Notification {
    /// Notification text.
    pub text: String,
    /// Display style.
    pub style: NotificationStyle,
    /// Screen position.
    pub position: NotificationPosition,
    /// Animation phase.
    pub phase: NotificationPhase,
    /// Elapsed time.
    pub elapsed: f32,
    /// Duration.
    pub duration: f32,
    /// Stack offset.
    pub stack_offset: f32,
    /// Alpha.
    pub alpha: f32,
}

/// Animation phases for notification.
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub enum NotificationPhase {
    #[default]
    EnterIn,
    Hold,
    Exit,
}

/// Event emitted when a notification is spawned.
#[derive(Message, Reflect)]
pub struct NotificationEvent {
    pub text: String,
    pub style: NotificationStyle,
    pub position: NotificationPosition,
    pub icon: NotificationIcon,
}

/// Resource tracking active notifications.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct NotificationQueue {
    /// Active notifications.
    pub notifications: Vec<Entity>,
}

/// System to spawn notifications from events.
pub fn notification_spawn_system(
    mut events: MessageReader<NotificationEvent>,
    mut commands: Commands,
    mut queue: ResMut<NotificationQueue>,
) {
    for event in events.read() {
        let entity = commands
            .spawn((
                Notification {
                    text: event.text.clone(),
                    style: event.style,
                    position: event.position,
                    phase: NotificationPhase::EnterIn,
                    elapsed: 0.0,
                    duration: default_duration(),
                    stack_offset: 0.0,
                    alpha: 0.0,
                },
                Name::new("Notification"),
            ))
            .id();

        queue.notifications.push(entity);

        // Limit stack size
        while queue.notifications.len() > 4 {
            if let Some(&oldest) = queue.notifications.first() {
                commands.entity(oldest).despawn();
                queue.notifications.remove(0);
            }
        }
    }
}

/// System to animate notifications.
pub fn notification_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut notification_query: Query<(Entity, &mut Notification)>,
    mut queue: ResMut<NotificationQueue>,
) {
    for (entity, mut notification) in notification_query.iter_mut() {
        notification.elapsed += time.delta_secs();

        match notification.phase {
            NotificationPhase::EnterIn => {
                // Slide in
                notification.alpha = (notification.elapsed / 0.3).min(1.0);
                if notification.alpha >= 1.0 {
                    notification.phase = NotificationPhase::Hold;
                    notification.elapsed = 0.0;
                }
            }
            NotificationPhase::Hold => {
                notification.alpha = 1.0;
                if notification.elapsed >= notification.duration {
                    notification.phase = NotificationPhase::Exit;
                    notification.elapsed = 0.0;
                }
            }
            NotificationPhase::Exit => {
                // Fade out
                notification.alpha = (1.0 - notification.elapsed / 0.3).max(0.0);

                if notification.alpha <= 0.0 {
                    commands.entity(entity).despawn();
                    queue.notifications.retain(|&e| e != entity);
                }
            }
        }
    }

    // Update stack positions
    let mut offset = 0.0;
    for entity in &queue.notifications {
        if let Ok((_, mut notif)) = notification_query.get_mut(*entity) {
            notif.stack_offset = offset;
            offset += 1.0;
        }
    }
}

/// Get icon text for notification icon.
pub fn get_notification_icon_text(icon: NotificationIcon) -> &'static str {
    match icon {
        NotificationIcon::None => "",
        NotificationIcon::Star => "★",
        NotificationIcon::Coin => "●",
        NotificationIcon::Key => "🔑",
        NotificationIcon::Heart => "❤",
        NotificationIcon::Warning => "⚠",
    }
}

/// Plugin for notification systems.
pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationQueue>()
            .add_message::<NotificationEvent>()
            .add_systems(Update, notification_spawn_system)
            .add_systems(Update, notification_animation_system);
    }
}
