use std::sync::Arc;

use clap::{Parser, ValueEnum};
use mcp_rust_sdk::transport::websocket::WebSocketTransport;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing::{error, info};

use rust_mcp::mcp::McpOdooHandler;
use rust_mcp::mcp::cursor_stdio::CursorStdioTransport;
use rust_mcp::mcp::http as mcp_http;
use rust_mcp::mcp::registry::Registry;
use rust_mcp::mcp::runtime::ServerCompat;
use rust_mcp::mcp::tools::OdooClientPool;

#[derive(Debug, Clone, ValueEnum)]
enum TransportMode {
    Stdio,
    Ws,
    Http,
}

#[derive(Debug, Parser)]
#[command(name = "odoo-mcp-rust", version, about = "Odoo MCP server (Rust)")]
struct Cli {
    /// Transport mode (stdio for Claude Desktop, ws for standalone server)
    #[arg(long, value_enum, default_value_t = TransportMode::Stdio)]
    transport: TransportMode,

    /// Listen address for ws mode, e.g. 0.0.0.0:8787
    #[arg(long, default_value = "127.0.0.1:8787")]
    listen: String,

    /// Enable destructive cleanup tools (off by default)
    #[arg(long, env = "ODOO_ENABLE_CLEANUP_TOOLS", default_value_t = false)]
    enable_cleanup_tools: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let pool = OdooClientPool::from_env()?;
    let registry = Arc::new(Registry::from_env());
    registry.initial_load().await?;
    registry.start_watchers();

    // Cleanup tool gating is handled via tool guards (e.g. requiresEnvTrue=ODOO_ENABLE_CLEANUP_TOOLS).
    // We keep the CLI flag for compatibility, but it only affects the env var via clap env binding.
    let handler = Arc::new(McpOdooHandler::new(pool, registry));

    match cli.transport {
        TransportMode::Stdio => run_stdio(handler).await?,
        TransportMode::Ws => run_ws(handler, &cli.listen).await?,
        TransportMode::Http => run_http(handler, &cli.listen).await?,
    }

    Ok(())
}

async fn run_stdio(handler: Arc<McpOdooHandler>) -> anyhow::Result<()> {
    let (transport, _sender) = CursorStdioTransport::new();
    let server = ServerCompat::new(Arc::new(transport), handler);

    info!("MCP server starting (stdio)");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

async fn run_ws(handler: Arc<McpOdooHandler>, listen: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(listen).await?;
    info!("MCP server listening (ws) on {}", listen);

    loop {
        let (stream, addr) = listener.accept().await?;
        let handler = handler.clone();
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let transport = WebSocketTransport::from_stream(ws_stream);
                    let server = ServerCompat::new(Arc::new(transport), handler);
                    info!("Accepted ws connection from {}", addr);
                    if let Err(e) = server.start().await {
                        error!("ws server error: {}", e);
                    }
                }
                Err(e) => error!("ws accept error: {}", e),
            }
        });
    }
}

async fn run_http(handler: Arc<McpOdooHandler>, listen: &str) -> anyhow::Result<()> {
    info!("MCP server listening (http) on {}", listen);
    mcp_http::serve(handler, listen).await
}
