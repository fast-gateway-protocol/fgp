# iMessage Daemon

Fast iMessage gateway for macOS using direct SQLite queries. **480x faster** than MCP stdio.

**Platform:** macOS only

## Installation

```bash
fgp install imessage
fgp start imessage
```

Or from source:

```bash
cd imessage && cargo build --release
./target/release/fgp-imessage-daemon start
```

## Requirements

- **macOS** with Messages.app
- **Full Disk Access** for Terminal (System Preferences → Privacy & Security → Full Disk Access)
- **Automation** permission for AppleScript message sending

## Methods

### Reading Messages

#### imessage.recent

Get recent messages across all contacts.

```json
{
  "method": "imessage.recent",
  "params": {
    "days": 7,
    "limit": 20
  }
}
```

**Response:**
```json
{
  "result": {
    "messages": [
      {
        "text": "Hey, are you free tomorrow?",
        "date": "2026-01-15T10:30:00Z",
        "is_from_me": false,
        "phone": "+14155551234",
        "contact_name": "John Doe"
      }
    ],
    "count": 20,
    "days": 7
  }
}
```

#### imessage.unread

Get unread messages.

```json
{
  "method": "imessage.unread",
  "params": {
    "limit": 50
  }
}
```

**Response:**
```json
{
  "result": {
    "unread_count": 3,
    "messages": [...]
  }
}
```

### Analytics

#### imessage.analytics

Get messaging statistics.

```json
{
  "method": "imessage.analytics",
  "params": {
    "contact": "John",
    "days": 30
  }
}
```

**Response:**
```json
{
  "result": {
    "period_days": 30,
    "total_messages": 4629,
    "sent_count": 1843,
    "received_count": 2786,
    "avg_per_day": 154.3,
    "busiest_hour": 14,
    "busiest_day": "Monday",
    "top_contacts": [...],
    "attachment_count": 45,
    "reaction_count": 89
  }
}
```

#### imessage.followup

Get items needing follow-up (unanswered questions, stale conversations).

```json
{
  "method": "imessage.followup",
  "params": {
    "days": 30,
    "stale": 3
  }
}
```

**Response:**
```json
{
  "result": {
    "unanswered_questions": [...],
    "stale_conversations": [...],
    "total_items": 5
  }
}
```

### Discovery

#### imessage.handles

List active phone/email handles from recent messages.

```json
{
  "method": "imessage.handles",
  "params": {
    "days": 30,
    "limit": 50
  }
}
```

#### imessage.unknown

List messages from senders not in contacts.

```json
{
  "method": "imessage.unknown",
  "params": {
    "days": 30,
    "limit": 20
  }
}
```

#### imessage.discover

Discover potential contacts from frequent unknown senders.

```json
{
  "method": "imessage.discover",
  "params": {
    "days": 90,
    "min_messages": 3
  }
}
```

### Dashboard

#### imessage.bundle

Bundle multiple queries for dashboard use (reduces round trips).

```json
{
  "method": "imessage.bundle",
  "params": {
    "include": "unread_count,recent,analytics,followup_count"
  }
}
```

**Response:**
```json
{
  "result": {
    "unread_count": 5,
    "recent": [...],
    "analytics": {...},
    "followup_count": 3
  }
}
```

## CLI Examples

```bash
# Recent messages
fgp-imessage recent --limit 10 --json

# Unread messages
fgp-imessage unread --json

# Messages with a contact
fgp-imessage messages "John" --limit 20 --json

# Send a message
fgp-imessage send "John" "Hey, want to grab lunch?"

# Analytics
fgp-imessage analytics --days 30 --json

# Follow-up items
fgp-imessage followup --json

# Bundle query via daemon
fgp-imessage-client bundle --params '{"include":"unread_count,recent,analytics"}'
```

## Performance

| Operation | FGP Daemon | MCP Stdio | Speedup |
|-----------|------------|-----------|---------|
| Recent messages | **8ms** | 2,300ms | **292x** |
| Unread messages | **10ms** | 2,300ms | **230x** |
| Analytics | **5ms** | 2,400ms | **480x** |
| Follow-up | **12ms** | 2,400ms | **200x** |
| Bundle query | **15ms** | N/A | Batched |

## Architecture

```
┌─────────────────┐     UNIX Socket      ┌──────────────────┐
│  AI Agent       │ ◄──────────────────► │  FGP iMessage    │
│  (Claude, etc.) │    NDJSON protocol   │  Daemon (Rust)   │
└─────────────────┘                      └────────┬─────────┘
                                                  │
                                    ┌─────────────┼─────────────┐
                                    │             │             │
                               ┌────▼────┐  ┌─────▼─────┐  ┌────▼────┐
                               │ chat.db │  │ Contacts  │  │Messages │
                               │ SQLite  │  │   Cache   │  │   App   │
                               └─────────┘  └───────────┘  └─────────┘
                                                            AppleScript
```

## Socket Location

`~/.fgp/services/imessage/daemon.sock`
