//! Markdown-based memory search backend.
//!
//! Provides a lightweight, grep-style search over workspace `.md` files.
//! No database required — searches files directly from the filesystem.
//! Does not support embeddings or vector search.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::backend::{MemoryBackend, MemoryBackendKind};
use super::search::MemoryChunk;

/// Markdown backend — grep-based search over `.md` files in the workspace.
pub struct MarkdownBackend {
    workspace: PathBuf,
}

impl MarkdownBackend {
    /// Create a new markdown backend rooted at `workspace`.
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

impl MemoryBackend for MarkdownBackend {
    fn kind(&self) -> MemoryBackendKind {
        MemoryBackendKind::Markdown
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryChunk>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let terms: Vec<&str> = query_lower.split_whitespace().collect();

        if terms.is_empty() {
            return Ok(results);
        }

        // Search all .md files recursively
        let pattern = format!("{}/**/*.md", self.workspace.display());
        for entry in glob::glob(&pattern)
            .into_iter()
            .flatten()
            .filter_map(|r| r.ok())
        {
            if !entry.is_file() {
                continue;
            }
            let content = match fs::read_to_string(&entry) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative = entry
                .strip_prefix(&self.workspace)
                .map(|r| r.display().to_string())
                .unwrap_or_else(|_| entry.display().to_string());

            // Score each line for query term matches
            for (i, line) in content.lines().enumerate() {
                let line_lower = line.to_lowercase();
                let matched = terms.iter().filter(|t| line_lower.contains(*t)).count();
                if matched > 0 {
                    let score = matched as f64 / terms.len().max(1) as f64;
                    results.push(MemoryChunk::new(
                        relative.clone(),
                        (i + 1) as i32,
                        (i + 1) as i32,
                        line.to_string(),
                        score,
                    ));
                }
            }
        }

        // Sort by score descending, take top `limit`
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }

    fn chunk_count(&self) -> Result<usize> {
        let mut count = 0;
        let pattern = format!("{}/**/*.md", self.workspace.display());
        for entry in glob::glob(&pattern)
            .into_iter()
            .flatten()
            .filter_map(|r| r.ok())
        {
            if entry.is_file()
                && let Ok(content) = fs::read_to_string(&entry)
            {
                count += content.lines().count();
            }
        }
        Ok(count)
    }

    fn file_chunk_count(&self, path: &Path) -> Result<usize> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(content.lines().count())
        } else {
            Ok(0)
        }
    }
}
