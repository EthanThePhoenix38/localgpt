//! Connectivity-ordered generation — ensures layers are built in dependency order.
//!
//! Layer order:
//! 0: Ground     — terrain, paths, water, floors
//! 1: Structure  — buildings, walls, bridges, stairs, fences
//! 2: Prop       — furniture, vehicles, crates
//! 3: Vegetation — trees, bushes, grass
//! 4: Decoration — small visual props, particles
//! 5: Character  — NPCs (on walkable surfaces)
//! 6: Lighting   — lights
//! 7: Audio      — sound emitters

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::tier::SemanticRole;

/// Generation layer — defines the order in which entity types are generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationLayer {
    Ground = 0,
    Structure = 1,
    Prop = 2,
    Vegetation = 3,
    Decoration = 4,
    Character = 5,
    Lighting = 6,
    Audio = 7,
}

impl GenerationLayer {
    /// Get the generation layer for a semantic role.
    pub fn from_role(role: &SemanticRole) -> Self {
        match role {
            SemanticRole::Ground => GenerationLayer::Ground,
            SemanticRole::Structure => GenerationLayer::Structure,
            SemanticRole::Prop => GenerationLayer::Prop,
            SemanticRole::Vegetation => GenerationLayer::Vegetation,
            SemanticRole::Decoration => GenerationLayer::Decoration,
            SemanticRole::Character => GenerationLayer::Character,
            SemanticRole::Lighting => GenerationLayer::Lighting,
            SemanticRole::Audio => GenerationLayer::Audio,
            SemanticRole::Untagged => GenerationLayer::Decoration,
        }
    }

    /// Get the dependencies for this layer (layers that must be complete before this one).
    pub fn dependencies(&self) -> &[GenerationLayer] {
        match self {
            GenerationLayer::Ground => &[],
            GenerationLayer::Structure => &[GenerationLayer::Ground],
            GenerationLayer::Prop => &[GenerationLayer::Ground, GenerationLayer::Structure],
            GenerationLayer::Vegetation => &[GenerationLayer::Ground],
            GenerationLayer::Decoration => &[
                GenerationLayer::Ground,
                GenerationLayer::Structure,
                GenerationLayer::Prop,
            ],
            GenerationLayer::Character => &[GenerationLayer::Ground, GenerationLayer::Structure],
            GenerationLayer::Lighting => &[GenerationLayer::Ground, GenerationLayer::Structure],
            GenerationLayer::Audio => &[GenerationLayer::Ground, GenerationLayer::Structure],
        }
    }

    /// All layers in generation order.
    pub fn all() -> &'static [GenerationLayer] {
        &[
            GenerationLayer::Ground,
            GenerationLayer::Structure,
            GenerationLayer::Prop,
            GenerationLayer::Vegetation,
            GenerationLayer::Decoration,
            GenerationLayer::Character,
            GenerationLayer::Lighting,
            GenerationLayer::Audio,
        ]
    }
}

/// Status of a generation layer within a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerStatus {
    /// Layer has not been started.
    NotStarted,
    /// Layer is currently being generated.
    InProgress,
    /// Layer generation is complete.
    Complete,
    /// Layer needs re-validation due to lower layer changes.
    NeedsRevalidation,
}

/// Tracks generation progress per region per layer.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct GenerationState {
    /// region_id → layer statuses (indexed by GenerationLayer as usize)
    pub layers: HashMap<String, [LayerStatus; 8]>,
}

impl GenerationState {
    /// Initialize a new region with all layers NotStarted.
    pub fn init_region(&mut self, region_id: &str) {
        self.layers
            .entry(region_id.to_string())
            .or_insert([LayerStatus::NotStarted; 8]);
    }

    /// Get the status of a specific layer in a region.
    pub fn get_status(&self, region_id: &str, layer: GenerationLayer) -> LayerStatus {
        self.layers
            .get(region_id)
            .map(|layers| layers[layer as usize])
            .unwrap_or(LayerStatus::NotStarted)
    }

    /// Set the status of a specific layer in a region.
    pub fn set_status(&mut self, region_id: &str, layer: GenerationLayer, status: LayerStatus) {
        let layers = self
            .layers
            .entry(region_id.to_string())
            .or_insert([LayerStatus::NotStarted; 8]);
        layers[layer as usize] = status;
    }

    /// Check if all dependencies for a layer are met in a region.
    pub fn dependencies_met(&self, region_id: &str, layer: GenerationLayer) -> bool {
        for dep in layer.dependencies() {
            let status = self.get_status(region_id, *dep);
            if status != LayerStatus::Complete {
                return false;
            }
        }
        true
    }

