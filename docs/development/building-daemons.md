# Building Custom Daemons

Create your own FGP daemon to integrate any service.

## Prerequisites

- Rust 1.70+
- `fgp-daemon` crate

## Project Setup

```bash
cargo new my-daemon
cd my-daemon
```

Add to `Cargo.toml`:

```toml
[dependencies]
fgp-daemon = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
anyhow = "1"
```

## Basic Structure

```rust
use fgp_daemon::{FgpServer, FgpService};
use std::collections::HashMap;
use serde_json::Value;
use anyhow::Result;

struct MyService {
    // Your service state
}

impl MyService {
    fn new() -> Self {
        Self {}
    }
}

impl FgpService for MyService {
    fn name(&self) -> &str {
        "my-service"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "my-service.hello" | "hello" => self.handle_hello(params),
            "my-service.echo" | "echo" => self.handle_echo(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<(&str, &str)> {
        vec![
            ("my-service.hello", "Return a greeting"),
            ("my-service.echo", "Echo back the input"),
        ]
    }
}

impl MyService {
    fn handle_hello(&self, _params: HashMap<String, Value>) -> Result<Value> {
        Ok(serde_json::json!({
            "message": "Hello from my-service!"
        }))
    }

    fn handle_echo(&self, params: HashMap<String, Value>) -> Result<Value> {
        let input = params.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("(no input)");

        Ok(serde_json::json!({
            "echoed": input
        }))
    }
}

fn main() -> Result<()> {
    let service = MyService::new();
    let socket_path = dirs::home_dir()
        .unwrap()
        .join(".fgp/services/my-service/daemon.sock");

    let server = FgpServer::new(service, socket_path)?;
    server.serve()
}
```

## Run Your Daemon

```bash
# Create socket directory
mkdir -p ~/.fgp/services/my-service

# Build and run
cargo run

# Test it
echo '{"id":"1","v":1,"method":"hello","params":{}}' | \
  nc -U ~/.fgp/services/my-service/daemon.sock
```

## Adding CLI

Use `clap` for a proper CLI:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "my-daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
    Health,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            let service = MyService::new();
            let server = FgpServer::new(service, socket_path())?;
            server.serve()
        }
        Commands::Stop => {
            // Send stop command to running daemon
            send_command("stop")
        }
        Commands::Health => {
            // Check daemon health
            send_command("health")
        }
    }
}
```

## Best Practices

### Error Handling

Return meaningful errors:

```rust
fn handle_fetch(&self, params: HashMap<String, Value>) -> Result<Value> {
    let url = params.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

    // ...
}
```

### Logging

Use `tracing` for structured logging:

```rust
use tracing::{info, error};

fn handle_request(&self, method: &str) -> Result<Value> {
    info!(method = method, "Processing request");
    // ...
}
```

### Connection Pooling

For external services, maintain connection pools:

```rust
struct MyService {
    client: reqwest::Client,
    // Connection pool is maintained by the client
}

impl MyService {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}
```

### State Management

Use thread-safe state for concurrent requests:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct MyService {
    cache: Arc<RwLock<HashMap<String, String>>>,
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let service = MyService::new();
        let result = service.dispatch("hello", HashMap::new()).unwrap();
        assert!(result.get("message").is_some());
    }
}
```

## Next Steps

- See [Protocol Reference](../protocol/overview.md) for message format details
- Check [existing daemons](https://github.com/fast-gateway-protocol) for examples
