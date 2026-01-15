# FGP - Fast Gateway Protocol

[![CI](https://github.com/fast-gateway-protocol/fgp/actions/workflows/ci.yml/badge.svg)](https://github.com/fast-gateway-protocol/fgp/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://fast-gateway-protocol.github.io/fgp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Daemon-based architecture for AI agent tools. 19x faster than MCP stdio.**

FGP replaces slow MCP stdio servers with persistent UNIX socket daemons. Instead of spawning a new process for each tool call (~2.3s overhead), FGP keeps daemons warm and ready (~10-50ms latency).

## Performance

### Browser Automation (vs Playwright MCP)

| Operation | FGP Browser | Playwright MCP | Speedup |
|-----------|-------------|----------------|---------|
| Navigate  | **8ms**     | 2,328ms        | **292x** |
| Snapshot  | **9ms**     | 2,484ms        | **276x** |
| Screenshot| **30ms**    | 1,635ms        | **54x** |

### Multi-Step Workflow Benchmark

4-step workflow: Navigate → Snapshot → Click → Snapshot

| Tool | Total Time | vs MCP |
|------|------------|--------|
| **FGP Browser** | **585ms** | **19x faster** |
| Vercel agent-browser | 733ms | 15x faster |
| Playwright MCP | 11,211ms | baseline |

### API Daemons

All methods tested at **100% success rate** (3 iterations each):

#### Gmail Daemon (PyO3 + Google API)

| Method | Mean | Min | Max | Payload |
|--------|------|-----|-----|---------|
| inbox | 881ms | 743ms | 1092ms | 2.4KB |
| search | 748ms | 680ms | 874ms | 2.4KB |
| thread | **116ms** | 105ms | 126ms | 795B |
| unread | 985ms | 916ms | 1047ms | 1.7KB |

#### Calendar Daemon (PyO3 + Google API)

| Method | Mean | Min | Max | Payload |
|--------|------|-----|-----|---------|
| today | 315ms | 145ms | 612ms | 48B |
| upcoming | 241ms | 223ms | 272ms | 444B |
| search | **177ms** | 136ms | 206ms | 46B |
| free_slots | 198ms | 145ms | 258ms | 65B |

#### GitHub Daemon (Native Rust + gh CLI)

| Method | Mean | Min | Max | Payload |
|--------|------|-----|-----|---------|
| user | 418ms | 307ms | 575ms | 199B |
| repos | 569ms | 476ms | 665ms | 2.8KB |
| notifications | 521ms | 512ms | 535ms | 9.8KB |
| issues | **390ms** | 343ms | 460ms | 75B |

### Summary by Daemon

| Daemon | Avg Latency | Architecture |
|--------|-------------|--------------|
| **Calendar** | **233ms** | PyO3 + Google API |
| **GitHub** | **474ms** | Native Rust + gh CLI |
| **Gmail** | **683ms** | PyO3 + Google API |

**Key insight:** Latency is dominated by external API calls, not FGP overhead (~5-10ms). For MCP, add ~2.3s cold-start to every call.

## Why FGP?

LLM agents make many sequential tool calls. Cold-start overhead compounds:

| Agent Workflow | Tool Calls | MCP Overhead | FGP Overhead | Time Saved |
|----------------|------------|--------------|--------------|------------|
| Check email | 2 | 4.6s | 0.02s | **4.6s** |
| Browse + fill form | 5 | 11.5s | 0.05s | **11.4s** |
| Full productivity check | 10 | 23s | 0.1s | **22.9s** |
| Complex agent task | 20 | 46s | 0.2s | **45.8s** |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     AI Agent / Claude                    │
├─────────────────────────────────────────────────────────┤
│                   FGP UNIX Sockets                       │
│   ~/.fgp/services/{browser,gmail,calendar,github}/      │
├──────────┬──────────┬──────────┬──────────┬────────────┤
│ Browser  │  Gmail   │ Calendar │  GitHub  │   ...      │
│ Daemon   │  Daemon  │  Daemon  │  Daemon  │            │
│ (Rust)   │  (PyO3)  │  (PyO3)  │  (Rust)  │            │
├──────────┴──────────┴──────────┴──────────┴────────────┤
│           Chrome    │  Google APIs  │  gh CLI          │
└─────────────────────────────────────────────────────────┘
```

**Key design decisions:**
- **UNIX sockets** - Zero network overhead, file-based permissions
- **NDJSON protocol** - Human-readable, streaming-friendly
- **Per-service daemons** - Independent scaling, fault isolation
- **Rust core** - Sub-millisecond latency, low memory (~10MB)

## Quick Start

### Browser Daemon

```bash
cd browser && cargo build --release

# Start daemon
./target/release/browser-gateway start

# Use it
browser-gateway open "https://example.com"
browser-gateway snapshot
browser-gateway click "button#submit"
browser-gateway screenshot /tmp/page.png
```

### Gmail Daemon

```bash
cd gmail && cargo build --release

# Start daemon (requires OAuth setup)
./target/release/fgp-gmail start

# Use it
fgp call gmail.inbox '{"limit": 5}'
fgp call gmail.search '{"query": "from:important"}'
```

### Calendar Daemon

```bash
cd calendar && cargo build --release

# Start daemon
./target/release/fgp-calendar start

# Use it
fgp call calendar.today
fgp call calendar.upcoming '{"days": 7}'
fgp call calendar.free_slots '{"duration_minutes": 30}'
```

### GitHub Daemon

```bash
cd github && cargo build --release

# Start daemon (uses gh CLI auth)
./target/release/fgp-github start

# Use it
fgp call github.repos '{"limit": 10}'
fgp call github.issues '{"repo": "owner/repo"}'
fgp call github.notifications
```

## FGP Protocol

All daemons use the same NDJSON-over-UNIX-socket protocol.

**Request:**
```json
{"id": "uuid", "v": 1, "method": "service.action", "params": {...}}
```

**Response:**
```json
{"id": "uuid", "ok": true, "result": {...}, "meta": {"server_ms": 12.5}}
```

**Built-in methods (all daemons):**
- `health` - Check daemon health
- `methods` - List available methods
- `stop` - Graceful shutdown

## Repository Structure

```
fgp/
├── daemon/          # Core SDK (Rust) - Build your own FGP daemons
├── daemon-py/       # Python SDK - For Python-based daemons
├── protocol/        # FGP protocol specification
├── cli/             # `fgp` CLI for daemon management
│
├── browser/         # Browser automation (Chrome DevTools Protocol)
├── gmail/           # Gmail daemon (Google API)
├── calendar/        # Google Calendar daemon
├── github/          # GitHub daemon (GraphQL + REST)
└── ...
```

## Status

| Component | Status | Performance |
|-----------|--------|-------------|
| browser | **Production** | 8ms navigate, 9ms snapshot |
| gmail | Beta | 116ms thread read, 881ms inbox |
| calendar | Beta | 177ms search, 233ms avg |
| github | Beta | 390ms issues, 474ms avg |
| daemon SDK | Stable | Core library |
| cli | WIP | Daemon management |

## Building a New Daemon

```rust
use fgp_daemon::{FgpServer, FgpService};

struct MyService { /* state */ }

impl FgpService for MyService {
    fn name(&self) -> &str { "my-service" }
    fn version(&self) -> &str { "1.0.0" }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "my-service.hello" => Ok(json!({"message": "Hello!"})),
            _ => bail!("Unknown method"),
        }
    }
}

fn main() {
    let server = FgpServer::new(MyService::new(), "~/.fgp/services/my-service/daemon.sock")?;
    server.serve()?;
}
```

## License

MIT

## Related

- [daemon](https://github.com/fast-gateway-protocol/daemon) - Core SDK
- [browser](https://github.com/fast-gateway-protocol/browser) - Browser daemon
