//! Navmesh — grid-based walkability analysis for world traversability.
//!
//! Implements a voxelized walkability grid that determines which areas of the
//! world are traversable by agents. Uses a heightfield approach:
//! 1. Sample terrain heights on a regular grid
//! 2. Mark cells as walkable based on slope and step height
//! 3. Mark cells as blocked by static obstacles
//! 4. Flood-fill to find connected walkable regions

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Configuration for navmesh generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavMeshSettings {
    /// Agent collision radius in meters.
    #[serde(default = "default_agent_radius")]
    pub agent_radius: f32,
    /// Agent height in meters.
    #[serde(default = "default_agent_height")]
    pub agent_height: f32,
    /// Maximum walkable slope in degrees.
    #[serde(default = "default_max_slope")]
    pub max_slope: f32,
    /// Maximum step-up height in meters.
    #[serde(default = "default_step_height")]
    pub step_height: f32,
    /// Grid cell size in meters (smaller = more detail, slower).
    #[serde(default = "default_cell_size")]
    pub cell_size: f32,
}

fn default_agent_radius() -> f32 {
    0.3
}
fn default_agent_height() -> f32 {
    1.8
}
fn default_max_slope() -> f32 {
    45.0
}
fn default_step_height() -> f32 {
    0.4
}
fn default_cell_size() -> f32 {
    0.5
}

impl Default for NavMeshSettings {
    fn default() -> Self {
        Self {
            agent_radius: default_agent_radius(),
            agent_height: default_agent_height(),
            max_slope: default_max_slope(),
            step_height: default_step_height(),
            cell_size: default_cell_size(),
        }
    }
}

/// Cell state in the walkability grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// Walkable surface.
    Walkable,
    /// Blocked by an obstacle.
    Blocked,
    /// Too steep to walk on.
    TooSteep,
    /// No geometry (void/off-world).
    Void,
}

/// A 2D grid representing walkability of the world.
#[derive(Debug, Clone)]
pub struct NavGrid {
    /// Grid cell states.
    pub cells: Vec<CellState>,
    /// Height at each cell (terrain height).
    pub heights: Vec<f32>,
    /// Grid dimensions (columns, rows).
    pub dims: (usize, usize),
    /// World-space origin of the grid (min corner).
    pub origin: Vec2,
    /// Cell size in world units.
    pub cell_size: f32,
    /// Connected component ID for each walkable cell (0 = not walkable).
    pub components: Vec<u32>,
    /// Number of connected components.
    pub component_count: u32,
}

impl NavGrid {
    /// Create a new navgrid covering the given world bounds.
    pub fn new(min: Vec2, max: Vec2, cell_size: f32) -> Self {
        let size = max - min;
        let cols = (size.x / cell_size).ceil() as usize;
        let rows = (size.y / cell_size).ceil() as usize;
        let total = cols * rows;

        Self {
            cells: vec![CellState::Void; total],
            heights: vec![0.0; total],
            dims: (cols, rows),
            origin: min,
            cell_size,
            components: vec![0; total],
            component_count: 0,
        }
    }

    /// Convert world position to grid coordinates.
    pub fn world_to_grid(&self, world_x: f32, world_z: f32) -> Option<(usize, usize)> {
        let lx = world_x - self.origin.x;
        let lz = world_z - self.origin.y;
        if lx < 0.0 || lz < 0.0 {
            return None;
        }
        let col = (lx / self.cell_size) as usize;
        let row = (lz / self.cell_size) as usize;
        if col < self.dims.0 && row < self.dims.1 {
            Some((col, row))
        } else {
            None
        }
    }

    /// Convert grid coordinates to world position (cell center).
    pub fn grid_to_world(&self, col: usize, row: usize) -> Vec3 {
        let x = self.origin.x + (col as f32 + 0.5) * self.cell_size;
        let z = self.origin.y + (row as f32 + 0.5) * self.cell_size;
        let idx = row * self.dims.0 + col;
        let y = if idx < self.heights.len() {
            self.heights[idx]
        } else {
            0.0
        };
        Vec3::new(x, y, z)
    }

    fn idx(&self, col: usize, row: usize) -> usize {
        row * self.dims.0 + col
    }

    /// Set terrain height and mark cell as potentially walkable.
    pub fn set_height(&mut self, col: usize, row: usize, height: f32) {
        let idx = self.idx(col, row);
        if idx < self.heights.len() {
            self.heights[idx] = height;
            self.cells[idx] = CellState::Walkable;
        }
    }

