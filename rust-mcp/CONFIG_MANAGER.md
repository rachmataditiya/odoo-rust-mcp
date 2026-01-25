# Config Manager - Web-Based Configuration Editor

A modern web-based configuration management system for Odoo Rust MCP Server with hot reload support.

## 🎯 Features

✅ **Web UI** - Beautiful, responsive configuration editor  
✅ **Hot Reload** - Changes apply instantly without service restart  
✅ **REST API** - Full API for programmatic configuration  
✅ **File Watcher** - Automatic detection of external file changes  
✅ **Multi-Config** - Manage instances, tools, prompts, and server settings  
✅ **Real-Time Validation** - JSON validation before saving  
✅ **Live Instance Display** - View active Odoo connections  

## 📋 Quick Start

### Option 1: Standalone Rust Server

```bash
cargo run --manifest-path rust-mcp/Cargo.toml -- \
  --transport ws \
  --listen 127.0.0.1:8787 \
  --config-server-port 3008
```

### Option 2: Docker Container

```bash
docker run -d \
  --name odoo-mcp-config \
  -p 3008:3008 \
  -v ~/.config/odoo-rust-mcp:/config \
  ghcr.io/rachmataditiya/odoo-rust-mcp:latest \
  --config-server-port 3008
```

### Option 3: Kubernetes/Helm

```bash
helm install odoo-mcp helm/odoo-rust-mcp/ \
  --set configServer.enabled=true \
  --set configServer.port=3000
```

## 🌐 Web Interface

Access the UI at: **http://localhost:3008** (inspired by Peugeot 3008)

### Configuration Tabs

#### 1. **Instances** 🏢
Configure Odoo instance connections:
```json
{
  "default": {
    "url": "http://localhost:8069",
    "db": "mydb",
    "apiKey": "your-api-key"
  },
  "production": {
    "url": "https://erp.company.com",
    "db": "production",
    "apiKey": "prod-key"
  }
}
```

#### 2. **Server** ⚙️
Server metadata and settings:
```json
{
  "serverName": "odoo-rust-mcp",
  "instructions": "Odoo MCP server with hot reload support",
  "protocolVersionDefault": "2025-11-05"
}
```

#### 3. **Tools** 🛠️
Available tools and their schemas (read-only by default):
```json
[
  {
    "name": "create_record",
    "description": "Create a new record in Odoo",
    "inputSchema": {
      "type": "object",
      "properties": { ... }
    }
  }
]
```

#### 4. **Prompts** 💬
System prompts that guide server behavior:
```json
[
  {
    "name": "analyze_sales",
    "description": "Analyze sales data",
    "arguments": [ ... ]
  }
]
```

## 🔌 REST API

All endpoints accept/return JSON and support hot reload.

### Health Check
```bash
GET /health
```

### Get Configuration
```bash
# Get Odoo instances
GET /api/config/instances

# Get server settings
GET /api/config/server

# Get tools
GET /api/config/tools

# Get prompts
GET /api/config/prompts
```

### Update Configuration
```bash
# Update instances (triggers hot reload)
POST /api/config/instances
Content-Type: application/json

{
  "default": {
    "url": "http://localhost:8069",
    "apiKey": "new-key"
  }
}

# Response: { "status": "saved" }
```

### Example cURL

```bash
# Load current config
curl http://localhost:3008/api/config/instances | jq

# Update instances
curl -X POST http://localhost:3008/api/config/instances \
  -H "Content-Type: application/json" \
  -d @instances.json
```

## 🔄 Hot Reload Mechanism

### How It Works

1. **File Watcher** (`notify` crate)
   - Monitors `~/.config/odoo-rust-mcp/` directory
   - Detects `.json` file changes
   - Broadcasts change events

2. **Change Notification**
   - Config server notifies main app via broadcast channel
   - No network calls required
   - Instant propagation

3. **Registry Update** (main app)
   - Registry listens for file change events
   - Reloads affected configuration
   - Updates internal state

4. **No Service Restart**
   - Running connections unaffected
   - New connections use updated config
   - Seamless user experience

### What Reloads Automatically

| File | Component | Reload Time |
|------|-----------|------------|
| `instances.json` | Connection Pool | < 100ms |
| `tools.json` | Tool Registry | < 50ms |
| `prompts.json` | Prompt Registry | < 50ms |
| `server.json` | Server Info | < 50ms |

## 📁 Configuration Files

