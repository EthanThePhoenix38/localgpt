//! Camera mode switching and control.
//!
//! Implements Spec 1.5: `gen_set_camera_mode` — Camera Mode Switching

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Current camera mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CameraPov {
    /// Camera at player eye level, no player mesh visible.
    FirstPerson,
    /// Camera orbits behind player.
    #[default]
    ThirdPerson,
    /// Camera above and behind at fixed pitch.
    TopDown,
    /// Fixed position camera looking at a target.
    Fixed,
}

/// Component for the player's camera.
#[derive(Component, Clone)]
pub struct PlayerCamera {
    /// Current camera mode.
    pub mode: CameraPov,
    /// Distance from player (third-person / top-down).
    pub distance: f32,
    /// Camera pitch in degrees.
    pub pitch: f32,
    /// Camera yaw in degrees.
    pub yaw: f32,
    /// Field of view in degrees.
    pub fov: f32,
    /// Fixed camera position (fixed mode only).
    pub fixed_position: Option<Vec3>,
    /// Fixed camera look-at target (fixed mode only).
    pub fixed_look_at: Option<Vec3>,
    /// Transition duration when switching modes.
    pub transition_duration: f32,
    /// Current transition progress (0.0 - 1.0).
    pub transition_progress: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            mode: CameraPov::ThirdPerson,
            distance: 5.0,
            pitch: -20.0,
            yaw: 0.0,
            fov: 60.0,
            fixed_position: None,
            fixed_look_at: None,
            transition_duration: 0.5,
            transition_progress: 1.0,
        }
    }
}

/// Parameters for setting camera mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCameraModeParams {
    /// Camera mode: "first_person", "third_person", "top_down", "fixed".
    pub mode: String,
    /// Distance from player (third-person / top-down).
    #[serde(default = "default_distance")]
    pub distance: f32,
    /// Initial pitch in degrees.
    #[serde(default = "default_pitch")]
    pub pitch: f32,
    /// Field of view in degrees.
    #[serde(default = "default_fov")]
    pub fov: f32,
    /// Transition duration in seconds.
    #[serde(default = "default_transition")]
    pub transition_duration: f32,
    /// Fixed camera position (fixed mode only).
    #[serde(default)]
    pub fixed_position: Option<[f32; 3]>,
    /// Fixed look-at target (fixed mode only).
    #[serde(default)]
    pub fixed_look_at: Option<[f32; 3]>,
}

fn default_distance() -> f32 {
    5.0
}
fn default_pitch() -> f32 {
    -20.0
}
fn default_fov() -> f32 {
    60.0
}
fn default_transition() -> f32 {
    0.5
}

impl SetCameraModeParams {
    /// Parse mode string to enum.
    pub fn mode_enum(&self) -> CameraPov {
        match self.mode.to_lowercase().as_str() {
            "first_person" | "firstperson" => CameraPov::FirstPerson,
            "top_down" | "topdown" => CameraPov::TopDown,
            "fixed" => CameraPov::Fixed,
            _ => CameraPov::ThirdPerson,
        }
    }
}

/// Resource for mouse sensitivity.
#[derive(Resource, Clone)]
pub struct MouseSensitivity {
    pub x: f32,
    pub y: f32,
}

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self { x: 0.1, y: 0.1 }
    }
}

/// System to update camera position based on mode.
pub fn camera_follow_system(
    _time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &PlayerCamera), Without<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (mut camera_transform, camera) in camera_query.iter_mut() {
        let target_position = match camera.mode {
            CameraPov::FirstPerson => {
                // Camera at eye height
                player_transform.translation + Vec3::new(0.0, 1.6, 0.0)
            }
            CameraPov::ThirdPerson => {
                // Camera behind and above player
                let yaw_rad = camera.yaw.to_radians();
                let pitch_rad = camera.pitch.to_radians();

                let offset = Vec3::new(
                    yaw_rad.sin() * camera.distance * pitch_rad.cos(),
                    -pitch_rad.sin() * camera.distance,
                    yaw_rad.cos() * camera.distance * pitch_rad.cos(),
                );

                player_transform.translation + offset
            }
            CameraPov::TopDown => {
                // Camera above player
                player_transform.translation + Vec3::new(0.0, camera.distance, 0.0)
            }
            CameraPov::Fixed => {
                // Use fixed position
                camera.fixed_position.unwrap_or(Vec3::new(0.0, 10.0, 10.0))
            }
        };

        // Smooth transition
        if camera.transition_progress < 1.0 {
            let t = camera.transition_progress;
            let eased = t * t * (3.0 - 2.0 * t); // Smoothstep

            camera_transform.translation =
                camera_transform.translation.lerp(target_position, eased);
        } else {
            camera_transform.translation = target_position;
        }

        // Update look direction
        let look_target = match camera.mode {
            CameraPov::Fixed => camera.fixed_look_at.unwrap_or(player_transform.translation),
            _ => player_transform.translation,
        };

        camera_transform.look_at(look_target, Vec3::Y);
    }
}

// Import from sibling module
use super::player::Player;

/// Plugin for camera systems.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseSensitivity::default())
            .add_systems(Update, camera_follow_system);
    }
}
