# FGP Screen Time

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast macOS Screen Time daemon via SQLite queries to `knowledgeC.db`. Access app usage analytics without Apple's Swift-only frameworks.

## Why?

Apple's Screen Time frameworks (DeviceActivity, ManagedSettings) are Swift-only and inaccessible from Rust. This daemon reads directly from the `knowledgeC.db` SQLite database:

| Operation | FGP | Alternative | Notes |
|-----------|-----|-------------|-------|
| Daily total | **8ms** | N/A | No public API exists |
| App usage | **12ms** | N/A | Direct DB access |
| Weekly summary | **50ms** | N/A | Aggregated query |

## Requirements

- macOS 12.0+
- **Full Disk Access** - Required to read `knowledgeC.db`
- Rust 1.70+ (for building)

## Installation

```bash
# Clone and build
git clone https://github.com/fast-gateway-protocol/screen-time.git
cd screen-time
cargo build --release

# Add to PATH (optional)
cp target/release/fgp-screen-time ~/.local/bin/
```

### Granting Full Disk Access

1. Open **System Settings** > **Privacy & Security** > **Full Disk Access**
2. Click the **+** button
3. Navigate to and add your terminal app (Terminal.app, iTerm, VS Code, etc.)
4. Restart the terminal

## Quick Start

```bash
# Check access status
fgp-screen-time auth

# Get today's screen time with app breakdown
fgp-screen-time daily-total

# Get screen time for a specific date
fgp-screen-time daily-total --date 2026-01-14

# Get top 5 most used apps (last 7 days)
fgp-screen-time most-used --limit 5

# Get usage for a specific app
fgp-screen-time app-usage --bundle-id com.google.Chrome

# Get 7-day weekly summary
fgp-screen-time weekly-summary

# Get hourly timeline for today
fgp-screen-time timeline
```

## Available Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `daily_total` | Total screen time for a day | `date` (optional, default: today) |
| `app_usage` | Usage for specific bundle ID | `bundle_id`, `days` (default: 7) |
| `weekly_summary` | 7-day summary with daily breakdown | - |
| `most_used` | Top N apps by usage | `limit` (default: 10), `days` (default: 7) |
| `usage_timeline` | Hourly breakdown | `date` (optional, default: today) |
| `auth` | Check Full Disk Access status | - |

## Example Output

### Daily Total
```json
{
  "date": "2026-01-15",
  "total_seconds": 28800,
  "total_formatted": "8h 0m",
  "breakdown": [
    {"bundle_id": "com.mitchellh.ghostty", "total_seconds": 14400, "total_formatted": "4h 0m"},
    {"bundle_id": "com.google.Chrome", "total_seconds": 10800, "total_formatted": "3h 0m"}
  ]
}
```

### Weekly Summary
```json
{
  "total_seconds": 201600,
  "total_formatted": "56h 0m",
  "daily_average_seconds": 28800,
  "days": [...]
}
```

## Daemon Mode

```bash
# Start the daemon
fgp-screen-time-daemon

# Query via socket
echo '{"id":"1","v":1,"method":"screen-time.daily_total","params":{}}' | \
  nc -U ~/.fgp/services/screen-time/daemon.sock
```

## Technical Details

### Database Location
```
~/Library/Application Support/Knowledge/knowledgeC.db
```

### Data Source
- Table: `ZOBJECT`
- Stream: `/app/usage`
- Timestamps: Mac Absolute Time (2001-01-01 epoch)

### What's Available
- App usage by bundle ID
- Session start/end times
- Total usage aggregation
- Historical data (retained by macOS)

### What's NOT Available
- App categories (not stored in knowledgeC.db)
- Website usage details
- Setting/modifying Screen Time limits
- Downtime schedules

## Troubleshooting

**"Screen Time data not accessible"**
- Grant Full Disk Access to your terminal in System Settings
- Restart the terminal after granting access

**"No data returned"**
- Screen Time must be enabled in System Settings
- Data collection takes time; new installs have limited history

**"Permission denied on database"**
- The database is read-only; this is expected
- Ensure Full Disk Access is granted

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# Test database access directly
sqlite3 "$HOME/Library/Application Support/Knowledge/knowledgeC.db" \
  "SELECT COUNT(*) FROM ZOBJECT WHERE ZSTREAMNAME = '/app/usage'"

# Run with debug logging
RUST_LOG=debug ./target/release/fgp-screen-time daily-total
```

## License

MIT - see [LICENSE](LICENSE)

## Related

- [FGP Daemon SDK](https://github.com/fast-gateway-protocol/daemon) - Build your own FGP daemons
- [knowledgeC.db Forensics](http://www.mac4n6.com/blog/2018/8/5/knowledge-is-power) - Database schema documentation
