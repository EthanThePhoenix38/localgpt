//! Region-level dirty tracking for incremental sync and drift detection.
//!
//! Uses latching: once a region is dirty, it stays dirty until explicitly cleared.

use bevy::prelude::*;
use std::collections::HashSet;

use super::registry::RegionMember;

/// Tracks which regions have been modified since last sync/drift check.
/// Uses latching: once a region is dirty, it stays dirty until explicitly cleared.
#[derive(Resource, Default)]
pub struct RegionDirtyFlags {
    pub dirty: HashSet<String>,
}

#[allow(dead_code)]
impl RegionDirtyFlags {
    pub fn mark_dirty(&mut self, region_id: &str) {
        self.dirty.insert(region_id.to_string());
    }

    pub fn clear(&mut self, region_id: &str) {
        self.dirty.remove(region_id);
    }

    pub fn clear_all(&mut self) {
        self.dirty.clear();
    }

    pub fn is_dirty(&self, region_id: &str) -> bool {
        self.dirty.contains(region_id)
    }
}

/// System that runs each frame to detect entity changes and latch dirty flags.
#[allow(clippy::type_complexity)]
pub fn track_region_dirty(
    mut flags: ResMut<RegionDirtyFlags>,
    changed: Query<&RegionMember, Or<(Changed<Transform>, Changed<Visibility>)>>,
) {
    for member in &changed {
        flags.dirty.insert(member.region_id.clone());
    }
}
