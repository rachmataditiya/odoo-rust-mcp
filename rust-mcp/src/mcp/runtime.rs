use std::sync::Arc;

use futures::StreamExt;
use serde_json::json;
use tokio::sync::RwLock;

use mcp_rust_sdk::error::{Error, ErrorCode};
use mcp_rust_sdk::protocol::{Request, Response, ResponseError};
use mcp_rust_sdk::server::ServerHandler;
use mcp_rust_sdk::transport::{Message, Transport};

use super::McpOdooHandler;

pub struct ServerCompat {
    transport: Arc<dyn Transport>,
    handler: Arc<McpOdooHandler>,
    initialized: Arc<RwLock<bool>>,
}

impl ServerCompat {
    pub fn new(transport: Arc<dyn Transport>, handler: Arc<McpOdooHandler>) -> Self {
        Self {
            transport,
            handler,
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), Error> {
        let mut stream = self.transport.receive();
        while let Some(message) = stream.next().await {
            match message? {
                Message::Request(request) => {
                    let response = match self.handle_request(request.clone()).await {
                        Ok(resp) => resp,
                        Err(err) => Response::error(request.id, ResponseError::from(err)),
                    };
                    self.transport.send(Message::Response(response)).await?;
                }
                Message::Notification(notification) => match notification.method.as_str() {
                    "exit" => break,
                    "initialized" => {
                        *self.initialized.write().await = true;
                    }
                    _ => {}
                },
                Message::Response(_) => {
                    return Err(Error::protocol(
                        ErrorCode::InvalidRequest,
                        "Server received unexpected response",
                    ));
                }
            }
        }
        Ok(())
    }

    async fn handle_request(&self, request: Request) -> Result<Response, Error> {
        let initialized = *self.initialized.read().await;

        match request.method.as_str() {
            "initialize" => {
                if initialized {
                    return Err(Error::protocol(
                        ErrorCode::InvalidRequest,
                        "Server already initialized",
                    ));
                }

                let params = request.params.unwrap_or(serde_json::json!({}));

                // Get protocol version from client or use default
                let default_protocol = self.handler.protocol_version_default().await;
                let protocol_version = params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(default_protocol);

                let server_name = self.handler.server_name().await;
                let instructions = self.handler.instructions().await;
                let odoo_instances = self.handler.instance_names();

                // Build MCP-compliant initialize response with protocolVersion, capabilities, serverInfo
                let result = json!({
                    "protocolVersion": protocol_version,
                    "capabilities": {
                        "tools": { "listChanged": true },
                        "prompts": { "listChanged": true },
                        "resources": {},
                        "experimental": {
                            "odooInstances": { "available": odoo_instances }
                        }
                    },
                    "serverInfo": {
                        "name": server_name,
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "instructions": instructions
                });

                Ok(Response::success(request.id, Some(result)))
            }
            "shutdown" => {
                if !initialized {
                    return Err(Error::protocol(
                        ErrorCode::ServerNotInitialized,
                        "Server not initialized",
                    ));
                }
                self.handler.shutdown().await?;
                Ok(Response::success(request.id, None))
            }
            _ => {
                // Allow tools/list and prompts/list without initialized notification
                // Some clients (like Cursor) may not send initialized before listing
                let allow_without_init = matches!(
                    request.method.as_str(),
                    "tools/list" | "prompts/list" | "resources/list"
                );
                if !initialized && !allow_without_init {
                    return Err(Error::protocol(
                        ErrorCode::ServerNotInitialized,
                        "Server not initialized",
                    ));
                }
                let result = self
                    .handler
                    .handle_method(&request.method, request.params)
                    .await?;
                Ok(Response::success(request.id, Some(result)))
            }
        }
    }
}
