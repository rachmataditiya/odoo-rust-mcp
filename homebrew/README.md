# Homebrew Formula for odoo-rust-mcp

This folder contains the Homebrew formula source for `rust-mcp`.

The actual Homebrew tap is hosted at: https://github.com/rachmataditiya/homebrew-odoo-rust-mcp

## For Users

### Quick Install

```bash
brew tap rachmataditiya/odoo-rust-mcp
brew install rust-mcp
```

### After Installation

1. Edit your Odoo credentials:
   ```bash
   nano ~/.config/odoo-rust-mcp/env
   ```

2. Start the service:
   ```bash
   brew services start rust-mcp
   ```

3. Service runs at: `http://127.0.0.1:8787/mcp`

### Service Authentication

**MCP HTTP Authentication:**

To enable Bearer token authentication for the HTTP service, you can either:

**Option 1: Via Config UI (Recommended)**
1. Start the service: `brew services start rust-mcp`
2. Open `http://localhost:3008` and login (default: `admin` / `changeme`)
3. Go to **Security** tab
4. Toggle **Enable MCP HTTP Authentication**
5. Click **Generate New Token** (or paste existing token)
6. Changes apply immediately (no restart needed)

**Option 2: Via Environment Variables**
Add to your `~/.config/odoo-rust-mcp/env`:

```bash
# Generate token: openssl rand -hex 32
MCP_AUTH_ENABLED=true
MCP_AUTH_TOKEN=your-secure-random-token-here
```

Then restart the service:

```bash
brew services restart rust-mcp
```

**Config UI Authentication:**

The Config UI requires login credentials. Default credentials are:
- Username: `admin`
- Password: `changeme`

**IMPORTANT:** Change the default password immediately after first login:
1. Open `http://localhost:3008`
2. Login with default credentials
3. Go to **Security** tab
4. Update username and/or password

Credentials are stored in `~/.config/odoo-rust-mcp/env`:
```bash
CONFIG_UI_USERNAME=admin
CONFIG_UI_PASSWORD=your-secure-password
```

### Cursor Configuration

**STDIO (direct process):**

```json
{
  "mcpServers": {
    "odoo": {
      "command": "/opt/homebrew/bin/rust-mcp-service",
      "args": ["--transport", "stdio"]
    }
  }
}
```

**HTTP (service mode):**

```json
{
  "mcpServers": {
    "odoo": {
      "url": "http://127.0.0.1:8787/mcp"
    }
  }
}
```

**HTTP with Bearer Token:**

```json
{
  "mcpServers": {
    "odoo": {
      "url": "http://127.0.0.1:8787/mcp",
      "headers": {
        "Authorization": "Bearer your-secure-random-token-here"
      }
    }
  }
}
```

For complete documentation, see: https://github.com/rachmataditiya/homebrew-odoo-rust-mcp

## For Maintainers

### Formula Location

- Source: `homebrew/Formula/rust-mcp.rb` (this repo)
- Published: https://github.com/rachmataditiya/homebrew-odoo-rust-mcp

### Updating the Formula

1. Update `Formula/rust-mcp.rb` in this repo
2. Copy to homebrew-odoo-rust-mcp repo
3. Commit and push both repos

### Automated Updates

The GitHub Actions workflow (`.github/workflows/release.yml`) automatically:
1. Generates SHA256 checksums for release artifacts
2. Updates the formula with new version and checksums
3. Pushes to the homebrew tap repository

**Required GitHub Settings:**

Set these in repository Settings > Secrets and variables:

| Type | Name | Value |
|------|------|-------|
| Variable | `HOMEBREW_TAP_REPO` | `rachmataditiya/homebrew-odoo-rust-mcp` |
| Secret | `HOMEBREW_TAP_TOKEN` | Personal access token with `repo` scope |

### Manual Checksum Generation

After creating a release, generate checksums:

```bash
VERSION=0.1.0
curl -sL "https://github.com/rachmataditiya/odoo-rust-mcp/releases/download/v${VERSION}/rust-mcp-aarch64-apple-darwin.tar.gz" | shasum -a 256
curl -sL "https://github.com/rachmataditiya/odoo-rust-mcp/releases/download/v${VERSION}/rust-mcp-x86_64-apple-darwin.tar.gz" | shasum -a 256
curl -sL "https://github.com/rachmataditiya/odoo-rust-mcp/releases/download/v${VERSION}/rust-mcp-x86_64-unknown-linux-gnu.tar.gz" | shasum -a 256
```
