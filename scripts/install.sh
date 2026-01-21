#!/bin/bash
set -e

# Odoo Rust MCP Server Installer
# Installs to /usr/local/bin (binary) and /usr/local/share/odoo-rust-mcp (config)
# Run from extracted release directory: ./install.sh [install|uninstall|service|service-uninstall]

BINARY_NAME="rust-mcp"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/usr/local/share/odoo-rust-mcp"
ENV_FILE="/etc/odoo-rust-mcp.env"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Get script directory (where the release was extracted)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       error "Unsupported OS: $(uname -s)" ;;
    esac
}

# Install from local extracted release
install() {
    info "Installing odoo-rust-mcp..."

    # Check if binary exists in current directory
    if [ ! -f "$SCRIPT_DIR/$BINARY_NAME" ]; then
        error "Binary '$BINARY_NAME' not found in $SCRIPT_DIR. Make sure you're running from the extracted release directory."
    fi

    info "Installing binary to $INSTALL_DIR..."
    sudo mkdir -p "$INSTALL_DIR"
    sudo cp "$SCRIPT_DIR/$BINARY_NAME" "$INSTALL_DIR/" || error "Failed to copy binary"
    sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

    info "Installing config files to $CONFIG_DIR..."
    sudo mkdir -p "$CONFIG_DIR"
    if [ -d "$SCRIPT_DIR/config" ]; then
        sudo cp -r "$SCRIPT_DIR/config/"* "$CONFIG_DIR/" || warn "Failed to copy config files"
    fi

    if [ -f "$SCRIPT_DIR/.env.example" ]; then
        sudo cp "$SCRIPT_DIR/.env.example" "$CONFIG_DIR/" || warn "Failed to copy .env.example"
    fi

    info "Installation complete!"
    echo ""
    echo "Binary installed to: $INSTALL_DIR/$BINARY_NAME"
    echo "Config files installed to: $CONFIG_DIR"
    echo ""
    echo "Usage:"
    echo "  Run directly (stdio):  $BINARY_NAME --transport stdio"
    echo "  Run as HTTP server:    $BINARY_NAME --transport http --listen 127.0.0.1:8787"
    echo "  Install as service:    ./install.sh service"
    echo ""
    echo "For Cursor/Claude Desktop, see README for configuration examples."
}

# Uninstall
uninstall() {
    info "Uninstalling odoo-rust-mcp..."

    # Stop and remove service first
    service_uninstall 2>/dev/null || true

    if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        sudo rm -f "$INSTALL_DIR/$BINARY_NAME"
        info "Removed $INSTALL_DIR/$BINARY_NAME"
    fi

    if [ -d "$CONFIG_DIR" ]; then
        read -p "Remove config directory $CONFIG_DIR? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo rm -rf "$CONFIG_DIR"
            info "Removed $CONFIG_DIR"
        fi
    fi

    if [ -f "$ENV_FILE" ]; then
        read -p "Remove environment file $ENV_FILE? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo rm -f "$ENV_FILE"
            info "Removed $ENV_FILE"
        fi
    fi

    info "Uninstall complete!"
}

# Install systemd service (Linux)
install_systemd_service() {
    info "Installing systemd service..."

    # Create environment file if not exists
    if [ ! -f "$ENV_FILE" ]; then
        info "Creating environment file at $ENV_FILE..."
        sudo tee "$ENV_FILE" > /dev/null << 'ENVEOF'
# Odoo MCP Server Environment Configuration
# Edit this file with your Odoo credentials

# Odoo 19+ (API Key authentication)
ODOO_URL=http://localhost:8069
ODOO_DB=mydb
ODOO_API_KEY=YOUR_API_KEY

# Odoo < 19 (Username/Password authentication)
# ODOO_URL=http://localhost:8069
# ODOO_DB=mydb
# ODOO_VERSION=18
# ODOO_USERNAME=admin
# ODOO_PASSWORD=admin

# MCP Authentication (HTTP transport)
# Generate a secure token: openssl rand -hex 32
MCP_AUTH_TOKEN=CHANGE_ME_TO_A_SECURE_TOKEN

# MCP Config paths
MCP_TOOLS_JSON=/usr/local/share/odoo-rust-mcp/tools.json
MCP_PROMPTS_JSON=/usr/local/share/odoo-rust-mcp/prompts.json
MCP_SERVER_JSON=/usr/local/share/odoo-rust-mcp/server.json
ENVEOF
        sudo chmod 600 "$ENV_FILE"
        warn "Please edit $ENV_FILE with your Odoo credentials and MCP_AUTH_TOKEN"
    fi

    # Create systemd service file
    sudo tee /etc/systemd/system/odoo-rust-mcp.service > /dev/null << SERVICEEOF
[Unit]
Description=Odoo Rust MCP Server
After=network.target

[Service]
Type=simple
User=nobody
Group=nogroup
EnvironmentFile=$ENV_FILE
ExecStart=$INSTALL_DIR/$BINARY_NAME --transport http --listen 127.0.0.1:8787
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
SERVICEEOF

    sudo systemctl daemon-reload
    sudo systemctl enable odoo-rust-mcp.service
    info "Systemd service installed and enabled"
    echo ""
    echo "Commands:"
    echo "  Start:   sudo systemctl start odoo-rust-mcp"
    echo "  Stop:    sudo systemctl stop odoo-rust-mcp"
    echo "  Status:  sudo systemctl status odoo-rust-mcp"
    echo "  Logs:    sudo journalctl -u odoo-rust-mcp -f"
    echo ""
    echo "Service listens on: http://127.0.0.1:8787/mcp"
    warn "Don't forget to edit $ENV_FILE with your Odoo credentials!"
}

