class RustMcp < Formula
  desc "Odoo MCP Server - Model Context Protocol server for Odoo integration"
  homepage "https://github.com/rachmataditiya/odoo-rust-mcp"
  version "0.3.27"
  license "AGPL-3.0-only"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rachmataditiya/odoo-rust-mcp/releases/download/v#{version}/rust-mcp-aarch64-apple-darwin.tar.gz"
      sha256 "463eb4a62bb8a51d2432dc6fa26a66f2c4ca1c21192afeb5a8e8644cb8e658b5"  # macos-arm64
    end

    if Hardware::CPU.intel?
      url "https://github.com/rachmataditiya/odoo-rust-mcp/releases/download/v#{version}/rust-mcp-x86_64-apple-darwin.tar.gz"
      sha256 "b4941498ea268e3016bc2477cde9f974d4380c3f7078d60e223a87599ab865de"  # macos-x64
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/rachmataditiya/odoo-rust-mcp/releases/download/v#{version}/rust-mcp-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "e44aa9c0ee0bc292f0122b98612851a21f6415fccc39505482cbab37cb9ab0fb"  # linux-x64
    end
  end

  def install
    bin.install "rust-mcp"
    # Install config files to share directory (defaults)
    (share/"odoo-rust-mcp").install Dir["config/*"] if Dir.exist?("config")
    # Install example env file
    (share/"odoo-rust-mcp").install ".env.example" if File.exist?(".env.example")
    # Install static files (React UI) if present
    if Dir.exist?("static/dist")
      (share/"odoo-rust-mcp/static/dist").install Dir["static/dist/*"]
    end

    # Create wrapper script that loads env file before running
    # Also creates config dir if it doesn't exist (fallback for post_install)
    wrapper_script = bin/"rust-mcp-service"
    wrapper_script.write <<~EOS
      #!/bin/bash
      CONFIG_DIR="$HOME/.config/odoo-rust-mcp"
      
      # Create config directory if it doesn't exist
      if [ ! -d "$CONFIG_DIR" ]; then
        mkdir -p "$CONFIG_DIR"
        echo "Created config directory: $CONFIG_DIR"
      fi
      
      # Create default instances.json for multi-instance configuration
      if [ ! -f "$CONFIG_DIR/instances.json" ]; then
        cat > "$CONFIG_DIR/instances.json" << 'INSTANCESEOF'
{
  "production": {
    "url": "http://localhost:8069",
    "db": "production",
    "apiKey": "YOUR_ODOO_19_API_KEY"
  },
  "staging": {
    "url": "http://localhost:8069",
    "db": "staging",
    "apiKey": "YOUR_STAGING_API_KEY"
  },
  "development": {
    "url": "http://localhost:8069",
    "db": "development",
    "version": "18",
    "username": "admin",
    "password": "admin"
  }
}
INSTANCESEOF
        chmod 600 "$CONFIG_DIR/instances.json"
        echo "Created default instances.json: $CONFIG_DIR/instances.json"
        echo "Please edit it with your Odoo credentials"
      fi
      
      # Create default env file if it doesn't exist
      if [ ! -f "$CONFIG_DIR/env" ]; then
        cat > "$CONFIG_DIR/env" << ENVEOF
# Odoo Rust MCP Server Configuration
# Edit this file with your settings

# =============================================================================
# Multi-Instance Configuration (Default - Recommended)
# =============================================================================
# Uses instances.json for multiple Odoo instances
ODOO_INSTANCES_JSON=$CONFIG_DIR/instances.json

# =============================================================================
# Single Instance Configuration (Alternative - uncomment if not using multi-instance)
# =============================================================================
# # Odoo 19+ (API Key authentication)
# ODOO_URL=http://localhost:8069
# ODOO_DB=mydb
# ODOO_API_KEY=YOUR_API_KEY
#
# # Odoo < 19 (Username/Password authentication)
# # ODOO_VERSION=18
# # ODOO_USERNAME=admin
# # ODOO_PASSWORD=admin

# =============================================================================
# Config UI Authentication
# =============================================================================
# Login credentials for the config web UI
# IMPORTANT: Change these default credentials!
CONFIG_UI_USERNAME=admin
CONFIG_UI_PASSWORD=changeme

# =============================================================================
# MCP HTTP Transport Authentication
# =============================================================================
# Enable/disable HTTP authentication (default: false)
MCP_AUTH_ENABLED=false
# Generate a secure token: openssl rand -hex 32
# MCP_AUTH_TOKEN=your-secure-random-token-here

