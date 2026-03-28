//! MCP conformance test suite.
//!
//! Validates all MCP tools against the MCP spec's tool-calling contract:
//! - Tool registration correctness (non-empty name, description, valid schema)
//! - Parameter schema validity (type=object, properties, required)
//! - Name uniqueness
//! - Name format (snake_case, no spaces)
//! - JSON-RPC dispatch (tools/list, tools/call, initialize, ping)
//! - Error handling (unknown tool, unknown method)
//! - Invalid parameters (missing required params)

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

use localgpt_core::agent::ToolSchema;
use localgpt_core::agent::tools::Tool;
use localgpt_core::mcp::server::{McpHandler, ToolHandler};

// ---------------------------------------------------------------------------
// Mock tools — lightweight tools to exercise the MCP contract without Bevy
// ---------------------------------------------------------------------------

/// A minimal tool with all required fields present.
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "echo".to_string(),
            description: "Echo back the input message.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo"
                    }
                },
                "required": ["message"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let message = args["message"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;
        Ok(message.to_string())
    }
}

/// A tool with no required parameters.
struct StatusTool;

#[async_trait]
impl Tool for StatusTool {
    fn name(&self) -> &str {
        "get_status"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "get_status".to_string(),
            description: "Get the current status.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "verbose": {
                        "type": "boolean",
                        "description": "Include extra details",
                        "default": false
                    }
                }
            }),
        }
    }

    async fn execute(&self, _arguments: &str) -> Result<String> {
        Ok("ok".to_string())
    }
}

/// A tool that always returns an error.
struct FailTool;

#[async_trait]
impl Tool for FailTool {
    fn name(&self) -> &str {
        "always_fail"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "always_fail".to_string(),
            description: "A tool that always fails for testing error handling.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _arguments: &str) -> Result<String> {
        anyhow::bail!("intentional failure")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_test_handler() -> ToolHandler {
    let workspace = PathBuf::from("/tmp/mcp-conformance-test");

    let mut tools: Vec<Box<dyn Tool>> =
        vec![Box::new(EchoTool), Box::new(StatusTool), Box::new(FailTool)];

    // Add real memory write tools for broader coverage
    tools.extend(localgpt_core::mcp::memory_tools::create_memory_write_tools(
        workspace,
    ));

    ToolHandler::new("test-server", tools)
}

/// Helper: dispatch a JSON-RPC request and return the response value.
async fn dispatch_request(handler: &ToolHandler, method: &str, params: Value) -> Value {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    handler
        .dispatch(&msg)
        .await
        .expect("requests with id must return a response")
}

// ---------------------------------------------------------------------------
// Tool registration correctness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_names_are_non_empty() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(!name.is_empty(), "Tool name must not be empty");
    }
}

#[tokio::test]
async fn tool_descriptions_are_non_empty() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        let desc = tool["description"].as_str().unwrap();
        assert!(
            !desc.is_empty(),
            "Tool '{}' must have a non-empty description",
            name
        );
    }
}

#[tokio::test]
async fn tool_schemas_have_valid_json_schema_type() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        let schema = &tool["inputSchema"];

        assert!(
            schema.is_object(),
            "Tool '{}' inputSchema must be a JSON object",
            name
        );

        assert_eq!(
            schema["type"].as_str(),
            Some("object"),
            "Tool '{}' inputSchema.type must be \"object\"",
            name
        );

        assert!(
            schema["properties"].is_object(),
            "Tool '{}' inputSchema must have \"properties\" object",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Parameter schema validity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_properties_have_type_field() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        let properties = tool["inputSchema"]["properties"].as_object().unwrap();

        for (prop_name, prop_schema) in properties {
            assert!(
                prop_schema.get("type").is_some() || prop_schema.get("$ref").is_some(),
                "Tool '{}' property '{}' must have a \"type\" or \"$ref\" field",
                name,
                prop_name
            );
        }
    }
}

#[tokio::test]
async fn required_fields_reference_existing_properties() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        let schema = &tool["inputSchema"];
        let properties = schema["properties"].as_object().unwrap();

        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            for req in required {
                let req_name = req.as_str().unwrap();
                assert!(
                    properties.contains_key(req_name),
                    "Tool '{}' lists '{}' as required but it is not in properties",
                    name,
                    req_name
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Name uniqueness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_names_are_unique() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    let mut seen = HashSet::new();
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            seen.insert(name.to_string()),
            "Duplicate tool name: '{}'",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Name format (MCP convention: snake_case, no spaces)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_names_follow_snake_case_convention() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    let valid_name = regex::Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            valid_name.is_match(name),
            "Tool name '{}' does not follow snake_case convention (must match [a-z][a-z0-9_]*)",
            name
        );
    }
}

