//! Placement tier and semantic role components for world-generation entities.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Placement importance tier — controls generation order and collision behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementTier {
    /// Focal-point entities placed first at hero slots.
    Hero,
    /// Mid-importance entities with collision checks.
    Medium,
    /// Small scatter elements filling residual space.
    Decorative,
    /// Not yet assigned a tier.
    Untiered,
}

impl PlacementTier {
    /// Parse from a string, returning `None` for unrecognized values.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hero" => Some(Self::Hero),
            "medium" => Some(Self::Medium),
            "decorative" => Some(Self::Decorative),
            "untiered" => Some(Self::Untiered),
            _ => None,
        }
    }

    /// Return the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hero => "hero",
            Self::Medium => "medium",
            Self::Decorative => "decorative",
            Self::Untiered => "untiered",
        }
    }
}

impl fmt::Display for PlacementTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Semantic role — what purpose an entity serves in the scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRole {
    Ground,
    Structure,
    Prop,
    Vegetation,
    Decoration,
    Character,
    Lighting,
    Audio,
    Untagged,
}

impl SemanticRole {
    /// Parse from a string, returning `None` for unrecognized values.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ground" => Some(Self::Ground),
            "structure" => Some(Self::Structure),
            "prop" => Some(Self::Prop),
            "vegetation" => Some(Self::Vegetation),
            "decoration" => Some(Self::Decoration),
            "character" => Some(Self::Character),
            "lighting" => Some(Self::Lighting),
            "audio" => Some(Self::Audio),
            "untagged" => Some(Self::Untagged),
            _ => None,
        }
    }

    /// Return the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ground => "ground",
            Self::Structure => "structure",
            Self::Prop => "prop",
            Self::Vegetation => "vegetation",
            Self::Decoration => "decoration",
            Self::Character => "character",
            Self::Lighting => "lighting",
            Self::Audio => "audio",
            Self::Untagged => "untagged",
        }
    }
}

impl fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
