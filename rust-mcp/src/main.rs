use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use mcp_rust_sdk::transport::websocket::WebSocketTransport;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing::{error, info, warn};

use rust_mcp::mcp::McpOdooHandler;
use rust_mcp::mcp::cursor_stdio::CursorStdioTransport;
use rust_mcp::mcp::http as mcp_http;
use rust_mcp::mcp::registry::Registry;
use rust_mcp::mcp::runtime::ServerCompat;
use rust_mcp::mcp::tools::OdooClientPool;

/// Get user config directory: ~/.config/odoo-rust-mcp/
/// We use ~/.config/ for cross-platform consistency with the shell wrapper
fn get_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".config").join("odoo-rust-mcp"))
}

/// Get share directory for default configs (platform-specific)
fn get_share_dir() -> Option<PathBuf> {
    // Check common locations for Homebrew/system-installed configs
    let candidates = [
        // Homebrew Apple Silicon
        PathBuf::from("/opt/homebrew/share/odoo-rust-mcp"),
        // Homebrew Intel Mac
        PathBuf::from("/usr/local/share/odoo-rust-mcp"),
        // Linux (APT, manual install)
        PathBuf::from("/usr/share/rust-mcp"),
        PathBuf::from("/usr/local/share/odoo-rust-mcp"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

/// Default env file template
const DEFAULT_ENV_TEMPLATE: &str = r#"# Odoo Rust MCP Server Configuration
# Edit this file with your Odoo credentials

# Odoo 19+ (API Key authentication)
ODOO_URL=http://localhost:8069
ODOO_DB=mydb
ODOO_API_KEY=YOUR_API_KEY

# Odoo < 19 (Username/Password authentication)
# ODOO_URL=http://localhost:8069
# ODOO_DB=mydb
# ODOO_VERSION=18
# ODOO_USERNAME=admin
# ODOO_PASSWORD=admin

# MCP Authentication (HTTP transport)
# Generate a secure token: openssl rand -hex 32
# MCP_AUTH_TOKEN=your-secure-random-token-here
"#;

/// Setup user config directory and load environment variables
fn setup_user_config() {
    let Some(config_dir) = get_config_dir() else {
        warn!("Could not determine user config directory");
        return;
    };

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            warn!("Failed to create config directory {:?}: {}", config_dir, e);
        } else {
            info!("Created config directory: {:?}", config_dir);
        }
    }

    // Create default env file if it doesn't exist
    let env_file = config_dir.join("env");
    if !env_file.exists() {
        if let Err(e) = fs::write(&env_file, DEFAULT_ENV_TEMPLATE) {
            warn!("Failed to create default env file {:?}: {}", env_file, e);
        } else {
            // Set restrictive permissions on env file (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&env_file, fs::Permissions::from_mode(0o600));
            }
            info!("Created default env file: {:?}", env_file);
            info!("Please edit it with your Odoo credentials");
        }
    }

    // Load environment variables from env file
    if env_file.exists() {
        load_env_file(&env_file);
    }

    // Set default MCP config paths if not already set
    if let Some(share_dir) = get_share_dir() {
        set_default_env("MCP_TOOLS_JSON", share_dir.join("tools.json"));
        set_default_env("MCP_PROMPTS_JSON", share_dir.join("prompts.json"));
        set_default_env("MCP_SERVER_JSON", share_dir.join("server.json"));
    }
}

/// Load environment variables from a file (simple key=value format)
fn load_env_file(path: &PathBuf) {
    let Ok(file) = fs::File::open(path) else {
        warn!("Could not open env file: {:?}", path);
        return;
    };

    info!("Loading environment from: {:?}", path);
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let Ok(line) = line else { continue };
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Only set if not already set (env vars take precedence)
            if std::env::var(key).is_err() {
                // SAFETY: We're setting env vars at startup before any threads are spawned
                unsafe {
                    std::env::set_var(key, value);
                }
                // Mask sensitive values in logs
                let display_value =
                    if key.contains("PASSWORD") || key.contains("API_KEY") || key.contains("TOKEN")
                    {
                        "***"
                    } else {
                        value
                    };
                info!("  Set {}={}", key, display_value);
            }
        }
    }
}

/// Set environment variable if not already set
fn set_default_env(key: &str, value: PathBuf) {
    if std::env::var(key).is_err()
        && let Some(s) = value.to_str()
    {
        // SAFETY: We're setting env vars at startup before any threads are spawned
        unsafe {
            std::env::set_var(key, s);
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum TransportMode {
    Stdio,
    Ws,
    Http,
}

#[derive(Debug, Parser)]
#[command(name = "odoo-mcp-rust", version, about = "Odoo MCP server (Rust)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

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

#[derive(Debug, Parser)]
enum Command {
    /// Validate Odoo instance configuration without starting the server
    #[command(about = "Validate Odoo configuration")]
    ValidateConfig {
        /// Optional path to env file (defaults to ~/.config/odoo-rust-mcp/env)
        #[arg(long)]
        env_file: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Auto-load user config from ~/.config/odoo-rust-mcp/
    setup_user_config();

    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Command::ValidateConfig { env_file } => {
                return validate_config(env_file).await;
            }
        }
    }

    // Otherwise, start the server
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

async fn validate_config(_env_file: Option<PathBuf>) -> anyhow::Result<()> {
    // The environment is already loaded by setup_user_config()
    // The --env-file option is for future extensibility

    // Load Odoo environment configuration
    let env = rust_mcp::odoo::config::load_odoo_env()?;
    let instances: Vec<String> = env.instances.keys().cloned().collect();

    if instances.is_empty() {
        eprintln!("No Odoo instances configured");
        return Err(anyhow::anyhow!("No instances found in configuration"));
    }

    println!("Validating {} Odoo instance(s)...\n", instances.len());

    let mut all_ok = true;

    for instance_name in &instances {
        let instance_cfg = &env.instances[instance_name];
        print!("• {} ({}): ", instance_name, instance_cfg.url);

        match rust_mcp::odoo::unified_client::OdooClient::new(instance_cfg) {
            Ok(client) => {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    client.health_check(),
                ).await {
                    Ok(true) => {
                        println!("✓ OK");
                    }
                    Ok(false) => {
                        println!("✗ FAIL - health check failed");
                        all_ok = false;
                    }
                    Err(_) => {
                        println!("✗ FAIL - timeout");
                        all_ok = false;
                    }
                }
            }
            Err(e) => {
                println!("✗ FAIL - {}", e);
                all_ok = false;
            }
        }
    }

    println!();
    if all_ok {
        println!("✓ All instances validated successfully!");
        Ok(())
    } else {
        eprintln!("✗ One or more instances failed validation");
        Err(anyhow::anyhow!("Validation failed"))
    }
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