    /// Mark higher layers as needing revalidation when a lower layer changes.
    pub fn invalidate_above(&mut self, region_id: &str, changed_layer: GenerationLayer) {
        let layers = self
            .layers
            .entry(region_id.to_string())
            .or_insert([LayerStatus::NotStarted; 8]);

        let threshold = changed_layer as usize;
        for layer in &mut layers[(threshold + 1)..] {
            if *layer == LayerStatus::Complete {
                *layer = LayerStatus::NeedsRevalidation;
            }
        }
    }

    /// Get missing dependencies for a layer in a region.
    pub fn missing_dependencies(
        &self,
        region_id: &str,
        layer: GenerationLayer,
    ) -> Vec<GenerationLayer> {
        layer
            .dependencies()
            .iter()
            .filter(|dep| self.get_status(region_id, **dep) != LayerStatus::Complete)
            .copied()
            .collect()
    }

    /// Get the next layer that should be generated for a region.
    pub fn next_layer(&self, region_id: &str) -> Option<GenerationLayer> {
        for layer in GenerationLayer::all() {
            let status = self.get_status(region_id, *layer);
            if (status == LayerStatus::NotStarted || status == LayerStatus::NeedsRevalidation)
                && self.dependencies_met(region_id, *layer)
            {
                return Some(*layer);
            }
        }
        None
    }

    /// Check if all layers are complete for a region.
    pub fn is_complete(&self, region_id: &str) -> bool {
        self.layers
            .get(region_id)
            .map(|layers| layers.iter().all(|s| *s == LayerStatus::Complete))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_order() {
        assert!(GenerationLayer::Ground < GenerationLayer::Structure);
        assert!(GenerationLayer::Structure < GenerationLayer::Prop);
        assert!(GenerationLayer::Prop < GenerationLayer::Vegetation);
    }

    #[test]
    fn test_ground_has_no_deps() {
        assert!(GenerationLayer::Ground.dependencies().is_empty());
    }

    #[test]
    fn test_structure_depends_on_ground() {
        let deps = GenerationLayer::Structure.dependencies();
        assert!(deps.contains(&GenerationLayer::Ground));
    }

    #[test]
    fn test_new_region_all_not_started() {
        let mut state = GenerationState::default();
        state.init_region("village");
        for layer in GenerationLayer::all() {
            assert_eq!(state.get_status("village", *layer), LayerStatus::NotStarted);
        }
    }

    #[test]
    fn test_dependencies_met_ground() {
        let state = GenerationState::default();
        assert!(state.dependencies_met("village", GenerationLayer::Ground));
    }

    #[test]
    fn test_dependencies_not_met_structure() {
        let state = GenerationState::default();
        assert!(!state.dependencies_met("village", GenerationLayer::Structure));
    }

    #[test]
    fn test_dependencies_met_after_ground_complete() {
        let mut state = GenerationState::default();
        state.init_region("village");
        state.set_status("village", GenerationLayer::Ground, LayerStatus::Complete);
        assert!(state.dependencies_met("village", GenerationLayer::Structure));
    }

    #[test]
    fn test_next_layer() {
        let mut state = GenerationState::default();
        state.init_region("village");
        assert_eq!(state.next_layer("village"), Some(GenerationLayer::Ground));
        state.set_status("village", GenerationLayer::Ground, LayerStatus::Complete);
        // Structure, Vegetation, and Lighting all have deps met now, but Structure comes first
        assert_eq!(
            state.next_layer("village"),
            Some(GenerationLayer::Structure)
        );
    }

    #[test]
    fn test_invalidate_above() {
        let mut state = GenerationState::default();
        state.init_region("village");
        for layer in GenerationLayer::all() {
            state.set_status("village", *layer, LayerStatus::Complete);
        }
        assert!(state.is_complete("village"));

        // Ground changes → everything above needs revalidation
        state.invalidate_above("village", GenerationLayer::Ground);
        assert_eq!(
            state.get_status("village", GenerationLayer::Ground),
            LayerStatus::Complete
        );
        assert_eq!(
            state.get_status("village", GenerationLayer::Structure),
            LayerStatus::NeedsRevalidation
        );
        assert_eq!(
            state.get_status("village", GenerationLayer::Audio),
            LayerStatus::NeedsRevalidation
        );
    }

    #[test]
    fn test_missing_dependencies() {
        let state = GenerationState::default();
        let missing = state.missing_dependencies("village", GenerationLayer::Prop);
        assert!(missing.contains(&GenerationLayer::Ground));
        assert!(missing.contains(&GenerationLayer::Structure));
    }

    #[test]
    fn test_from_role() {
        assert_eq!(
            GenerationLayer::from_role(&SemanticRole::Ground),
            GenerationLayer::Ground
        );
        assert_eq!(
            GenerationLayer::from_role(&SemanticRole::Vegetation),
            GenerationLayer::Vegetation
        );
        assert_eq!(
            GenerationLayer::from_role(&SemanticRole::Character),
            GenerationLayer::Character
        );
    }
}
