#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== Odoo MCP Agent Skills & Config Installer ===${NC}"
echo "Installing Odoo MCP skills for Claude Code and Antigravity CLI..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILL_SRC="$SCRIPT_DIR/skills/odoo-mcp/SKILL.md"

if [ ! -f "$SKILL_SRC" ]; then
  echo -e "${YELLOW}Warning: Source skill not found at $SKILL_SRC${NC}"
  exit 1
fi

# 1. Claude Code setup
CLAUDE_DIR="$HOME/.claude"
if [ -d "$CLAUDE_DIR" ] || command -v claude >/dev/null 2>&1; then
  echo -e "\n${BLUE}[1/2] Configuring Claude Code...${NC}"
  mkdir -p "$CLAUDE_DIR/skills/odoo-mcp"
  cp "$SKILL_SRC" "$CLAUDE_DIR/skills/odoo-mcp/SKILL.md"
  echo -e "${GREEN}✓ Installed skill to $CLAUDE_DIR/skills/odoo-mcp/SKILL.md${NC}"

  # Register MCP in ~/.claude/mcp.json if present or create it
  MCP_JSON="$CLAUDE_DIR/mcp.json"
  if [ ! -f "$MCP_JSON" ]; then
    cat > "$MCP_JSON" << 'EOF'
{
  "mcpServers": {
    "odoo": {
      "type": "http",
      "url": "http://127.0.0.1:8787/mcp"
    }
  }
}
EOF
    echo -e "${GREEN}✓ Created $MCP_JSON${NC}"
  fi
fi

# 2. Antigravity CLI (AGY) setup
GEMINI_DIR="$HOME/.gemini/config"
if [ -d "$GEMINI_DIR" ] || command -v agy >/dev/null 2>&1; then
  echo -e "\n${BLUE}[2/2] Configuring Antigravity CLI (AGY)...${NC}"
  mkdir -p "$GEMINI_DIR/skills/odoo-mcp"
  cp "$SKILL_SRC" "$GEMINI_DIR/skills/odoo-mcp/SKILL.md"
  echo -e "${GREEN}✓ Installed skill to $GEMINI_DIR/skills/odoo-mcp/SKILL.md${NC}"

  MCP_CONFIG="$GEMINI_DIR/mcp_config.json"
  if [ -f "$MCP_CONFIG" ]; then
    if ! grep -q '"odoo"' "$MCP_CONFIG"; then
      # Add odoo using python/node or jq if available
      python3 -c '
import json, sys
p = sys.argv[1]
with open(p, "r") as f:
    d = json.load(f)
if "mcpServers" not in d:
    d["mcpServers"] = {}
d["mcpServers"]["odoo"] = {"url": "http://127.0.0.1:8787/mcp"}
with open(p, "w") as f:
    json.dump(d, f, indent=2)
' "$MCP_CONFIG"
      echo -e "${GREEN}✓ Added odoo MCP server to $MCP_CONFIG${NC}"
    else
      echo -e "${GREEN}✓ Odoo MCP server already registered in $MCP_CONFIG${NC}"
    fi
  else
    mkdir -p "$GEMINI_DIR"
    cat > "$MCP_CONFIG" << 'EOF'
{
  "mcpServers": {
    "odoo": {
      "url": "http://127.0.0.1:8787/mcp"
    }
  }
}
EOF
    echo -e "${GREEN}✓ Created $MCP_CONFIG${NC}"
  fi
fi

echo -e "\n${GREEN}=== Installation Complete! ===${NC}"
echo "Claude Code and Antigravity now have full autonomous Odoo ERP capabilities and skills."
