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

### Run with Docker Compose (.env)

Create `.env` in the repo root (example in `dotenv.example`), then:

```bash
docker compose up --build
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
