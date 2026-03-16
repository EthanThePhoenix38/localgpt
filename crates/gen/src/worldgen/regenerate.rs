//! Incremental regeneration — only regenerate content in dirty blockout regions.
//!
//! When the blockout changes, this module determines what needs to be regenerated
//! and in what order. Unchanged regions keep their entities.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Type of change that triggered dirty status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Region bounds changed (resize or move).
    BoundsChanged,
    /// Density parameters changed.
    DensityChanged,
    /// Region was removed.
    Removed,
    /// Terrain parameters changed.
    TerrainChanged,
    /// Region was added.
    Added,
}

/// A dirty region that needs regeneration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirtyRegion {
    pub region_id: String,
    pub change_type: ChangeType,
}

/// Preview of what regeneration will do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerationPreview {
    pub regions: Vec<RegionPreview>,
    pub navmesh_rebuild: bool,
    pub total_entities_removed: usize,
    pub total_entities_estimated: usize,
}

/// Preview for a single region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionPreview {
    pub region_id: String,
    pub change_type: ChangeType,
    pub entities_to_remove: usize,
    pub entities_estimated_new: usize,
    pub description: String,
}

/// Content-addressable cache key for a region's parameters.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RegionCacheKey {
    pub region_id: String,
    pub bounds_hash: u64,
    pub density_hash: u64,
    pub seed: u32,
}

/// Cache of previously generated entity lists.
#[derive(Debug, Clone, Default)]
pub struct RegenerationCache {
    /// region_id → (cache_key, entity_names)
    pub entries: HashMap<String, (RegionCacheKey, Vec<String>)>,
}

impl RegenerationCache {
    /// Check if a region's content is still valid (parameters haven't changed).
    pub fn is_valid(&self, key: &RegionCacheKey) -> bool {
        self.entries
            .get(&key.region_id)
            .map(|(cached_key, _)| cached_key == key)
            .unwrap_or(false)
    }

    /// Store generated entity names for a region.
    pub fn store(&mut self, key: RegionCacheKey, entity_names: Vec<String>) {
        self.entries
            .insert(key.region_id.clone(), (key, entity_names));
    }

    /// Invalidate a region's cache.
    pub fn invalidate(&mut self, region_id: &str) {
        self.entries.remove(region_id);
    }
}

/// Plan what needs to be regenerated based on dirty flags.
pub fn plan_regeneration(
    dirty: &[DirtyRegion],
    existing_entity_counts: &HashMap<String, usize>,
    density_map: &HashMap<String, f32>,
    area_map: &HashMap<String, f32>,
) -> RegenerationPreview {
    let mut preview = RegenerationPreview {
        regions: Vec::new(),
        navmesh_rebuild: false,
        total_entities_removed: 0,
        total_entities_estimated: 0,
    };

    // Sort dirty regions by processing order
    let mut sorted_dirty: Vec<_> = dirty.to_vec();
    sorted_dirty.sort_by_key(|d| match d.change_type {
        ChangeType::TerrainChanged => 0,
        ChangeType::Removed => 1,
        ChangeType::BoundsChanged => 2,
        ChangeType::DensityChanged => 3,
        ChangeType::Added => 4,
    });

    for dirty_region in &sorted_dirty {
        let entity_count = existing_entity_counts
            .get(&dirty_region.region_id)
            .copied()
            .unwrap_or(0);

        let estimated_new = match dirty_region.change_type {
            ChangeType::Removed => 0,
            ChangeType::DensityChanged => {
                let density = density_map
                    .get(&dirty_region.region_id)
                    .copied()
                    .unwrap_or(0.5);
                let area = area_map
                    .get(&dirty_region.region_id)
                    .copied()
                    .unwrap_or(100.0);
                (area * density * 0.2).round() as usize
            }
            _ => entity_count.max(5),
        };

        let description = match dirty_region.change_type {
            ChangeType::BoundsChanged => format!(
                "{} entities will be removed, ~{} regenerated at new bounds",
                entity_count, estimated_new
            ),
            ChangeType::DensityChanged => format!(
                "density change: ~{} entities expected (was {})",
                estimated_new, entity_count
            ),
            ChangeType::Removed => format!("{} entities will be removed", entity_count),
            ChangeType::TerrainChanged => {
                format!("{} entities will snap to new terrain height", entity_count)
            }
            ChangeType::Added => format!("new region, ~{} entities will be generated", estimated_new),
        };

        preview.total_entities_removed += entity_count;
        preview.total_entities_estimated += estimated_new;

        preview.regions.push(RegionPreview {
            region_id: dirty_region.region_id.clone(),
            change_type: dirty_region.change_type.clone(),
            entities_to_remove: entity_count,
            entities_estimated_new: estimated_new,
            description,
        });
    }

    // Navmesh rebuild needed if any bounds or terrain changed
    preview.navmesh_rebuild = sorted_dirty.iter().any(|d| {
        matches!(
            d.change_type,
            ChangeType::BoundsChanged | ChangeType::TerrainChanged | ChangeType::Removed | ChangeType::Added
        )
    });

    preview
}

