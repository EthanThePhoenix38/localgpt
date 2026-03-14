//! Avatar-based camera control with V key POV switching and Tab key FreeFly toggle.
//!
//! The avatar always exists in the scene. By default the camera follows the avatar
//! in 3rd-person view. V toggles 1st/3rd person. Tab detaches the camera for
//! free-fly spectator mode; Tab again re-attaches.

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use super::commands::PointOfView;
use super::plugin::FlyCam;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// 3rd-person camera offset relative to avatar (up 2.5m, back 5m).
const THIRD_PERSON_OFFSET: Vec3 = Vec3::new(0.0, 2.5, 5.0);

// ---------------------------------------------------------------------------
// Resources & Components
// ---------------------------------------------------------------------------

/// Camera attachment mode.
#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub enum CameraMode {
    /// Camera follows the avatar (1st or 3rd person).
    Attached,
    /// Camera detached, moves independently (spectator).
    #[default]
    FreeFly,
}

/// Which POV is active when camera is attached.
#[derive(Resource)]
pub struct PovState {
    pub pov: PointOfView,
}

impl Default for PovState {
    fn default() -> Self {
        Self {
            pov: PointOfView::ThirdPerson,
        }
    }
}

/// Avatar movement configuration (runtime-adjustable).
#[derive(Resource)]
pub struct AvatarMovementConfig {
    pub move_speed: f32,
    pub look_sensitivity: f32,
    pub eye_height: f32,
    /// Camera pitch angle (radians), clamped ±89°.
    pub pitch: f32,
    /// Yaw angle (radians) — avatar's facing direction.
    pub yaw: f32,
    /// If true, avatar cannot go below `eye_height`.
    pub ground_clamp: bool,
}

impl Default for AvatarMovementConfig {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            look_sensitivity: 0.003,
            eye_height: 1.8,
            pitch: 0.0,
            yaw: 0.0,
            ground_clamp: true,
        }
    }
}

/// Marker component for the avatar mesh entity.
#[derive(Component)]
pub struct AvatarEntity;

// ---------------------------------------------------------------------------
// Run conditions
// ---------------------------------------------------------------------------

pub fn in_attached_mode(mode: Res<CameraMode>) -> bool {
    *mode == CameraMode::Attached
}

pub fn in_freefly_mode(mode: Res<CameraMode>) -> bool {
    *mode == CameraMode::FreeFly
}

// ---------------------------------------------------------------------------
// Movement systems (run when CameraMode == Attached)
// ---------------------------------------------------------------------------

/// WASD moves the avatar relative to its yaw rotation. Space/Shift for vertical.
pub fn avatar_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: ResMut<AvatarMovementConfig>,
    mut query: Query<&mut Transform, With<AvatarEntity>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    // Movement directions based on yaw only (ignoring pitch)
    let yaw_rot = Quat::from_rotation_y(config.yaw);
    let forward = yaw_rot * Vec3::NEG_Z;
    let right = yaw_rot * Vec3::X;

    let mut velocity = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        velocity += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        velocity -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        velocity -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        velocity += right;
    }
    if keys.pressed(KeyCode::Space) {
        velocity += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        velocity -= Vec3::Y;
    }

    if velocity != Vec3::ZERO {
        transform.translation += velocity.normalize() * config.move_speed * time.delta_secs();
    }

    // Ground clamp
    if config.ground_clamp {
        transform.translation.y = transform.translation.y.max(config.eye_height);
    }

    // Keep avatar rotation in sync with yaw (only Y-axis rotation)
    transform.rotation = yaw_rot;
}

/// Right-click + mouse drag to rotate the avatar (yaw) and camera pitch.
pub fn avatar_look(
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion_reader: MessageReader<MouseMotion>,
    mut config: ResMut<AvatarMovementConfig>,
) {
    let delta: Vec2 = motion_reader.read().map(|e| e.delta).sum();
    if delta == Vec2::ZERO || !mouse.pressed(MouseButton::Right) {
        return;
    }

    // Yaw — rotate avatar around Y axis
    config.yaw -= delta.x * config.look_sensitivity;

    // Pitch — camera-only, clamped ±89°
    let max_pitch = 89.0_f32.to_radians();
    config.pitch = (config.pitch - delta.y * config.look_sensitivity).clamp(-max_pitch, max_pitch);
}

