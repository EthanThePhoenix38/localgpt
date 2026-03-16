//! MCP server for LocalGPT Gen — exposes gen tools + core tools over stdio.
//!
//! This allows external CLI backends (gemini-cli, claude cli, codex) and
//! MCP-capable editors (VS Code, Zed, Cursor) to drive the Bevy scene.
//!
//! Exposed tools:
//! - All gen tools (spawn, modify, camera, audio, behaviors, world, etc.)
//! - memory_search, memory_get (read), memory_save, memory_log (write)
//! - web_fetch, web_search

use std::sync::Arc;

use anyhow::Result;

use localgpt_core::agent::tools::Tool;
use localgpt_core::config::Config;

use crate::gen3d::GenBridge;

/// Run the MCP stdio server with gen tools + core tools.
pub async fn run_mcp_server(bridge: Arc<GenBridge>, config: Config) -> Result<()> {
    let tools = create_mcp_tools(bridge, &config)?;
    localgpt_core::mcp::server::run_mcp_stdio_server(tools, "localgpt-gen").await
}

/// Create the combined tool set for the MCP server:
/// gen tools + safe core tools + memory write tools.
///
/// CLI tools (bash, read_file, write_file, edit_file) are excluded because
/// external CLI backends already have their own file/shell tools.
fn create_mcp_tools(bridge: Arc<GenBridge>, config: &Config) -> Result<Vec<Box<dyn Tool>>> {
    use localgpt_core::agent::tools::create_safe_tools;
    use localgpt_core::mcp::memory_tools::create_memory_write_tools;
    use localgpt_core::memory::MemoryManager;

    let workspace = config.workspace_path();

    // Core read tools: memory_search, memory_get, web_fetch, web_search
    let memory = MemoryManager::new_with_agent(&config.memory, "gen-mcp")?;
    let memory = Arc::new(memory);
    let mut tools = create_safe_tools(config, Some(memory))?;

    // Core write tools: memory_save, memory_log
    tools.extend(create_memory_write_tools(workspace));

    // Gen tools: all scene manipulation tools
    tools.extend(crate::gen3d::tools::create_gen_tools(bridge.clone()));

    // P1/P2/P3/P4/P5 tools: character + interaction + terrain + UI + physics
    tools.extend(crate::mcp::avatar_tools::create_character_tools(
        bridge.clone(),
    ));
    tools.extend(crate::mcp::interaction_tools::create_interaction_tools(
        bridge.clone(),
    ));
    tools.extend(crate::mcp::terrain_tools::create_terrain_tools(
        bridge.clone(),
    ));
    tools.extend(crate::mcp::ui_tools::create_ui_tools(bridge.clone()));
    tools.extend(crate::mcp::physics_tools::create_physics_tools(
        bridge.clone(),
    ));

    // WG1 tools: worldgen blockout pipeline
    tools.extend(crate::mcp::worldgen_tools::create_worldgen_tools(
        bridge.clone(),
    ));

    // AI1 tools: AI asset generation
    tools.extend(crate::mcp::asset_gen_tools::create_asset_gen_tools(bridge));

    Ok(tools)
}
