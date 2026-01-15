# API Reference

## Rust SDK

### FgpServer

The main server struct for running daemons.

```rust
use fgp_daemon::FgpServer;

let server = FgpServer::new(service, socket_path)?;
server.serve()?;
```

#### Methods

##### `new(service: impl FgpService, socket_path: impl AsRef<Path>) -> Result<Self>`

Create a new server instance.

##### `serve(self) -> Result<()>`

Start serving requests. Blocks until shutdown.

### FgpService Trait

Implement this trait to create a daemon.

```rust
pub trait FgpService: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value>;
    fn method_list(&self) -> Vec<(&str, &str)>;
}
```

#### Required Methods

##### `name(&self) -> &str`

Return the service name (e.g., "browser").

##### `version(&self) -> &str`

Return the service version (e.g., "0.1.0").

##### `dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value>`

Handle an incoming request. Called for each method invocation.

##### `method_list(&self) -> Vec<(&str, &str)>`

Return list of (method_name, description) tuples.

### FgpClient

Client for calling daemons.

```rust
use fgp_daemon::FgpClient;

let client = FgpClient::connect(socket_path).await?;
let result = client.call("browser.open", json!({"url": "https://example.com"})).await?;
```

#### Methods

##### `connect(socket_path: impl AsRef<Path>) -> Result<Self>`

Connect to a daemon.

##### `call(&self, method: &str, params: Value) -> Result<Value>`

Call a method and wait for response.

##### `health(&self) -> Result<HealthResponse>`

Check daemon health.

##### `stop(&self) -> Result<()>`

Request daemon shutdown.

## Python SDK

### FgpService Class

```python
from fgp_daemon import FgpService, FgpServer

class MyService(FgpService):
    def name(self) -> str:
        return "my-service"

    def version(self) -> str:
        return "0.1.0"

    def dispatch(self, method: str, params: dict) -> dict:
        if method == "hello":
            return {"message": "Hello!"}
        raise ValueError(f"Unknown method: {method}")

server = FgpServer(MyService(), "~/.fgp/services/my-service/daemon.sock")
server.serve()
```

### FgpClient Class

```python
from fgp_daemon import FgpClient

client = FgpClient("~/.fgp/services/browser/daemon.sock")
result = client.call("browser.open", {"url": "https://example.com"})
print(result)
```

## Protocol Types

### Request

```typescript
interface Request {
  id: string;      // Unique request ID
  v: number;       // Protocol version (1)
  method: string;  // Method to call
  params?: object; // Method parameters
}
```

### Response

```typescript
interface Response {
  id: string;           // Matches request ID
  ok: boolean;          // Success indicator
  result?: any;         // Result (if ok=true)
  error?: string;       // Error message (if ok=false)
  meta: {
    server_ms: number;  // Processing time
    protocol_v: number; // Protocol version
  };
}
```

### HealthResponse

```typescript
interface HealthResponse {
  status: "healthy" | "degraded";
  uptime_secs: number;
  version: string;
}
```

### MethodInfo

```typescript
interface MethodInfo {
  name: string;
  description: string;
  params: string[];
}
```