/// Determine which entities to remove during regeneration.
/// Returns entity names that should be despawned.
pub fn entities_to_remove(
    region_id: &str,
    change_type: &ChangeType,
    all_entities: &[(String, String, bool)], // (name, region_id, is_manual)
    preserve_manual: bool,
) -> Vec<String> {
    all_entities
        .iter()
        .filter(|(_, rid, is_manual)| {
            rid == region_id && !(preserve_manual && *is_manual)
        })
        .map(|(name, _, _)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_empty() {
        let preview = plan_regeneration(&[], &HashMap::new(), &HashMap::new(), &HashMap::new());
        assert!(preview.regions.is_empty());
        assert!(!preview.navmesh_rebuild);
    }

    #[test]
    fn test_plan_removed_region() {
        let dirty = vec![DirtyRegion {
            region_id: "village".to_string(),
            change_type: ChangeType::Removed,
        }];
        let mut counts = HashMap::new();
        counts.insert("village".to_string(), 15);

        let preview = plan_regeneration(&dirty, &counts, &HashMap::new(), &HashMap::new());
        assert_eq!(preview.regions.len(), 1);
        assert_eq!(preview.total_entities_removed, 15);
        assert!(preview.navmesh_rebuild);
    }

    #[test]
    fn test_plan_density_change() {
        let dirty = vec![DirtyRegion {
            region_id: "forest".to_string(),
            change_type: ChangeType::DensityChanged,
        }];
        let mut counts = HashMap::new();
        counts.insert("forest".to_string(), 20);
        let mut density = HashMap::new();
        density.insert("forest".to_string(), 0.8f32);
        let mut area = HashMap::new();
        area.insert("forest".to_string(), 400.0f32);

        let preview = plan_regeneration(&dirty, &counts, &density, &area);
        assert_eq!(preview.regions.len(), 1);
        assert!(!preview.navmesh_rebuild); // Density changes don't need navmesh rebuild
    }

    #[test]
    fn test_processing_order() {
        let dirty = vec![
            DirtyRegion {
                region_id: "a".to_string(),
                change_type: ChangeType::DensityChanged,
            },
            DirtyRegion {
                region_id: "b".to_string(),
                change_type: ChangeType::TerrainChanged,
            },
            DirtyRegion {
                region_id: "c".to_string(),
                change_type: ChangeType::Removed,
            },
        ];
        let preview = plan_regeneration(&dirty, &HashMap::new(), &HashMap::new(), &HashMap::new());
        // Terrain first, then removed, then density
        assert_eq!(preview.regions[0].region_id, "b");
        assert_eq!(preview.regions[1].region_id, "c");
        assert_eq!(preview.regions[2].region_id, "a");
    }

    #[test]
    fn test_entities_to_remove_preserves_manual() {
        let entities = vec![
            ("tree_1".to_string(), "forest".to_string(), false),
            ("tree_2".to_string(), "forest".to_string(), false),
            ("custom_statue".to_string(), "forest".to_string(), true),
        ];
        let to_remove = entities_to_remove("forest", &ChangeType::BoundsChanged, &entities, true);
        assert_eq!(to_remove.len(), 2);
        assert!(!to_remove.contains(&"custom_statue".to_string()));
    }

    #[test]
    fn test_entities_to_remove_no_preserve() {
        let entities = vec![
            ("tree_1".to_string(), "forest".to_string(), false),
            ("custom_statue".to_string(), "forest".to_string(), true),
        ];
        let to_remove = entities_to_remove("forest", &ChangeType::Removed, &entities, false);
        assert_eq!(to_remove.len(), 2);
    }

    #[test]
    fn test_cache_valid() {
        let mut cache = RegenerationCache::default();
        let key = RegionCacheKey {
            region_id: "village".to_string(),
            bounds_hash: 123,
            density_hash: 456,
            seed: 42,
        };
        cache.store(key.clone(), vec!["ent_1".to_string()]);
        assert!(cache.is_valid(&key));

        let different_key = RegionCacheKey {
            region_id: "village".to_string(),
            bounds_hash: 999,
            density_hash: 456,
            seed: 42,
        };
        assert!(!cache.is_valid(&different_key));
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = RegenerationCache::default();
        let key = RegionCacheKey {
            region_id: "village".to_string(),
            bounds_hash: 123,
            density_hash: 456,
            seed: 42,
        };
        cache.store(key.clone(), vec![]);
        assert!(cache.is_valid(&key));
        cache.invalidate("village");
        assert!(!cache.is_valid(&key));
    }
}