    /// Mark a cell as blocked by an obstacle.
    pub fn set_blocked(&mut self, col: usize, row: usize) {
        let idx = self.idx(col, row);
        if idx < self.cells.len() {
            self.cells[idx] = CellState::Blocked;
        }
    }

    /// Apply slope analysis — mark cells as TooSteep based on neighbor height differences.
    pub fn analyze_slopes(&mut self, max_slope_degrees: f32, step_height: f32) {
        let max_height_diff = self.cell_size * max_slope_degrees.to_radians().tan();
        let (cols, rows) = self.dims;

        // Need to collect results first since we can't borrow mutably while iterating
        let mut steep_cells = Vec::new();

        for row in 0..rows {
            for col in 0..cols {
                let idx = self.idx(col, row);
                if self.cells[idx] != CellState::Walkable {
                    continue;
                }

                let h = self.heights[idx];
                let mut too_steep = false;

                // Check all 4 cardinal neighbors
                let neighbors = [
                    (col.wrapping_sub(1), row),
                    (col + 1, row),
                    (col, row.wrapping_sub(1)),
                    (col, row + 1),
                ];

                for (nc, nr) in neighbors {
                    if nc < cols && nr < rows {
                        let nidx = self.idx(nc, nr);
                        if self.cells[nidx] == CellState::Walkable {
                            let diff = (self.heights[nidx] - h).abs();
                            if diff > max_height_diff && diff > step_height {
                                too_steep = true;
                                break;
                            }
                        }
                    }
                }

                if too_steep {
                    steep_cells.push(idx);
                }
            }
        }

        for idx in steep_cells {
            self.cells[idx] = CellState::TooSteep;
        }
    }

