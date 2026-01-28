# Odoo Rust MCP - APT Repository

This directory contains the APT repository configuration for distributing rust-mcp on Debian/Ubuntu systems.

## Installation

### Quick Install

```bash
# Add GPG key
curl -fsSL https://rachmataditiya.github.io/odoo-rust-mcp/pubkey.gpg | sudo gpg --dearmor -o /usr/share/keyrings/rust-mcp.gpg

# Add repository
echo "deb [signed-by=/usr/share/keyrings/rust-mcp.gpg] https://rachmataditiya.github.io/odoo-rust-mcp stable main" | sudo tee /etc/apt/sources.list.d/rust-mcp.list

# Install
sudo apt update
sudo apt install rust-mcp
```

### Configuration

After installation, edit your configuration:

```bash
nano ~/.config/rust-mcp/env
```

### Start Service

```bash
# Start the service
sudo systemctl start rust-mcp

# Enable on boot
sudo systemctl enable rust-mcp

# Check status
sudo systemctl status rust-mcp
```

### Service Endpoint

Once running, the MCP server is available at: `http://127.0.0.1:8787/mcp`

## Uninstall

```bash
sudo apt remove rust-mcp
sudo rm /etc/apt/sources.list.d/rust-mcp.list
sudo rm /usr/share/keyrings/rust-mcp.gpg
```

## Repository Structure

```
apt-repo/
├── conf/
│   ├── distributions    # Repository configuration
│   └── options          # reprepro options
├── dists/               # Distribution metadata (auto-generated)
├── pool/                # Package files (auto-generated)
└── pubkey.gpg           # Public GPG key for verification
```

## For Maintainers

The repository is automatically updated by GitHub Actions when a new release is tagged.
Packages are signed with a GPG key stored in GitHub Secrets.
