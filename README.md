## odoo-rust-mcp

[![CI](https://github.com/rachmataditiya/odoo-rust-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/rachmataditiya/odoo-rust-mcp/actions/workflows/ci.yml)
[![Release](https://github.com/rachmataditiya/odoo-rust-mcp/actions/workflows/release.yml/badge.svg)](https://github.com/rachmataditiya/odoo-rust-mcp/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/rachmataditiya/odoo-rust-mcp/branch/main/graph/badge.svg)](https://codecov.io/gh/rachmataditiya/odoo-rust-mcp)
[![GitHub release](https://img.shields.io/github/v/release/rachmataditiya/odoo-rust-mcp)](https://github.com/rachmataditiya/odoo-rust-mcp/releases)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Homebrew](https://img.shields.io/badge/homebrew-available-orange)](https://rachmataditiya.github.io/homebrew-odoo-rust-mcp/)
[![Debian/Ubuntu](https://img.shields.io/badge/apt-available-blue)](https://rachmataditiya.github.io/odoo-rust-mcp/)
[![Docker](https://img.shields.io/badge/docker-ghcr.io%2Fodoo--rust--mcp-2496ED?logo=docker&logoColor=white)](https://github.com/rachmataditiya/odoo-rust-mcp/pkgs/container/odoo-rust-mcp)
[![Kubernetes](https://img.shields.io/badge/kubernetes-ready-326CE5?logo=kubernetes&logoColor=white)](https://github.com/rachmataditiya/odoo-rust-mcp/tree/main/k8s)
[![Helm](https://img.shields.io/badge/helm-chart-0F1689?logo=helm&logoColor=white)](https://github.com/rachmataditiya/odoo-rust-mcp/tree/main/helm)

Rust implementation of an **Odoo MCP server** (Model Context Protocol), supporting:
- **Odoo 19+**: JSON-2 External API (`/json/2/...`) with API key authentication
- **Odoo < 19**: JSON-RPC (`/jsonrpc`) with username/password authentication

### Features

- MCP over **stdio** (Cursor / Claude Desktop style)
- MCP over **Streamable HTTP** (Cursor remote transport)
- MCP over **SSE** (legacy HTTP+SSE transport)
- MCP over **WebSocket** (standalone server; not used by Cursor)
- **Web UI for configuration** on port 3000 (tools, prompts, server, instances JSON)
- **Multi-instance** support via `ODOO_INSTANCES`
- **Metadata caching** with configurable TTL to reduce Odoo API calls
- **Health check endpoint** for monitoring and configuration validation
- **MCP Resources** with `odoo://` URI scheme for resource discovery
- **New tools**: list models, check access rights, bulk create
- Optional cleanup tools gated behind `ODOO_ENABLE_CLEANUP_TOOLS=true`

### Repository layout

- `rust-mcp/`: the Rust MCP server implementation (Cargo workspace root)

### Requirements

- Rust toolchain (cargo)
- Odoo instance:
  - **Odoo 19+**: Requires External JSON-2 API and API keys
  - **Odoo < 19**: Works with standard JSON-RPC endpoint (username/password)

Notes: For Odoo 19+, access to JSON-2 external API is only available on **Custom** plans. For Odoo < 19, the standard JSON-RPC endpoint is used which is available on all editions.

### Configuration (environment variables)

#### Multi-instance (recommended)

**Option A: JSON file (recommended for readability)**

See `instances.example.json` in the repository root for a complete example configuration.

Create `~/.config/odoo-rust-mcp/instances.json`:

```json
{
  "production": {
    "url": "https://mycompany.example.com",
    "db": "mycompany",
    "apiKey": "YOUR_API_KEY"
  },
  "staging": {
    "url": "https://staging.mycompany.example.com",
    "apiKey": "YOUR_API_KEY"
  },
  "legacy": {
    "url": "https://legacy.mycompany.example.com",
    "db": "legacy_db",
    "version": "18",
    "username": "admin",
    "password": "admin_password"
  }
}
```

Then in `~/.config/odoo-rust-mcp/env`:

```bash
ODOO_INSTANCES_JSON=/path/to/instances.json
```

**Option B: Inline JSON (single line)**

Set `ODOO_INSTANCES` directly in env file (must be single line):

```bash
ODOO_INSTANCES={"production":{"url":"https://mycompany.example.com","apiKey":"xxx"},"staging":{"url":"https://staging.example.com","apiKey":"yyy"}}
```

**Instance configuration fields:**

| Field | Required | Description |
|-------|----------|-------------|
| `url` | Yes | Odoo server URL |
| `db` | Odoo < 19 | Database name (required for legacy, optional for 19+) |
| `apiKey` | Odoo 19+ | API key for authentication |
| `version` | No | Odoo version (e.g., "17", "18"). If < 19, uses username/password |
| `username` | Odoo < 19 | Username for JSON-RPC authentication |
| `password` | Odoo < 19 | Password for JSON-RPC authentication |

Notes:
- `db` is optional for Odoo 19+ (only needed when Host header isn't enough to select DB).
- `db` is **required** for Odoo < 19 (legacy mode).
- `version` determines authentication mode: `< 19` uses username/password, `>= 19` uses API key.
- Extra fields in the JSON are ignored.
- If an instance omits `apiKey`, the server will fall back to the global `ODOO_API_KEY` (if set).
- If an instance omits `username`/`password`, the server will fall back to `ODOO_USERNAME`/`ODOO_PASSWORD`.

#### Single-instance (fallback)

**Odoo 19+ (API Key):**
```bash
export ODOO_URL="https://mycompany.example.com"
export ODOO_API_KEY="YOUR_API_KEY"
export ODOO_DB="mycompany"   # optional
```

**Odoo < 19 (Username/Password):**
```bash
export ODOO_URL="https://mycompany.example.com"
export ODOO_DB="mycompany"   # required for legacy
export ODOO_VERSION="18"     # triggers legacy mode
export ODOO_USERNAME="admin"
export ODOO_PASSWORD="your_password"
```

URL convenience: `ODOO_URL` may be given as `localhost:8069` (it will be normalized to `http://localhost:8069`).

#### Local example (Odoo 19+)

```bash
export ODOO_URL="localhost:8069"
export ODOO_DB="v19_pos"
export ODOO_API_KEY="YOUR_API_KEY"
```

#### Local example (Odoo 18 or earlier)

```bash
export ODOO_URL="localhost:8069"
export ODOO_DB="v18_db"
export ODOO_VERSION="18"
export ODOO_USERNAME="admin"
export ODOO_PASSWORD="admin"
```

### Tools, prompts, and server metadata via JSON (no recompile)

This server is **fully declarative**:

- **`tools.json`** defines:
  - tool `name` / `description`
  - tool `inputSchema` (JSON Schema; must be Cursor-friendly)
  - tool `op` (maps tool calls to a primitive operation executed by Rust)
  - optional `guards` (e.g. `requiresEnvTrue`)
- **`prompts.json`** defines prompt `name` / `description` / `content`
- **`server.json`** defines initialize metadata (`serverName`, `instructions`, default protocol version)

Cursor schema constraints:
- Avoid `anyOf`, `oneOf`, `allOf`, `$ref`, `definitions`, and `\"type\": [...]` (type arrays). These are rejected to prevent Cursor schema parsing issues.

Auto-reload:
- The server watches these files and reloads them on change.
- If a JSON file is missing at startup, the server will **create it from built-in seed defaults** (embedded from `rust-mcp/config-defaults/*`).

Editable vs seed defaults (important):
- **Edit these runtime files** for day-to-day configuration:
  - `rust-mcp/config/tools.json`
  - `rust-mcp/config/prompts.json`
  - `rust-mcp/config/server.json`
- **Do not edit** `rust-mcp/config-defaults/*` for normal configuration changes.
  - Those files are embedded into the binary as seed defaults (used only to auto-create missing runtime config files).
  - If you change `config-defaults/*`, you must rebuild the binary/image for changes to take effect.

Environment variables (optional overrides):

```bash
export MCP_TOOLS_JSON="config/tools.json"
export MCP_PROMPTS_JSON="config/prompts.json"
export MCP_SERVER_JSON="config/server.json"
```

Sample files are provided in:
- `rust-mcp/config/tools.json`
- `rust-mcp/config/prompts.json`
- `rust-mcp/config/server.json`

Seed defaults (used only when files are missing):
- `rust-mcp/config-defaults/tools.json`
- `rust-mcp/config-defaults/prompts.json`
- `rust-mcp/config-defaults/server.json`

#### Disabling Tools

To disable specific tools (e.g., create and update operations for read-only access), you have two options:

**Option 1: Remove tools from the array (recommended)**

Simply remove the tool entries from the `tools` array in `tools.json`. For example, to disable `odoo_create` and `odoo_update`:

```json
{
  "tools": [
    {
      "name": "odoo_search",
      ...
    },
    {
      "name": "odoo_search_read",
      ...
    },
    {
      "name": "odoo_read",
      ...
    }
    // odoo_create and odoo_update are removed - they won't be available
  ]
}
```

**Option 2: Use guards for conditional disabling**

Use `guards` with `requiresEnvTrue` to conditionally enable/disable tools based on environment variables:

```json
{
  "tools": [
    {
      "name": "odoo_create",
      "description": "Create a new Odoo record...",
      "guards": {
        "requiresEnvTrue": "ODOO_ENABLE_WRITE_OPERATIONS"
      },
      "inputSchema": { ... },
      "op": { ... }
    },
    {
      "name": "odoo_update",
      "description": "Update existing Odoo records...",
      "guards": {
        "requiresEnvTrue": "ODOO_ENABLE_WRITE_OPERATIONS"
      },
      "inputSchema": { ... },
      "op": { ... }
    }
  ]
}
```

Then set the environment variable to enable:
```bash
export ODOO_ENABLE_WRITE_OPERATIONS=true
```

**Example: Read-only configuration**

Here's a minimal `tools.json` for read-only access (no create, update, delete, or batch operations):

```json
{
  "tools": [
    {
      "name": "odoo_search",
      "description": "Search for Odoo records...",
      "inputSchema": { ... },
      "op": { "type": "search", "map": { ... } }
    },
    {
      "name": "odoo_search_read",
      "description": "Search and read Odoo records...",
      "inputSchema": { ... },
      "op": { "type": "search_read", "map": { ... } }
    },
    {
      "name": "odoo_read",
      "description": "Read specific Odoo records...",
      "inputSchema": { ... },
      "op": { "type": "read", "map": { ... } }
    },
    {
      "name": "odoo_count",
      "description": "Count records matching domain...",
      "inputSchema": { ... },
      "op": { "type": "search_count", "map": { ... } }
    },
    {
      "name": "odoo_get_model_metadata",
      "description": "Get model metadata...",
      "inputSchema": { ... },
      "op": { "type": "get_model_metadata", "map": { ... } }
    },
    {
      "name": "odoo_list_models",
      "description": "List available Odoo models...",
      "inputSchema": { ... },
      "op": { "type": "list_models", "map": { ... } }
    }
    // Note: odoo_create, odoo_update, odoo_delete, odoo_create_batch are excluded
  ]
}
```

After modifying `tools.json`, the server will automatically reload the configuration (no restart needed).


### Advanced Features

#### Metadata Caching

The server caches Odoo model metadata to reduce repeated API calls. Configure cache TTL:

```bash
export ODOO_METADATA_CACHE_TTL_SECS=3600  # default: 1 hour
```

Cache is in-memory and shared across all instances. Useful for reducing latency when working with large models or frequent metadata queries.

#### Health Check Endpoint

When running in HTTP mode, a health check endpoint is available:

```bash
GET /health
```

Response:

```json
{
  "status": "ok",
  "version": "0.3.0",
  "instance": {
    "name": "default",
    "reachable": true
  }
}
```

#### Configuration Validation

Validate your Odoo instance configuration before starting the server:

```bash
./rust-mcp validate-config
```

This command tests connectivity to all configured instances and reports any authentication or connection issues.

#### MCP Resources

The server exposes Odoo resources via the MCP Resources protocol using `odoo://` URIs:

- `odoo://instances` - List configured Odoo instances
- `odoo://{instance}/models` - List accessible models in an instance
- `odoo://{instance}/metadata/{model}` - Get field metadata for a model

MCP clients that support resources can use these to discover available Odoo models and fields dynamically.

#### Configuration Server (Web UI)

A web-based configuration interface is available on port **3000** for managing `tools.json`, `prompts.json`, `server.json`, and `instances.json` without manual file editing.

**Access the config UI:**

- Local: `http://localhost:3000`
- Docker Compose: `http://localhost:3000` or via Traefik `http://mcp-config.localhost`
- Kubernetes: Exposed via service on port 3000

**Features:**

- Edit all JSON configuration files via interactive UI
- Real-time JSON validation before saving
- Tool enable/disable checkboxes (e.g., toggle `ODOO_ENABLE_CLEANUP_TOOLS`)
- Automatic file watching and hot reload (no server restart needed)
- REST API endpoints for programmatic access:
  - `GET /api/config/{instances|tools|prompts|server}` - Retrieve config
  - `POST /api/config/{instances|tools|prompts|server}` - Update config
  - `GET /health` - Health check

**Environment variable:**

```bash
export ODOO_CONFIG_SERVER_PORT=3000  # default: 3000
```

**Example: Enable/disable cleanup tools via UI**

1. Open `http://localhost:3000`
2. Go to **Tools** tab
3. Find the `odoo_create_batch` tool (or other cleanup operations)
4. Toggle the **ODOO_ENABLE_CLEANUP_TOOLS** checkbox
5. Save changes (file is auto-reloaded)

### Installation

#### Option 1: Homebrew (macOS/Linux) - Recommended

```bash
# Install
brew tap rachmataditiya/odoo-rust-mcp
brew install rust-mcp

# Configure (edit with your Odoo credentials)
nano ~/.config/odoo-rust-mcp/env

# Start as background service
brew services start rust-mcp
```

Or install directly in one command:

```bash
brew install rachmataditiya/odoo-rust-mcp/rust-mcp
```

**What gets installed:**

| Component | Location |
|-----------|----------|
| Binary | `/opt/homebrew/bin/rust-mcp` |
| Service wrapper | `/opt/homebrew/bin/rust-mcp-service` |
| User configs | `~/.config/odoo-rust-mcp/` (auto-created) |
| Service logs | `/opt/homebrew/var/log/rust-mcp.log` |

**User config directory (`~/.config/odoo-rust-mcp/`):**

```
├── env              # Environment variables - EDIT THIS with Odoo credentials
├── tools.json       # MCP tools definition
├── prompts.json     # MCP prompts definition  
└── server.json      # MCP server metadata
```

**Service commands:**

```bash
brew services start rust-mcp      # Start service
brew services stop rust-mcp       # Stop service
brew services restart rust-mcp    # Restart after config changes
brew services list                # Check status
tail -f /opt/homebrew/var/log/rust-mcp.log  # View logs
```

Service endpoint: `http://127.0.0.1:8787/mcp`

**For Cursor/Claude Desktop/Windsurf with Homebrew:**

The binary automatically loads config from `~/.config/odoo-rust-mcp/env`, so you can use it directly:

```json
{
  "mcpServers": {
    "odoo": {
      "command": "/opt/homebrew/bin/rust-mcp",
      "args": ["--transport", "stdio"]
    }
  }
}
```

**Note:** Starting from v0.2.4, the binary (`rust-mcp`) automatically:
- Creates `~/.config/odoo-rust-mcp/` directory if it doesn't exist
- Creates a default `env` template file
- Loads environment variables from `~/.config/odoo-rust-mcp/env`
- Sets default MCP config paths from Homebrew share directory

This means you can use `rust-mcp` directly without the shell wrapper `rust-mcp-service`. This is especially important for MCP clients like Windsurf that don't support shell script execution.

For full Homebrew documentation, see: https://rachmataditiya.github.io/homebrew-odoo-rust-mcp/

#### Option 2: APT (Debian/Ubuntu)

```bash
# Add GPG key
curl -fsSL https://rachmataditiya.github.io/odoo-rust-mcp/pubkey.gpg | sudo gpg --dearmor -o /usr/share/keyrings/rust-mcp.gpg

# Add repository
echo "deb [signed-by=/usr/share/keyrings/rust-mcp.gpg] https://rachmataditiya.github.io/odoo-rust-mcp stable main" | sudo tee /etc/apt/sources.list.d/rust-mcp.list

# Install
sudo apt update
sudo apt install rust-mcp

# Configure (edit with your Odoo credentials)
nano ~/.config/rust-mcp/env

# Start service
sudo systemctl start rust-mcp
sudo systemctl enable rust-mcp
```

**What gets installed:**

| Component | Location |
|-----------|----------|
| Binary | `/usr/bin/rust-mcp` |
| Service wrapper | `/usr/bin/rust-mcp-service` |
| Default configs | `/usr/share/rust-mcp/` |
| User configs | `~/.config/rust-mcp/` (auto-created on install) |
| Systemd service | `/lib/systemd/system/rust-mcp.service` |

**Service commands:**

```bash
sudo systemctl start rust-mcp      # Start service
sudo systemctl stop rust-mcp       # Stop service
sudo systemctl restart rust-mcp    # Restart after config changes
sudo systemctl status rust-mcp     # Check status
journalctl -u rust-mcp -f          # View logs
```

Service endpoint: `http://127.0.0.1:8787/mcp`

#### Option 3: Download and install

Download the latest release for your platform from [GitHub Releases](https://github.com/rachmataditiya/odoo-rust-mcp/releases):

| Platform | File |
|----------|------|
| Linux x86_64 | `rust-mcp-x86_64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `rust-mcp-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (Apple Silicon) | `rust-mcp-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `rust-mcp-x86_64-pc-windows-msvc.zip` |

Extract and install:

**Linux / macOS:**
```bash
tar -xzf rust-mcp-<platform>.tar.gz
cd rust-mcp-<platform>
./install.sh
```

This installs:
- Binary to `/usr/local/bin/rust-mcp`
- Config files to `/usr/local/share/odoo-rust-mcp/`

To uninstall:
```bash
./install.sh uninstall
```

**Windows (PowerShell as Administrator):**
```powershell
Expand-Archive rust-mcp-x86_64-pc-windows-msvc.zip -DestinationPath rust-mcp
cd rust-mcp
.\install.ps1
```

This installs:
- Binary to `C:\Program Files\odoo-rust-mcp\`
- Config files to `C:\ProgramData\odoo-rust-mcp\`
- Adds binary to system PATH

To uninstall:
```powershell
.\install.ps1 -Uninstall
```

**Manual (without installer):**
```bash
# Linux/macOS - just run directly
tar -xzf rust-mcp-<platform>.tar.gz
./rust-mcp --transport stdio

# Windows
Expand-Archive rust-mcp-x86_64-pc-windows-msvc.zip -DestinationPath .
.\rust-mcp.exe --transport stdio
```

The release archive includes:
- `rust-mcp` (or `rust-mcp.exe` on Windows) - the binary
- `config/` - default configuration files (tools.json, prompts.json, server.json)
- `.env.example` - example environment variables
- `install.sh` (Linux/macOS) or `install.ps1` (Windows) - installer script

#### Option 4: Build from source

```bash
cd rust-mcp
cargo build --release
```

### Run as Background Service

The installer scripts support running the MCP server as a background service (HTTP transport on port 8787).

**Linux (systemd):**
```bash
./install.sh service
```

Commands:
- Start: `sudo systemctl start odoo-rust-mcp`
- Stop: `sudo systemctl stop odoo-rust-mcp`
- Status: `sudo systemctl status odoo-rust-mcp`
- Logs: `sudo journalctl -u odoo-rust-mcp -f`

Config: `/etc/odoo-rust-mcp.env`

**macOS (launchd):**
```bash
./install.sh service
```

Commands:
- Start: `launchctl load ~/Library/LaunchAgents/com.odoo.rust-mcp.plist`
- Stop: `launchctl unload ~/Library/LaunchAgents/com.odoo.rust-mcp.plist`
- Logs: `tail -f ~/.config/odoo-rust-mcp/stdout.log`

Config: `~/.config/odoo-rust-mcp/env`

**Windows (Scheduled Task):**
```powershell
.\install.ps1 -Service
```

Commands (PowerShell as Admin):
- Start: `Start-ScheduledTask -TaskName OdooRustMcpService`
- Stop: `Stop-ScheduledTask -TaskName OdooRustMcpService`
- Status: `Get-ScheduledTask -TaskName OdooRustMcpService | Select-Object State`

Config: `C:\ProgramData\odoo-rust-mcp\env.ps1`

**Service endpoint:** `http://127.0.0.1:8787/mcp`

To remove service only:
```bash
# Linux/macOS
./install.sh service-uninstall

# Windows
.\install.ps1 -ServiceUninstall
```

### Run (stdio)

```bash
cd rust-mcp
./target/release/rust-mcp --transport stdio
```

### Run (Streamable HTTP)

```bash
cd rust-mcp
./target/release/rust-mcp --transport http --listen 127.0.0.1:8787
```

Endpoints:

- Streamable HTTP: `http://127.0.0.1:8787/mcp`
- Legacy SSE: `http://127.0.0.1:8787/sse` (paired with `POST /messages`)
- Health check: `http://127.0.0.1:8787/health` (no auth required)
- OpenAPI spec: `http://127.0.0.1:8787/openapi.json` (no auth required)

### HTTP API Documentation

The HTTP API is documented using OpenAPI 3.0 specification. You can:

- View the OpenAPI spec: `http://localhost:8787/openapi.json`
- Use the spec with tools like Swagger UI, Postman, or n8n for integration
- The spec documents all endpoints including request/response schemas

For integration with tools like n8n or Dify, refer to the OpenAPI specification for complete API details.

### Authentication (HTTP Transport)

The HTTP transport supports Bearer token authentication as per the [MCP specification](https://modelcontextprotocol.io/specification/draft/basic/authorization).

**Enable authentication:**
```bash
export MCP_AUTH_TOKEN=your-secure-random-token-here
```

When `MCP_AUTH_TOKEN` is set, all HTTP requests must include the `Authorization` header:
```
Authorization: Bearer your-secure-random-token-here
```

**Example with curl:**
```bash
curl -X POST http://127.0.0.1:8787/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secure-random-token-here" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}'
```

**Notes:**
- If `MCP_AUTH_TOKEN` is not set, authentication is disabled (not recommended for production)
- STDIO transport does not use HTTP authentication (credentials come from environment)
- Generate a secure token: `openssl rand -hex 32`

### Run (WebSocket / standalone server)

```bash
cd rust-mcp
./target/release/rust-mcp --transport ws --listen 127.0.0.1:8787
```

### Run with Docker Compose

Create `.env` in the repo root (example in `dotenv.example`), then:

```bash
docker compose up --build
```

By default, the container runs **HTTP** transport and exposes `http://localhost:8787/mcp`.

**Features:**

- Custom network `mcp-network` for integration with other containers (n8n, dify, etc.)
- Health check for container orchestration
- Volume mounts for config files
- Support for `ODOO_INSTANCES_JSON` for multi-instance setup
- Resource limits and labels for service discovery

**Multi-instance configuration:**

1. Create `instances.json` with your Odoo instances:
```json
{
  "production": {"url": "https://odoo.example.com", "db": "prod", "apiKey": "xxx"},
  "staging": {"url": "https://staging.example.com", "db": "staging", "apiKey": "yyy"}
}
```

2. Mount and configure in `.env`:
```bash
ODOO_INSTANCES_JSON=/config/instances.json
```

3. Uncomment the volume mount in `docker-compose.yml`:
```yaml
volumes:
  - ./instances.json:/config/instances.json:ro
```

**Integration with other containers:**

The MCP server runs on the `mcp-network` network. Other containers can connect using:

```yaml
# In your other service's docker-compose.yml
services:
  n8n:
    networks:
      - mcp-network
    environment:
      MCP_URL: "http://odoo-mcp:8787/mcp"

networks:
  mcp-network:
    external: true
```

See `docker-compose.override.example.yml` for more integration examples.

### Run with Kubernetes

The project includes Kubernetes manifests in `k8s/` for production deployments.

**Quick start with raw manifests:**

```bash
# Apply all manifests
kubectl apply -k k8s/

# Or apply individually
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml    # Edit first with your credentials!
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml   # Optional, for external access
```

**Verify deployment:**

```bash
kubectl -n odoo-mcp get pods
kubectl -n odoo-mcp logs -f deployment/odoo-mcp
```

**Port-forward for local testing:**

```bash
kubectl -n odoo-mcp port-forward svc/odoo-mcp 8787:8787
# Access: http://127.0.0.1:8787/mcp
```

**Features:**

- Namespace isolation
- ConfigMap for MCP configuration files
- Secret for Odoo credentials
- Deployment with:
  - Resource limits/requests
  - Liveness/readiness/startup probes
  - Pod anti-affinity for high availability
  - Non-root security context
- ClusterIP Service for internal access
- Ingress for external access with TLS
- Kustomization for environment management

### Run with Helm

For more flexible deployments, use the Helm chart in `helm/odoo-rust-mcp/`.

**Install:**

```bash
# Add your values (see helm/odoo-rust-mcp/values.yaml for all options)
cat > my-values.yaml <<EOF
odoo:
  url: "http://odoo-service:8069"
  db: "mydb"
  apiKey: "your-api-key"

mcp:
  authToken: "your-secure-token"

ingress:
  enabled: true
  hosts:
    - host: mcp.example.com
      paths:
        - path: /
          pathType: Prefix
EOF

# Install
helm install odoo-mcp ./helm/odoo-rust-mcp -f my-values.yaml -n odoo-mcp --create-namespace
```

**Upgrade:**

```bash
helm upgrade odoo-mcp ./helm/odoo-rust-mcp -f my-values.yaml -n odoo-mcp
```

**Uninstall:**

```bash
helm uninstall odoo-mcp -n odoo-mcp
```

**Key configuration options:**

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `2` |
| `odoo.url` | Odoo server URL | `http://odoo-service:8069` |
| `odoo.apiKey` | API key for Odoo 19+ | `""` |
| `odooInstances.json` | Multi-instance JSON config | `""` |
| `mcp.authToken` | HTTP auth token (recommended) | `""` |
| `ingress.enabled` | Enable ingress | `false` |
| `autoscaling.enabled` | Enable HPA | `false` |

**Production recommendations:**

1. **Always set `mcp.authToken`** for HTTP authentication
2. **Use external secrets** (Vault, AWS Secrets Manager) instead of storing credentials in values.yaml
3. **Enable pod disruption budgets** for zero-downtime updates
4. **Configure resource limits** based on your workload
5. **Use Ingress with TLS** for external access

### Cleanup tools (disabled by default)

Cleanup tools are only listed and callable when enabled:

```bash
export ODOO_ENABLE_CLEANUP_TOOLS=true
```

### Tools

Tools are defined by `tools.json` (authoritative). The default seed includes tools like:
- `odoo_search`, `odoo_search_read`, `odoo_read`, `odoo_create`, `odoo_update`, `odoo_delete`
- `odoo_execute`, `odoo_count`, `odoo_workflow_action`, `odoo_generate_report`, `odoo_get_model_metadata`
- `odoo_list_models`, `odoo_check_access`, `odoo_create_batch`
- cleanup tools (`odoo_database_cleanup`, `odoo_deep_cleanup`) guarded by `ODOO_ENABLE_CLEANUP_TOOLS=true`

Supported `op.type` values (used in `tools.json`):
- `search`, `search_read`, `read`, `create`, `write`, `unlink`
- `search_count`, `workflow_action`, `execute`
- `generate_report`, `get_model_metadata`
- `list_models`, `check_access`, `create_batch`
- `database_cleanup`, `deep_cleanup`

### Prompts

Prompts are defined by `prompts.json` (authoritative). The default seed includes:
- `odoo_common_models`
- `odoo_domain_filters`

### Example tool calls

### Tool result format (important)

Most tools return an MCP response whose `content[0].text` is a **JSON string**.

- In the examples below, **Request** shows what you pass as tool arguments.
- **Decoded result** shows the JSON payload after you parse `content[0].text`.

Search and read:

```json
{
  "instance": "default",
  "model": "res.partner",
  "domain": [["is_company", "=", true]],
  "fields": ["name", "email"],
  "limit": 10,
  "order": "name ASC"
}
```

Decoded result (shape):

```json
{
  "records": [],
  "count": 0
}
```

Read by ids:

```json
{
  "instance": "default",
  "model": "res.partner",
  "ids": [1, 2, 3],
  "fields": ["id", "name", "email"]
}
```

Decoded result (shape):

```json
{
  "records": []
}
```

Create:

```json
{
  "instance": "default",
  "model": "res.partner",
  "values": {
    "name": "ACME Demo",
    "is_company": true
  }
}
```

Decoded result (shape):

```json
{
  "id": 123,
  "success": true
}
```

Update:

```json
{
  "instance": "default",
  "model": "res.partner",
  "ids": [123],
  "values": {
    "email": "demo@acme.test"
  }
}
```

Decoded result (shape):

```json
{
  "success": true,
  "updated_count": 1
}
```

Delete:

```json
{
  "instance": "default",
  "model": "res.partner",
  "ids": [123]
}
```

Decoded result (shape):

```json
{
  "success": true,
  "deleted_count": 1
}
```

Search (IDs only):

```json
{
  "instance": "default",
  "model": "res.partner",
  "domain": [["is_company", "=", true]],
  "limit": 10,
  "order": "name ASC"
}
```

Decoded result (shape):

```json
{
  "ids": [1, 2, 3],
  "count": 3
}
```

Count:

```json
{
  "instance": "default",
  "model": "res.partner",
  "domain": [["id", ">", 0]]
}
```

Decoded result (shape):

```json
{
  "count": 18
}
```

Execute action/button (workflow):

```json
{
  "instance": "default",
  "model": "sale.order",
  "ids": [42],
  "action": "action_confirm"
}
```

Decoded result (shape):

```json
{
  "result": null,
  "executed_on": [42]
}
```

Execute arbitrary method:

```json
{
  "instance": "default",
  "model": "res.partner",
  "method": "name_get",
  "args": [[1, 2, 3]]
}
```

Decoded result (shape):

```json
{
  "result": []
}
```

Model metadata (fields/types):

```json
{
  "instance": "default",
  "model": "sale.order"
}
```

Decoded result (shape):

```json
{
  "model": {
    "name": "sale.order",
    "description": "Sales Order",
    "fields": {}
  }
}
```

Generate report (PDF as base64):

```json
{
  "instance": "default",
  "reportName": "sale.report_saleorder",
  "ids": [42]
}
```

Decoded result (shape):

```json
{
  "pdf_base64": "JVBERi0xLjQKJ...<omitted>...",
  "report_name": "sale.report_saleorder",
  "record_ids": [42]
}
```

List models (with optional filtering):

```json
{
  "instance": "default",
  "domain": [["transient", "=", false]],
  "limit": 50,
  "offset": 0
}
```

Decoded result (shape):

```json
{
  "models": [
    {"id": 2, "model": "ir.actions", "name": "Actions"},
    {"id": 3, "model": "ir.model", "name": "Models"}
  ]
}
```

Check access rights:

```json
{
  "instance": "default",
  "model": "res.partner",
  "operation": "write",
  "ids": [1, 2, 3]
}
```

Decoded result (shape):

```json
{
  "has_access": true,
  "model": "res.partner",
  "operation": "write",
  "model_level": true,
  "record_level": true
}
```

Bulk create records (max 100):

```json
{
  "instance": "default",
  "model": "res.partner",
  "values_list": [
    {"name": "Partner 1", "email": "p1@example.com"},
    {"name": "Partner 2", "email": "p2@example.com"}
  ]
}
```

Decoded result (shape):

```json
{
  "ids": [101, 102],
  "created_count": 2,
  "success": true
}
```

### Claude Desktop config example

Set your MCP server command to the built binary:

**Odoo 19+ (API Key):**
```json
{
  "mcpServers": {
    "odoo-rust": {
      "command": "/absolute/path/to/odoo-rust-mcp/rust-mcp/target/release/rust-mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "ODOO_INSTANCES": "{\"production\":{\"url\":\"https://mycompany.example.com\",\"db\":\"mycompany\",\"apiKey\":\"YOUR_API_KEY\"}}"
      }
    }
  }
}
```

**Odoo < 19 (Username/Password):**
```json
{
  "mcpServers": {
    "odoo-rust": {
      "command": "/absolute/path/to/odoo-rust-mcp/rust-mcp/target/release/rust-mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "ODOO_INSTANCES": "{\"legacy\":{\"url\":\"https://mycompany.example.com\",\"db\":\"mycompany\",\"version\":\"18\",\"username\":\"admin\",\"password\":\"your_password\"}}"
      }
    }
  }
}
```

### Cursor config example

Cursor supports `stdio`, `SSE`, and `Streamable HTTP` transports. See Cursor docs: [`cursor.com/docs/context/mcp`](https://cursor.com/docs/context/mcp).

#### Cursor (recommended): stdio

Put this in `~/.cursor/mcp.json` (or `${workspaceFolder}/.cursor/mcp.json`):

**IMPORTANT:** For stdio transport, you must use **absolute paths** for config files because Cursor runs the binary from a different working directory.

**Odoo 19+ (complete example):**
```json
{
  "mcpServers": {
    "odoo-rust-mcp": {
      "type": "stdio",
      "command": "/absolute/path/to/odoo-rust-mcp/rust-mcp/target/release/rust-mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "ODOO_URL": "http://localhost:8069",
        "ODOO_DB": "mydb",
        "ODOO_API_KEY": "YOUR_API_KEY",
        "MCP_TOOLS_JSON": "/absolute/path/to/odoo-rust-mcp/config/tools.json",
        "MCP_PROMPTS_JSON": "/absolute/path/to/odoo-rust-mcp/config/prompts.json",
        "MCP_SERVER_JSON": "/absolute/path/to/odoo-rust-mcp/config/server.json"
      }
    }
  }
}
```

**Odoo < 19 (complete example):**
```json
{
  "mcpServers": {
    "odoo-rust-mcp": {
      "type": "stdio",
      "command": "/absolute/path/to/odoo-rust-mcp/rust-mcp/target/release/rust-mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "ODOO_URL": "http://localhost:8069",
        "ODOO_DB": "mydb",
        "ODOO_VERSION": "18",
        "ODOO_USERNAME": "admin",
        "ODOO_PASSWORD": "admin",
        "MCP_TOOLS_JSON": "/absolute/path/to/odoo-rust-mcp/config/tools.json",
        "MCP_PROMPTS_JSON": "/absolute/path/to/odoo-rust-mcp/config/prompts.json",
        "MCP_SERVER_JSON": "/absolute/path/to/odoo-rust-mcp/config/server.json"
      }
    }
  }
}
```

**Using .env file:**
```json
{
  "mcpServers": {
    "odoo-rust-mcp": {
      "type": "stdio",
      "command": "/absolute/path/to/odoo-rust-mcp/rust-mcp/target/release/rust-mcp",
      "args": ["--transport", "stdio"],
      "envFile": "/absolute/path/to/odoo-rust-mcp/.env"
    }
  }
}
```

Note: When using `envFile`, ensure your `.env` contains absolute paths for `MCP_TOOLS_JSON`, `MCP_PROMPTS_JSON`, and `MCP_SERVER_JSON`.

#### Cursor: Streamable HTTP (remote / multi-user)

Run the server with `--transport http` and set:

```json
{
  "mcpServers": {
    "odoo-rust-mcp": {
      "url": "http://127.0.0.1:8787/mcp"
    }
  }
}
```

**With Bearer Token Authentication:**

If you have `MCP_AUTH_TOKEN` set on the server, configure Cursor with the token in headers:

```json
{
  "mcpServers": {
    "odoo-rust-mcp": {
      "url": "http://127.0.0.1:8787/mcp",
      "headers": {
        "Authorization": "Bearer your-secure-random-token-here"
      }
    }
  }
}
```

Note: Generate a secure token with `openssl rand -hex 32` and set `MCP_AUTH_TOKEN` on the server.

### Test / smoke

Run unit tests (no warnings):

```bash
cd rust-mcp
RUSTFLAGS='-Dwarnings' cargo test
```

There is also a small WS smoke client that can validate end-to-end MCP calls (connect → initialize → list tools → call a few tools):

```bash
cd rust-mcp
cargo run --release --bin ws_smoke_client -- \
  --url ws://127.0.0.1:8787 \
  --instance default \
  --model res.partner
```

Example output (local run against `res.partner`):

```text
tools/list: 11 tools
- odoo_search
- odoo_search_read
- odoo_read
- odoo_create
- odoo_update
- odoo_delete
- odoo_execute
- odoo_count
- odoo_workflow_action
- odoo_generate_report
- odoo_get_model_metadata
odoo_count result: {"count":18}
odoo_search_read count: 2
odoo_search_read sample records: [{"id":46,"name":"Kasir C"},{"id":45,"name":"Kasir B"}]
prompts/list: odoo_common_models, odoo_domain_filters
```

### Security

- Do **not** commit `.env` or any file containing API keys/passwords.
- Prefer using dedicated bot users with minimal access rights in Odoo for automation.
