//! MCP stdio server — exposes LocalGPT tools to external AI backends.
//!
//! Reads JSON-RPC 2.0 messages from stdin, dispatches tool calls,
//! and writes responses to stdout. Used by both `localgpt mcp-server`
//! and `localgpt-gen --mcp-server`.

use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info};

use super::super::agent::tools::Tool;

/// Run the MCP stdio server loop with the given tool set.
///
/// Blocks until stdin is closed (i.e., the MCP client disconnects).
pub async fn run_mcp_stdio_server(tools: Vec<Box<dyn Tool>>, server_name: &str) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    info!(
        "MCP server '{}' ready: {} tools: {:?}",
        server_name,
        tools.len(),
        tool_names
    );

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            info!("MCP server: stdin closed, shutting down");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                debug!("MCP server: ignoring non-JSON line: {}", e);
                continue;
            }
        };

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        // Notifications (no id) — handle but don't respond
        if id.is_none() {
            match method {
                "notifications/initialized" => {
                    info!("MCP server: client initialized");
                }
                "notifications/cancelled" => {
                    debug!("MCP server: received cancellation");
                }
                _ => {
                    debug!("MCP server: ignoring notification '{}'", method);
                }
            }
            continue;
        }

        let id = id.unwrap();

        let response = match method {
            "initialize" => handle_initialize(server_name),
            "tools/list" => handle_tools_list(&tools),
            "tools/call" => handle_tools_call(&tools, &params).await,
            "ping" => Ok(json!({})),
            _ => Err(json!({
                "code": -32601,
                "message": format!("Method not found: {}", method),
            })),
        };

        let response_msg = match response {
            Ok(result) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result,
            }),
            Err(error) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": error,
            }),
        };

        let mut out = serde_json::to_string(&response_msg)?;
        out.push('\n');
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}

fn handle_initialize(server_name: &str) -> Result<Value, Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": server_name,
            "version": env!("CARGO_PKG_VERSION"),
        }
    }))
}

fn handle_tools_list(tools: &[Box<dyn Tool>]) -> Result<Value, Value> {
    let tool_defs: Vec<Value> = tools
        .iter()
        .map(|t| {
            let schema = t.schema();
            json!({
                "name": schema.name,
                "description": schema.description,
                "inputSchema": schema.parameters,
            })
        })
        .collect();

    Ok(json!({ "tools": tool_defs }))
}

async fn handle_tools_call(tools: &[Box<dyn Tool>], params: &Value) -> Result<Value, Value> {
    let tool_name = params.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
        json!({
            "code": -32602,
            "message": "Missing 'name' parameter",
        })
    })?;

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let tool = tools
        .iter()
        .find(|t| t.name() == tool_name)
        .ok_or_else(|| {
            json!({
                "code": -32602,
                "message": format!("Unknown tool: {}", tool_name),
            })
        })?;

    let args_str = serde_json::to_string(&arguments).unwrap_or_default();

    match tool.execute(&args_str).await {
        Ok(output) => Ok(json!({
            "content": [{
                "type": "text",
                "text": output,
            }]
        })),
        Err(e) => Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Error: {}", e),
            }],
            "isError": true,
        })),
    }
}
