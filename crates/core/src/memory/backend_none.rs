//! No-op memory search backend.
//!
//! All search operations return empty results. Workspace file reading
//! (MEMORY.md, SOUL.md, etc.) still works via `MemoryManager`.

use anyhow::Result;

use super::backend::{MemoryBackend, MemoryBackendKind};
use super::search::MemoryChunk;

/// No-op backend — memory search disabled.
pub struct NoneBackend;

impl Default for NoneBackend {
    fn default() -> Self {
        Self
    }
}

impl NoneBackend {
    pub fn new() -> Self {
        Self
    }
}

impl MemoryBackend for NoneBackend {
    fn kind(&self) -> MemoryBackendKind {
        MemoryBackendKind::None
    }

    fn search(&self, _query: &str, _limit: usize) -> Result<Vec<MemoryChunk>> {
        Ok(Vec::new())
    }
}
