# Odoo Rust MCP Server - AI Coding Agent Instructions

## 🎯 Project Overview

A **dual-stack Model Context Protocol (MCP) server** for Odoo integration:
- **Backend**: Rust (`rust-mcp/`) - MCP protocol implementation, Odoo client abstraction, tools execution
- **Frontend**: React TypeScript (`config-ui/`) - Web-based configuration editor for servers, tools, prompts, and multi-instance Odoo connections
- Supports **Odoo 19+** (JSON-2 External API with API keys) and **Odoo <19** (JSON-RPC with username/password)

## 🏗️ Architecture Patterns

### Dual Client Pattern (`src/odoo/`)
The codebase maintains **abstraction over two Odoo authentication methods**:
- **Modern**: `OdooClient` (Odoo 19+) - JSON-2 API with API key auth
- **Legacy**: `LegacyClient` (Odoo <19) - JSON-RPC with username/password auth
- **Unified**: `OdooClient` trait abstracts both implementations

**Key file**: `src/odoo/unified_client.rs` - All tool operations go through this abstraction.

### MCP Handler Pattern (`src/mcp/`)
Core request handler implements `ServerHandler` trait with these methods:
- `tools/list` - Returns tools from registry (tools.json)
- `tools/call` - Routes to `execute_op()` in `mcp/tools.rs`
- `prompts/list`, `prompts/get` - Returns prompts from registry
- `resources/list`, `resources/read` - Returns Odoo instance metadata via `odoo://` URIs

**Tool execution flow**: `tools/call` → `execute_op()` → `op_search`, `op_read`, `op_create`, etc. → Odoo client

### Config Manager Architecture (`src/config_manager/`)
**Hot-reload capable** configuration system:
- **Manager** (`manager.rs`) - CRUD operations for JSON configs (instances, tools, prompts, server)
- **Watcher** (`watcher.rs`) - File system monitoring for external changes
- **Server** (`server.rs`) - HTTP REST API and Axum web server on port 3008
- **React UI** (`config-ui/`) - Consumes REST API, syncs to manager

**Critical workflow**: File change → Watcher detects → Registry reloads → MCP handler reflects changes

### Registry Pattern (`src/mcp/registry.rs`)
Centralized configuration store:
- Loads `tools.json`, `prompts.json`, `server.json` from `~/.config/odoo-rust-mcp/`
- Caches parsed tool definitions with guard evaluation (`requiresEnv`, `requiresEnvTrue`)
- Guards gate access to dangerous tools (cleanup tools require `ODOO_ENABLE_CLEANUP_TOOLS=true`)

## 🔄 Key Workflows

### Adding a New Tool
1. Define tool structure in `src/mcp/registry.rs` (e.g., `OpSpec` with `op_type` and `map`)
2. Implement handler function: `async fn op_my_tool(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError>`
3. Add `op_type` case in `execute_op()` router
4. Add JSON definition to `config/tools.json`
5. Test with unit tests in `src/mcp/tools.rs` module tests

### Testing
- **Rust**: `cargo test --lib` (193 lib tests), `cargo test --all-targets` (includes integration tests in `tests/`)
- **TypeScript**: `npm test` (205 tests), `npm run test:coverage` (Istanbul provider)
- **CI**: GitHub Actions (parallel: build-ui, check, fmt, clippy, test, test-multiplatform, coverage)
- **Important**: Don't use `2>&1` in commands - stdout/stderr already go to terminal

### Code Quality
- **Format**: `cargo fmt --all` before commit
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings` (must pass)
- Both checked in CI (`fmt` and `clippy` jobs)

### Build Outputs
- **Rust Binary**: `target/debug/rust-mcp` or `target/release/rust-mcp`
- **UI Static**: Built to `rust-mcp/static/dist/` (embedded in binary via `include_dir!`)
- **Coverage**: Rust via `cargo-tarpaulin`, TS via `vitest` with `@vitest/coverage-istanbul`

## 📋 Project Structure

```
rust-mcp/                    # Rust server (Cargo workspace)
  src/
    main.rs                  # CLI entry point (stdio, HTTP, WebSocket transports)
    lib.rs                   # Library root (pub mod cleanup, config_manager, mcp, odoo)
    mcp/
      mod.rs                 # McpOdooHandler (ServerHandler implementation)
      tools.rs               # Tool execution logic (1400+ lines, helper functions)
      registry.rs            # Config loading, tool/prompt definitions
      cursor_stdio.rs        # Stdio transport for Cursor/Claude Desktop
      http.rs                # Axum server with MCP-over-HTTP transport
      cache.rs               # Metadata caching with TTL
      prompts.rs             # Hardcoded prompts for domain-specific guidance
      resources.rs           # odoo:// URI scheme parsing
      runtime.rs             # ServerCompat wrapper for MCP compatibility
    odoo/
      unified_client.rs      # OdooClient trait (abstraction over modern + legacy)
      client.rs              # Modern JSON-2 API client (Odoo 19+)
      legacy_client.rs       # Legacy JSON-RPC client (Odoo <19)
      config.rs              # Instance config parsing and auth mode detection
      types.rs               # OdooError, serialization types
    config_manager/
      manager.rs             # Config CRUD (load/save instances, tools, prompts)
      server.rs              # Axum HTTP server (REST API, UI serving)
      watcher.rs             # File watcher for hot reload
      mod.rs                 # Exports
    cleanup/
      database.rs            # Database cleanup tool
      deep.rs                # Deep record cleanup with relationships
      mod.rs                 # Exports
  config/                    # Default configs (checked into repo)
    tools.json               # All tool definitions
    prompts.json             # Static prompts
    server.json              # Server metadata
  tests/                     # Integration tests (*.rs files)

