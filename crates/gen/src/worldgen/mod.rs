//! WorldGen — procedural world generation pipeline.
//!
//! Implements the blockout-first workflow:
//! 1. `gen_plan_layout` — text → structured BlockoutSpec
//! 2. `gen_apply_blockout` — BlockoutSpec → coarse 3D scene
//! 3. `gen_populate_region` — fill regions with content

pub mod blockout;

pub use blockout::*;
