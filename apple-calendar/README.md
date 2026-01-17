# FGP Apple Calendar

## Doctrine

See [DOCTRINE.md](./DOCTRINE.md).


[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast Apple Calendar daemon using native EventKit framework. Direct access to macOS calendar data without spawning subprocesses.

## Why?

MCP stdio tools spawn a new process for every call (~2.3s overhead). FGP Apple Calendar uses native EventKit bindings via Rust's objc2 crate, keeping the daemon warm and ready:

| Operation | FGP | MCP Estimate | Speedup |
|-----------|-----|--------------|---------|
| List calendars | **5ms** | ~2,300ms | **460x** |
| Get today's events | **8ms** | ~2,300ms | **288x** |
| Search events | **12ms** | ~2,300ms | **192x** |

## Requirements

- macOS 12.0+
- Calendar access permission (granted on first run)
- Rust 1.70+ (for building)

## Installation

```bash
# Clone and build
git clone https://github.com/fast-gateway-protocol/apple-calendar.git
cd apple-calendar
cargo build --release

# Add to PATH (optional)
cp target/release/fgp-apple-calendar ~/.local/bin/
```

## Quick Start

```bash
# Check authorization status
fgp-apple-calendar auth

# List all calendars
fgp-apple-calendar calendars

# Get today's events
fgp-apple-calendar today

# Get events for a specific date
fgp-apple-calendar events --date 2026-01-20

# Get events for a date range
fgp-apple-calendar range --start 2026-01-15 --end 2026-01-22

# Search events by title
fgp-apple-calendar search --query "meeting"

# Get upcoming events
fgp-apple-calendar upcoming --days 7
```

## Available Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `calendars` | List all calendars | - |
| `today` | Get today's events | - |
| `events` | Get events for a date | `date` (YYYY-MM-DD) |
| `range` | Get events in date range | `start`, `end` |
| `search` | Search events by title | `query`, `days` (optional) |
| `upcoming` | Get upcoming events | `days` (default: 7) |
| `auth` | Check calendar access status | - |

## Daemon Mode

```bash
# Start the daemon
fgp-apple-calendar-daemon

# Query via socket
echo '{"id":"1","v":1,"method":"apple-calendar.today","params":{}}' | \
  nc -U ~/.fgp/services/apple-calendar/daemon.sock
```

## Integration with Claude Code

Add to your Claude Code skill:

```yaml
dependencies:
  - fgp-apple-calendar
```

Then call methods via the FGP protocol:

```json
{"method": "apple-calendar.today", "params": {}}
```

## Troubleshooting

**"Calendar access not granted"**
- Open System Settings > Privacy & Security > Calendars
- Enable access for the terminal/IDE running the daemon

**"No calendars found"**
- Ensure you have at least one calendar configured in Calendar.app
- iCloud calendars require being signed in

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug ./target/release/fgp-apple-calendar today
```

## License

MIT - see [LICENSE](LICENSE)

## Related

- [FGP Daemon SDK](https://github.com/fast-gateway-protocol/daemon) - Build your own FGP daemons
- [FGP Protocol Spec](https://github.com/fast-gateway-protocol/protocol) - NDJSON over UNIX sockets
