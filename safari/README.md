# FGP Safari Daemon

## Doctrine

See [DOCTRINE.md](./DOCTRINE.md).


Fast Safari gateway for macOS - direct SQLite access to browser history, cloud tabs, and bookmarks.

## Performance

| Operation | MCP (est.) | FGP | Speedup |
|-----------|-----------|-----|---------|
| Cold-start CLI | ~2,300ms | 26ms | **~90x** |
| Daemon mode | ~2,300ms | ~5ms | **~460x** |

## Features

- **History**: Recent browsing history with timestamps and visit counts
- **Search**: Full-text search across URLs and page titles
- **Top Sites**: Most visited sites with visit counts
- **Stats**: Browsing statistics (visits, unique pages, active days)
- **Cloud Tabs**: Tabs synced from iPhone/iPad via iCloud
- **Bundle**: Combine multiple queries in single request

## Installation

```bash
# Build from source
cd safari && cargo build --release

# Copy to PATH
cp target/release/fgp-safari ~/.local/bin/
```

## Usage

### CLI Commands

```bash
# Get recent history (last 7 days, 50 items)
fgp-safari history

# Search history
fgp-safari search "github" --days 30 --limit 20

# Top visited sites
fgp-safari top-sites --days 7 --limit 10

# Browsing stats
fgp-safari stats --days 30

# iCloud tabs from other devices
fgp-safari cloud-tabs

# JSON output
fgp-safari history --json

# Health check
fgp-safari health

# List available methods
fgp-safari methods
```

### Daemon Mode

```bash
# Start daemon (foreground)
fgp-safari start

# Or as background daemon
fgp-safari-daemon

# Stop daemon
fgp-safari stop
```

### FGP Protocol

Once running as daemon, query via socket:

```bash
echo '{"id":"1","v":1,"method":"safari.history","params":{"days":7,"limit":20}}' | \
  nc -U ~/.fgp/services/safari/daemon.sock
```

## Available Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `safari.history` | Recent browsing history | `days`, `limit` |
| `safari.search` | Search by URL/title | `query`*, `days`, `limit` |
| `safari.top_sites` | Most visited sites | `days`, `limit` |
| `safari.stats` | Browsing statistics | `days` |
| `safari.cloud_tabs` | iCloud synced tabs | - |
| `safari.bundle` | Combine queries | `include` |

## Permissions

**NO Full Disk Access required** - Safari databases are accessible with standard file permissions:

- `~/Library/Safari/History.db` - Browser history
- `~/Library/Safari/CloudTabs.db` - iCloud synced tabs (optional)
- `~/Library/Safari/Bookmarks.plist` - Bookmarks (planned)

## Database Schema

### History.db

```sql
-- Main tables
history_items (id, url, visit_count, domain_expansion)
history_visits (id, history_item, visit_time, title)

-- Timestamps: Core Data format (seconds since 2001-01-01)
```

### CloudTabs.db

```sql
cloud_tabs (tab_uuid, device_uuid, title, url, is_pinned)
cloud_tab_devices (device_uuid, device_name)
```

## Development

```bash
# Build debug
cargo build

# Build release (optimized)
cargo build --release

# Run tests
cargo test
```

## License

MIT
