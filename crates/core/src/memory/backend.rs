//! Pluggable memory search backend trait.
//!
//! The `MemoryManager` handles workspace file reading (MEMORY.md, SOUL.md, etc.)
//! which is always filesystem-based. This module defines the `MemoryBackend` trait
//! that abstracts the search index, allowing different storage engines:
//!
//! - **SQLite** (default): Full FTS5 + optional vector similarity via `MemoryIndex`
//! - **Markdown**: Lightweight grep-style search over workspace `.md` files
//! - **None**: Search disabled; workspace file reading still works

use std::path::Path;

use anyhow::Result;

pub use crate::config::MemoryBackendKind;

use super::search::MemoryChunk;

/// Trait for pluggable memory search backends.
///
/// The `MemoryManager` handles workspace file reading (MEMORY.md, SOUL.md, etc.)
/// which is always filesystem-based. This trait abstracts only the search index.
pub trait MemoryBackend: Send + Sync {
    /// Backend kind identifier.
    fn kind(&self) -> MemoryBackendKind;

    /// Search memory for matching chunks using keyword/FTS search.
    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryChunk>>;

    /// Search with a raw FTS query (pre-expanded keywords).
    fn search_fts_raw(&self, fts_query: &str, limit: usize) -> Result<Vec<MemoryChunk>> {
        self.search(fts_query, limit)
    }

    /// Hybrid search combining FTS and vector similarity.
    fn search_hybrid(
        &self,
        fts_query: &str,
        embedding: Option<&[f32]>,
        model: &str,
        limit: usize,
        fts_weight: f64,
        vector_weight: f64,
    ) -> Result<Vec<MemoryChunk>> {
        let _ = (embedding, model, fts_weight, vector_weight);
        self.search(fts_query, limit)
    }

    /// Index a file for searching. Returns `true` if the file was updated.
    fn index_file(&self, path: &Path, force: bool) -> Result<bool> {
        let _ = (path, force);
        Ok(false)
    }

    /// Remove a file from the index.
    fn remove_file(&self, relative_path: &str) -> Result<()> {
        let _ = relative_path;
        Ok(())
    }

    /// List all indexed file paths.
    fn indexed_files(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Total number of indexed chunks.
    fn chunk_count(&self) -> Result<usize> {
        Ok(0)
    }

    /// Number of chunks for a specific file.
    fn file_chunk_count(&self, path: &Path) -> Result<usize> {
        let _ = path;
        Ok(0)
    }

    /// On-disk size of the index in bytes.
    fn size_bytes(&self) -> Result<u64> {
        Ok(0)
    }

    /// Whether this backend supports vector embeddings.
    fn supports_embeddings(&self) -> bool {
        false
    }

    /// Get chunks that don't have embeddings yet (id, text).
    fn chunks_without_embeddings(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let _ = limit;
        Ok(Vec::new())
    }

    /// Store an embedding vector for a chunk.
    fn store_embedding(&self, chunk_id: &str, embedding: &[f32], model: &str) -> Result<()> {
        let _ = (chunk_id, embedding, model);
        Ok(())
    }

    /// Get a cached embedding by text hash.
    fn get_cached_embedding(
        &self,
        provider: &str,
        model: &str,
        text_hash: &str,
    ) -> Result<Option<Vec<f32>>> {
        let _ = (provider, model, text_hash);
        Ok(None)
    }

    /// Cache an embedding for future reuse.
    fn cache_embedding(
        &self,
        provider: &str,
        model: &str,
        provider_key: &str,
        text_hash: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let _ = (provider, model, provider_key, text_hash, embedding);
        Ok(())
    }

    /// Count of chunks that have embeddings for a given model.
    fn embedded_chunk_count(&self, model: &str) -> Result<usize> {
        let _ = model;
        Ok(0)
    }

    /// Insert a pre-chunked content piece (used by session indexer).
    fn insert_chunk(
        &self,
        virtual_path: &str,
        content: &str,
        line_start: usize,
        line_end: usize,
    ) -> Result<()> {
        let _ = (virtual_path, content, line_start, line_end);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::backend_markdown::MarkdownBackend;
    use crate::memory::backend_none::NoneBackend;
    use crate::memory::backend_sqlite::SqliteBackend;
    use crate::memory::index::MemoryIndex;

    #[test]
    fn test_none_backend_returns_empty() {
        let backend = NoneBackend::new();
        assert_eq!(backend.kind(), MemoryBackendKind::None);
        let results = backend.search("test query", 10).unwrap();
        assert!(results.is_empty());
        assert_eq!(backend.chunk_count().unwrap(), 0);
    }

    #[test]
    fn test_markdown_backend_search() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        std::fs::write(
            workspace.join("test.md"),
            "Hello world\nThis is a test\nGoodbye",
        )
        .unwrap();

        let backend = MarkdownBackend::new(workspace.to_path_buf());
        assert_eq!(backend.kind(), MemoryBackendKind::Markdown);

        let results = backend.search("hello", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("Hello"));

        let results = backend.search("nonexistent xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_sqlite_backend_via_trait() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        std::fs::create_dir_all(workspace).unwrap();
        let db_path = workspace.join("test.sqlite");

        let index = MemoryIndex::new_with_db_path(workspace, &db_path).unwrap();
        let backend = SqliteBackend::new(index);
        assert_eq!(backend.kind(), MemoryBackendKind::Sqlite);
        assert!(backend.supports_embeddings());
        assert_eq!(backend.chunk_count().unwrap(), 0);
    }

    #[test]
    fn test_backend_kind_default() {
        assert_eq!(MemoryBackendKind::default(), MemoryBackendKind::Sqlite);
    }

    #[test]
    fn test_backend_kind_display() {
        assert_eq!(MemoryBackendKind::Sqlite.to_string(), "sqlite");
        assert_eq!(MemoryBackendKind::Markdown.to_string(), "markdown");
        assert_eq!(MemoryBackendKind::None.to_string(), "none");
    }

    #[test]
    fn test_markdown_backend_chunk_count() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        std::fs::write(workspace.join("a.md"), "line1\nline2\nline3").unwrap();
        std::fs::write(workspace.join("b.md"), "one\ntwo").unwrap();

        let backend = MarkdownBackend::new(workspace.to_path_buf());
        // 3 lines + 2 lines = 5
        assert_eq!(backend.chunk_count().unwrap(), 5);
    }

    #[test]
    fn test_none_backend_default_methods() {
        let backend = NoneBackend::new();
        assert!(!backend.supports_embeddings());
        assert!(backend.indexed_files().unwrap().is_empty());
        assert_eq!(backend.size_bytes().unwrap(), 0);
        assert!(backend.chunks_without_embeddings(10).unwrap().is_empty());
        assert_eq!(backend.embedded_chunk_count("model").unwrap(), 0);
        assert!(
            backend
                .get_cached_embedding("p", "m", "h")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_markdown_backend_file_chunk_count() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        let file = workspace.join("test.md");
        std::fs::write(&file, "line1\nline2\nline3\nline4").unwrap();

        let backend = MarkdownBackend::new(workspace.to_path_buf());
        assert_eq!(backend.file_chunk_count(&file).unwrap(), 4);

        // Non-existent file returns 0
        let missing = workspace.join("missing.md");
        assert_eq!(backend.file_chunk_count(&missing).unwrap(), 0);
    }
}
