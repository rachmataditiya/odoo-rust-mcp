# GPG Key Setup for APT Repository

This guide explains how to generate and configure GPG keys for signing the APT repository.

## Prerequisites

Install GnuPG:

```bash
# macOS
brew install gnupg

# Ubuntu/Debian
sudo apt-get install gnupg
```

## Generate GPG Key

```bash
# Generate a new GPG key (no passphrase for automation)
gpg --batch --gen-key <<EOF
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: Odoo Rust MCP
Name-Email: apt@odoo-rust-mcp.local
Expire-Date: 0
%no-protection
%commit
EOF

# List keys to verify
gpg --list-keys
```

## Export Keys

```bash
# Export public key (for users to import)
gpg --armor --export "Odoo Rust MCP" > apt-repo/pubkey.gpg

# Export private key (for GitHub Secrets)
gpg --armor --export-secret-keys "Odoo Rust MCP" | base64 > /tmp/private-key-base64.txt

# View the base64 encoded private key
cat /tmp/private-key-base64.txt
```

## Configure GitHub Secrets

1. Go to your repository Settings > Secrets and variables > Actions
2. Add the following secrets:

| Secret Name | Value |
|------------|-------|
| `APT_GPG_PRIVATE_KEY` | Contents of `/tmp/private-key-base64.txt` |
| `APT_GPG_PASSPHRASE` | (empty if no passphrase was set) |

3. Add the following repository variable:

| Variable Name | Value |
|--------------|-------|
| `APT_REPO_ENABLED` | `true` |

## Test Signing

```bash
# Test signing a file
echo "test" > /tmp/test.txt
gpg --armor --detach-sign /tmp/test.txt

# Verify signature
gpg --verify /tmp/test.txt.asc
```

## Commit Public Key

After generating and exporting the keys:

```bash
git add apt-repo/pubkey.gpg
git commit -m "Add GPG public key for APT repository"
git push
```

## Security Notes

- The private key should ONLY be stored in GitHub Secrets
- Never commit the private key to the repository
- The public key (`pubkey.gpg`) is safe to commit and share
- Consider using a dedicated GPG key only for APT signing
