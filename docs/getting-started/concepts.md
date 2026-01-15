# Core Concepts

## Daemons

A **daemon** is a long-running process that:

- Listens on a UNIX socket
- Accepts NDJSON requests
- Returns NDJSON responses
- Manages connections to external services

Daemons stay warm between requests, eliminating cold-start overhead.

## Protocol

FGP uses **NDJSON over UNIX sockets**:

- **NDJSON**: Newline-delimited JSON - one JSON object per line
- **UNIX sockets**: Local inter-process communication (no network overhead)

### Request Format

```json
{"id": "uuid", "v": 1, "method": "service.action", "params": {...}}
```

| Field | Description |
|-------|-------------|
| `id` | Unique request identifier |
| `v` | Protocol version (currently 1) |
| `method` | Action to perform |
| `params` | Method-specific parameters |

### Response Format

```json
{"id": "uuid", "ok": true, "result": {...}, "meta": {"server_ms": 12.5}}
```

| Field | Description |
|-------|-------------|
| `id` | Matches request ID |
| `ok` | Success indicator |
| `result` | Method result (if ok=true) |
| `error` | Error message (if ok=false) |
| `meta` | Timing and debug info |

## Services

Each daemon provides a **service** with multiple **methods**:

```
browser.open       - Navigate to URL
browser.click      - Click element
browser.snapshot   - Get ARIA tree
gmail.list         - List emails
gmail.send         - Send email
```

## Sessions

Some daemons support **sessions** for isolated contexts:

- **Browser**: Separate cookies, localStorage, history
- **Gmail**: Different accounts

Sessions are identified by name and created on demand.

## Socket Locations

Daemons create sockets at:

```
~/.fgp/services/<daemon-name>/daemon.sock
```

Examples:
- `~/.fgp/services/browser/daemon.sock`
- `~/.fgp/services/gmail/daemon.sock`

## Built-in Methods

All daemons support these methods:

| Method | Description |
|--------|-------------|
| `health` | Check daemon status |
| `methods` | List available methods |
| `stop` | Graceful shutdown |

## Lifecycle

```
1. Start daemon     →  fgp start browser
2. Daemon listens   →  ~/.fgp/services/browser/daemon.sock
3. Agent connects   →  UNIX socket connection
4. Send requests    →  {"method": "browser.open", ...}
5. Receive results  →  {"ok": true, "result": {...}}
6. Stop daemon      →  fgp stop browser
```

## Comparison with MCP

| Aspect | MCP stdio | FGP |
|--------|-----------|-----|
| Process model | New process per call | Persistent daemon |
| Startup time | ~2.3s | ~1ms |
| Connection | stdin/stdout | UNIX socket |
| State | Stateless | Stateful |
| Concurrent calls | No | Yes |
