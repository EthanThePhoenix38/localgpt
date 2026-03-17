//! MCP tools for experiment queue management.
//!
//! Three tools for queuing, listing, and checking experiment status.
//! Used by external AI backends via the MCP server.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use localgpt_core::agent::providers::ToolSchema;
use localgpt_core::agent::tools::Tool;

use crate::experiment::{
    Experiment, ExperimentStatus, ExperimentTracker, parse_variation, prompt_to_slug,
};

// ---------------------------------------------------------------------------
// gen_queue_experiment
// ---------------------------------------------------------------------------

/// Queue a world generation experiment for background processing.
pub struct GenQueueExperimentTool {
    tracker: Arc<ExperimentTracker>,
}

impl GenQueueExperimentTool {
    pub fn new(tracker: Arc<ExperimentTracker>) -> Self {
        Self { tracker }
    }
}

#[async_trait]
impl Tool for GenQueueExperimentTool {
    fn name(&self) -> &str {
        "gen_queue_experiment"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_queue_experiment".to_string(),
            description: "Queue a world generation experiment. The experiment \
                will be processed in the background. Use gen_experiment_status \
                to check progress."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "World generation prompt"
                    },
                    "name": {
                        "type": "string",
                        "description": "World name (used for skill folder name)"
                    },
                    "style": {
                        "type": "string",
                        "description": "Optional style hint or memory reference"
                    },
                    "variations": {
                        "type": "object",
                        "description": "Optional variation spec",
                        "properties": {
                            "axis": {
                                "type": "string",
                                "description": "Variation axis (e.g., 'lighting', 'density')"
                            },
                            "values": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Values to try for each variation"
                            }
                        }
                    }
                },
                "required": ["prompt", "name"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;

        let prompt = args["prompt"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'prompt'"))?;
        let name = args["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'name'"))?;
        let style = args["style"].as_str().map(|s| s.to_string());

        // Check for variation spec in args
        let variations = args.get("variations");
        let axis = variations.and_then(|v| v["axis"].as_str());
        let values: Option<Vec<String>> = variations.and_then(|v| {
            v["values"].as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        });

        // Also check for inline variation syntax in the prompt
        let (base_prompt, var_axis, var_values) =
            if let (Some(ax), Some(vals)) = (axis, values.as_ref()) {
                (prompt.to_string(), Some(ax.to_string()), Some(vals.clone()))
            } else if let Some((base, ax, vals)) = parse_variation(prompt) {
                (base, Some(ax), Some(vals))
            } else {
                (prompt.to_string(), None, None)
            };

        let mut experiment_ids = Vec::new();

        if let (Some(ax), Some(vals)) = (var_axis, var_values) {
            // Create variation group
            let group_id = format!("vg-{}", prompt_to_slug(name));
            for val in &vals {
                let var_prompt = format!("{} — {}: {}", base_prompt, ax, val);
                let slug = format!("{}-{}", prompt_to_slug(name), prompt_to_slug(val));
                let mut exp = Experiment::new(&var_prompt, &slug);
                exp.style = style.clone();
                exp.variation_group = Some(group_id.clone());
                exp.variation = Some((ax.clone(), val.clone()));
                self.tracker.append(&exp)?;
                experiment_ids.push(exp.id);
            }
        } else {
            // Single experiment
            let slug = prompt_to_slug(name);
            let mut exp = Experiment::new(&base_prompt, &slug);
            exp.style = style;
            self.tracker.append(&exp)?;
            experiment_ids.push(exp.id);
        }

        Ok(json!({
            "queued": experiment_ids.len(),
            "experiment_ids": experiment_ids,
        })
        .to_string())
    }
}

// ---------------------------------------------------------------------------
// gen_list_experiments
// ---------------------------------------------------------------------------

/// List all experiments and their statuses.
pub struct GenListExperimentsTool {
    tracker: Arc<ExperimentTracker>,
}

impl GenListExperimentsTool {
    pub fn new(tracker: Arc<ExperimentTracker>) -> Self {
        Self { tracker }
    }
}

#[async_trait]
impl Tool for GenListExperimentsTool {
    fn name(&self) -> &str {
        "gen_list_experiments"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_list_experiments".to_string(),
            description: "List all queued, running, and completed experiments \
                with their statuses, paths, and thumbnails."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["all", "pending", "running", "completed", "failed"],
                        "default": "all"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 20,
                        "description": "Max experiments to return"
                    }
                }
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;

        let status_filter = args["status"].as_str().unwrap_or("all");
        let limit = args["limit"].as_u64().unwrap_or(20) as usize;

        let all = self.tracker.read_all()?;

        let filtered: Vec<&Experiment> = all
            .iter()
            .filter(|e| match status_filter {
                "pending" => e.status == ExperimentStatus::Pending,
                "running" => e.status == ExperimentStatus::Running,
                "completed" => e.status == ExperimentStatus::Completed,
                "failed" => e.status == ExperimentStatus::Failed,
                _ => true,
            })
            .rev() // newest first
            .take(limit)
            .collect();

        let entries: Vec<serde_json::Value> = filtered
            .iter()
            .map(|e| {
                json!({
                    "id": e.id,
                    "prompt": e.prompt,
                    "status": format!("{}", e.status),
                    "output_path": e.output_path,
                    "screenshot_path": e.screenshot_path,
                    "entity_count": e.entity_count,
                    "duration_ms": e.duration_ms,
                    "queued_at": e.queued_at.to_rfc3339(),
                    "error": e.error,
                })
            })
            .collect();

        Ok(json!({
            "total": all.len(),
            "showing": entries.len(),
            "experiments": entries,
        })
        .to_string())
    }
}

// ---------------------------------------------------------------------------
// gen_experiment_status
// ---------------------------------------------------------------------------

/// Get detailed status of a specific experiment by ID.
pub struct GenExperimentStatusTool {
    tracker: Arc<ExperimentTracker>,
}

impl GenExperimentStatusTool {
    pub fn new(tracker: Arc<ExperimentTracker>) -> Self {
        Self { tracker }
    }
}

#[async_trait]
impl Tool for GenExperimentStatusTool {
    fn name(&self) -> &str {
        "gen_experiment_status"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_experiment_status".to_string(),
            description: "Get detailed status of a specific experiment by ID, \
                including output path, screenshot, entity count, and duration."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Experiment ID"
                    }
                },
                "required": ["id"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;

        let id = args["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'id'"))?;

        match self.tracker.get(id)? {
            Some(exp) => Ok(serde_json::to_string_pretty(&exp)?),
            None => Ok(json!({
                "error": format!("Experiment '{}' not found", id)
            })
            .to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Create experiment queue MCP tools.
pub fn create_experiment_tools(tracker: Arc<ExperimentTracker>) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(GenQueueExperimentTool::new(tracker.clone())),
        Box::new(GenListExperimentsTool::new(tracker.clone())),
        Box::new(GenExperimentStatusTool::new(tracker)),
    ]
}
