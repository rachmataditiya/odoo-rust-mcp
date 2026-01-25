# Contributing to odoo-rust-mcp

Thank you for your interest in contributing to odoo-rust-mcp! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to see if the problem has already been reported. If it has, add a comment to the existing issue instead of creating a duplicate.

When creating a bug report, please include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Environment details (OS, Rust version, Odoo version, etc.)
- Relevant logs or error messages
- Any additional context that might be helpful

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) when creating a new issue.

### Suggesting Features

Feature suggestions are welcome! Please:

- Check existing issues to see if the feature has already been suggested
- Provide a clear description of the feature and its use case
- Explain why this feature would be beneficial
- Consider potential implementation approaches (if applicable)

Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md) when creating a new issue.

### Pull Requests

1. **Fork the repository** and create a branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make your changes** following the coding standards below

3. **Test your changes**:
   ```bash
   cd rust-mcp
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit your changes** with clear, descriptive commit messages:
   ```bash
   git commit -m "Add feature: description of what you did"
   ```

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a Pull Request**:
   - Use the [PR template](.github/pull_request_template.md)
   - Link to any related issues
   - Provide a clear description of changes
   - Ensure all CI checks pass

## Development Setup

### Prerequisites

- **Rust toolchain**: Install via [rustup](https://rustup.rs/)
  - Minimum Rust version: Check `rust-mcp/rust-toolchain.toml` or `rust-mcp/Cargo.toml`
- **Cargo**: Comes with Rust
- **Odoo instance**: For testing (Odoo 19+ or < 19)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/rachmataditiya/odoo-rust-mcp.git
cd odoo-rust-mcp

# Build the project
cd rust-mcp
cargo build

# Build in release mode
cargo build --release
```

### Running Tests

```bash
cd rust-mcp

# Run all tests
cargo test

# Run tests with warnings as errors
RUSTFLAGS='-Dwarnings' cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Running Linters

```bash
cd rust-mcp

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all
```

### Running the Server Locally

**Stdio transport (for Cursor/Claude Desktop):**
```bash
cd rust-mcp
./target/release/rust-mcp --transport stdio
```

**HTTP transport:**
```bash
cd rust-mcp
./target/release/rust-mcp --transport http --listen 127.0.0.1:8787
```

**WebSocket transport:**
```bash
cd rust-mcp
./target/release/rust-mcp --transport ws --listen 127.0.0.1:8787
```

### Configuration

Set up your Odoo instance configuration:

```bash
# Create config directory
mkdir -p ~/.config/odoo-rust-mcp

