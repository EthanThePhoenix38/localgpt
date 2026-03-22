//! Camera mode switching and control.
//!
//! Implements Spec 1.5: `gen_set_camera_mode` — Camera Mode Switching

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "physics")]
use avian3d::prelude::*;

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

/// System to handle camera input (mouse look).
pub fn camera_input_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_query: Query<&mut PlayerCamera>,
    mut player_query: Query<&mut Transform, With<Player>>,
    sensitivity: Res<MouseSensitivity>,
) {
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    for event in mouse_motion.read() {
        match camera.mode {
            CameraPov::FirstPerson | CameraPov::ThirdPerson => {
                // Yaw rotates player
                player_transform.rotate_y(-event.delta.x * sensitivity.x.to_radians());

                // Pitch rotates camera (clamped)
                camera.pitch -= event.delta.y * sensitivity.y;
                camera.pitch = camera.pitch.clamp(-89.0, 89.0);
            }
            _ => {}
        }
    }
}

/// System to update camera position based on mode and spring arm (with physics collision avoidance).
#[cfg(feature = "physics")]
pub fn camera_follow_system(
    time: Res<Time>,
    spatial_query: Option<SpatialQuery>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
) {
    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };

    for (mut camera_transform, mut camera) in camera_query.iter_mut() {
        let mut target_position = compute_target_position(&camera, player_transform);

        // Spring Arm (Collision Avoidance) for Third Person
        if camera.mode == CameraPov::ThirdPerson {
            if let Some(ref spatial_query) = spatial_query {
                let origin = player_transform.translation + Vec3::new(0.0, 1.5, 0.0);
                let dir = target_position - origin;
                let dist = dir.length();

                if dist > 0.001 {
                    let direction = dir.normalize();

                    // Raycast to check for obstructions (filter out player entity)
                    let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

                    if let Some(hit) = spatial_query.cast_ray(
                        origin,
                        Dir3::new(direction).unwrap(),
                        dist,
                        true,
                        &filter,
                    ) {
                        // Hit something, pull camera in
                        target_position = origin + direction * (hit.distance - 0.2).max(0.5);
                    }
                }
            }
        }

        apply_camera_transform(
            &time,
            &mut camera_transform,
            &mut camera,
            target_position,
            player_transform,
        );
    }
}

/// System to update camera position based on mode (without physics collision avoidance).
#[cfg(not(feature = "physics"))]
pub fn camera_follow_system(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (mut camera_transform, mut camera) in camera_query.iter_mut() {
        let target_position = compute_target_position(&camera, player_transform);

        apply_camera_transform(
            &time,
            &mut camera_transform,
            &mut camera,
            target_position,
            player_transform,
        );
    }
}

/// Compute the desired camera target position based on mode and player transform.
fn compute_target_position(camera: &PlayerCamera, player_transform: &Transform) -> Vec3 {
    match camera.mode {
        CameraPov::FirstPerson => {
            // Camera at eye height (approx 1.6m)
            player_transform.translation + Vec3::new(0.0, 1.6, 0.0)
        }
        CameraPov::ThirdPerson => {
            let player_yaw = player_transform.rotation.to_euler(EulerRot::YXZ).0;
            let pitch_rad = camera.pitch.to_radians();

            // Calculate offset in local space then rotate by player yaw
            let local_offset = Vec3::new(
                0.0,
                -pitch_rad.sin() * camera.distance,
                pitch_rad.cos() * camera.distance,
            );

            let rotation = Quat::from_rotation_y(player_yaw);
            let world_offset = rotation * local_offset;

            player_transform.translation + Vec3::new(0.0, 1.5, 0.0) + world_offset
        }
        CameraPov::TopDown => {
            // Camera above player
            player_transform.translation + Vec3::new(0.0, camera.distance, 0.0)
        }
        CameraPov::Fixed => {
            // Use fixed position
            camera.fixed_position.unwrap_or(Vec3::new(0.0, 10.0, 10.0))
        }
    }
}

