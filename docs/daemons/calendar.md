# Calendar Daemon

Google Calendar integration via Google API.

## Installation

```bash
fgp install calendar
fgp start calendar
```

## Authentication

Uses OAuth2, same flow as Gmail daemon.

## Methods

### calendar.today

Get today's events.

```json
{
  "method": "calendar.today",
  "params": {}
}
```

### calendar.upcoming

Get upcoming events.

```json
{
  "method": "calendar.upcoming",
  "params": {
    "days": 7
  }
}
```

### calendar.search

Search for events.

```json
{
  "method": "calendar.search",
  "params": {
    "query": "meeting"
  }
}
```

### calendar.free_slots

Find free time slots.

```json
{
  "method": "calendar.free_slots",
  "params": {
    "duration_minutes": 30,
    "days": 3
  }
}
```

### calendar.create

Create a new event.

```json
{
  "method": "calendar.create",
  "params": {
    "title": "Team Meeting",
    "start": "2025-01-15T10:00:00",
    "end": "2025-01-15T11:00:00"
  }
}
```

## CLI Examples

```bash
# Today's schedule
fgp call calendar today

# Find free time
fgp call calendar free_slots --duration 30

# Create event
fgp call calendar create --title "Meeting" --start "2025-01-15T10:00:00"
```

## Status

**Beta** - Core operations work.