    /// Apply agent radius erosion — mark cells near obstacles as blocked.
    pub fn erode(&mut self, agent_radius: f32) {
        let radius_cells = (agent_radius / self.cell_size).ceil() as i32;
        let (cols, rows) = self.dims;

        // Find all blocked cells first
        let cells = &self.cells;
        let mut blocked = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let idx = r * cols + c;
                if cells[idx] == CellState::Blocked {
                    blocked.push((c, r));
                }
            }
        }

        // Erode around each blocked cell
        let mut to_block = Vec::new();
        for (bc, br) in blocked {
            for dr in -radius_cells..=radius_cells {
                for dc in -radius_cells..=radius_cells {
                    let nc = bc as i32 + dc;
                    let nr = br as i32 + dr;
                    if nc >= 0 && nc < cols as i32 && nr >= 0 && nr < rows as i32 {
                        let dist_sq = (dc * dc + dr * dr) as f32;
                        if dist_sq <= (radius_cells * radius_cells) as f32 {
                            to_block.push((nc as usize, nr as usize));
                        }
                    }
                }
            }
        }

        for (c, r) in to_block {
            let idx = self.idx(c, r);
            if self.cells[idx] == CellState::Walkable {
                self.cells[idx] = CellState::Blocked;
            }
        }
    }

    /// Flood-fill to identify connected walkable components.
    pub fn find_components(&mut self) {
        let (cols, rows) = self.dims;
        self.components = vec![0; cols * rows];
        let mut component_id = 0u32;

        for row in 0..rows {
            for col in 0..cols {
                let idx = self.idx(col, row);
                if self.cells[idx] == CellState::Walkable && self.components[idx] == 0 {
                    component_id += 1;
                    self.flood_fill(col, row, component_id);
                }
            }
        }

        self.component_count = component_id;
    }

    fn flood_fill(&mut self, start_col: usize, start_row: usize, component_id: u32) {
        let (cols, rows) = self.dims;
        let mut queue = VecDeque::new();
        queue.push_back((start_col, start_row));

        while let Some((col, row)) = queue.pop_front() {
            let idx = self.idx(col, row);
            if self.cells[idx] != CellState::Walkable || self.components[idx] != 0 {
                continue;
            }
            self.components[idx] = component_id;

            // 4-connected neighbors
            if col > 0 {
                queue.push_back((col - 1, row));
            }
            if col + 1 < cols {
                queue.push_back((col + 1, row));
            }
            if row > 0 {
                queue.push_back((col, row - 1));
            }
            if row + 1 < rows {
                queue.push_back((col, row + 1));
            }
        }
    }

    /// Get walkable coverage as a fraction (0.0 - 1.0).
    pub fn walkable_coverage(&self) -> f32 {
        let total_non_void = self
            .cells
            .iter()
            .filter(|c| **c != CellState::Void)
            .count();
        if total_non_void == 0 {
            return 0.0;
        }
        let walkable = self
            .cells
            .iter()
            .filter(|c| **c == CellState::Walkable)
            .count();
        walkable as f32 / total_non_void as f32
    }

    /// Check if two world positions are in the same connected component.
    pub fn are_connected(&self, a: Vec3, b: Vec3) -> bool {
        let Some((ac, ar)) = self.world_to_grid(a.x, a.z) else {
            return false;
        };
        let Some((bc, br)) = self.world_to_grid(b.x, b.z) else {
            return false;
        };
        let ai = self.idx(ac, ar);
        let bi = self.idx(bc, br);
        if ai >= self.components.len() || bi >= self.components.len() {
            return false;
        }
        let ca = self.components[ai];
        let cb = self.components[bi];
        ca != 0 && ca == cb
    }

    /// Find a path between two world positions using A* on the grid.
    pub fn find_path(&self, from: Vec3, to: Vec3) -> Option<Vec<Vec3>> {
        let (fc, fr) = self.world_to_grid(from.x, from.z)?;
        let (tc, tr) = self.world_to_grid(to.x, to.z)?;

        let fi = self.idx(fc, fr);
        let ti = self.idx(tc, tr);

        if self.cells[fi] != CellState::Walkable || self.cells[ti] != CellState::Walkable {
            return None;
        }
        if self.components[fi] != self.components[ti] || self.components[fi] == 0 {
            return None;
        }

        // A* pathfinding
        let (cols, rows) = self.dims;
        let heuristic = |c: usize, r: usize| -> f32 {
            let dc = (c as f32 - tc as f32).abs();
            let dr = (r as f32 - tr as f32).abs();
            dc + dr
        };

        let mut open: Vec<(usize, usize)> = vec![(fc, fr)];
        let mut g_score: HashMap<(usize, usize), f32> = HashMap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        g_score.insert((fc, fr), 0.0);

        let mut visited: HashSet<(usize, usize)> = HashSet::new();

        while !open.is_empty() {
            // Find node with lowest f-score
            let mut best_idx = 0;
            let mut best_f = f32::MAX;
            for (i, &(c, r)) in open.iter().enumerate() {
                let g = g_score.get(&(c, r)).copied().unwrap_or(f32::MAX);
                let f = g + heuristic(c, r);
                if f < best_f {
                    best_f = f;
                    best_idx = i;
                }
            }

            let (cc, cr) = open.swap_remove(best_idx);
            if cc == tc && cr == tr {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = (tc, tr);
                path.push(self.grid_to_world(current.0, current.1));
                while let Some(&prev) = came_from.get(&current) {
                    path.push(self.grid_to_world(prev.0, prev.1));
                    current = prev;
                }
                path.reverse();
                return Some(path);
            }

            if !visited.insert((cc, cr)) {
                continue;
            }

            let cg = g_score.get(&(cc, cr)).copied().unwrap_or(f32::MAX);

            // 4-connected neighbors
            let neighbors = [
                (cc.wrapping_sub(1), cr),
                (cc + 1, cr),
                (cc, cr.wrapping_sub(1)),
                (cc, cr + 1),
            ];

            for (nc, nr) in neighbors {
                if nc >= cols || nr >= rows {
                    continue;
                }
                let ni = self.idx(nc, nr);
                if self.cells[ni] != CellState::Walkable {
                    continue;
                }
                if visited.contains(&(nc, nr)) {
                    continue;
                }

                let new_g = cg + self.cell_size;
                let old_g = g_score.get(&(nc, nr)).copied().unwrap_or(f32::MAX);
                if new_g < old_g {
                    g_score.insert((nc, nr), new_g);
                    came_from.insert((nc, nr), (cc, cr));
                    open.push((nc, nr));
                }
            }
        }

        None // No path found
    }

    /// Find blocked areas within a bounding region.
    pub fn find_blocked_areas(&self, center: Vec2, radius: f32) -> Vec<BlockedArea> {
        let mut areas = Vec::new();
        let (cols, rows) = self.dims;

        // Cluster nearby blocked cells
        let mut visited = HashSet::new();

        for row in 0..rows {
            for col in 0..cols {
                let world_pos = self.grid_to_world(col, row);
                let dist = Vec2::new(world_pos.x - center.x, world_pos.z - center.y).length();
                if dist > radius {
                    continue;
                }

                let idx = self.idx(col, row);
                if self.cells[idx] != CellState::Blocked && self.cells[idx] != CellState::TooSteep
                {
                    continue;
                }
                if visited.contains(&(col, row)) {
                    continue;
                }

                // Flood-fill to find extent of blocked area
                let mut cluster = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back((col, row));

                while let Some((c, r)) = queue.pop_front() {
                    if !visited.insert((c, r)) {
                        continue;
                    }
                    let i = self.idx(c, r);
                    if self.cells[i] != CellState::Blocked && self.cells[i] != CellState::TooSteep
                    {
                        continue;
                    }
                    cluster.push(self.grid_to_world(c, r));

                    if c > 0 {
                        queue.push_back((c - 1, r));
                    }
                    if c + 1 < cols {
                        queue.push_back((c + 1, r));
                    }
                    if r > 0 {
                        queue.push_back((c, r - 1));
                    }
                    if r + 1 < rows {
                        queue.push_back((c, r + 1));
                    }
                }

                if cluster.len() >= 2 {
                    let avg = cluster.iter().copied().sum::<Vec3>() / cluster.len() as f32;
                    let max_dist = cluster
                        .iter()
                        .map(|p| (*p - avg).length())
                        .fold(0.0f32, f32::max);

                    let reason = if self.cells[idx] == CellState::TooSteep {
                        "steep_slope"
                    } else {
                        "object_blocking_path"
                    };

                    areas.push(BlockedArea {
                        center: [avg.x, avg.y, avg.z],
                        radius: max_dist.max(self.cell_size),
                        reason: reason.to_string(),
                    });
                }
            }
        }

        areas
    }
}