/// Apply camera transform: smooth transition + look direction.
fn apply_camera_transform(
    time: &Time,
    camera_transform: &mut Transform,
    camera: &mut PlayerCamera,
    target_position: Vec3,
    player_transform: &Transform,
) {
    // Smooth transition
    if camera.transition_progress < 1.0 {
        camera.transition_progress += time.delta_secs() / camera.transition_duration;
        camera.transition_progress = camera.transition_progress.min(1.0);

        let t = camera.transition_progress;
        let eased = t * t * (3.0 - 2.0 * t); // Smoothstep

        camera_transform.translation = camera_transform.translation.lerp(target_position, eased);
    } else {
        camera_transform.translation = target_position;
    }

    // Update look direction
    let look_target = match camera.mode {
        CameraPov::Fixed => camera.fixed_look_at.unwrap_or(player_transform.translation),
        CameraPov::ThirdPerson | CameraPov::TopDown => {
            player_transform.translation + Vec3::new(0.0, 1.0, 0.0)
        }
        CameraPov::FirstPerson => {
            // Look forward based on player yaw and camera pitch
            let player_yaw = player_transform.rotation.to_euler(EulerRot::YXZ).0;
            let pitch_rad = camera.pitch.to_radians();

            let forward = Vec3::new(
                player_yaw.sin() * pitch_rad.cos(),
                pitch_rad.sin(),
                player_yaw.cos() * pitch_rad.cos(),
            );
            camera_transform.translation + forward
        }
    };

    camera_transform.look_at(look_target, Vec3::Y);
}