# Uninstall systemd service (Linux)
uninstall_systemd_service() {
    info "Removing systemd service..."
    sudo systemctl stop odoo-rust-mcp.service 2>/dev/null || true
    sudo systemctl disable odoo-rust-mcp.service 2>/dev/null || true
    sudo rm -f /etc/systemd/system/odoo-rust-mcp.service
    sudo systemctl daemon-reload
    info "Systemd service removed"
}

# Install launchd service (macOS)
install_launchd_service() {
    info "Installing launchd service..."

    local plist_path="$HOME/Library/LaunchAgents/com.odoo.rust-mcp.plist"
    local user_config_dir="$HOME/.config/odoo-rust-mcp"
    local user_env_file="$user_config_dir/env"

    # Create user config directory
    mkdir -p "$user_config_dir"

    # Create environment file if not exists
    if [ ! -f "$user_env_file" ]; then
        info "Creating environment file at $user_env_file..."
        cat > "$user_env_file" << 'ENVEOF'
# Odoo MCP Server Environment Configuration
# Edit this file with your Odoo credentials

# Odoo 19+ (API Key authentication)
ODOO_URL=http://localhost:8069
ODOO_DB=mydb
ODOO_API_KEY=YOUR_API_KEY

# Odoo < 19 (Username/Password authentication)
# ODOO_URL=http://localhost:8069
# ODOO_DB=mydb
# ODOO_VERSION=18
# ODOO_USERNAME=admin
# ODOO_PASSWORD=admin

# MCP Authentication (HTTP transport)
# Generate a secure token: openssl rand -hex 32
MCP_AUTH_TOKEN=CHANGE_ME_TO_A_SECURE_TOKEN

# MCP Config paths
MCP_TOOLS_JSON=/usr/local/share/odoo-rust-mcp/tools.json
MCP_PROMPTS_JSON=/usr/local/share/odoo-rust-mcp/prompts.json
MCP_SERVER_JSON=/usr/local/share/odoo-rust-mcp/server.json
ENVEOF
        chmod 600 "$user_env_file"
        warn "Please edit $user_env_file with your Odoo credentials and MCP_AUTH_TOKEN"
    fi

    # Create launchd plist
    mkdir -p "$HOME/Library/LaunchAgents"
    cat > "$plist_path" << PLISTEOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.odoo.rust-mcp</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>-c</string>
        <string>set -a; source $user_env_file; set +a; exec $INSTALL_DIR/$BINARY_NAME --transport http --listen 127.0.0.1:8787</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$user_config_dir/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>$user_config_dir/stderr.log</string>
</dict>
</plist>
PLISTEOF

    launchctl unload "$plist_path" 2>/dev/null || true
    launchctl load "$plist_path"
    info "Launchd service installed and started"
    echo ""
    echo "Commands:"
    echo "  Start:   launchctl load $plist_path"
    echo "  Stop:    launchctl unload $plist_path"
    echo "  Logs:    tail -f $user_config_dir/stdout.log"
    echo ""
    echo "Service listens on: http://127.0.0.1:8787/mcp"
    warn "Don't forget to edit $user_env_file with your Odoo credentials!"
}

# Uninstall launchd service (macOS)
uninstall_launchd_service() {
    info "Removing launchd service..."
    local plist_path="$HOME/Library/LaunchAgents/com.odoo.rust-mcp.plist"
    launchctl unload "$plist_path" 2>/dev/null || true
    rm -f "$plist_path"
    info "Launchd service removed"
}

# Install service (auto-detect OS)
service_install() {
    # First ensure binary is installed
    if [ ! -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        install
    fi

    local os=$(detect_os)
    case "$os" in
        linux) install_systemd_service ;;
        macos) install_launchd_service ;;
    esac
}

# Uninstall service (auto-detect OS)
service_uninstall() {
    local os=$(detect_os)
    case "$os" in
        linux) uninstall_systemd_service ;;
        macos) uninstall_launchd_service ;;
    esac
}

# Show help
show_help() {
    echo "Odoo Rust MCP Server Installer"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  install           Install binary and config files (default)"
    echo "  uninstall         Uninstall binary, config, and service"
    echo "  service           Install and start as background service"
    echo "  service-uninstall Remove background service"
    echo ""
    echo "Examples:"
    echo "  $0                # Install binary only"
    echo "  $0 service        # Install and run as HTTP service"
    echo "  $0 uninstall      # Remove everything"
}

# Main
case "${1:-install}" in
    install)           install ;;
    uninstall)         uninstall ;;
    service)           service_install ;;
    service-uninstall) service_uninstall ;;
    help|--help|-h)    show_help ;;
    *)                 show_help; exit 1 ;;
esac