#[tokio::test]
async fn tool_names_have_no_spaces() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            !name.contains(' '),
            "Tool name '{}' must not contain spaces",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatch: tools/list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_list_returns_result_with_tools_array() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/list", json!({})).await;

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(
        resp.get("error").is_none(),
        "tools/list should not return an error"
    );

    let tools = resp["result"]["tools"]
        .as_array()
        .expect("result.tools must be an array");
    // We registered 5 tools: echo, get_status, always_fail, memory_save, memory_log
    assert_eq!(tools.len(), 5, "Expected 5 registered tools");
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatch: tools/call
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_call_echo_returns_content() {
    let handler = create_test_handler();
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({
            "name": "echo",
            "arguments": { "message": "hello world" }
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp.get("error").is_none(), "echo should succeed");

    let content = resp["result"]["content"]
        .as_array()
        .expect("result.content must be an array");
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "hello world");
}

#[tokio::test]
async fn tools_call_returns_mcp_content_format() {
    let handler = create_test_handler();
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({
            "name": "get_status",
            "arguments": {}
        }),
    )
    .await;

    // MCP spec: successful tool call returns { content: [{ type: "text", text: "..." }] }
    let content = resp["result"]["content"].as_array().unwrap();
    assert!(!content.is_empty(), "Content array must not be empty");
    for item in content {
        assert!(
            item.get("type").is_some(),
            "Each content item must have a 'type' field"
        );
        assert!(
            item.get("text").is_some(),
            "Each text content item must have a 'text' field"
        );
    }
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatch: initialize
// ---------------------------------------------------------------------------

#[tokio::test]
async fn initialize_returns_capabilities_and_server_info() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "initialize", json!({})).await;

    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp.get("error").is_none());

    let result = &resp["result"];
    assert!(
        result["protocolVersion"].is_string(),
        "Must return protocolVersion"
    );
    assert!(
        result["capabilities"].is_object(),
        "Must return capabilities"
    );
    assert!(
        result["capabilities"]["tools"].is_object(),
        "Must declare tools capability"
    );
    assert!(
        result["serverInfo"]["name"].is_string(),
        "Must return serverInfo.name"
    );
    assert_eq!(result["serverInfo"]["name"], "test-server");
    assert!(
        result["serverInfo"]["version"].is_string(),
        "Must return serverInfo.version"
    );
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatch: ping
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ping_returns_empty_result() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "ping", json!({})).await;

    assert_eq!(resp["jsonrpc"], "2.0");
    assert!(resp.get("error").is_none());
    assert!(resp["result"].is_object());
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatch: notifications (no id)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn notifications_return_no_response() {
    let handler = create_test_handler();

    // Notifications have no "id" field — server must not respond.
    let msg = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
    });
    let result = handler.dispatch(&msg).await;
    assert!(
        result.is_none(),
        "Notifications must not produce a response"
    );

    let msg = json!({
        "jsonrpc": "2.0",
        "method": "notifications/cancelled",
    });
    let result = handler.dispatch(&msg).await;
    assert!(
        result.is_none(),
        "Notifications must not produce a response"
    );
}

// ---------------------------------------------------------------------------
// Error handling: unknown tool
// ---------------------------------------------------------------------------

#[tokio::test]
async fn calling_unknown_tool_returns_error() {
    let handler = create_test_handler();
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({
            "name": "nonexistent_tool",
            "arguments": {}
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    let error = &resp["error"];
    assert!(error.is_object(), "Must return an error object");
    assert!(error["code"].is_number(), "Error must have a numeric code");
    assert!(
        error["message"].is_string(),
        "Error must have a message string"
    );
    let msg = error["message"].as_str().unwrap();
    assert!(
        msg.contains("nonexistent_tool"),
        "Error message should mention the unknown tool name, got: {}",
        msg
    );
}

// ---------------------------------------------------------------------------
// Error handling: unknown method
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unknown_method_returns_method_not_found_error() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "bogus/method", json!({})).await;

    assert_eq!(resp["jsonrpc"], "2.0");
    let error = &resp["error"];
    assert!(error.is_object());
    assert_eq!(
        error["code"].as_i64(),
        Some(-32601),
        "Unknown method should return JSON-RPC -32601 (Method not found)"
    );
}