/// A blocked area in the navmesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedArea {
    pub center: [f32; 3],
    pub radius: f32,
    pub reason: String,
}

/// Navigability validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigabilityResult {
    pub navigable: bool,
    pub coverage_percent: f32,
    pub path_found: Option<bool>,
    pub path_length: Option<f32>,
    pub disconnected_regions: Vec<String>,
    pub blocked_areas: Vec<BlockedArea>,
    pub component_count: u32,
    pub warnings: Vec<String>,
}

/// Bevy resource holding the current navmesh grid.
#[derive(Resource, Default)]
pub struct NavMeshResource {
    pub grid: Option<NavGrid>,
    pub settings: NavMeshSettings,
    /// Whether the navmesh needs regeneration.
    pub dirty: bool,
}

/// Marker component for navmesh debug visualization entities.
#[derive(Component)]
pub struct NavMeshDebugMesh;

/// Build a NavGrid from scene data.
///
/// `terrain_heights` — function that returns terrain height at (x, z).
/// `obstacles` — list of (position, half_extents) for blocking objects.
/// `world_min`/`world_max` — world bounds.
pub fn build_navgrid(
    world_min: Vec2,
    world_max: Vec2,
    settings: &NavMeshSettings,
    terrain_height_fn: impl Fn(f32, f32) -> Option<f32>,
    obstacles: &[(Vec3, Vec3)], // (center, half_extents)
) -> NavGrid {
    let mut grid = NavGrid::new(world_min, world_max, settings.cell_size);
    let (cols, rows) = grid.dims;

    // Sample terrain heights
    for row in 0..rows {
        for col in 0..cols {
            let world = grid.grid_to_world(col, row);
            if let Some(h) = terrain_height_fn(world.x, world.z) {
                grid.set_height(col, row, h);
            }
        }
    }

    // Mark obstacle cells as blocked
    for &(center, half_extents) in obstacles {
        let min_x = center.x - half_extents.x;
        let max_x = center.x + half_extents.x;
        let min_z = center.z - half_extents.z;
        let max_z = center.z + half_extents.z;

        // Find grid cells that overlap the obstacle AABB
        if let Some((c0, r0)) = grid.world_to_grid(min_x, min_z) {
            let (c1, r1) = grid
                .world_to_grid(max_x, max_z)
                .unwrap_or((cols - 1, rows - 1));
            for r in r0..=r1.min(rows - 1) {
                for c in c0..=c1.min(cols - 1) {
                    grid.set_blocked(c, r);
                }
            }
        }
    }

    // Apply slope analysis
    grid.analyze_slopes(settings.max_slope, settings.step_height);

    // Erode walkable area by agent radius
    grid.erode(settings.agent_radius);

    // Find connected components
    grid.find_components();

    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let grid = NavGrid::new(Vec2::ZERO, Vec2::new(10.0, 10.0), 1.0);
        assert_eq!(grid.dims, (10, 10));
        assert_eq!(grid.walkable_coverage(), 0.0);
    }

    #[test]
    fn test_flat_terrain_fully_walkable() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0, // No erosion for this test
            ..Default::default()
        };
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &[],
        );
        assert_eq!(grid.walkable_coverage(), 1.0);
        assert_eq!(grid.component_count, 1);
    }

    #[test]
    fn test_obstacle_blocks_cells() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            ..Default::default()
        };
        let obstacle = (Vec3::new(5.0, 0.0, 5.0), Vec3::new(1.0, 1.0, 1.0));
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &[obstacle],
        );
        assert!(grid.walkable_coverage() < 1.0);
    }

    #[test]
    fn test_wall_creates_two_components() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            ..Default::default()
        };
        // Wall across the middle from x=0 to x=10 at z=5
        let mut obstacles = Vec::new();
        for x in 0..10 {
            obstacles.push((
                Vec3::new(x as f32 + 0.5, 0.0, 5.5),
                Vec3::new(0.5, 1.0, 0.5),
            ));
        }
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &obstacles,
        );
        assert_eq!(grid.component_count, 2);
    }

    #[test]
    fn test_pathfinding_simple() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            ..Default::default()
        };
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &[],
        );
        let path = grid.find_path(Vec3::new(1.0, 0.0, 1.0), Vec3::new(8.0, 0.0, 8.0));
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() >= 2);
    }

    #[test]
    fn test_pathfinding_blocked() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            ..Default::default()
        };
        let mut obstacles = Vec::new();
        for x in 0..10 {
            obstacles.push((
                Vec3::new(x as f32 + 0.5, 0.0, 5.5),
                Vec3::new(0.5, 1.0, 0.5),
            ));
        }
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &obstacles,
        );
        let path = grid.find_path(Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 9.0));
        assert!(path.is_none());
    }

    #[test]
    fn test_connectivity_check() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            ..Default::default()
        };
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &[],
        );
        assert!(grid.are_connected(
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(8.0, 0.0, 8.0)
        ));
    }

    #[test]
    fn test_steep_slope_detection() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            max_slope: 30.0,
            step_height: 0.4,
            ..Default::default()
        };
        // Create a cliff: left half at y=0, right half at y=10
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |x, _| Some(if x < 5.0 { 0.0 } else { 10.0 }),
            &[],
        );
        // Cells near the cliff edge should be marked as too steep
        assert!(grid.component_count >= 2);
    }

    #[test]
    fn test_erosion() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 1.0,
            ..Default::default()
        };
        let obstacle = (Vec3::new(5.0, 0.0, 5.0), Vec3::new(0.5, 1.0, 0.5));
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &[obstacle],
        );
        // With erosion, more cells should be blocked than just the obstacle
        let blocked_count = grid
            .cells
            .iter()
            .filter(|c| **c == CellState::Blocked)
            .count();
        assert!(blocked_count > 1);
    }

    #[test]
    fn test_blocked_area_detection() {
        let settings = NavMeshSettings {
            cell_size: 1.0,
            agent_radius: 0.0,
            ..Default::default()
        };
        let obstacle = (Vec3::new(5.0, 0.0, 5.0), Vec3::new(2.0, 1.0, 2.0));
        let grid = build_navgrid(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            &settings,
            |_, _| Some(0.0),
            &[obstacle],
        );
        let areas = grid.find_blocked_areas(Vec2::new(5.0, 5.0), 10.0);
        assert!(!areas.is_empty());
    }

    #[test]
    fn test_navmesh_settings_defaults() {
        let settings = NavMeshSettings::default();
        assert_eq!(settings.agent_radius, 0.3);
        assert_eq!(settings.agent_height, 1.8);
        assert_eq!(settings.max_slope, 45.0);
        assert_eq!(settings.step_height, 0.4);
        assert_eq!(settings.cell_size, 0.5);
    }
}