# Create env file with your Odoo credentials
cat > ~/.config/odoo-rust-mcp/env <<EOF
ODOO_URL=http://localhost:8069
ODOO_DB=mydb
ODOO_API_KEY=your_api_key
EOF
```

For multi-instance setup, see the [README.md](README.md#configuration-environment-variables) for details.

## Coding Standards

### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code (follows rustfmt defaults)
- Run `cargo clippy` and fix all warnings
- Prefer explicit error handling with `Result<T, E>` over panics
- Use meaningful variable and function names
- Add documentation comments for public APIs:
  ```rust
  /// Brief description.
  ///
  /// Detailed description if needed.
  /// 
  /// # Examples
  /// 
  /// ```
  /// // example code
  /// ```
  pub fn my_function() -> Result<()> {
      // ...
  }
  ```

### Project Structure

- **`rust-mcp/src/`**: Main source code
  - `main.rs`: Entry point, CLI argument parsing
  - `mcp/`: MCP protocol implementation
  - `odoo/`: Odoo API client and configuration
  - `config_manager/`: Configuration server and UI
- **`rust-mcp/config/`**: Runtime configuration files (tools.json, prompts.json, server.json)
- **`rust-mcp/config-defaults/`**: Seed defaults embedded in binary
- **`config/`**: Example configuration files
- **`k8s/`**: Kubernetes manifests
- **`helm/`**: Helm charts
- **`.github/workflows/`**: CI/CD workflows

### Adding New Tools

Tools are defined declaratively in `tools.json`. To add a new tool:

1. **Add the tool definition** to `rust-mcp/config/tools.json`:
   ```json
   {
     "name": "odoo_my_new_tool",
     "description": "Description of what the tool does",
     "inputSchema": {
       "type": "object",
       "properties": {
         "instance": { "type": "string" },
         "model": { "type": "string" }
       },
       "required": ["instance", "model"]
     },
     "op": {
       "type": "my_operation_type",
       "map": {
         "instance": "instance",
         "model": "model"
       }
     }
   }
   ```

2. **Implement the operation** in `rust-mcp/src/odoo/`:
   - Add a new operation type handler
   - Ensure it follows the existing patterns

3. **Update seed defaults** in `rust-mcp/config-defaults/tools.json` (for new installations)

4. **Add tests** for the new tool

5. **Update documentation** in README.md if needed

**Important**: Avoid using `anyOf`, `oneOf`, `allOf`, `$ref`, `definitions`, or type arrays in JSON Schema, as Cursor rejects these.

### Adding New Prompts

Prompts are defined in `prompts.json`:

1. Add the prompt to `rust-mcp/config/prompts.json`
2. Update `rust-mcp/config-defaults/prompts.json` for seed defaults
3. Document the prompt's purpose

### Commit Message Guidelines

- Use clear, descriptive commit messages
- Start with a verb in imperative mood: "Add", "Fix", "Update", "Remove", etc.
- Keep the first line under 72 characters
- Add a blank line between the subject and body
- Reference issues in the body: "Fixes #123" or "Closes #456"

Examples:
```
Add support for Odoo 19 JSON-2 API

Implements authentication via API keys and uses the new /json/2/ endpoint
for Odoo 19+ instances. Falls back to JSON-RPC for older versions.

Fixes #42
```

```
Fix config server port default

Changes default port from 3000 to 3008 (inspired by Peugeot 3008).
Updates all documentation and deployment configs accordingly.
```

## Testing Guidelines

### Unit Tests

- Write unit tests for all new functions and modules
- Test both success and error cases
- Use descriptive test names: `test_should_return_error_when_invalid_input()`

### Integration Tests

- Test end-to-end workflows when possible
- Use the smoke test client for WebSocket testing:
  ```bash
  cargo run --release --bin ws_smoke_client -- \
    --url ws://127.0.0.1:8787 \
    --instance default \
    --model res.partner
  ```

### Manual Testing

- Test with both Odoo 19+ (API key) and Odoo < 19 (username/password)
- Test multi-instance configurations
- Verify configuration server UI functionality
- Test all transport modes (stdio, HTTP, WebSocket, SSE)

## Documentation

### Code Documentation

- Document all public APIs with doc comments
- Include examples in doc comments where helpful
- Keep documentation up-to-date with code changes

### README Updates

- Update README.md when adding new features
- Keep installation instructions current
- Update configuration examples when adding new options
- Maintain accuracy of all code examples

### Configuration Documentation

- Document new environment variables
- Update example configuration files
- Explain new tool options in `tools.json`

## Review Process

1. All pull requests require review before merging
2. Maintainers will review code quality, tests, and documentation
3. Address review feedback promptly
4. Ensure all CI checks pass before requesting re-review

## Release Process

Releases are managed through GitHub Actions:

1. Version is bumped in `rust-mcp/Cargo.toml`
2. Homebrew formula is updated in `homebrew/Formula/rust-mcp.rb`
3. Git tag is created and pushed
4. GitHub Actions builds and releases binaries
5. Homebrew tap is updated automatically

Contributors don't need to handle releases; maintainers will manage this process.

## Getting Help

- **Questions?** Open a [discussion](https://github.com/rachmataditiya/odoo-rust-mcp/discussions) or use the [question issue template](.github/ISSUE_TEMPLATE/question.md)
- **Found a bug?** Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- **Security issue?** See [SECURITY.md](.github/SECURITY.md)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (AGPL-3.0).

Thank you for contributing to odoo-rust-mcp! 🚗
