#!/bin/bash
set -e

# Odoo Rust MCP Server Installer
# Installs to /usr/local/bin (binary) and /usr/local/share/odoo-rust-mcp (config)
# Run from extracted release directory: ./install.sh

BINARY_NAME="rust-mcp"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/usr/local/share/odoo-rust-mcp"

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
    echo "Quick start:"
    echo "  1. Copy and edit the example environment file:"
    echo "     cp $CONFIG_DIR/.env.example ~/.odoo-mcp.env"
    echo "     # Edit ~/.odoo-mcp.env with your Odoo credentials"
    echo ""
    echo "  2. Run the server:"
    echo "     $BINARY_NAME --transport stdio"
    echo ""
    echo "  3. For Cursor, add to ~/.cursor/mcp.json:"
    echo '     {'
    echo '       "mcpServers": {'
    echo '         "odoo-rust-mcp": {'
    echo '           "type": "stdio",'
    echo "           \"command\": \"$INSTALL_DIR/$BINARY_NAME\","
    echo '           "args": ["--transport", "stdio"],'
    echo '           "env": {'
    echo '             "ODOO_URL": "http://localhost:8069",'
    echo '             "ODOO_DB": "mydb",'
    echo '             "ODOO_API_KEY": "YOUR_API_KEY",'
    echo "             \"MCP_TOOLS_JSON\": \"$CONFIG_DIR/tools.json\","
    echo "             \"MCP_PROMPTS_JSON\": \"$CONFIG_DIR/prompts.json\","
    echo "             \"MCP_SERVER_JSON\": \"$CONFIG_DIR/server.json\""
    echo '           }'
    echo '         }'
    echo '       }'
    echo '     }'
}

# Uninstall
uninstall() {
    info "Uninstalling odoo-rust-mcp..."

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

    info "Uninstall complete!"
}

# Main
case "${1:-install}" in
    install)   install ;;
    uninstall) uninstall ;;
    *)         echo "Usage: $0 [install|uninstall]"; exit 1 ;;
esac
