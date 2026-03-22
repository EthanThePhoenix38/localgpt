//! Holds .md + .ron file pairs in memory until flushed to disk.
//!
//! Ensures atomic writes: both files in a pair succeed or neither is written.

use bevy::prelude::*;
use std::collections::HashMap;
use std::io;
use std::path::Path;

/// Holds .md + .ron file pairs in memory until flushed to disk.
/// Ensures atomic writes: both files in a pair succeed or neither is written.
#[derive(Resource, Default)]
pub struct PendingWrites {
    /// domain path → (md_content, ron_content)
    writes: HashMap<String, (String, String)>,
}

#[allow(dead_code)]
impl PendingWrites {
    pub fn insert(&mut self, domain: &str, md_content: String, ron_content: String) {
        self.writes
            .insert(domain.to_string(), (md_content, ron_content));
    }

    pub fn has(&self, domain: &str) -> bool {
        self.writes.contains_key(domain)
    }

    pub fn remove(&mut self, domain: &str) -> Option<(String, String)> {
        self.writes.remove(domain)
    }

    pub fn domains(&self) -> impl Iterator<Item = &String> {
        self.writes.keys()
    }

    /// Flush a single domain to disk. Writes to .tmp first, then renames for atomicity.
    pub fn flush(&mut self, domain: &str, base_dir: &Path) -> io::Result<()> {
        let (md, ron) = self.writes.remove(domain).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "domain not in pending writes")
        })?;

        let md_path = base_dir.join(format!("{domain}.md"));
        let ron_path = base_dir.join(format!("{domain}.ron"));
        let md_tmp = base_dir.join(format!("{domain}.md.tmp"));
        let ron_tmp = base_dir.join(format!("{domain}.ron.tmp"));

        // Write to .tmp files first
        std::fs::write(&md_tmp, &md)?;
        std::fs::write(&ron_tmp, &ron)?;

        // Rename both .tmp to final paths
        if let Err(e) = std::fs::rename(&md_tmp, &md_path) {
            let _ = std::fs::remove_file(&md_tmp);
            let _ = std::fs::remove_file(&ron_tmp);
            return Err(e);
        }
        if let Err(e) = std::fs::rename(&ron_tmp, &ron_path) {
            let _ = std::fs::remove_file(&ron_tmp);
            return Err(e);
        }

        Ok(())
    }

    /// Flush all pending writes to disk.
    pub fn flush_all(&mut self, base_dir: &Path) -> io::Result<()> {
        let domains: Vec<String> = self.writes.keys().cloned().collect();
        for domain in domains {
            self.flush(&domain, base_dir)?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.writes.len()
    }
}
