# Protocol Overview

FGP uses a simple, human-readable protocol: **NDJSON over UNIX sockets**.

## Why This Design?

### NDJSON (Newline-Delimited JSON)

- **Human-readable**: Easy to debug with `cat` and `jq`
- **Streaming-friendly**: Each message is a complete line
- **Universal**: Every language has JSON support
- **Schema-flexible**: Easy to extend

### UNIX Sockets

- **Zero network overhead**: Direct IPC
- **File-based security**: Use filesystem permissions
- **No port conflicts**: Each daemon has its own socket file
- **Faster than TCP**: No TCP handshake, no network stack

## Message Format

### Request

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "v": 1,
  "method": "browser.open",
  "params": {
    "url": "https://example.com"
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique request identifier (UUID recommended) |
| `v` | integer | Yes | Protocol version (must be 1) |
| `method` | string | Yes | Daemon method to call |
| `params` | object | No | Method parameters |

### Success Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "ok": true,
  "result": {
    "title": "Example Domain",
    "url": "https://example.com/"
  },
  "meta": {
    "server_ms": 8.2,
    "protocol_v": 1
  }
}
```

### Error Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "ok": false,
  "error": "Element not found: #nonexistent",
  "meta": {
    "server_ms": 2.1,
    "protocol_v": 1
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Matches request ID |
| `ok` | boolean | `true` for success, `false` for error |
| `result` | any | Method result (only if ok=true) |
| `error` | string | Error message (only if ok=false) |
| `meta` | object | Server metadata |

## Meta Fields

The `meta` object provides debugging information:

| Field | Type | Description |
|-------|------|-------------|
| `server_ms` | float | Server-side processing time in milliseconds |
| `protocol_v` | integer | Protocol version used |

## Connection Flow

```
1. Client connects to UNIX socket
2. Client sends request (single line JSON + newline)
3. Server processes request
4. Server sends response (single line JSON + newline)
5. Connection remains open for more requests
```

## Example Session

```bash
# Connect to daemon
nc -U ~/.fgp/services/browser/daemon.sock

# Send requests (type each line)
{"id":"1","v":1,"method":"health","params":{}}
{"id":"2","v":1,"method":"browser.open","params":{"url":"https://example.com"}}
{"id":"3","v":1,"method":"browser.snapshot","params":{}}
```

## Concurrent Requests

FGP supports concurrent requests on the same connection. The `id` field matches responses to requests:

```
→ {"id":"a","v":1,"method":"browser.open","params":{"url":"https://a.com"}}
→ {"id":"b","v":1,"method":"browser.open","params":{"url":"https://b.com"}}
← {"id":"b","ok":true,"result":{...}}  # b finishes first
← {"id":"a","ok":true,"result":{...}}  # a finishes second
```

## Next Steps

- [Request/Response Details](messages.md)
- [Built-in Methods](builtin.md)
