//! Write-side memory tools for MCP server.
//!
//! These complement the read-side tools (memory_search, memory_get) from
//! `agent::tools` by allowing MCP clients to write to LocalGPT's memory
//! in a structured way — without needing direct file access.

use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::debug;

use crate::agent::providers::ToolSchema;
use crate::agent::tools::Tool;

/// Append content to MEMORY.md (long-term curated knowledge).
pub struct MemorySaveTool {
    workspace: PathBuf,
}

impl MemorySaveTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for MemorySaveTool {
    fn name(&self) -> &str {
        "memory_save"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "memory_save".to_string(),
            description: "Append content to MEMORY.md — long-term curated knowledge (user info, preferences, key decisions). Use for important facts that should persist across sessions.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to append to MEMORY.md (markdown format)"
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

        let memory_file = self.workspace.join("MEMORY.md");
        debug!("memory_save: appending to {}", memory_file.display());

        // Ensure workspace exists
        std::fs::create_dir_all(&self.workspace)?;

        // Append with a newline separator
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&memory_file)?;

        // Add a blank line before the new content if file is not empty
        let metadata = file.metadata()?;
        if metadata.len() > 0 {
            writeln!(file)?;
        }
        write!(file, "{}", content)?;

        Ok(format!(
            "Saved to MEMORY.md ({} bytes appended)",
            content.len()
        ))
    }
}

/// Append an entry to today's daily log (memory/YYYY-MM-DD.md).
pub struct MemoryLogTool {
    workspace: PathBuf,
}

impl MemoryLogTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for MemoryLogTool {
    fn name(&self) -> &str {
        "memory_log"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "memory_log".to_string(),
            description: "Append an entry to today's daily log (memory/YYYY-MM-DD.md). Use for session notes, task progress, and observations that are date-specific.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to append to today's daily log (markdown format)"
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let memory_dir = self.workspace.join("memory");
        let log_file = memory_dir.join(format!("{}.md", today));
        debug!("memory_log: appending to {}", log_file.display());

        // Ensure memory dir exists
        std::fs::create_dir_all(&memory_dir)?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        // Add header if file is new
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            writeln!(file, "# {}\n", today)?;
        } else {
            writeln!(file)?;
        }
        write!(file, "{}", content)?;

        Ok(format!(
            "Logged to memory/{}.md ({} bytes)",
            today,
            content.len()
        ))
    }
}

/// Create the write-side memory tools.
pub fn create_memory_write_tools(workspace: PathBuf) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(MemorySaveTool::new(workspace.clone())),
        Box::new(MemoryLogTool::new(workspace)),
    ]
}
