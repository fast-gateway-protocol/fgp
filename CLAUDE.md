# FGP - Fast Gateway Protocol

**Launch Claude Code from this directory:** `cd ~/projects/fgp && claude`

## Repository Structure Note

This is a **workspace directory** containing multiple related repos:
- `daemon/` - https://github.com/fast-gateway-protocol/daemon
- `browser/` - https://github.com/fast-gateway-protocol/browser
- Other directories - See https://github.com/fast-gateway-protocol for full list

## What is FGP?

FGP (Fast Gateway Protocol) is a **daemon-based architecture** that replaces slow MCP stdio servers with persistent UNIX socket daemons. Daemons stay warm across sessions, eliminating cold-start latency.

**Key insight:** LLM agents make many sequential tool calls. FGP provides consistent sub-10ms latency.

### Performance Summary

| Context | Improvement | Why |
|---------|-------------|-----|
| **Cold start** (first call) | 10-20x faster | No process spawn, no init |
| **Warm calls** (same session) | 3-12x faster | Lower protocol overhead |
| **Local ops** (SQLite) | 50x faster | No subprocess spawn |

### Benchmarked Results

| Tool | FGP Daemon | Alternative | Speedup |
|------|------------|-------------|---------|
| Browser navigate | 2-8ms | 27ms (MCP warm) | **3-12x** |
| Browser snapshot | 0.7-9ms | 2-3ms (MCP warm) | **3x** |
| Screen Time | 1-5ms | 60ms (Python subprocess) | **50x** |
| iMessage | 5-10ms | 80ms (Python subprocess) | **10x** |

> **Note:** MCP servers stay warm within a Claude Code session. Speedup claims are for warm-to-warm comparisons unless noted.

---

## Repository Structure

```
fgp/
├── daemon/          # Core SDK (Rust) - Build your own FGP daemons
├── daemon-py/       # Python SDK - For Python-based daemons
├── protocol/        # FGP protocol specification (NDJSON over UNIX sockets)
├── cli/             # `fgp` CLI for managing daemons
│
├── imessage/        # iMessage daemon (macOS - SQLite + AppleScript)
├── browser/         # Browser automation daemon (Chrome DevTools Protocol)
├── screen-time/     # Screen Time daemon (macOS - knowledgeC.db)
├── gmail/           # Gmail daemon (Google API)
├── calendar/        # Google Calendar daemon
├── github/          # GitHub daemon (GraphQL + REST)
├── fly/             # Fly.io daemon (GraphQL)
├── neon/            # Neon Postgres daemon (HTTP API)
└── vercel/          # Vercel daemon (REST API)
```

---

## Quick Reference

### Daemon SDK (`daemon/`)

The core library for building FGP daemons in Rust.

**Key files:**
- `src/server.rs` - FgpServer with concurrent connection handling
- `src/service.rs` - FgpService trait (implement this for your daemon)
- `src/protocol.rs` - NDJSON request/response format
- `src/client.rs` - Async client for calling daemons

**Creating a new daemon:**
```rust
use fgp_daemon::{FgpServer, FgpService};

struct MyService { /* state */ }

impl FgpService for MyService {
    fn name(&self) -> &str { "my-service" }
    fn version(&self) -> &str { "1.0.0" }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "my-service.do_something" => { /* ... */ }
            _ => bail!("Unknown method"),
        }
    }
}

fn main() {
    let server = FgpServer::new(MyService::new(), "~/.fgp/services/my-service/daemon.sock")?;
    server.serve()?;
}
```

### Browser Gateway (`browser/`)

**Status:** Production-ready (Phase 3 complete)

Fast browser automation using Chrome DevTools Protocol directly (no Playwright overhead).

**Key files:**
- `src/browser/client.rs` - BrowserClient with multi-session support
- `src/browser/aria.rs` - Single-pass ARIA tree extraction
- `src/service.rs` - BrowserService handlers
- `src/main.rs` - CLI interface

**Available methods:**
| Method | Description |
|--------|-------------|
| `browser.open` | Navigate to URL |
| `browser.snapshot` | Get ARIA accessibility tree |
| `browser.screenshot` | Capture PNG screenshot |
| `browser.click` | Click element by selector |
| `browser.fill` | Fill input field |
| `browser.press` | Press key |
| `browser.select` | Select dropdown option |
| `browser.check` | Check/uncheck checkbox |
| `browser.hover` | Hover over element |
| `browser.scroll` | Scroll page/element |
| `browser.press_combo` | Key with modifiers (Ctrl+A) |
| `browser.upload` | Upload file to input |
| `session.new` | Create isolated session |
| `session.list` | List active sessions |
| `session.close` | Close session |

**CLI usage:**
```bash
browser-gateway start                    # Start daemon
browser-gateway open "https://example.com"
browser-gateway snapshot                 # Get ARIA tree
browser-gateway fill "input#search" "query"
browser-gateway click "button[type=submit]"
browser-gateway screenshot /tmp/page.png
```

