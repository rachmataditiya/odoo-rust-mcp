#!/bin/bash
# Script to bump version across all files before release

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <new-version>"
    echo "Example: $0 0.3.15"
    exit 1
fi

NEW_VERSION="$1"
OLD_VERSION=$(grep -E '^version = ' rust-mcp/Cargo.toml | cut -d'"' -f2)

if [ -z "$OLD_VERSION" ]; then
    echo "Error: Could not find current version in rust-mcp/Cargo.toml"
    exit 1
fi

echo "Bumping version from $OLD_VERSION to $NEW_VERSION"

# Determine sed command based on OS (Linux vs BSD/macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" rust-mcp/Cargo.toml
    sed -i '' "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/" config-ui/package.json
else
    # Linux
    sed -i "s/^version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" rust-mcp/Cargo.toml
    sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/" config-ui/package.json
fi

if [ $? -ne 0 ]; then
    echo "Error: Failed to update version files"
    exit 1
fi

echo "✓ Updated rust-mcp/Cargo.toml"
echo "✓ Updated config-ui/package.json"

# Show changes
echo ""
echo "Version changes:"
git diff rust-mcp/Cargo.toml config-ui/package.json

echo ""
echo "✓ Version bump completed successfully!"