# =============================================================================
# MCP Config Paths
# =============================================================================
# Path to MCP configuration files (tools, prompts, server settings)
MCP_TOOLS_JSON=$CONFIG_DIR/tools.json
MCP_PROMPTS_JSON=$CONFIG_DIR/prompts.json
MCP_SERVER_JSON=$CONFIG_DIR/server.json
ENVEOF
        chmod 600 "$CONFIG_DIR/env"
        echo "Created default env file: $CONFIG_DIR/env"
      fi
      
      # Copy default config files to user directory if they don't exist
      SHARE_DIR="#{HOMEBREW_PREFIX}/share/odoo-rust-mcp"
      for config_file in tools.json prompts.json server.json; do
        if [ ! -f "$CONFIG_DIR/$config_file" ] && [ -f "$SHARE_DIR/$config_file" ]; then
          cp "$SHARE_DIR/$config_file" "$CONFIG_DIR/$config_file"
          chmod 600 "$CONFIG_DIR/$config_file"
          echo "Copied default $config_file to: $CONFIG_DIR/$config_file"
        fi
      done
      
      # Migrate existing env file: add MCP config paths if missing (upgrade support)
      if [ -f "$CONFIG_DIR/env" ] && ! grep -q "MCP_TOOLS_JSON" "$CONFIG_DIR/env"; then
        cat >> "$CONFIG_DIR/env" << MIGRATEEOF

# =============================================================================
# MCP Config Paths (added in v0.3.27)
# =============================================================================
# Path to MCP configuration files (tools, prompts, server settings)
MCP_TOOLS_JSON=$CONFIG_DIR/tools.json
MCP_PROMPTS_JSON=$CONFIG_DIR/prompts.json
MCP_SERVER_JSON=$CONFIG_DIR/server.json
MIGRATEEOF
        echo "Migration: Added MCP config paths to env file"
      fi
      
      # Load environment from user config if exists
      if [ -f "$CONFIG_DIR/env" ]; then
        set -a
        source "$CONFIG_DIR/env"
        set +a
      fi
      
      # Set default MCP config paths to user config dir (fallback to share dir)
      export MCP_TOOLS_JSON="${MCP_TOOLS_JSON:-$CONFIG_DIR/tools.json}"
      export MCP_PROMPTS_JSON="${MCP_PROMPTS_JSON:-$CONFIG_DIR/prompts.json}"
      export MCP_SERVER_JSON="${MCP_SERVER_JSON:-$CONFIG_DIR/server.json}"
      
      # Set default instances.json path if not already set
      export ODOO_INSTANCES_JSON="${ODOO_INSTANCES_JSON:-$CONFIG_DIR/instances.json}"
      
      # Change to share directory so static files can be found
      cd "#{HOMEBREW_PREFIX}/share/odoo-rust-mcp"
      
      exec "#{opt_bin}/rust-mcp" "$@"
    EOS
    # Ensure executable permission is set correctly
    wrapper_script.chmod 0755
  end

  # Service configuration for `brew services start rust-mcp`
  # Service uses binary directly (v0.2.4+ auto-loads config)
  # Supports ODOO_INSTANCES_JSON file for readable multi-instance config
  service do
    run [opt_bin/"rust-mcp", "--transport", "http", "--listen", "127.0.0.1:8787"]
    keep_alive true
    log_path var/"log/rust-mcp.log"
    error_log_path var/"log/rust-mcp.log"
  end

  def caveats
    <<~EOS
      Configuration directory: ~/.config/odoo-rust-mcp/
      
      The config directory and default files will be automatically created
      the first time you run 'rust-mcp-service'.
      
      Quick start:
        1. Run once to create config: rust-mcp-service --help
        2. Edit instances: nano ~/.config/odoo-rust-mcp/instances.json
        3. Start service: brew services start rust-mcp

      Multi-instance configuration (default):
        ~/.config/odoo-rust-mcp/instances.json - Configure multiple Odoo instances
        ~/.config/odoo-rust-mcp/env - Environment settings

      Service commands:
        brew services start rust-mcp
        brew services stop rust-mcp
        brew services restart rust-mcp

      Service endpoint: http://127.0.0.1:8787/mcp
      Service logs: #{var}/log/rust-mcp.log

      For Cursor IDE configuration:
        See: https://github.com/rachmataditiya/odoo-rust-mcp#readme
    EOS
  end

  test do
    assert_match "rust-mcp", shell_output("#{bin}/rust-mcp --help")
  end
end
