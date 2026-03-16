//! WorldGen — procedural world generation pipeline.
//!
//! Implements the blockout-first workflow:
//! 1. `gen_plan_layout` — text → structured BlockoutSpec
//! 2. `gen_apply_blockout` — BlockoutSpec → coarse 3D scene
//! 3. `gen_populate_region` — fill regions with content
//! 4. Navmesh generation and traversability validation (WG2)
//! 5. Connectivity-ordered generation (WG6.2)
//! 6. GLTF mesh segmentation (WG6.3)

pub mod blockout;
pub mod navmesh;
pub mod ordering;
pub mod segment;
pub mod tier;

pub use blockout::*;
pub use navmesh::*;
pub use ordering::*;
pub use segment::*;
pub use tier::*;
