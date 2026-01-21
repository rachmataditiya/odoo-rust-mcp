pub mod prompts;
pub mod runtime;
pub mod tools;
pub mod cursor_stdio;
pub mod http;

use std::collections::HashMap;
use async_trait::async_trait;
use mcp_rust_sdk::error::{Error, ErrorCode};
use mcp_rust_sdk::server::ServerHandler;
use mcp_rust_sdk::types::{ClientCapabilities, Implementation, ServerCapabilities};
use serde_json::{json, Value};

use crate::mcp::prompts::{get_prompt_result, list_prompts_result};
use crate::mcp::tools::{call_tool, tool_defs, OdooClientPool};

#[derive(Clone)]
pub struct McpOdooHandler {
    pool: OdooClientPool,
    enable_cleanup_tools: bool,
}

impl McpOdooHandler {
    pub fn new(pool: OdooClientPool, enable_cleanup_tools: bool) -> Self {
        Self {
            pool,
            enable_cleanup_tools,
        }
    }

    pub fn instance_names(&self) -> Vec<String> {
        self.pool.instance_names()
    }
}

fn protocol_err(message: impl Into<String>) -> Error {
    Error::protocol(ErrorCode::InvalidRequest, message)
}

#[async_trait]
impl ServerHandler for McpOdooHandler {
    async fn initialize(
        &self,
        _implementation: Implementation,
        _capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities, Error> {
        // mcp_rust_sdk ServerCapabilities is currently "custom" only, so we advertise tools/prompts in custom.
        let mut custom = HashMap::new();
        custom.insert("tools".to_string(), json!({}));
        custom.insert("prompts".to_string(), json!({}));
        custom.insert(
            "odooInstances".to_string(),
            json!({ "available": self.pool.instance_names() }),
        );
        Ok(ServerCapabilities {
            custom: Some(custom),
        })
    }

    async fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_method(&self, method: &str, params: Option<Value>) -> Result<Value, Error> {
        match method {
            "tools/list" => {
                let tools = tool_defs(self.enable_cleanup_tools);
                Ok(json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = params.ok_or_else(|| protocol_err("Missing params for tools/call"))?;
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| protocol_err("tools/call missing 'name'"))?;
                let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

                if !self.enable_cleanup_tools
                    && matches!(name, "odoo_database_cleanup" | "odoo_deep_cleanup")
                {
                    return Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&json!({
                                "error": "Cleanup tools are disabled. Set ODOO_ENABLE_CLEANUP_TOOLS=true (or --enable-cleanup-tools) to enable.",
                                "tool": name,
                            })).unwrap_or_else(|_| "{\"error\":\"disabled\"}".to_string())
                        }],
                        "isError": true
                    }));
                }

                match call_tool(&self.pool, name, args).await {
                    Ok(v) => Ok(v),
                    Err(e) => Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&json!({
                                "error": e.to_string(),
                                "tool": name,
                            })).unwrap_or_else(|_| "{\"error\":\"unknown\"}".to_string())
                        }],
                        "isError": true
                    })),
                }
            }
            "prompts/list" => Ok(list_prompts_result()),
            "prompts/get" => {
                let params = params.ok_or_else(|| protocol_err("Missing params for prompts/get"))?;
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| protocol_err("prompts/get missing 'name'"))?;
                get_prompt_result(name).ok_or_else(|| protocol_err(format!("Unknown prompt: {name}")))
            }
            _ => Err(protocol_err(format!("Unknown method: {method}"))),
        }
    }
}