config-ui/                   # React/TypeScript UI
  src/
    components/              # React components (tabs, forms, editor)
    hooks/                   # useConfig, useAuth custom hooks
    __tests__/               # 205 tests (vitest)
    types.ts                 # TypeScript types mirroring Rust config
  vite.config.ts             # Vite bundler (builds to rust-mcp/static/dist)
  vitest.config.ts           # Test config (istanbul coverage provider)

.github/
  workflows/
    ci.yml                   # Multi-stage CI (build-ui → quality checks → coverage)
  instructions/
    instructions.md          # Existing instructions (avoid 2>&1)
```

## 🚀 Critical Commands & Development

### Local Development
```bash
# Terminal 1: Rust server with config UI
cargo run --manifest-path rust-mcp/Cargo.toml -- --config-server-port 3008

# Terminal 2: React dev server (hot reload on config-ui changes)
cd config-ui && npm run dev   # Runs on :5173, proxied through Vite

# Access UI: http://localhost:3008 (after server starts)
# Or: http://localhost:5173 (dev mode with HMR)
```

### Testing During Development
```bash
# Run affected tests (watch mode)
cargo test --lib                              # Rust lib tests
cd config-ui && npm test                      # TypeScript tests

# Full coverage (run before PR)
cargo tarpaulin --all-targets --all-features # Rust coverage
cd config-ui && npm run test:coverage         # TS coverage (Istanbul)
```

### Pre-Commit Checklist
```bash
# REQUIRED: Run these BEFORE committing
cargo fmt --all                               # Format Rust code
cargo clippy --all-targets --all-features -- -D warnings  # Lint Rust code

cd config-ui && npm run lint && npm test     # Lint & test TypeScript

# Do NOT commit if any of these fail!
# CI will block the commit if fmt/clippy have errors.
```

**Important**: The CI pipeline has dedicated jobs for `fmt` and `clippy`:
- If `cargo fmt` produces different output, CI will **FAIL**
- If `cargo clippy` has warnings, CI will **FAIL**
- Always run these locally before pushing

## 🔐 Important Patterns & Gotchas

### 1. **Tool Guards & Cleanup Tools**
- Dangerous tools (database cleanup, deep cleanup) require `ODOO_ENABLE_CLEANUP_TOOLS=true`
- Guards are evaluated in `registry.rs::eval_guard()` - check `requiresEnv` conditions
- Add guards when creating privileged operations

### 2. **Async/Await Everywhere**
- All Odoo operations are async (reqwest, tokio)
- Use `async fn` in tool implementations
- `OdooClientPool::get()` requires `.await`

### 3. **Configuration Paths & Defaults**
- User config: `~/.config/odoo-rust-mcp/` (created on first run)
- Default configs in `config/` folder are copied to user home on startup
- `ODOO_INSTANCES` env var overrides instances.json location
- See `main.rs::get_config_dir()` and `copy_default_config_if_missing()`

### 4. **Multi-Instance Pooling**
- `OdooClientPool` maintains a hashmap of clients per instance
- `pool.get(instance_name)` creates client if missing, else returns cached
- Instance availability checked via `pool.instance_names()` (used in `initialize()`)

### 5. **JSON Pointer Mapping**
- Tools use `/path/to/field` syntax to extract args from JSON via `json_pointer` crate
- Helper functions: `ptr()`, `req_str()`, `opt_str()`, `opt_i64()`, `opt_vec_string()`
- Map field names to pointers in `tools.json` under `"map": { "field": "/path/to/field" }`

### 6. **React Hook Rules**
- Tests CANNOT call hooks directly (they need React context)
- Test types that use hooks; don't test hook behavior in isolation
- Use `source-import.test.tsx` pattern: import types, test type shapes, not hook execution

### 7. **Coverage Reporting**
- Rust: Tarpaulin reports only lib tests (not integration tests in `tests/` folder)
- TypeScript: Istanbul provides file-by-file breakdown; only counts imported source code
- If coverage is low, check that tests actually **import and use** source code

## 📦 Environment Variables (Key Ones)

| Variable | Purpose | Example |
|----------|---------|---------|
| `ODOO_INSTANCES` | Override instances JSON path | `~/.config/odoo-rust-mcp/instances.json` |
| `ODOO_ENABLE_CLEANUP_TOOLS` | Gate cleanup operations | `true` |
| `RUST_LOG` | Tracing level | `debug,odoo=trace` |
| `ODOO_CONFIG_DIR` | Config directory | `~/.config/odoo-rust-mcp` |
| `MCP_AUTH_ENABLED` | HTTP basic auth | `true` |

## 🎬 Common Development Scenarios

### Scenario: Add a new Odoo operation tool
1. Add `op_my_operation()` function in `src/mcp/tools.rs`
2. Add case in `execute_op()` router: `"my_operation" => op_my_operation(...).await`
3. Add JSON to `config/tools.json` with tool schema
4. Write unit test in `src/mcp/tools.rs::tests` module
5. Update `src/cleanup/deep.rs` if it affects record relationships

### Scenario: Modify config schema (e.g., add field to instances)
1. Update `src/odoo/config.rs::InstanceConfig` struct
2. Update `config-ui/src/types.ts::InstanceConfig` type to match
3. Add migration in `src/config_manager/manager.rs` if backwards compat needed
4. Test with manual config changes or unit tests

### Scenario: Debug why Codecov dropped
1. Run `cargo tarpaulin --all-targets --all-features` locally (check Rust coverage)
2. Run `cd config-ui && npm run test:coverage` (check TS file-by-file)
3. Verify files are actually **imported** in tests (Istanbul only counts imported source)
4. Check CI logs for `coverage` step in `.github/workflows/ci.yml`

---

**Last Updated**: 2025-01-27  
**Workspace Root**: `/workspaces/odoo-rust-mcp`
