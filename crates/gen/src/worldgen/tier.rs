//! Placement tiers and semantic roles for hierarchical scene composition.
//!
//! PlacementTier controls generation order (hero → medium → decorative).
//! SemanticRole enables bulk operations by category (vegetation, structure, etc.).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Placement tier — controls generation priority and visual hierarchy.
#[derive(
    Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[reflect(Component)]
#[serde(rename_all = "lowercase")]
pub enum PlacementTier {
    /// Major landmarks, buildings, focal points. Placed first.
    Hero,
    /// Trees, walls, bridges, fences, medium structures. Placed second.
    Medium,
    /// Flowers, rocks, grass, small props, ground clutter. Placed last.
    Decorative,
    /// Manually placed entities not part of hierarchical generation.
    #[default]
    Untiered,
}

impl PlacementTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hero => "hero",
            Self::Medium => "medium",
            Self::Decorative => "decorative",
            Self::Untiered => "untiered",
        }
    }

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
}

impl fmt::Display for PlacementTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Semantic role — categorizes entities by function for bulk operations.
#[derive(
    Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[reflect(Component)]
#[serde(rename_all = "lowercase")]
pub enum SemanticRole {
    /// Terrain, floors, ground planes, paths.
    Ground,
    /// Buildings, walls, bridges, fences, stairs.
    Structure,
    /// Furniture, crates, barrels, vehicles, functional objects.
    Prop,
    /// Trees, bushes, grass, flowers.
    Vegetation,
    /// Small visual details, particles, ambient effects.
    Decoration,
    /// NPCs, player.
    Character,
    /// Light sources.
    Lighting,
    /// Sound emitters.
    Audio,
    /// Default for manually placed entities.
    #[default]
    Untagged,
}

impl SemanticRole {
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
}

impl fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_roundtrip() {
        let tier = PlacementTier::Hero;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"hero\"");
        let back: PlacementTier = serde_json::from_str(&json).unwrap();
        assert_eq!(back, tier);
    }

    #[test]
    fn role_roundtrip() {
        let role = SemanticRole::Vegetation;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"vegetation\"");
        let back: SemanticRole = serde_json::from_str(&json).unwrap();
        assert_eq!(back, role);
    }

    #[test]
    fn tier_from_str() {
        assert_eq!(
            PlacementTier::from_str_opt("hero"),
            Some(PlacementTier::Hero)
        );
        assert_eq!(
            PlacementTier::from_str_opt("decorative"),
            Some(PlacementTier::Decorative)
        );
        assert_eq!(PlacementTier::from_str_opt("invalid"), None);
    }

    #[test]
    fn role_from_str() {
        assert_eq!(
            SemanticRole::from_str_opt("ground"),
            Some(SemanticRole::Ground)
        );
        assert_eq!(
            SemanticRole::from_str_opt("structure"),
            Some(SemanticRole::Structure)
        );
        assert_eq!(SemanticRole::from_str_opt("invalid"), None);
    }

    #[test]
    fn all_tiers_roundtrip() {
        for tier in [
            PlacementTier::Hero,
            PlacementTier::Medium,
            PlacementTier::Decorative,
            PlacementTier::Untiered,
        ] {
            let s = tier.as_str();
            assert_eq!(PlacementTier::from_str_opt(s), Some(tier));
        }
    }

    #[test]
    fn all_roles_roundtrip() {
        for role in [
            SemanticRole::Ground,
            SemanticRole::Structure,
            SemanticRole::Prop,
            SemanticRole::Vegetation,
            SemanticRole::Decoration,
            SemanticRole::Character,
            SemanticRole::Lighting,
            SemanticRole::Audio,
            SemanticRole::Untagged,
        ] {
            let s = role.as_str();
            assert_eq!(SemanticRole::from_str_opt(s), Some(role));
        }
    }
}
