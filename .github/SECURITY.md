# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

1. **Do NOT** open a public issue for security vulnerabilities
2. Use [GitHub Security Advisories](https://github.com/fast-gateway-protocol/fgp/security/advisories/new) to report privately
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes

### What to Expect

- **Acknowledgment**: We will acknowledge receipt within 48 hours
- **Assessment**: We will assess the vulnerability and determine severity
- **Timeline**: We aim to address critical vulnerabilities within 7 days
- **Disclosure**: We will coordinate disclosure timing with you

### Scope

The following are in scope for security reports:

- FGP daemon implementations
- Protocol vulnerabilities
- Authentication/authorization issues
- Data exposure risks
- Socket permission issues

### Out of Scope

- Issues in dependencies (report to upstream maintainers)
- Social engineering attacks
- Physical security issues
- Issues requiring unlikely user interaction

## Security Best Practices

When using FGP daemons:

1. **Socket Permissions**: FGP uses UNIX sockets with file-based permissions. Ensure socket directories have appropriate permissions (700)
2. **API Keys**: Store API keys in environment variables, not in code
3. **Updates**: Keep daemons updated to the latest versions
4. **Isolation**: Run daemons with minimal required permissions

## Security Architecture

FGP daemons are designed with security in mind:

- **Local-only**: UNIX sockets prevent network exposure
- **No auth by default**: Relies on file system permissions
- **Process isolation**: Each daemon runs independently
- **No persistent storage**: Sensitive data not persisted to disk

Thank you for helping keep FGP secure.
