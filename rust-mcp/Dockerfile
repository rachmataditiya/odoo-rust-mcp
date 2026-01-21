FROM rust:1.91.1-slim-bookworm AS builder

WORKDIR /app

# (Optional) needed by some crates; keep minimal for this repo
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
COPY config-defaults ./config-defaults

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-mcp /usr/local/bin/rust-mcp

EXPOSE 8787

# Default to Streamable HTTP (Cursor remote transport).
# Override with `--transport stdio|ws` as needed.
CMD ["rust-mcp", "--transport", "http", "--listen", "0.0.0.0:8787"]