All files are stored in: **`~/.config/odoo-rust-mcp/`**

```
~/.config/odoo-rust-mcp/
├── instances.json      # Odoo connections
├── server.json         # Server metadata
├── tools.json          # Available tools
└── prompts.json        # System prompts
```

### File Permissions (Security)
```bash
chmod 700 ~/.config/odoo-rust-mcp/
chmod 600 ~/.config/odoo-rust-mcp/*.json
```

## 🛡️ Security

### Default Configuration

- **Bind Address**: `127.0.0.1` (localhost only)
- **No Authentication**: Edit in trusted environments only
- **Port**: `3008` (configurable, inspired by Peugeot 3008)

### Production Deployment

For public/remote access, use a reverse proxy with authentication:

#### Nginx Example
```nginx
server {
    listen 80;
    server_name config.mcp.internal;
    
    # Require authentication
    auth_basic "Restricted Access";
    auth_basic_user_file /etc/nginx/.htpasswd;
    
    location / {
        proxy_pass http://127.0.0.1:3008;
        proxy_set_header Host $host;
    }
}
```

Generate htpasswd:
```bash
htpasswd -c /etc/nginx/.htpasswd admin
```

## 📊 Environment Variables

```bash
# Enable config server (required)
export ODOO_CONFIG_SERVER_PORT=3008

# Or set config directory
export ODOO_CONFIG_DIR=~/.config/odoo-rust-mcp

# Set instances from file
export ODOO_INSTANCES_JSON=/path/to/instances.json

# Logging
export RUST_LOG=info,odoo_rust_mcp::config_manager=debug
```

## 🧪 Testing

### Unit Tests
```bash
# Test config manager
cargo test --lib config_manager --manifest-path rust-mcp/Cargo.toml

# Test specific module
cargo test --lib config_manager::manager
```

### Integration Tests
```bash
# Start config server
cargo run --manifest-path rust-mcp/Cargo.toml -- \
  --config-server-port 3008 &

# Test endpoints
curl http://localhost:3008/health
curl http://localhost:3008/api/config/instances
```

## 🚀 Advanced Usage

### Programmatic Configuration Updates

```rust
use rust_mcp::config_manager::ConfigManager;
use std::path::PathBuf;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = ConfigManager::new(
        PathBuf::from("~/.config/odoo-rust-mcp")
    );
    
    // Load current config
    let instances = manager.load_instances().await?;
    
    // Modify
    let mut new_config = instances;
    new_config["production"] = json!({
        "url": "https://erp.company.com",
        "apiKey": "secret"
    });
    
    // Save (triggers hot reload)
    manager.save_instances(new_config).await?;
    
    Ok(())
}
```

### Using the File Watcher

```rust
use rust_mcp::config_manager::ConfigWatcher;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let watcher = ConfigWatcher::new(
        PathBuf::from("~/.config/odoo-rust-mcp")
    )?;
    
    let mut rx = watcher.subscribe();
    
    // Listen for changes
    while let Ok(filename) = rx.recv().await {
        println!("Config changed: {}", filename);
    }
    
    Ok(())
}
```

## 📚 API Documentation

Complete API docs with examples available at:
- [DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md) - Deployment instructions
- [README.md](../README.md) - Main project README

## 🐛 Troubleshooting

### Config server won't start
```bash
# Check if port is in use
lsof -i :3008

# Use different port
--config-server-port 3009

# Check logs
RUST_LOG=debug cargo run
```

### Changes not appearing
1. Verify file permissions: `chmod 600 ~/.config/odoo-rust-mcp/*.json`
2. Check watcher logs: `RUST_LOG=debug`
3. Restart service if needed

### JSON validation errors
```bash
# Validate JSON file
jq empty ~/.config/odoo-rust-mcp/instances.json

# Pretty print
jq . ~/.config/odoo-rust-mcp/instances.json
```

### Web UI not loading
- Check browser console for errors
- Verify CORS is enabled (should be by default)
- Check network tab in DevTools

## 📦 Dependencies

- **axum** - Web framework
- **tokio** - Async runtime
- **notify** - File watching
- **serde_json** - JSON handling
- **tower-http** - CORS support

## 📝 License

Same as main project (AGPL-3.0-only)

## 🤝 Contributing

Config manager is part of the main Odoo Rust MCP project. Contributions welcome!

For issues or feature requests: [GitHub Issues](https://github.com/rachmataditiya/odoo-rust-mcp/issues)
