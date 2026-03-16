//! MCP tool handlers for WorldGen: procedural blockout pipeline.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::gen3d::GenBridge;
use crate::gen3d::commands::*;
use crate::worldgen;
use localgpt_core::agent::ToolSchema;
use localgpt_core::agent::tools::Tool;

// ---------------------------------------------------------------------------
// gen_plan_layout
// ---------------------------------------------------------------------------

pub struct GenPlanLayoutTool {
    bridge: Arc<GenBridge>,
}

impl GenPlanLayoutTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenPlanLayoutTool {
    fn name(&self) -> &str {
        "gen_plan_layout"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_plan_layout".to_string(),
            description: "Generate a structured world layout plan from a text description. Returns a BlockoutSpec JSON that can be reviewed/adjusted before applying with gen_apply_blockout.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "Natural language world description (e.g., 'a medieval village with a forest and lake')"
                    },
                    "size": {
                        "type": "array",
                        "items": { "type": "number" },
                        "default": [50, 50],
                        "description": "World dimensions in meters [X, Z]"
                    },
                    "seed": {
                        "type": "integer",
                        "description": "Random seed for deterministic generation (optional)"
                    }
                },
                "required": ["prompt"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;

        let prompt = args["prompt"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: prompt"))?;

        let size = args["size"]
            .as_array()
            .map(|a| {
                [
                    a[0].as_f64().unwrap_or(50.0) as f32,
                    a[1].as_f64().unwrap_or(50.0) as f32,
                ]
            })
            .unwrap_or([50.0, 50.0]);

        let seed = args["seed"].as_u64().map(|v| v as u32);

        let cmd = GenCommand::PlanLayout {
            prompt: prompt.to_string(),
            size,
            seed,
        };
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::BlockoutPlan { spec_json } => Ok(spec_json),
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Layout plan generated".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// gen_apply_blockout
// ---------------------------------------------------------------------------

pub struct GenApplyBlockoutTool {
    bridge: Arc<GenBridge>,
}

impl GenApplyBlockoutTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenApplyBlockoutTool {
    fn name(&self) -> &str {
        "gen_apply_blockout"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_apply_blockout".to_string(),
            description: "Generate a 3D blockout scene from a BlockoutSpec. Creates terrain, region debug volumes, hero slot markers, and connecting paths.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "spec": {
                        "type": "object",
                        "description": "BlockoutSpec JSON (from gen_plan_layout or manually constructed)"
                    },
                    "show_debug_volumes": {
                        "type": "boolean",
                        "default": true,
                        "description": "Render translucent region volumes for visualization"
                    },
                    "generate_terrain": {
                        "type": "boolean",
                        "default": true,
                        "description": "Generate terrain mesh from spec"
                    },
                    "generate_paths": {
                        "type": "boolean",
                        "default": true,
                        "description": "Generate path geometry between regions"
                    }
                },
                "required": ["spec"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;

        let spec_value = args
            .get("spec")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spec"))?;

        let spec: worldgen::BlockoutSpec = serde_json::from_value(spec_value.clone())
            .map_err(|e| anyhow::anyhow!("Invalid BlockoutSpec: {}", e))?;

        // Validate before sending
        spec.validate()
            .map_err(|e| anyhow::anyhow!("BlockoutSpec validation failed: {}", e))?;

        let show_debug_volumes = args["show_debug_volumes"].as_bool().unwrap_or(true);
        let generate_terrain = args["generate_terrain"].as_bool().unwrap_or(true);
        let generate_paths = args["generate_paths"].as_bool().unwrap_or(true);

        let cmd = GenCommand::ApplyBlockout {
            spec,
            show_debug_volumes,
            generate_terrain,
            generate_paths,
        };
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::BlockoutApplied {
                entities_spawned,
                regions,
                paths,
            } => {
                Ok(format!(
                    "Blockout applied: {} entities spawned ({} regions, {} paths). Use gen_populate_region to fill regions with content.",
                    entities_spawned, regions, paths
                ))
            }
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Blockout applied".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// gen_populate_region
// ---------------------------------------------------------------------------

pub struct GenPopulateRegionTool {
    bridge: Arc<GenBridge>,
}

impl GenPopulateRegionTool {
    pub fn new(bridge: Arc<GenBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for GenPopulateRegionTool {
    fn name(&self) -> &str {
        "gen_populate_region"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "gen_populate_region".to_string(),
            description: "Fill a blockout region with 3D content (foliage, decorations) based on its density and biome parameters. Hero slots should be filled manually with gen_spawn_primitive.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "region_id": {
                        "type": "string",
                        "description": "ID of the region to populate (from BlockoutSpec)"
                    },
                    "style_hint": {
                        "type": "string",
                        "description": "Additional style guidance (e.g., 'autumn colors', 'overgrown ruins')"
                    },
                    "replace_existing": {
                        "type": "boolean",
                        "default": false,
                        "description": "Clear existing content in region before populating"
                    }
                },
                "required": ["region_id"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;

        let region_id = args["region_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: region_id"))?;

        let style_hint = args["style_hint"].as_str().map(String::from);
        let replace_existing = args["replace_existing"].as_bool().unwrap_or(false);

        let cmd = GenCommand::PopulateRegion {
            region_id: region_id.to_string(),
            style_hint,
            replace_existing,
        };
        let response = self.bridge.send(cmd).await?;
        match response {
            GenResponse::RegionPopulated {
                region_id,
                entities_spawned,
            } => Ok(format!(
                "Region '{}' populated with {} entities",
                region_id, entities_spawned
            )),
            GenResponse::Error { message } => Err(anyhow::anyhow!("{}", message)),
            _ => Ok("Region populated".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

pub fn create_worldgen_tools(bridge: Arc<GenBridge>) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(GenPlanLayoutTool::new(bridge.clone())),
        Box::new(GenApplyBlockoutTool::new(bridge.clone())),
        Box::new(GenPopulateRegionTool::new(bridge)),
    ]
}
