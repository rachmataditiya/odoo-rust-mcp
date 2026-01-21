## odoo-rust-mcp

Rust implementation of an **Odoo MCP server** (Model Context Protocol), supporting:
- **Odoo 19+**: JSON-2 External API (`/json/2/...`) with API key authentication
- **Odoo < 19**: JSON-RPC (`/jsonrpc`) with username/password authentication

### Features

- MCP over **stdio** (Cursor / Claude Desktop style)
- MCP over **Streamable HTTP** (Cursor remote transport)
- MCP over **SSE** (legacy HTTP+SSE transport)
- MCP over **WebSocket** (standalone server; not used by Cursor)
- **Multi-instance** support via `ODOO_INSTANCES`
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

Set `ODOO_INSTANCES` to JSON:

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

### Installation

#### Option 1: Download and install (recommended)

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

#### Option 2: Build from source

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

### Run with Docker Compose (.env)

Create `.env` in the repo root (example in `dotenv.example`), then:

```bash
docker compose up --build
```

By default, the container runs **HTTP** transport and exposes `http://localhost:8787/mcp`.

If you want JSON-driven tools/prompts/server metadata in Docker, either bake the files into the image or mount them and set:
- `MCP_TOOLS_JSON`
- `MCP_PROMPTS_JSON`
- `MCP_SERVER_JSON`

### Cleanup tools (disabled by default)

Cleanup tools are only listed and callable when enabled:

```bash
export ODOO_ENABLE_CLEANUP_TOOLS=true
```

### Tools

Tools are defined by `tools.json` (authoritative). The default seed includes tools like:
- `odoo_search`, `odoo_search_read`, `odoo_read`, `odoo_create`, `odoo_update`, `odoo_delete`
- `odoo_execute`, `odoo_count`, `odoo_workflow_action`, `odoo_generate_report`, `odoo_get_model_metadata`
- cleanup tools (`odoo_database_cleanup`, `odoo_deep_cleanup`) guarded by `ODOO_ENABLE_CLEANUP_TOOLS=true`

Supported `op.type` values (used in `tools.json`):
- `search`, `search_read`, `read`, `create`, `write`, `unlink`
- `search_count`, `workflow_action`, `execute`
- `generate_report`, `get_model_metadata`
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

There is also a Python smoke tester that validates Cursor-style stdio + Streamable HTTP:

```bash
python3 ./mcp_test.py stdio --bin ./rust-mcp/target/release/rust-mcp --env-file .env
python3 ./mcp_test.py http --url http://127.0.0.1:8787/mcp
```

### Security

- Do **not** commit `.env` or any file containing API keys/passwords.
- Prefer using dedicated bot users with minimal access rights in Odoo for automation.
