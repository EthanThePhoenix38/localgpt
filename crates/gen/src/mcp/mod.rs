//! MCP tool handlers for LocalGPT Gen.
//!
//! This module contains tool implementations that bridge the agent
//! to Bevy scene manipulation.

pub mod avatar_tools;
pub mod interaction_tools;
pub mod physics_tools;
pub mod terrain_tools;
pub mod ui_tools;

// Re-export character tools from crate root module
pub use crate::character_tools::create_avatar_tools;