// ---------------------------------------------------------------------------
// Error handling: missing name in tools/call
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_call_without_name_returns_error() {
    let handler = create_test_handler();
    let resp = dispatch_request(&handler, "tools/call", json!({ "arguments": {} })).await;

    assert_eq!(resp["jsonrpc"], "2.0");
    let error = &resp["error"];
    assert!(error.is_object(), "Missing 'name' should return an error");
    assert_eq!(
        error["code"].as_i64(),
        Some(-32602),
        "Missing name should return JSON-RPC -32602 (Invalid params)"
    );
}

// ---------------------------------------------------------------------------
// Error handling: tool execution failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_execution_error_returns_is_error_flag() {
    let handler = create_test_handler();
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({
            "name": "always_fail",
            "arguments": {}
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    // MCP spec: tool execution errors are returned in the result with isError=true,
    // not as a JSON-RPC error.
    assert!(
        resp.get("error").is_none(),
        "Tool errors use result.isError, not JSON-RPC error"
    );
    let result = &resp["result"];
    assert_eq!(
        result["isError"].as_bool(),
        Some(true),
        "Failed tool must set isError=true"
    );
    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty());
    let text = content[0]["text"].as_str().unwrap();
    assert!(
        text.contains("intentional failure"),
        "Error text should contain the failure message, got: {}",
        text
    );
}

// ---------------------------------------------------------------------------
// Invalid parameters: missing required params
// ---------------------------------------------------------------------------

#[tokio::test]
async fn echo_tool_with_missing_required_param_returns_error() {
    let handler = create_test_handler();
    // echo requires "message" but we pass empty args
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({
            "name": "echo",
            "arguments": {}
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    // The tool should return an execution error (isError=true)
    let result = &resp["result"];
    assert_eq!(
        result["isError"].as_bool(),
        Some(true),
        "Missing required param should cause tool error with isError=true"
    );
}

#[tokio::test]
async fn memory_save_with_missing_content_returns_error() {
    let handler = create_test_handler();
    // memory_save requires "content" but we pass empty args
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({
            "name": "memory_save",
            "arguments": {}
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    let result = &resp["result"];
    assert_eq!(
        result["isError"].as_bool(),
        Some(true),
        "memory_save with missing 'content' should return isError=true"
    );
}

// ---------------------------------------------------------------------------
// JSON-RPC response envelope correctness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn all_responses_include_jsonrpc_and_id() {
    let handler = create_test_handler();

    let methods = [
        ("initialize", json!({})),
        ("tools/list", json!({})),
        (
            "tools/call",
            json!({"name": "echo", "arguments": {"message": "test"}}),
        ),
        ("ping", json!({})),
        ("bogus/method", json!({})),
    ];

    for (i, (method, params)) in methods.iter().enumerate() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": i + 1,
            "method": method,
            "params": params,
        });
        let resp = handler.dispatch(&msg).await.unwrap();
        assert_eq!(
            resp["jsonrpc"], "2.0",
            "Response for '{}' must have jsonrpc: \"2.0\"",
            method
        );
        assert_eq!(
            resp["id"].as_u64(),
            Some((i + 1) as u64),
            "Response for '{}' must echo back the request id",
            method
        );
    }
}

#[tokio::test]
async fn response_has_either_result_or_error_never_both() {
    let handler = create_test_handler();

    // Successful call
    let resp = dispatch_request(
        &handler,
        "tools/call",
        json!({"name": "echo", "arguments": {"message": "hi"}}),
    )
    .await;
    assert!(
        resp.get("result").is_some() && resp.get("error").is_none(),
        "Successful response must have result but not error"
    );

    // Error call (unknown method)
    let resp = dispatch_request(&handler, "bogus/method", json!({})).await;
    assert!(
        resp.get("error").is_some() && resp.get("result").is_none(),
        "Error response must have error but not result"
    );
}

// ---------------------------------------------------------------------------
// Tool schema consistency: name() matches schema().name
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tool_name_matches_schema_name() {
    // Validate that each tool's name() method returns the same value as schema().name
    let workspace = PathBuf::from("/tmp/mcp-conformance-test");

    let tools: Vec<Box<dyn Tool>> =
        vec![Box::new(EchoTool), Box::new(StatusTool), Box::new(FailTool)];
    let memory_tools = localgpt_core::mcp::memory_tools::create_memory_write_tools(workspace);

    for tool in tools.iter().chain(memory_tools.iter()) {
        let name = tool.name();
        let schema = tool.schema();
        assert_eq!(
            name, schema.name,
            "Tool.name() '{}' must match ToolSchema.name '{}'",
            name, schema.name
        );
    }
}
