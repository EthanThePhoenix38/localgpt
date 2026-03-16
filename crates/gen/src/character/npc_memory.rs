//! NPC persistent memory system (AI2.3).
//!
//! NPCs accumulate memories from interactions that persist across save/load.

use serde::{Deserialize, Serialize};

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: f64,
    pub content: String,
    pub importance: f32,
}

/// NPC memory store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcMemory {
    pub capacity: usize,
    pub entries: Vec<MemoryEntry>,
    pub auto_memorize: bool,
}

impl NpcMemory {
    pub fn new(capacity: usize, auto_memorize: bool) -> Self {
        Self {
            capacity,
            entries: Vec::new(),
            auto_memorize,
        }
    }

    /// Add a memory entry. If at capacity, evict the least important/oldest entry.
    pub fn add_memory(&mut self, content: String, importance: f32, timestamp: f64) {
        if self.entries.len() >= self.capacity {
            self.evict_one(timestamp);
        }
        self.entries.push(MemoryEntry {
            timestamp,
            content,
            importance,
        });
    }

    /// Get the N most relevant memories for brain context.
    pub fn top_memories(&self, n: usize, current_time: f64) -> Vec<&MemoryEntry> {
        let mut scored: Vec<(f32, &MemoryEntry)> = self
            .entries
            .iter()
            .map(|e| {
                let age = (current_time - e.timestamp).max(0.0) as f32;
                let recency = (-age / 300.0).exp(); // decay over 5 minutes
                (e.importance * 0.6 + recency * 0.4, e)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(n).map(|(_, e)| e).collect()
    }

    /// Format top memories as context string for brain prompt.
    pub fn format_for_context(&self, n: usize, current_time: f64) -> Option<String> {
        let top = self.top_memories(n, current_time);
        if top.is_empty() {
            return None;
        }
        let lines: Vec<String> = top.iter().map(|e| format!("- \"{}\"", e.content)).collect();
        Some(lines.join("\n"))
    }

    /// Evict the entry with the lowest combined score.
    fn evict_one(&mut self, current_time: f64) {
        if self.entries.is_empty() {
            return;
        }
        let mut worst_idx = 0;
        let mut worst_score = f32::MAX;
        for (i, e) in self.entries.iter().enumerate() {
            let age = (current_time - e.timestamp).max(0.0) as f32;
            let recency = (-age / 300.0).exp();
            let score = e.importance * 0.6 + recency * 0.4;
            if score < worst_score {
                worst_score = score;
                worst_idx = i;
            }
        }
        self.entries.remove(worst_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_memory() {
        let mut mem = NpcMemory::new(10, true);
        mem.add_memory("Met the blacksmith".to_string(), 0.8, 100.0);
        assert_eq!(mem.entries.len(), 1);
        assert_eq!(mem.entries[0].content, "Met the blacksmith");
        assert_eq!(mem.entries[0].importance, 0.8);
        assert_eq!(mem.entries[0].timestamp, 100.0);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut mem = NpcMemory::new(3, false);
        mem.add_memory("old low importance".to_string(), 0.1, 0.0);
        mem.add_memory("medium".to_string(), 0.5, 50.0);
        mem.add_memory("recent high".to_string(), 0.9, 100.0);

        // At capacity; adding one more should evict the worst-scored entry
        mem.add_memory("new entry".to_string(), 0.7, 150.0);
        assert_eq!(mem.entries.len(), 3);

        // The old low-importance entry should have been evicted
        let contents: Vec<&str> = mem.entries.iter().map(|e| e.content.as_str()).collect();
        assert!(
            !contents.contains(&"old low importance"),
            "Expected old low-importance entry to be evicted, got: {:?}",
            contents
        );
    }

    #[test]
    fn test_top_memories_ordering() {
        let mut mem = NpcMemory::new(10, true);
        mem.add_memory("low importance old".to_string(), 0.1, 0.0);
        mem.add_memory("high importance old".to_string(), 0.9, 10.0);
        mem.add_memory("medium recent".to_string(), 0.5, 300.0);

        let top = mem.top_memories(2, 300.0);
        assert_eq!(top.len(), 2);
        // High importance should appear (either because of importance or recency)
        let top_contents: Vec<&str> = top.iter().map(|e| e.content.as_str()).collect();
        // medium recent should rank well (recent + moderate importance)
        assert!(top_contents.contains(&"medium recent"));
    }

    #[test]
    fn test_format_for_context() {
        let mut mem = NpcMemory::new(10, true);
        mem.add_memory("Saw a dragon".to_string(), 0.9, 100.0);
        mem.add_memory("Ate breakfast".to_string(), 0.3, 100.0);

        let ctx = mem.format_for_context(5, 100.0);
        assert!(ctx.is_some());
        let text = ctx.unwrap();
        assert!(text.contains("Saw a dragon"));
        assert!(text.contains("Ate breakfast"));
    }

    #[test]
    fn test_empty_memory() {
        let mem = NpcMemory::new(10, true);
        assert!(mem.top_memories(5, 0.0).is_empty());
        assert!(mem.format_for_context(5, 0.0).is_none());
    }
}