**Performance (vs Playwright MCP warm):**
- Navigate: 2-8ms vs 27ms (3-12x faster)
- ARIA snapshot: 0.7-9ms vs 2-3ms (3x faster)
- Screenshot: 25-30ms vs 50-80ms (2x faster)

### Gmail Daemon (`gmail/`)

Fast Gmail operations via Google API.

**Key methods:** `gmail.list`, `gmail.read`, `gmail.send`, `gmail.search`

### Calendar Daemon (`calendar/`)

Google Calendar integration.

**Key methods:** `calendar.list`, `calendar.create`, `calendar.update`

### GitHub Daemon (`github/`)

GitHub operations via GraphQL and REST.

**Key methods:** `github.issues`, `github.prs`, `github.repos`

---

## FGP Protocol

All daemons use the same NDJSON-over-UNIX-socket protocol.

**Request format:**
```json
{"id": "uuid", "v": 1, "method": "service.action", "params": {...}}
```

**Response format:**
```json
{"id": "uuid", "ok": true, "result": {...}, "meta": {"server_ms": 12.5, "protocol_v": 1}}
```

**Built-in methods (all daemons):**
- `health` - Check daemon health
- `methods` - List available methods
- `stop` - Graceful shutdown

**Socket locations:** `~/.fgp/services/<name>/daemon.sock`

---

## Development Workflow

### Building

```bash
# Build all Rust crates
cargo build --release

# Build specific daemon
cd browser && cargo build --release
```

### Testing a daemon

```bash
# Start daemon
./target/release/browser-gateway start

# Check health
./target/release/browser-gateway health

# Call method via CLI
./target/release/browser-gateway open "https://example.com"

# Stop daemon
./target/release/browser-gateway stop
```

### Running benchmarks

Benchmarks are in `~/LIFE-PLANNER/benchmarks/`:

```bash
# Browser benchmark (FGP vs agent-browser vs Playwright MCP)
python3 ~/LIFE-PLANNER/benchmarks/browser_benchmark.py --iterations 5

# Workflow benchmarks
python3 ~/LIFE-PLANNER/benchmarks/browser_workflow_benchmark.py --iterations 5
```

---

## Architecture Decisions

### Why UNIX sockets?
- Zero network overhead (local IPC)
- File-based permissions (no auth needed)
- 10-100x faster than TCP for local calls

### Why NDJSON?
- Human-readable for debugging
- Streaming-friendly (newline-delimited)
- Universal language support

### Why Rust for daemons?
- Sub-millisecond latency
- Low memory footprint (~10MB per daemon)
- Safe concurrency with tokio

### Why per-service daemons (not monolith)?
- Independent scaling/restart
- Fault isolation
- Simpler debugging

---

## Common Tasks

### Add a new method to browser daemon

1. Add handler in `browser/src/service.rs`:
   ```rust
   fn handle_my_method(&self, params: HashMap<String, Value>) -> Result<Value> { ... }
   ```

2. Add to dispatch match in `service.rs`:
   ```rust
   "browser.my_method" | "my_method" => self.handle_my_method(params),
   ```

3. Add to `method_list()` for documentation

4. Add CLI command in `browser/src/main.rs`

### Create a new daemon

1. Copy `daemon/examples/echo_daemon.rs` as template
2. Implement `FgpService` trait
3. Add CLI with clap
4. Register socket in `~/.fgp/services/<name>/`

### Debug a daemon

```bash
# Check if running
pgrep -f browser-gateway

# View logs (if using tracing)
RUST_LOG=debug ./target/release/browser-gateway start

# Test socket directly
echo '{"id":"1","v":1,"method":"health","params":{}}' | nc -U ~/.fgp/services/browser/daemon.sock
```

---

## Related Files

- **Benchmarks:** `~/LIFE-PLANNER/benchmarks/browser_*.py`
- **Plans:** `~/.claude/plans/toasty-brewing-wolf.md` (Phase 3 plan)
- **Skill integration:** `~/.claude/skills/browser-gateway/`

---

## Status by Component

| Component | Status | Notes |
|-----------|--------|-------|
| `daemon` | Stable | Core SDK, concurrent server |
| `screen-time` | Production | 50x faster vs subprocess, macOS only |
| `imessage` | Production | 10-50x faster vs subprocess, macOS only |
| `browser` | Production | 3-12x faster warm, 17x cold start |
| `gmail` | Beta | Basic operations |
| `calendar` | Beta | Basic operations |
| `github` | Beta | Issues, PRs |
| `fly` | Alpha | Deployment ops |
| `neon` | Alpha | SQL operations |
| `vercel` | Alpha | Deployment ops |
| `cli` | WIP | Daemon management |
