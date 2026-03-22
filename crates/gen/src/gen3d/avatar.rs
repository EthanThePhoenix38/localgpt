//! Camera mode control: FreeFly spectator and Player (physics-based character).
//!
//! Tab toggles between Player ↔ FreeFly when a player entity exists.
//! FreeFly gives WASD + mouse spectator movement. Player mode activates
//! the physics-based character controller (Tnua + Avian).

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Active camera control mode.
///
/// Two modes: FreeFly (spectator) and Player (physics character via gen_spawn_player).
/// Tab cycles between them when a player entity exists.
#[derive(Resource, Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CameraMode {
    /// Camera detached, moves independently (spectator). WASD + mouse look.
    #[default]
    FreeFly,
    /// Camera follows the physics-based player entity (Tnua + Avian).
    /// Activated by gen_spawn_player. WASD/Space/Shift go to player_input_system.
    Player,
}

// ---------------------------------------------------------------------------
// Run conditions
// ---------------------------------------------------------------------------

pub fn in_freefly_mode(mode: Res<CameraMode>) -> bool {
    *mode == CameraMode::FreeFly
}

/// Run condition: camera is following the physics-based player (gen_spawn_player).
pub fn in_player_mode(mode: Res<CameraMode>) -> bool {
    *mode == CameraMode::Player
}

// ---------------------------------------------------------------------------
// Toggle system
// ---------------------------------------------------------------------------

/// Tab key toggles between Player and FreeFly camera modes.
///
/// If a Player entity exists: Player ↔ FreeFly.
/// If no Player: stays in FreeFly (nothing to attach to).
pub fn handle_camera_mode_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<CameraMode>,
    player_q: Query<Entity, With<crate::character::Player>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    let has_player = player_q.iter().next().is_some();

    *mode = match *mode {
        CameraMode::Player => {
            info!("Camera mode: FreeFly (detached from player)");
            CameraMode::FreeFly
        }
        CameraMode::FreeFly => {
            if has_player {
                info!("Camera mode: Player");
                CameraMode::Player
            } else {
                info!("Camera mode: No player spawned, staying in FreeFly");
                return;
            }
        }
    };
}
