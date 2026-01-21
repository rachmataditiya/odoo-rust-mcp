## odoo-rust-mcp

Rust implementation of an **Odoo MCP server** (Model Context Protocol), using **Odoo 19 JSON-2 External API** (`/json/2/...`) for fast HTTP-based access (no XML-RPC).

### Features

- MCP over **stdio** (Claude Desktop style)
- MCP over **WebSocket** (standalone server)
- **Multi-instance** support via `ODOO_INSTANCES`
- Optional cleanup tools gated behind `ODOO_ENABLE_CLEANUP_TOOLS=true`

### Repository layout

- `rust-mcp/`: the Rust MCP server implementation (Cargo workspace root)

### Requirements

- Rust toolchain (cargo)
- Odoo instance that supports **External JSON-2 API** and **API keys** (Odoo 19+)

Notes: Odoo states access to JSON-2 external API is only available on **Custom** plans. If your Odoo doesn’t expose `/json/2`, this server won’t work as-is.

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
  }
}
```

Notes:
- `db` is optional (only needed when Host header isn’t enough to select DB).
- Extra fields in the JSON are ignored.
- If an instance omits `apiKey`, the server will fall back to the global `ODOO_API_KEY` (if set).

#### Single-instance (fallback)

```bash
export ODOO_URL="https://mycompany.example.com"
export ODOO_API_KEY="YOUR_API_KEY"
export ODOO_DB="mycompany"   # optional
```

URL convenience: `ODOO_URL` may be given as `localhost:8069` (it will be normalized to `http://localhost:8069`).

#### Local example (your setup)

```bash
export ODOO_URL="localhost:8069"
export ODOO_DB="v19_pos"
export ODOO_API_KEY="YOUR_API_KEY"
```

### Build

```bash
cd rust-mcp
cargo build --release
```

### Run (stdio)

```bash
cd rust-mcp
./target/release/rust-mcp --transport stdio
```

### Run (WebSocket / standalone server)

```bash
cd rust-mcp
./target/release/rust-mcp --transport ws --listen 127.0.0.1:8787
```

### Cleanup tools (disabled by default)

Cleanup tools are only listed and callable when enabled:

```bash
export ODOO_ENABLE_CLEANUP_TOOLS=true
```

### Tools

Core tools:
- `odoo_search`
- `odoo_search_read`
- `odoo_read`
- `odoo_create`
- `odoo_update`
- `odoo_delete`
- `odoo_execute`
- `odoo_count`
- `odoo_workflow_action`
- `odoo_generate_report` (returns base64 PDF)
- `odoo_get_model_metadata`

Cleanup tools (feature-flag):
- `odoo_database_cleanup`
- `odoo_deep_cleanup`

### Prompts

- `odoo_common_models`
- `odoo_domain_filters`

### Example tool calls

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

Read by ids:

```json
{
  "instance": "default",
  "model": "res.partner",
  "ids": [1, 2, 3],
  "fields": ["id", "name", "email"]
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

Execute action/button (workflow):

```json
{
  "instance": "default",
  "model": "sale.order",
  "ids": [42],
  "action": "action_confirm"
}
```

Model metadata (fields/types):

```json
{
  "instance": "default",
  "model": "sale.order"
}
```

### Claude Desktop config example

Set your MCP server command to the built binary:

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

### Security

- Do **not** commit `.env` or any file containing API keys/passwords.
- Prefer using dedicated bot users with minimal access rights in Odoo for automation.
