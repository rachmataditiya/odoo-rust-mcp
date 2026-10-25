# Odoo Rust MCP Server - Multi-stage Docker Build
#
# Build: docker build -t odoo-rust-mcp .
# Run:   docker run -p 8787:8787 --env-file .env odoo-rust-mcp

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.91.1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies (including Node.js for React UI)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
  && rm -rf /var/lib/apt/lists/*

# Install Node.js 20
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
  && apt-get install -y --no-install-recommends nodejs \
  && rm -rf /var/lib/apt/lists/*

# Prepare Rust build files first
WORKDIR /app/rust-mcp
COPY rust-mcp/Cargo.toml rust-mcp/Cargo.lock ./
COPY rust-mcp/src ./src
COPY rust-mcp/tests ./tests
COPY rust-mcp/config-defaults ./config-defaults
# Copy openapi directory (required for include_str! in code)
COPY rust-mcp/openapi ./openapi
# Create static directory structure
RUN mkdir -p ./static

# Copy any static assets from build context (just .gitkeep, but structure matters)
COPY rust-mcp/static/ ./static/

# Build React UI
WORKDIR /app/config-ui
COPY config-ui/package.json config-ui/package-lock.json* ./
RUN npm ci || npm install

COPY config-ui ./
RUN npm run build

# React UI is now built at /app/rust-mcp/static/dist

# Verify UI build output
RUN if [ -d /app/rust-mcp/static/dist ] && [ "$(ls -A /app/rust-mcp/static/dist 2>/dev/null)" ]; then \
        echo "✓ UI build output found at /app/rust-mcp/static/dist"; \
        find /app/rust-mcp/static/dist -type f | head -5; \
    else \
        echo "⚠ Warning: UI build output not found at expected location"; \
    fi

# Back to Rust directory for final build
WORKDIR /app/rust-mcp

# Build release binary
# Note: cargo builds to target/release/ by default
RUN cargo build --release --verbose 2>&1 | tail -100

# Verify binary exists
RUN if [ ! -f /app/rust-mcp/target/release/rust-mcp ]; then \
        echo "ERROR: Binary not found at /app/rust-mcp/target/release/rust-mcp"; \
        echo "Contents of /app/rust-mcp/target/release:"; \
        ls -la /app/rust-mcp/target/release/ 2>/dev/null || echo "Directory does not exist"; \
        exit 1; \
    else \
        echo "✓ Binary found at /app/rust-mcp/target/release/rust-mcp"; \
        ls -lh /app/rust-mcp/target/release/rust-mcp; \
    fi

# Let Docker fail with a clear error if binary doesn't exist
# This will help diagnose the issue

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

# OCI Labels for container registry
LABEL org.opencontainers.image.title="Odoo Rust MCP Server"
LABEL org.opencontainers.image.description="Model Context Protocol server for Odoo integration"
LABEL org.opencontainers.image.source="https://github.com/rachmataditiya/odoo-rust-mcp"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.vendor="Rachmat Aditiya"

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
  && rm -rf /var/lib/apt/lists/*

# Create non-root user for security (Kubernetes best practice)
RUN groupadd -r mcp && useradd -r -g mcp mcp

# Create config directory
RUN mkdir -p /config && chown mcp:mcp /config

WORKDIR /app

# Copy binary from builder
# Note: Binary is built in /app/rust-mcp/target/release/rust-mcp (not /app/target/release/rust-mcp)
COPY --from=builder /app/rust-mcp/target/release/rust-mcp /usr/local/bin/rust-mcp

# Copy static files (React UI) from builder
# Note: Working directory is /app, so static/dist will be at /app/static/dist
COPY --from=builder --chown=mcp:mcp /app/rust-mcp/static /app/static

# Copy default config files
COPY --chown=mcp:mcp config /config

# Switch to non-root user
USER mcp

# Expose HTTP ports (MCP server and config server)
EXPOSE 8787 3008

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
  CMD curl -sf http://localhost:8787/mcp -X POST -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","id":1,"method":"ping"}' || exit 1

# Environment defaults
ENV MCP_TOOLS_JSON=/config/tools.json
ENV MCP_PROMPTS_JSON=/config/prompts.json
ENV MCP_SERVER_JSON=/config/server.json

# Default to Streamable HTTP transport
# Override with: --transport stdio|ws
CMD ["rust-mcp", "--transport", "http", "--listen", "0.0.0.0:8787"]
