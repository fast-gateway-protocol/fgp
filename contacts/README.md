# FGP Contacts Daemon

Fast Contacts gateway for macOS - direct SQLite access to AddressBook database.

## Performance

| Operation | MCP (est.) | FGP | Speedup |
|-----------|-----------|-----|---------|
| Cold-start CLI | ~2,300ms | 13ms | **~180x** |
| Daemon mode | ~2,300ms | ~3ms | **~770x** |

## Features

- **List**: Get all contacts with emails and phone numbers
- **Search**: Fuzzy search contacts by name
- **By Email**: Lookup contact by email address
- **By Phone**: Lookup contact by phone number (matches last 4 digits)
- **Recent**: Recently modified contacts
- **Stats**: Contact database statistics

## Permissions

**Requires Full Disk Access** to read the AddressBook database.

To grant access:
1. System Settings → Privacy & Security → Full Disk Access
2. Add your terminal application (Terminal.app, iTerm2, etc.)

**Database location:** `~/Library/Application Support/AddressBook/AddressBook-v22.abcddb`

## Installation

```bash
# Build from source
cd contacts && cargo build --release

# Copy to PATH
cp target/release/fgp-contacts ~/.local/bin/
```

## Usage

### CLI Commands

```bash
# List all contacts
fgp-contacts list

# Search by name
fgp-contacts search "john" --limit 10

# Find by email
fgp-contacts by-email "john@example.com"

# Find by phone
fgp-contacts by-phone "555-1234"

# Recently modified contacts
fgp-contacts recent --days 7

# Statistics
fgp-contacts stats

# JSON output
fgp-contacts list --json

# Health check
fgp-contacts health
```

### Daemon Mode

```bash
# Start daemon (foreground)
fgp-contacts start

# Or as background daemon
fgp-contacts-daemon

# Stop daemon
fgp-contacts stop
```

### FGP Protocol

Once running as daemon, query via socket:

```bash
echo '{"id":"1","v":1,"method":"contacts.search","params":{"query":"john"}}' | \
  nc -U ~/.fgp/services/contacts/daemon.sock
```

## Available Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `contacts.list` | List all contacts | `limit` |
| `contacts.search` | Search by name | `query`*, `limit` |
| `contacts.by_email` | Find by email | `email`* |
| `contacts.by_phone` | Find by phone | `phone`* |
| `contacts.recent` | Recently modified | `days`, `limit` |
| `contacts.stats` | Database statistics | - |

## Database Schema

Core Data SQLite database with key tables:

```sql
-- Main contact record
ZABCDRECORD (Z_PK, ZFIRSTNAME, ZLASTNAME, ZORGANIZATION, ...)

-- Email addresses (linked via ZOWNER)
ZABCDEMAILADDRESS (Z_PK, ZOWNER, ZADDRESS, ZLABEL, ZISPRIMARY)

-- Phone numbers (linked via ZOWNER)
ZABCDPHONENUMBER (Z_PK, ZOWNER, ZFULLNUMBER, ZLABEL, ZISPRIMARY)
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
