# Security Policy

## Supported Versions

We actively support the following versions of odoo-rust-mcp with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| 0.2.x   | :white_check_mark: |
| < 0.2   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security vulnerability, please follow these steps:

### 1. **Do NOT** create a public GitHub issue

Security vulnerabilities should be reported privately to prevent exploitation.

### 2. Report via GitHub Security Advisories (Recommended)

1. Go to [https://github.com/rachmataditiya/odoo-rust-mcp/security/advisories/new](https://github.com/rachmataditiya/odoo-rust-mcp/security/advisories/new)
2. Click "Report a vulnerability"
3. Fill out the security advisory form with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if you have one)

### 3. Alternative: Email (if GitHub is not accessible)

If you cannot access GitHub Security Advisories, you can email the repository owner directly. Please include:
- A clear description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Your contact information

### 4. What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Initial Assessment**: We will provide an initial assessment within 7 days
- **Updates**: We will keep you informed of our progress
- **Resolution**: We will work to resolve the issue as quickly as possible
- **Disclosure**: We will coordinate with you on public disclosure timing

## Security Best Practices

### For Users

1. **Keep your installation updated**: Always use the latest stable version
2. **Secure your API keys**: Never commit API keys or passwords to version control
3. **Use environment variables**: Store sensitive credentials in environment variables, not in config files
4. **Restrict network access**: If running in HTTP mode, use authentication tokens (`MCP_AUTH_TOKEN`)
5. **Use minimal permissions**: Create dedicated Odoo users with minimal required access rights
6. **Monitor logs**: Regularly check logs for suspicious activity

### For Developers

1. **Never commit secrets**: Use `.gitignore` to exclude `.env` files and credentials
2. **Validate input**: Always validate and sanitize user input
3. **Use secure defaults**: Prefer secure configurations by default
4. **Keep dependencies updated**: Regularly update dependencies to patch known vulnerabilities
5. **Follow Rust security best practices**: Use `cargo audit` to check for vulnerable dependencies

## Security Checklist for Contributions

When contributing code, please ensure:

- [ ] No hardcoded credentials or API keys
- [ ] Input validation for all user-provided data
- [ ] Proper error handling (no sensitive information in error messages)
- [ ] Dependencies are up-to-date and free of known vulnerabilities
- [ ] Authentication/authorization is properly implemented
- [ ] No SQL injection or similar vulnerabilities (if applicable)
- [ ] Secure communication (HTTPS/TLS) for network operations
- [ ] Proper handling of sensitive data in logs

## Known Security Considerations

### API Key Storage

- API keys are stored in environment variables or configuration files
- Never commit API keys to version control
- Use secure file permissions for configuration files (e.g., `chmod 600`)

### Network Security

- HTTP transport supports Bearer token authentication via `MCP_AUTH_TOKEN`
- Always use authentication tokens in production environments
- Consider using HTTPS/TLS for production deployments

### Odoo Integration

- Use dedicated bot users with minimal required permissions
- Regularly audit Odoo user permissions
- Monitor Odoo access logs for suspicious activity

### Configuration Files

- Configuration files may contain sensitive information
- Ensure proper file permissions on configuration directories
- Use secure deployment methods (Kubernetes secrets, Docker secrets, etc.)

## Dependency Security

We use `cargo audit` to check for vulnerable dependencies. If you discover a vulnerability in a dependency:

1. Report it via GitHub Security Advisories
2. Include the dependency name and version
3. Reference the CVE or security advisory if available

## Security Updates

Security updates will be:
- Released as patch versions (e.g., 0.3.3 → 0.3.4)
- Documented in release notes
- Tagged with security labels on GitHub
- Backported to supported versions when possible

## Responsible Disclosure

We follow responsible disclosure practices:

1. **Private reporting**: Vulnerabilities are reported privately
2. **Timely fixes**: We work to fix issues promptly
3. **Coordinated disclosure**: We coordinate public disclosure with the reporter
4. **Credit**: We credit security researchers (with permission) in release notes

## Security Resources

- [Rust Security Advisory Database](https://rustsec.org/)
- [Cargo Audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Secure Code Guidelines](https://anssi-fr.github.io/rust-guide/)

## Questions?

If you have questions about security that are not vulnerabilities, please:
- Open a [discussion](https://github.com/rachmataditiya/odoo-rust-mcp/discussions)
- Use the [question issue template](.github/ISSUE_TEMPLATE/question.md)

Thank you for helping keep odoo-rust-mcp secure! 🔒
