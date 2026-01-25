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

# Build React UI first
WORKDIR /app/config-ui
COPY config-ui/package.json config-ui/package-lock.json* ./
RUN npm ci || npm install

COPY config-ui ./
RUN npm run build

# Build Rust binary
WORKDIR /app/rust-mcp
COPY rust-mcp/Cargo.toml rust-mcp/Cargo.lock ./
COPY rust-mcp/src ./src
COPY rust-mcp/tests ./tests
COPY rust-mcp/config-defaults ./config-defaults
COPY rust-mcp/openapi ./openapi
COPY rust-mcp/static ./static

# Copy built UI to static/dist (Vite outputs to ../rust-mcp/static/dist from config-ui)
RUN mkdir -p static/dist && cp -r ../rust-mcp/static/dist/* static/dist/ 2>/dev/null || echo "Note: UI build output should be at rust-mcp/static/dist"

# Build release binary
RUN cargo build --release

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

# OCI Labels for container registry
LABEL org.opencontainers.image.title="Odoo Rust MCP Server"
LABEL org.opencontainers.image.description="Model Context Protocol server for Odoo integration"
LABEL org.opencontainers.image.source="https://github.com/rachmataditiya/odoo-rust-mcp"
LABEL org.opencontainers.image.licenses="AGPL-3.0-only"
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
COPY --from=builder /app/target/release/rust-mcp /usr/local/bin/rust-mcp

# Copy default config files
COPY --chown=mcp:mcp config /config

# Switch to non-root user
USER mcp

# Expose HTTP port
EXPOSE 8787

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
