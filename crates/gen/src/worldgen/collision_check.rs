//! Collision-aware placement — validates entity placements against existing geometry.
//!
//! Before finalizing medium-tier placements:
//! 1. AABB overlap test against existing entities
//! 2. Ground snap via terrain height lookup
//! 3. Minimum clearance enforcement between tiers

use serde::{Deserialize, Serialize};

/// AABB (axis-aligned bounding box) for overlap testing.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    pub fn new(center: [f32; 3], half_extents: [f32; 3]) -> Self {
        Self {
            min: [
                center[0] - half_extents[0],
                center[1] - half_extents[1],
                center[2] - half_extents[2],
            ],
            max: [
                center[0] + half_extents[0],
                center[1] + half_extents[1],
                center[2] + half_extents[2],
            ],
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Expand by a clearance margin.
    pub fn expanded(&self, margin: f32) -> Self {
        Self {
            min: [
                self.min[0] - margin,
                self.min[1] - margin,
                self.min[2] - margin,
            ],
            max: [
                self.max[0] + margin,
                self.max[1] + margin,
                self.max[2] + margin,
            ],
        }
    }
}

/// Result of a placement check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlacementResult {
    /// Placement is valid at the original position.
    Valid { position: [f32; 3] },
    /// Placement was adjusted to avoid collision.
    Adjusted {
        original: [f32; 3],
        adjusted: [f32; 3],
        reason: String,
    },
    /// Placement was rejected (no valid position found).
    Rejected { position: [f32; 3], reason: String },
}

/// Check if a proposed placement overlaps any existing entities.
///
/// `proposed` — AABB of the new entity at the proposed position.
/// `clearance` — minimum clearance from existing entities.
/// `existing` — list of existing entity AABBs.
/// `max_attempts` — number of alternative positions to try.
pub fn check_placement(
    proposed: Aabb,
    clearance: f32,
    existing: &[Aabb],
    max_attempts: u32,
) -> PlacementResult {
    let expanded = proposed.expanded(clearance);

    // Check for overlaps
    let has_overlap = existing.iter().any(|e| expanded.intersects(e));

    if !has_overlap {
        return PlacementResult::Valid {
            position: proposed.center(),
        };
    }

    // Try alternative positions
    let original = proposed.center();
    let half_w = (proposed.max[0] - proposed.min[0]) * 0.5;
    let half_d = (proposed.max[2] - proposed.min[2]) * 0.5;

    let offsets: Vec<[f32; 2]> = vec![
        [half_w * 2.0, 0.0],
        [-half_w * 2.0, 0.0],
        [0.0, half_d * 2.0],
        [0.0, -half_d * 2.0],
        [half_w * 1.5, half_d * 1.5],
        [-half_w * 1.5, half_d * 1.5],
        [half_w * 1.5, -half_d * 1.5],
        [-half_w * 1.5, -half_d * 1.5],
    ];

    for (i, offset) in offsets.iter().enumerate() {
        if i >= max_attempts as usize {
            break;
        }

        let alt_center = [
            original[0] + offset[0],
            original[1],
            original[2] + offset[1],
        ];
        let alt_half = [
            (proposed.max[0] - proposed.min[0]) * 0.5,
            (proposed.max[1] - proposed.min[1]) * 0.5,
            (proposed.max[2] - proposed.min[2]) * 0.5,
        ];
        let alt = Aabb::new(alt_center, alt_half).expanded(clearance);

        if !existing.iter().any(|e| alt.intersects(e)) {
            return PlacementResult::Adjusted {
                original,
                adjusted: alt_center,
                reason: format!(
                    "Offset by [{:.1}, {:.1}] to avoid collision",
                    offset[0], offset[1]
                ),
            };
        }
    }

    PlacementResult::Rejected {
        position: original,
        reason: format!(
            "No valid position found after {} attempts — area too crowded",
            max_attempts
        ),
    }
}

/// Snap a position to terrain height.
///
/// `position` — proposed [x, y, z].
/// `entity_half_height` — half the entity's vertical extent.
/// `terrain_height_fn` — returns terrain height at (x, z), or None if no terrain.
pub fn ground_snap(
    position: [f32; 3],
    entity_half_height: f32,
    terrain_height_fn: impl Fn(f32, f32) -> Option<f32>,
) -> Option<[f32; 3]> {
    let terrain_y = terrain_height_fn(position[0], position[2])?;
    Some([position[0], terrain_y + entity_half_height, position[2]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_no_overlap() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Aabb::new([5.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_aabb_overlap() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Aabb::new([1.5, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_placement_valid_no_obstacles() {
        let proposed = Aabb::new([5.0, 0.0, 5.0], [1.0, 1.0, 1.0]);
        let result = check_placement(proposed, 0.0, &[], 3);
        assert!(matches!(result, PlacementResult::Valid { .. }));
    }

    #[test]
    fn test_placement_valid_far_from_obstacle() {
        let proposed = Aabb::new([5.0, 0.0, 5.0], [1.0, 1.0, 1.0]);
        let existing = vec![Aabb::new([20.0, 0.0, 20.0], [1.0, 1.0, 1.0])];
        let result = check_placement(proposed, 0.5, &existing, 3);
        assert!(matches!(result, PlacementResult::Valid { .. }));
    }

    #[test]
    fn test_placement_adjusted() {
        let proposed = Aabb::new([5.0, 0.0, 5.0], [1.0, 1.0, 1.0]);
        // Single obstacle at proposed position — alternatives should find open space
        let existing = vec![Aabb::new([5.0, 0.0, 5.0], [0.5, 0.5, 0.5])];
        let result = check_placement(proposed, 0.0, &existing, 8);
        assert!(matches!(result, PlacementResult::Adjusted { .. }));
    }

    #[test]
    fn test_placement_rejected_crowded() {
        let proposed = Aabb::new([5.0, 0.0, 5.0], [1.0, 1.0, 1.0]);
        // Surround with obstacles
        let existing: Vec<Aabb> = (-3..=3)
            .flat_map(|x| {
                (-3..=3).map(move |z| {
                    Aabb::new(
                        [5.0 + x as f32 * 2.0, 0.0, 5.0 + z as f32 * 2.0],
                        [0.8, 1.0, 0.8],
                    )
                })
            })
            .collect();
        let result = check_placement(proposed, 0.5, &existing, 3);
        assert!(matches!(result, PlacementResult::Rejected { .. }));
    }

    #[test]
    fn test_ground_snap() {
        let pos = [5.0, 10.0, 5.0];
        let result = ground_snap(pos, 1.0, |_, _| Some(2.0));
        assert!(result.is_some());
        let snapped = result.unwrap();
        assert_eq!(snapped[1], 3.0); // terrain (2.0) + half height (1.0)
    }

    #[test]
    fn test_ground_snap_no_terrain() {
        let pos = [5.0, 10.0, 5.0];
        let result = ground_snap(pos, 1.0, |_, _| None);
        assert!(result.is_none());
    }

    #[test]
    fn test_clearance_expanded() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let expanded = a.expanded(0.5);
        assert_eq!(expanded.min[0], -1.5);
        assert_eq!(expanded.max[0], 1.5);
    }
}