// Import from sibling module
use super::player::Player;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_camera_default() {
        let cam = PlayerCamera::default();
        assert_eq!(cam.mode, CameraPov::ThirdPerson);
        assert_eq!(cam.distance, 5.0);
        assert_eq!(cam.pitch, -20.0);
        assert_eq!(cam.fov, 60.0);
        assert!(cam.fixed_position.is_none());
    }

    #[test]
    fn test_set_camera_mode_params_enum() {
        let params = SetCameraModeParams {
            mode: "first_person".to_string(),
            distance: 5.0,
            pitch: -20.0,
            fov: 60.0,
            transition_duration: 0.5,
            fixed_position: None,
            fixed_look_at: None,
        };
        assert_eq!(params.mode_enum(), CameraPov::FirstPerson);

        let params2 = SetCameraModeParams {
            mode: "top_down".to_string(),
            ..params.clone()
        };
        assert_eq!(params2.mode_enum(), CameraPov::TopDown);

        let params3 = SetCameraModeParams {
            mode: "fixed".to_string(),
            ..params
        };
        assert_eq!(params3.mode_enum(), CameraPov::Fixed);
    }

    #[test]
    fn test_mouse_sensitivity_default() {
        let sens = MouseSensitivity::default();
        assert_eq!(sens.x, 0.1);
        assert_eq!(sens.y, 0.1);
    }

    #[test]
    fn test_camera_pov_variants() {
        assert_eq!(CameraPov::default(), CameraPov::ThirdPerson);
        assert_ne!(CameraPov::FirstPerson, CameraPov::ThirdPerson);
        assert_ne!(CameraPov::TopDown, CameraPov::Fixed);
    }

    #[test]
    fn test_mode_enum_aliases() {
        let params = SetCameraModeParams {
            mode: "firstperson".to_string(),
            distance: 5.0,
            pitch: -20.0,
            fov: 60.0,
            transition_duration: 0.5,
            fixed_position: None,
            fixed_look_at: None,
        };
        assert_eq!(params.mode_enum(), CameraPov::FirstPerson);

        let params2 = SetCameraModeParams {
            mode: "topdown".to_string(),
            ..params.clone()
        };
        assert_eq!(params2.mode_enum(), CameraPov::TopDown);

        // Unknown falls back to ThirdPerson
        let params3 = SetCameraModeParams {
            mode: "invalid".to_string(),
            ..params
        };
        assert_eq!(params3.mode_enum(), CameraPov::ThirdPerson);
    }

    #[test]
    fn test_compute_first_person_position() {
        let camera = PlayerCamera {
            mode: CameraPov::FirstPerson,
            ..default()
        };
        let player = Transform::from_translation(Vec3::new(1.0, 0.0, 2.0));
        let pos = compute_target_position(&camera, &player);
        assert_eq!(pos, Vec3::new(1.0, 1.6, 2.0));
    }

    #[test]
    fn test_compute_top_down_position() {
        let camera = PlayerCamera {
            mode: CameraPov::TopDown,
            distance: 10.0,
            ..default()
        };
        let player = Transform::from_translation(Vec3::new(3.0, 0.0, 5.0));
        let pos = compute_target_position(&camera, &player);
        assert_eq!(pos, Vec3::new(3.0, 10.0, 5.0));
    }

    #[test]
    fn test_compute_fixed_position() {
        let camera = PlayerCamera {
            mode: CameraPov::Fixed,
            fixed_position: Some(Vec3::new(10.0, 20.0, 30.0)),
            ..default()
        };
        let player = Transform::IDENTITY;
        let pos = compute_target_position(&camera, &player);
        assert_eq!(pos, Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_compute_fixed_position_default() {
        let camera = PlayerCamera {
            mode: CameraPov::Fixed,
            fixed_position: None,
            ..default()
        };
        let player = Transform::IDENTITY;
        let pos = compute_target_position(&camera, &player);
        // Default fixed position
        assert_eq!(pos, Vec3::new(0.0, 10.0, 10.0));
    }

    #[test]
    fn test_player_camera_transition() {
        let cam = PlayerCamera {
            transition_progress: 0.5,
            transition_duration: 1.0,
            ..default()
        };
        assert!(cam.transition_progress < 1.0);
        assert_eq!(cam.transition_duration, 1.0);
    }
}

/// System to toggle camera POV between ThirdPerson and FirstPerson with the V key.
///
/// Resets `transition_progress` to trigger smooth blend between views.
pub fn camera_pov_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut PlayerCamera>,
    mut notifications: MessageWriter<crate::ui::notification::NotificationEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyV) {
        return;
    }

    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };

    camera.mode = match camera.mode {
        CameraPov::ThirdPerson => CameraPov::FirstPerson,
        CameraPov::FirstPerson => CameraPov::ThirdPerson,
        // For other modes (TopDown, Fixed), toggle to ThirdPerson
        _ => CameraPov::ThirdPerson,
    };

    // Reset transition for smooth blend
    camera.transition_progress = 0.0;

    let label = match camera.mode {
        CameraPov::FirstPerson => "First Person",
        CameraPov::ThirdPerson => "Third Person",
        _ => "Camera",
    };
    info!("Camera POV: {label}");
    notifications.write(crate::ui::notification::NotificationEvent {
        text: format!("Camera: {label}"),
        style: crate::ui::notification::NotificationStyle::Toast,
        position: crate::ui::notification::NotificationPosition::Top,
        icon: crate::ui::notification::NotificationIcon::None,
    });
}

/// Hide/show the player mesh based on first-person mode.
/// In first-person, the player capsule is inside the camera, so hide it.
pub fn player_mesh_visibility_system(
    camera_query: Query<&PlayerCamera>,
    mut player_query: Query<&mut Visibility, With<Player>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    for mut visibility in player_query.iter_mut() {
        *visibility = if camera.mode == CameraPov::FirstPerson {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// Plugin for camera systems.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseSensitivity::default())
            .add_systems(
                Update,
                (
                    camera_pov_toggle_system,
                    camera_input_system,
                    camera_follow_system,
                    player_mesh_visibility_system,
                )
                    .chain()
                    .run_if(crate::gen3d::avatar::in_player_mode),
            );
    }
}
