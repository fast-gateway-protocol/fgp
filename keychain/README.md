# FGP Keychain

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast macOS Keychain daemon using the native Security framework. Secure password storage and retrieval without spawning `security` CLI subprocesses.

## Why?

The `security` CLI spawns a new process for every call (~100-300ms overhead). FGP Keychain uses Rust's security-framework crate for direct API access:

| Operation | FGP | security CLI | Speedup |
|-----------|-----|--------------|---------|
| Find password | **3ms** | ~150ms | **50x** |
| Set password | **5ms** | ~200ms | **40x** |
| Delete password | **3ms** | ~150ms | **50x** |

## Requirements

- macOS 10.15+
- **Code signing required** - The binary must be signed to access Keychain APIs
- Rust 1.70+ (for building)

## Installation

```bash
# Clone and build
git clone https://github.com/fast-gateway-protocol/keychain.git
cd keychain
cargo build --release

# IMPORTANT: Sign the binaries
codesign -s - ./target/release/fgp-keychain
codesign -s - ./target/release/fgp-keychain-daemon

# Add to PATH (optional)
cp target/release/fgp-keychain ~/.local/bin/
```

## Quick Start

```bash
# Check keychain access status
fgp-keychain auth

# Store a password
fgp-keychain set-generic --service "myapp" --account "user@example.com" --password "secret123"

# Retrieve a password
fgp-keychain find-generic --service "myapp" --account "user@example.com"

# Check if password exists
fgp-keychain exists --service "myapp" --account "user@example.com"

# Delete a password
fgp-keychain delete --service "myapp" --account "user@example.com"
```

## Available Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `find_generic` | Find a generic password | `service`, `account` |
| `set_generic` | Store/update a password | `service`, `account`, `password` |
| `delete` | Delete a password | `service`, `account` |
| `exists` | Check if password exists | `service`, `account` |
| `auth` | Check keychain access status | - |

## Daemon Mode

```bash
# Start the daemon (must be signed first!)
fgp-keychain-daemon

# Query via socket
echo '{"id":"1","v":1,"method":"keychain.find_generic","params":{"service":"myapp","account":"user"}}' | \
  nc -U ~/.fgp/services/keychain/daemon.sock
```

## Security Considerations

- **Code signing is mandatory** - Unsigned binaries cannot access the Keychain
- **First access prompts** - macOS will ask for permission on first access to each password
- **Click "Always Allow"** - To grant permanent access without repeated prompts
- **Passwords are never logged** - The daemon explicitly avoids logging sensitive data
- **No bulk export** - There's no "dump all passwords" method by design

## User Prompts

On first access to a specific password, macOS will display:

> "fgp-keychain wants to use your confidential information stored in [item] in your keychain."

Options:
- **Deny** - Block access
- **Allow** - Allow this once
- **Always Allow** - Grant permanent access (recommended for automation)

## Troubleshooting

**"Keychain access not available"**
- Ensure the binary is code-signed: `codesign -dv ./target/release/fgp-keychain`
- Re-sign if needed: `codesign -s - ./target/release/fgp-keychain`

**"Password not found"**
- Verify the exact service and account names
- Passwords are case-sensitive
- Check Keychain Access.app to verify the entry exists

**"User interaction not allowed"**
- The daemon is running without a user session
- Keychain requires a logged-in user context
- Don't run as a system daemon or via SSH without proper setup

## Development

```bash
# Build
cargo build --release

# Sign for development
codesign -s - ./target/release/fgp-keychain

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug ./target/release/fgp-keychain auth
```

## License

MIT - see [LICENSE](LICENSE)

## Related

- [FGP Daemon SDK](https://github.com/fast-gateway-protocol/daemon) - Build your own FGP daemons
- [security-framework](https://crates.io/crates/security-framework) - Rust bindings to macOS Security framework
