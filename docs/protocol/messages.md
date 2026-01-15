# Request/Response Details

## Request Structure

### Complete Request Example

```json
{
  "id": "req-001",
  "v": 1,
  "method": "browser.fill",
  "params": {
    "selector": "input#email",
    "value": "user@example.com",
    "session": "login-flow"
  }
}
```

### Method Naming

Methods follow the pattern: `<service>.<action>`

- `browser.open` - Browser service, open action
- `gmail.send` - Gmail service, send action
- `session.new` - Session management

Some daemons accept shorthand:
- `open` → `browser.open`
- `click` → `browser.click`

### Parameter Types

Parameters are method-specific. Common patterns:

```json
// URL parameter
{"url": "https://example.com"}

// Selector parameter
{"selector": "button.submit"}

// Session parameter (optional)
{"session": "my-session"}

// Multiple parameters
{
  "selector": "input",
  "value": "hello",
  "timeout": 5000
}
```

## Response Structure

### Success Response

```json
{
  "id": "req-001",
  "ok": true,
  "result": {
    "filled": true,
    "selector": "input#email"
  },
  "meta": {
    "server_ms": 12.3,
    "protocol_v": 1
  }
}
```

### Error Response

```json
{
  "id": "req-001",
  "ok": false,
  "error": "Timeout waiting for element: input#email",
  "meta": {
    "server_ms": 5002.1,
    "protocol_v": 1
  }
}
```

### Error Codes

Errors are returned as human-readable strings. Common patterns:

| Error | Meaning |
|-------|---------|
| `Element not found: <selector>` | Selector didn't match any element |
| `Timeout waiting for: <selector>` | Element didn't appear in time |
| `Navigation failed: <url>` | Page failed to load |
| `Unknown method: <method>` | Method doesn't exist |
| `Invalid params: <detail>` | Parameter validation failed |

## Result Types

Results vary by method:

### Void Results

Some methods return minimal confirmation:

```json
{"ok": true, "result": {"clicked": true}}
```

### Data Results

Methods that fetch data return structured results:

```json
{
  "ok": true,
  "result": {
    "title": "Page Title",
    "url": "https://example.com/page",
    "aria_tree": "document...",
    "timestamp": "2025-01-14T12:00:00Z"
  }
}
```

### Binary Results

Binary data (screenshots) are base64-encoded or saved to files:

```json
{
  "ok": true,
  "result": {
    "path": "/tmp/screenshot.png",
    "bytes": 45231
  }
}
```

## Timing Information

The `meta.server_ms` field shows server processing time:

```json
"meta": {
  "server_ms": 8.2  // 8.2 milliseconds
}
```

This excludes:
- Client connection time
- Network latency (minimal for UNIX sockets)
- Client JSON parsing

## Best Practices

### Generate Unique IDs

Use UUIDs or incrementing counters:

```python
import uuid
request_id = str(uuid.uuid4())
```

### Handle Errors

Always check `ok` before accessing `result`:

```python
response = send_request(...)
if response["ok"]:
    process(response["result"])
else:
    handle_error(response["error"])
```

### Set Reasonable Timeouts

Client-side timeouts should exceed expected operation time:

```python
# Browser navigation might take a few seconds
response = send_request({"method": "browser.open", ...}, timeout=10.0)
```
