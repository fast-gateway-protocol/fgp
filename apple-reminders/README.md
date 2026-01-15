# FGP Apple Reminders

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast Apple Reminders daemon using native EventKit framework. Direct access to macOS reminders without spawning subprocesses.

## Why?

MCP stdio tools spawn a new process for every call (~2.3s overhead). FGP Apple Reminders uses native EventKit bindings via Rust's objc2 crate:

| Operation | FGP | MCP Estimate | Speedup |
|-----------|-----|--------------|---------|
| List reminders | **15ms** | ~2,300ms | **153x** |
| Get incomplete | **12ms** | ~2,300ms | **192x** |
| Create reminder | **8ms** | ~2,300ms | **288x** |

## Requirements

- macOS 12.0+
- Reminders access permission (granted on first run)
- Rust 1.70+ (for building)

## Installation

```bash
# Clone and build
git clone https://github.com/fast-gateway-protocol/apple-reminders.git
cd apple-reminders
cargo build --release

# Add to PATH (optional)
cp target/release/fgp-apple-reminders ~/.local/bin/
```

## Quick Start

```bash
# Check authorization status
fgp-apple-reminders auth

# List all reminder lists
fgp-apple-reminders lists

# Get all reminders
fgp-apple-reminders all

# Get incomplete reminders
fgp-apple-reminders incomplete

# Get completed reminders
fgp-apple-reminders completed

# Get reminders due today
fgp-apple-reminders due-today

# Get overdue reminders
fgp-apple-reminders overdue

# Search reminders by title
fgp-apple-reminders search --query "groceries"

# Create a new reminder
fgp-apple-reminders create --title "Buy milk" --list "Shopping"
```

## Available Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `lists` | List all reminder lists | - |
| `all` | Get all reminders | `list` (optional) |
| `incomplete` | Get incomplete reminders | `list` (optional) |
| `completed` | Get completed reminders | `list` (optional) |
| `due_today` | Get reminders due today | - |
| `overdue` | Get overdue reminders | - |
| `search` | Search by title | `query` |
| `create` | Create a reminder | `title`, `list` (optional), `notes` (optional) |
| `complete` | Mark reminder complete | `id` |
| `auth` | Check reminders access | - |

## Daemon Mode

```bash
# Start the daemon
fgp-apple-reminders-daemon

# Query via socket
echo '{"id":"1","v":1,"method":"apple-reminders.incomplete","params":{}}' | \
  nc -U ~/.fgp/services/apple-reminders/daemon.sock
```

## Integration with Claude Code

Add to your Claude Code skill:

```yaml
dependencies:
  - fgp-apple-reminders
```

Then call methods via the FGP protocol:

```json
{"method": "apple-reminders.incomplete", "params": {}}
```

## Troubleshooting

**"Reminders access not granted"**
- Open System Settings > Privacy & Security > Reminders
- Enable access for the terminal/IDE running the daemon

**"No reminder lists found"**
- Ensure you have at least one list in Reminders.app
- iCloud lists require being signed in

**"Reminder fetch timeout"**
- EventKit fetches are async; very large lists may take longer
- The daemon has a 30-second timeout for reminder fetches

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug ./target/release/fgp-apple-reminders incomplete
```

## License

MIT - see [LICENSE](LICENSE)

## Related

- [FGP Daemon SDK](https://github.com/fast-gateway-protocol/daemon) - Build your own FGP daemons
- [FGP Apple Calendar](https://github.com/fast-gateway-protocol/apple-calendar) - Calendar daemon using same EventKit
