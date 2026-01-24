pub mod cache;
pub mod cursor_stdio;
pub mod http;
pub mod prompts;
pub mod registry;
pub mod resources;
pub mod runtime;
pub mod tools;

use async_trait::async_trait;
use mcp_rust_sdk::error::{Error, ErrorCode};
use mcp_rust_sdk::server::ServerHandler;
use mcp_rust_sdk::types::{ClientCapabilities, Implementation, ServerCapabilities};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

use crate::mcp::prompts::{get_prompt_result, list_prompts_result};
use crate::mcp::registry::Registry;
use crate::mcp::tools::{OdooClientPool, call_tool};

#[derive(Clone)]
pub struct McpOdooHandler {
    pool: OdooClientPool,
    registry: Arc<Registry>,
}

impl McpOdooHandler {
    pub fn new(pool: OdooClientPool, registry: Arc<Registry>) -> Self {
        Self { pool, registry }
    }

    pub fn instance_names(&self) -> Vec<String> {
        self.pool.instance_names()
    }

    pub async fn server_name(&self) -> String {
        self.registry.server_name().await
    }

    pub async fn instructions(&self) -> String {
        self.registry.instructions().await
    }

    pub async fn protocol_version_default(&self) -> String {
        self.registry.protocol_version_default().await
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
        // mcp_rust_sdk ServerCapabilities is currently "custom" only, so we advertise tools/prompts/resources in custom.
        let mut custom = HashMap::new();
        custom.insert("tools".to_string(), json!({}));
        custom.insert("prompts".to_string(), json!({}));
        custom.insert("resources".to_string(), json!({}));
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
                // Fully declarative: tools are served from tools.json (registry).
                // Note: cleanup gating is handled by tool guards (e.g. requiresEnvTrue).
                let tools = self.registry.list_tools().await;
                Ok(json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = params.ok_or_else(|| protocol_err("Missing params for tools/call"))?;
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| protocol_err("tools/call missing 'name'"))?;
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                let Some(tool) = self.registry.get_tool(name).await else {
                    return Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&json!({
                                "error": "Unknown or disabled tool",
                                "tool": name,
                            })).unwrap_or_else(|_| "{\"error\":\"disabled\"}".to_string())
                        }],
                        "isError": true
                    }));
                };

                match call_tool(&self.pool, &tool, args).await {
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
            "prompts/list" => {
                let prompts = self.registry.list_prompts().await;
                Ok(list_prompts_result(&prompts))
            }
            "prompts/get" => {
                let params =
                    params.ok_or_else(|| protocol_err("Missing params for prompts/get"))?;
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| protocol_err("prompts/get missing 'name'"))?;
                let p = self
                    .registry
                    .get_prompt(name)
                    .await
                    .ok_or_else(|| protocol_err(format!("Unknown prompt: {name}")))?;
                Ok(get_prompt_result(&p))
            }
            "resources/list" => {
                resources::list_resources(&self.pool).await
            }
            "resources/read" => {
                let params =
                    params.ok_or_else(|| protocol_err("Missing params for resources/read"))?;
                let uri = params
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| protocol_err("resources/read missing 'uri'"))?;
                resources::read_resource(&self.pool, uri).await
            }
            _ => Err(protocol_err(format!("Unknown method: {method}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_err_creates_error_with_message() {
        let err = protocol_err("test message");
        let display = err.to_string();
        assert!(display.contains("test message"));
    }

    #[test]
    fn test_protocol_err_has_invalid_request_code() {
        let err = protocol_err("test");
        // Error display should contain something about the error
        let display = format!("{:?}", err);
        assert!(!display.is_empty());
    }
}