/// Scroll wheel adjusts avatar movement speed.
pub fn avatar_scroll_speed(
    mut scroll_reader: MessageReader<MouseWheel>,
    mut config: ResMut<AvatarMovementConfig>,
) {
    for event in scroll_reader.read() {
        config.move_speed = (config.move_speed * (1.0 + event.y * 0.1)).clamp(0.5, 100.0);
    }
}

// ---------------------------------------------------------------------------
// Camera follow system (runs after avatar_movement, when Attached)
// ---------------------------------------------------------------------------

/// Position the camera relative to the avatar based on the current POV.
pub(crate) fn camera_follow_avatar(
    pov_state: Res<PovState>,
    config: Res<AvatarMovementConfig>,
    avatar_q: Query<&Transform, (With<AvatarEntity>, Without<FlyCam>)>,
    mut cam_q: Query<&mut Transform, (With<FlyCam>, Without<AvatarEntity>)>,
    mut vis_q: Query<&mut Visibility, With<AvatarEntity>>,
) {
    let Ok(avatar_tf) = avatar_q.single() else {
        return;
    };
    let Ok(mut cam_tf) = cam_q.single_mut() else {
        return;
    };

    let avatar_pos = avatar_tf.translation;
    let yaw_rot = Quat::from_rotation_y(config.yaw);

    match pov_state.pov {
        PointOfView::FirstPerson => {
            // Camera at avatar eye level
            cam_tf.translation = avatar_pos + Vec3::Y * config.eye_height;
            // Camera rotation = yaw × pitch
            cam_tf.rotation = yaw_rot * Quat::from_rotation_x(config.pitch);
            // Hide avatar mesh
            if let Ok(mut vis) = vis_q.single_mut() {
                *vis = Visibility::Hidden;
            }
        }
        PointOfView::ThirdPerson => {
            // Rotate the offset by avatar yaw
            let world_offset = yaw_rot * THIRD_PERSON_OFFSET;
            cam_tf.translation = avatar_pos + world_offset;
            // Look at the avatar (slightly above center)
            let look_target = avatar_pos + Vec3::Y * config.eye_height * 0.5;
            cam_tf.look_at(look_target, Vec3::Y);
            // Show avatar mesh
            if let Ok(mut vis) = vis_q.single_mut() {
                *vis = Visibility::Inherited;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Toggle systems
// ---------------------------------------------------------------------------

/// V key toggles between 1st and 3rd person (only when attached).
pub fn handle_pov_toggle(keys: Res<ButtonInput<KeyCode>>, mut pov_state: ResMut<PovState>) {
    if keys.just_pressed(KeyCode::KeyV) {
        pov_state.pov = match pov_state.pov {
            PointOfView::FirstPerson => PointOfView::ThirdPerson,
            PointOfView::ThirdPerson => PointOfView::FirstPerson,
        };
        info!("POV toggled to {:?}", pov_state.pov);
    }
}

/// Tab key toggles between Attached and FreeFly camera modes.
pub fn handle_camera_mode_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<CameraMode>,
    mut vis_q: Query<&mut Visibility, With<AvatarEntity>>,
    avatar_config: Res<super::plugin::AvatarConfig>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        *mode = match *mode {
            CameraMode::Attached => {
                // Detaching — avatar stays visible
                if let Ok(mut vis) = vis_q.single_mut() {
                    *vis = Visibility::Inherited;
                }
                info!("Camera mode: FreeFly (detached)");
                CameraMode::FreeFly
            }
            CameraMode::FreeFly => {
                // Only allow attaching if avatar config is active
                if avatar_config.active.is_none() {
                    info!("Camera mode: No avatar configured, staying in FreeFly");
                    return;
                }
                // Re-attaching — camera_follow_avatar will position it next frame
                info!("Camera mode: Attached");
                CameraMode::Attached
            }
        };
    }
}
