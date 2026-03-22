//! SQLite-backed memory search backend.
//!
//! Wraps the existing `MemoryIndex` (FTS5 + optional vector search) to implement
//! the `MemoryBackend` trait. This is the default backend.

use std::path::Path;

use anyhow::Result;

use super::backend::{MemoryBackend, MemoryBackendKind};
use super::index::MemoryIndex;
use super::search::MemoryChunk;

/// SQLite backend — delegates all operations to [`MemoryIndex`].
pub struct SqliteBackend {
    index: MemoryIndex,
}

impl SqliteBackend {
    /// Wrap an existing `MemoryIndex` in a backend adapter.
    pub fn new(index: MemoryIndex) -> Self {
        Self { index }
    }

    /// Access the underlying `MemoryIndex`.
    pub fn index(&self) -> &MemoryIndex {
        &self.index
    }
}

impl MemoryBackend for SqliteBackend {
    fn kind(&self) -> MemoryBackendKind {
        MemoryBackendKind::Sqlite
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryChunk>> {
        self.index.search(query, limit)
    }

    fn search_fts_raw(&self, fts_query: &str, limit: usize) -> Result<Vec<MemoryChunk>> {
        self.index.search_fts_raw(fts_query, limit)
    }

    fn search_hybrid(
        &self,
        fts_query: &str,
        embedding: Option<&[f32]>,
        model: &str,
        limit: usize,
        fts_weight: f64,
        vector_weight: f64,
    ) -> Result<Vec<MemoryChunk>> {
        self.index.search_hybrid(
            fts_query,
            embedding,
            model,
            limit,
            fts_weight as f32,
            vector_weight as f32,
        )
    }

    fn index_file(&self, path: &Path, force: bool) -> Result<bool> {
        self.index.index_file(path, force)
    }

    fn remove_file(&self, relative_path: &str) -> Result<()> {
        self.index.remove_file(relative_path)
    }

    fn indexed_files(&self) -> Result<Vec<String>> {
        self.index.indexed_files()
    }

    fn chunk_count(&self) -> Result<usize> {
        self.index.chunk_count()
    }

    fn file_chunk_count(&self, path: &Path) -> Result<usize> {
        self.index.file_chunk_count(path)
    }

    fn size_bytes(&self) -> Result<u64> {
        self.index.size_bytes()
    }

    fn supports_embeddings(&self) -> bool {
        true
    }

    fn chunks_without_embeddings(&self, limit: usize) -> Result<Vec<(String, String)>> {
        self.index.chunks_without_embeddings(limit)
    }

    fn store_embedding(&self, chunk_id: &str, embedding: &[f32], model: &str) -> Result<()> {
        self.index.store_embedding(chunk_id, embedding, model)
    }

    fn get_cached_embedding(
        &self,
        provider: &str,
        model: &str,
        text_hash: &str,
    ) -> Result<Option<Vec<f32>>> {
        self.index.get_cached_embedding(provider, model, text_hash)
    }

    fn cache_embedding(
        &self,
        provider: &str,
        model: &str,
        provider_key: &str,
        text_hash: &str,
        embedding: &[f32],
    ) -> Result<()> {
        self.index
            .cache_embedding(provider, model, provider_key, text_hash, embedding)
    }

    fn embedded_chunk_count(&self, model: &str) -> Result<usize> {
        self.index.embedded_chunk_count(model)
    }

    fn insert_chunk(
        &self,
        virtual_path: &str,
        content: &str,
        line_start: usize,
        line_end: usize,
    ) -> Result<()> {
        self.index
            .insert_chunk(virtual_path, content, line_start, line_end)
    }
}
