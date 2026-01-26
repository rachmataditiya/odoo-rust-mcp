#!/bin/bash
# Complete release script: bump version, commit, push, and create tag

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <new-version>"
    echo "Example: $0 0.3.15"
    exit 1
fi

NEW_VERSION="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Run bump-version script
echo "Step 1: Bumping version..."
"$SCRIPT_DIR/bump-version.sh" "$NEW_VERSION"

if [ $? -ne 0 ]; then
    echo "Error: Version bump failed"
    exit 1
fi

# Commit changes
echo ""
echo "Step 2: Committing changes..."
git add rust-mcp/Cargo.toml config-ui/package.json

if ! git diff-index --quiet HEAD --; then
    git commit -m "chore: bump version to $NEW_VERSION"
else
    echo "Warning: No changes to commit"
fi

# Push to remote
echo ""
echo "Step 3: Pushing to remote..."
if ! git push; then
    echo "Error: Push failed"
    exit 1
fi

# Create and push tag
echo ""
echo "Step 4: Creating tag v$NEW_VERSION..."
if git tag "v$NEW_VERSION" 2>/dev/null; then
    if git push origin "v$NEW_VERSION"; then
        echo "✓ Tag pushed successfully"
    else
        echo "Error: Failed to push tag"
        exit 1
    fi
else
    echo "Error: Failed to create tag (might already exist)"
    exit 1
fi

echo ""
echo "================================"
echo "✓ Release v$NEW_VERSION created successfully!"
echo "================================"
echo ""
echo "GitHub Actions will now:"
echo "  - Build binaries for all platforms"
echo "  - Build Docker image"
echo "  - Build Debian package"
echo "  - Create GitHub release"
echo "  - Update Homebrew formula"
echo "  - Update APT repository"
echo ""
echo "Monitor progress:"
echo "  https://github.com/rachmataditiya/odoo-rust-mcp/actions"
echo """
