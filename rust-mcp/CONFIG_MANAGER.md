# Config Manager - Web-Based Configuration Editor

A modern web-based configuration management system for Odoo Rust MCP Server with hot reload support.

## 🎯 Features

✅ **Web UI** - Beautiful, responsive configuration editor  
✅ **Authentication** - Secure login with username/password (configurable via `.env`)  
✅ **Hot Reload** - Changes apply instantly without service restart  
✅ **REST API** - Full API for programmatic configuration  
✅ **File Watcher** - Automatic detection of external file changes  
✅ **Multi-Config** - Manage instances, tools, prompts, and server settings  
✅ **Real-Time Validation** - JSON validation before saving with automatic rollback  
✅ **Error Handling** - Automatic backup and restore on configuration errors  
✅ **MCP Auth Management** - Enable/disable HTTP authentication and generate tokens from UI  
✅ **Config UI Credentials** - Change login username and password from the UI  
✅ **User Notifications** - Success, error, and warning messages displayed in the UI  
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
Authorization: Bearer <session-token>

{
  "default": {
    "url": "http://localhost:8069",
    "apiKey": "new-key"
  }
}

# Response: { 
#   "success": true, 
#   "message": "Configuration saved successfully",
#   "warning": null,
#   "rollback_performed": false
# }
```

### MCP HTTP Authentication Management
```bash
# Get MCP auth status
GET /api/config/auth/status
Authorization: Bearer <session-token>

# Enable/disable MCP HTTP auth
POST /api/config/auth/enable
Content-Type: application/json
Authorization: Bearer <session-token>

{
  "enabled": true
}

# Generate new MCP auth token
POST /api/config/auth/token/generate
Authorization: Bearer <session-token>

# Response: { "token": "new-generated-token-here" }
```

### Config UI Credentials Management
```bash
# Update Config UI credentials
POST /api/config/auth/credentials
Content-Type: application/json
Authorization: Bearer <session-token>

{
  "username": "newadmin",
  "password": "newpassword"
}
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

| File/Setting | Component | Reload Time |
|--------------|-----------|------------|
| `instances.json` | Connection Pool | < 100ms |
| `tools.json` | Tool Registry | < 50ms |
| `prompts.json` | Prompt Registry | < 50ms |
| `server.json` | Server Info | < 50ms |
| `MCP_AUTH_ENABLED` | HTTP Auth Config | < 50ms |
| `MCP_AUTH_TOKEN` | HTTP Auth Config | < 50ms |

**Note:** MCP HTTP authentication settings can be changed from the Config UI and take effect immediately without restarting the service.

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

### Authentication

The Config UI requires login credentials that are stored in your `.env` file:

```bash
# Config UI Authentication (default credentials)
CONFIG_UI_USERNAME=admin      # default: admin
CONFIG_UI_PASSWORD=changeme   # default: changeme (CHANGE THIS!)
```

**First-time setup:**
1. Login with default credentials (`admin` / `changeme`)
2. Go to **Security** tab
3. Change the default password immediately

**Change credentials from UI:**
1. Login to Config UI
2. Go to **Security** tab
3. Update username and/or password
4. Changes are saved to `.env` file immediately

### MCP HTTP Authentication Management

You can enable/disable MCP HTTP authentication and generate tokens directly from the Config UI:

1. Login to Config UI
2. Go to **Security** tab
3. Toggle **Enable MCP HTTP Authentication**
4. Click **Generate New Token** (or paste existing token)
5. Changes apply immediately (hot-reload, no restart needed)

The settings are stored in `.env`:
```bash
MCP_AUTH_ENABLED=true
MCP_AUTH_TOKEN=your-generated-token-here
```

### Default Configuration

- **Bind Address**: `127.0.0.1` (localhost only)
- **Authentication**: Required (username/password from `.env`)
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

**Automatic Error Handling:**
- The Config UI automatically creates backups before saving
- If validation fails or file becomes corrupted, the previous version is automatically restored
- You'll see a notification in the UI if a rollback occurred
- Check the browser console for detailed error messages

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
