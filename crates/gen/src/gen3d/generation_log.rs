//! Generation log — records tool invocations during worldgen for replay and debugging.

use bevy::prelude::*;
use localgpt_world_types::GenLogEntry;
use serde_json::Value;
use std::io;
use std::path::Path;

/// Tracks all tool invocations during a generation session.
#[derive(Resource, Default)]
pub struct GenerationLog {
    pub entries: Vec<GenLogEntry>,
    pub current_phase: Option<String>,
    seq_counter: u32,
}

#[allow(dead_code)]
impl GenerationLog {
    pub fn log(&mut self, tool: &str, args: &Value, result_hash: Option<String>) {
        self.seq_counter += 1;
        self.entries.push(GenLogEntry {
            seq: self.seq_counter,
            tool: tool.to_string(),
            args: args.clone(),
            result_hash,
            phase: self.current_phase.clone(),
            timestamp: None,
        });
    }

    pub fn set_phase(&mut self, phase: &str) {
        self.current_phase = Some(phase.to_string());
    }

    pub fn clear_phase(&mut self) {
        self.current_phase = None;
    }

    pub fn write_jsonl(&self, path: &Path) -> io::Result<()> {
        let lines: Vec<String> = self
            .entries
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .collect();
        std::fs::write(path, lines.join("\n"))
    }

    pub fn has_phases(&self) -> bool {
        self.entries.iter().any(|e| e.phase.is_some())
    }
}
