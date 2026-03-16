//! Avatar & Character System (P1)
//!
//! These modules implement player and NPC characters with movement,
//! camera following, collision, and dialogue systems.

pub mod camera;
pub mod dialogue;
pub mod npc;
pub mod npc_brain;
pub mod npc_memory;
pub mod player;
pub mod spawn_point;

pub use camera::*;
pub use dialogue::*;
pub use npc::*;
pub use npc_brain::*;
pub use npc_memory::*;
pub use player::*;
pub use spawn_point::*;
