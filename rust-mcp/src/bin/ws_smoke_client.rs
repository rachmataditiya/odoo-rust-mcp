use clap::Parser;
use futures::{SinkExt, StreamExt};
use mcp_rust_sdk::protocol::{Notification, Request, RequestId};
use mcp_rust_sdk::transport::Message as McpMessage;
use serde_json::json;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug, Parser)]
#[command(name = "ws-smoke-client", about = "Minimal MCP-over-WS smoke client")]
struct Cli {
    /// WebSocket URL of the MCP server, e.g. ws://127.0.0.1:8787
    #[arg(long, default_value = "ws://127.0.0.1:8787")]
    url: String,

    /// Odoo instance name configured in the server (default: "default")
    #[arg(long, default_value = "default")]
    instance: String,

    /// Model to use for basic read operations
    #[arg(long, default_value = "res.partner")]
    model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let (mut ws, _resp) = connect_async(&cli.url).await?;
    let instance = cli.instance.clone();
    let model = cli.model.clone();

    async fn send(ws: &mut WsStream, msg: McpMessage) -> anyhow::Result<()> {
        let s = serde_json::to_string(&msg)?;
        ws.send(WsMessage::Text(s)).await?;
        Ok(())
    }

    async fn recv(ws: &mut WsStream) -> anyhow::Result<McpMessage> {
        loop {
            let m = ws
                .next()
                .await
                .ok_or_else(|| anyhow::anyhow!("websocket closed"))??;
            match m {
                WsMessage::Text(s) => return Ok(serde_json::from_str(&s)?),
                WsMessage::Binary(b) => return Ok(serde_json::from_slice(&b)?),
                WsMessage::Ping(_) | WsMessage::Pong(_) => continue,
                WsMessage::Close(_) => return Err(anyhow::anyhow!("websocket closed")),
                _ => continue,
            }
        }
    }

    // initialize
    let init = Request::new(
        "initialize",
        Some(json!({
            "implementation": { "name": "ws-smoke-client", "version": "0.1.0" },
            "capabilities": {},
            "protocolVersion": "2025-11-05"
        })),
        RequestId::Number(1),
    );
    send(&mut ws, McpMessage::Request(init)).await?;

    // wait init response
    let init_resp = recv(&mut ws).await?;
    match init_resp {
        McpMessage::Response(resp) => {
            if let Some(err) = resp.error {
                anyhow::bail!("initialize failed: {}", serde_json::to_string(&err)?);
            }
        }
        other => anyhow::bail!("expected initialize response, got: {other:?}"),
    }

    // initialized notification
    send(
        &mut ws,
        McpMessage::Notification(Notification::new("initialized", None)),
    )
    .await?;

    // tools/list
    let list_tools = Request::new("tools/list", Some(json!({})), RequestId::Number(2));
    send(&mut ws, McpMessage::Request(list_tools)).await?;
    let tools_resp = recv(&mut ws).await?;
    let tools_val = match tools_resp {
        McpMessage::Response(resp) => {
            if let Some(err) = resp.error {
                anyhow::bail!("tools/list failed: {}", serde_json::to_string(&err)?);
            }
            resp.result.unwrap_or_else(|| json!({}))
        }
        other => anyhow::bail!("expected tools/list response, got: {other:?}"),
    };
    let tool_names: Vec<String> = tools_val
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    println!("tools/list: {} tools", tool_names.len());
    for n in &tool_names {
        println!("- {n}");
    }

    // tools/call: odoo_count
    let count_req = Request::new(
        "tools/call",
        Some(json!({
            "name": "odoo_count",
            "arguments": {
                "instance": instance.clone(),
                "model": model.clone(),
                "domain": [[ "id", ">", 0 ]]
            }
        })),
        RequestId::Number(3),
    );
    send(&mut ws, McpMessage::Request(count_req)).await?;
    let count_resp = recv(&mut ws).await?;
    let count_text = match count_resp {
        McpMessage::Response(resp) => {
            if let Some(err) = resp.error {
                anyhow::bail!("odoo_count failed: {}", serde_json::to_string(&err)?);
            }
            let v = resp.result.unwrap_or_else(|| json!({}));
            v.get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|o| o.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("{}")
                .to_string()
        }
        other => anyhow::bail!("expected odoo_count response, got: {other:?}"),
    };
    let count_json: serde_json::Value = serde_json::from_str(&count_text).unwrap_or_else(|_| json!({ "raw": count_text }));
    println!("odoo_count result: {}", count_json);

    // tools/call: odoo_search_read (small sample)
    let sr_req = Request::new(
        "tools/call",
        Some(json!({
            "name": "odoo_search_read",
            "arguments": {
                "instance": instance.clone(),
                "model": model.clone(),
                "domain": [[ "id", ">", 0 ]],
                "fields": ["id", "name"],
                "limit": 2,
                "order": "id DESC"
            }
        })),
        RequestId::Number(4),
    );
    send(&mut ws, McpMessage::Request(sr_req)).await?;
    let sr_resp = recv(&mut ws).await?;
    let sr_text = match sr_resp {
        McpMessage::Response(resp) => {
            if let Some(err) = resp.error {
                anyhow::bail!("odoo_search_read failed: {}", serde_json::to_string(&err)?);
            }
            let v = resp.result.unwrap_or_else(|| json!({}));
            v.get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|o| o.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("{}")
                .to_string()
        }
        other => anyhow::bail!("expected odoo_search_read response, got: {other:?}"),
    };
    let sr_json: serde_json::Value = serde_json::from_str(&sr_text).unwrap_or_else(|_| json!({ "raw": sr_text }));
    let records = sr_json.get("records").cloned().unwrap_or(json!([]));
    let count = sr_json.get("count").cloned().unwrap_or(json!(null));
    println!("odoo_search_read count: {count}");
    println!("odoo_search_read sample records: {records}");

    // prompts/list
    let list_prompts = Request::new("prompts/list", Some(json!({})), RequestId::Number(5));
    send(&mut ws, McpMessage::Request(list_prompts)).await?;
    let prompts_resp = recv(&mut ws).await?;
    let prompts_val = match prompts_resp {
        McpMessage::Response(resp) => {
            if let Some(err) = resp.error {
                anyhow::bail!("prompts/list failed: {}", serde_json::to_string(&err)?);
            }
            resp.result.unwrap_or_else(|| json!({}))
        }
        other => anyhow::bail!("expected prompts/list response, got: {other:?}"),
    };
    let prompt_names: Vec<String> = prompts_val
        .get("prompts")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    println!("prompts/list: {}", prompt_names.join(", "));

    // exit
    send(&mut ws, McpMessage::Notification(Notification::new("exit", None))).await?;
    let _ = ws.close(None).await;

    Ok(())
}

