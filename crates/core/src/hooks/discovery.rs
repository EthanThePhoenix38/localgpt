//! Hook definition and discovery from filesystem

use std::fs;
use std::path::Path;

use directories::ProjectDirs;
use serde::Deserialize;
use tracing::{debug, warn};

/// Hook definition loaded from a .hook.json file
#[derive(Debug, Clone, Deserialize)]
pub struct HookDef {
    /// Unique name for this hook
    pub name: String,

    /// Event type this hook listens for (e.g., "before_tool_call")
    pub event: String,

    /// Shell command to execute
    pub command: String,

    /// Timeout in milliseconds (default: 5000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Whether this hook is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_timeout() -> u64 {
    5000
}

fn default_enabled() -> bool {
    true
}

impl HookDef {
    /// Check if this hook is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if this hook matches the given event name
    pub fn matches_event(&self, event_name: &str) -> bool {
        self.event == event_name
    }
}

/// Discover all hooks from the workspace and global hooks directories
///
/// Looks in:
/// - workspace/hooks/*.hook.json
/// - ~/.local/share/localgpt/hooks/*.hook.json (global hooks, XDG_DATA_HOME)
pub fn discover_hooks(workspace: &Path) -> Vec<HookDef> {
    let mut hooks = Vec::new();

    // Workspace hooks
    let workspace_hooks_dir = workspace.join("hooks");
    if let Ok(found) = load_hooks_from_dir(&workspace_hooks_dir) {
        debug!("Found {} hooks in workspace", found.len());
        hooks.extend(found);
    }

    // Global hooks from XDG data directory
    if let Some(proj_dirs) = ProjectDirs::from("app", "LocalGPT", "localgpt") {
        let global_hooks_dir = proj_dirs.data_dir().join("hooks");
        if let Ok(found) = load_hooks_from_dir(&global_hooks_dir) {
            debug!("Found {} global hooks", found.len());
            hooks.extend(found);
        }
    }

    hooks
}

/// Load all hook definitions from a directory
fn load_hooks_from_dir(dir: &Path) -> Result<Vec<HookDef>, std::io::Error> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut hooks = Vec::new();

    let entries = fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if file_stem.ends_with(".hook") {
                match load_hook_from_file(&path) {
                    Ok(hook) => hooks.push(hook),
                    Err(e) => {
                        warn!("Failed to load hook from {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(hooks)
}

/// Load a single hook definition from a JSON file
fn load_hook_from_file(path: &Path) -> Result<HookDef, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    let hook: HookDef = serde_json::from_str(&content)?;
    Ok(hook)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_hook_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let hook_file = temp_dir.path().join("test.hook.json");

        let hook_content = r#"{
            "name": "test-hook",
            "event": "before_tool_call",
            "command": "/bin/echo",
            "timeout_ms": 1000,
            "enabled": true
        }"#;

        fs::write(&hook_file, hook_content).unwrap();

        let hook = load_hook_from_file(&hook_file).unwrap();
        assert_eq!(hook.name, "test-hook");
        assert_eq!(hook.event, "before_tool_call");
        assert_eq!(hook.command, "/bin/echo");
        assert_eq!(hook.timeout_ms, 1000);
        assert!(hook.is_enabled());
    }

    #[test]
    fn test_default_values() {
        let temp_dir = TempDir::new().unwrap();
        let hook_file = temp_dir.path().join("minimal.hook.json");

        let hook_content = r#"{
            "name": "minimal",
            "event": "after_tool_call",
            "command": "true"
        }"#;

        fs::write(&hook_file, hook_content).unwrap();

        let hook = load_hook_from_file(&hook_file).unwrap();
        assert_eq!(hook.timeout_ms, 5000); // default
        assert!(hook.is_enabled()); // default
    }

    #[test]
    fn test_matches_event() {
        let hook = HookDef {
            name: "test".to_string(),
            event: "before_tool_call".to_string(),
            command: "true".to_string(),
            timeout_ms: 5000,
            enabled: true,
        };

        assert!(hook.matches_event("before_tool_call"));
        assert!(!hook.matches_event("after_tool_call"));
    }
}
