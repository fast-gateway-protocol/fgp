# Built-in Methods

Every FGP daemon supports these standard methods.

## health

Check if the daemon is running and responsive.

**Request:**
```json
{"id": "1", "v": 1, "method": "health", "params": {}}
```

**Response:**
```json
{
  "id": "1",
  "ok": true,
  "result": {
    "status": "healthy",
    "uptime_secs": 3600,
    "version": "0.1.0"
  }
}
```

**Use cases:**
- Monitoring daemon status
- Health checks in orchestration systems
- Verifying daemon is ready after start

## methods

List all available methods on the daemon.

**Request:**
```json
{"id": "1", "v": 1, "method": "methods", "params": {}}
```

**Response:**
```json
{
  "id": "1",
  "ok": true,
  "result": {
    "methods": [
      {
        "name": "browser.open",
        "description": "Navigate to a URL",
        "params": ["url", "session?"]
      },
      {
        "name": "browser.click",
        "description": "Click an element",
        "params": ["selector", "session?"]
      }
    ]
  }
}
```

**Use cases:**
- Discovering daemon capabilities
- Building dynamic clients
- Documentation generation

## stop

Gracefully shut down the daemon.

**Request:**
```json
{"id": "1", "v": 1, "method": "stop", "params": {}}
```

**Response:**
```json
{
  "id": "1",
  "ok": true,
  "result": {
    "message": "Shutting down"
  }
}
```

**Behavior:**
1. Daemon stops accepting new connections
2. In-flight requests complete
3. Resources are cleaned up
4. Process exits

**Use cases:**
- Clean shutdown before system restart
- Daemon management scripts
- Graceful restarts

## CLI Usage

These methods are also available via CLI:

```bash
# Check health
fgp call browser health

# List methods
fgp call browser methods

# Stop daemon
fgp stop browser  # or: fgp call browser stop
```

## Implementation Notes

When building a custom daemon, implement these methods in your service:

```rust
impl FgpService for MyService {
    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.handle_health(),
            "methods" => self.handle_methods(),
            "stop" => self.handle_stop(),
            // ... custom methods
            _ => bail!("Unknown method: {}", method),
        }
    }
}
```

The SDK provides default implementations, but you can override them for custom behavior.
