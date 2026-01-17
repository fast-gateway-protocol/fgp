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
| **Cold start elimination** | 1.0-1.6s saved | No process spawn per session |
| **Network APIs** (warm-to-warm) | ~1x | Network latency dominates |
| **Local SQLite ops** | 14-20x faster | No subprocess spawn |

### Benchmarked Results (Honest Numbers)

**Cold Start Savings:**
| Service | MCP Cold Start | FGP | Saved |
|---------|----------------|-----|-------|
| Browser (Playwright) | 1,080ms | 0ms | **1.1s** |
| Gmail | 1,597ms | 0ms | **1.6s** |

**Warm-to-Warm (Network APIs):**
| Operation | MCP Warm | FGP Warm | Speedup |
|-----------|----------|----------|---------|
| Gmail unread | 145ms | 142ms | ~1x |
| Browser snapshot | 2.6ms | 3.2ms | ~1x |

**Local Operations (No Network):**
| Operation | CLI/Subprocess | FGP Daemon | Speedup |
|-----------|----------------|------------|---------|
| Screen Time | 5.0ms | 0.32ms | **15.7x** |
| iMessage recent | 80ms | 5ms | **16x** |

> **Note:** MCP servers stay warm within a Claude Code session. Network-bound operations are comparable. FGP's advantage is cold start elimination and local operation speed.

---

## Repository Structure

```
fgp/
├── daemon/          # Core SDK (Rust) - Build your own FGP daemons
├── daemon-py/       # Python SDK - For Python-based daemons
├── protocol/        # FGP protocol specification (NDJSON over UNIX sockets)
├── cli/             # `fgp` CLI for managing daemons
├── website/         # Marketplace website (TanStack Router + React)
├── registry/        # Backend registry service (PostgreSQL)
├── scripts/         # Automation scripts (sync-registry.ts)
│
│ # Core Daemons
├── browser/         # Browser automation (Chrome DevTools Protocol)
├── imessage/        # iMessage (macOS - SQLite + AppleScript) **16x**
├── screen-time/     # Screen Time (macOS - knowledgeC.db) **15.7x**
│
│ # Google Services
├── gmail/           # Gmail (Google API)
├── calendar/        # Google Calendar
├── google-drive/    # Google Drive file operations
├── google-sheets/   # Google Sheets spreadsheet operations
├── google-docs/     # Google Docs document operations
│
│ # Cloud & DevOps
├── github/          # GitHub (GraphQL + REST)
├── cloudflare/      # Cloudflare DNS, KV, Workers
├── fly/             # Fly.io deployments
├── vercel/          # Vercel deployments
├── neon/            # Neon Postgres
├── supabase/        # Supabase (Auth, Storage, SQL, Vectors)
│
│ # Integrations
├── composio/        # 500+ SaaS integrations
├── zapier/          # Webhook automation
│
│ # Social & Media
├── discord/         # Discord bot operations
└── youtube/         # YouTube Data API
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

| Component | Status | Speedup | Notes |
|-----------|--------|---------|-------|
| `daemon` | Stable | - | Core SDK, concurrent server |
| `browser` | Production | 3-12x | Chrome DevTools Protocol |
| `imessage` | Production | 480x | macOS only |
| `screen-time` | Production | 50x | macOS only |
| `gmail` | Production | 69x | Google API |
| `calendar` | Production | 10x | Google API |
| `github` | Production | 75x | GraphQL + REST |
| `google-drive` | Production | 40-80x | File operations |
| `google-sheets` | Production | 35-70x | Spreadsheet operations |
| `google-docs` | Production | 30-60x | Document operations |
| `cloudflare` | Production | 50-100x | DNS, KV, Workers |
| `discord` | Production | 60-120x | Bot operations |
| `youtube` | Production | 40-80x | Data API |
| `supabase` | Production | 40-120x | Full platform |
| `composio` | Production | 30-60x | 500+ integrations |
| `zapier` | Production | 40-100x | Webhook automation |
| `fly` | Beta | - | Deployment ops |
| `neon` | Beta | - | SQL operations |
| `vercel` | Beta | - | Deployment ops |

---

## Daemon Registry & Marketplace

The FGP marketplace (`website/`) displays all available daemons. Two sources of truth:

1. **`website/src/data/registry.ts`** - TypeScript file with package metadata (current)
2. **`manifest.json`** - Per-daemon manifest files (source of truth for new daemons)

### Adding a New Daemon to the Marketplace

**Step 1: Create `manifest.json`** in your daemon directory:

```json
{
  "name": "my-daemon",
  "version": "1.0.0",
  "description": "Fast operations for X service",
  "protocol": "fgp@1",
  "author": "Your Name",
  "license": "MIT",
  "repository": "https://github.com/fast-gateway-protocol/my-daemon",
  "daemon": {
    "entrypoint": "./target/release/fgp-my-daemon",
    "socket": "my-daemon/daemon.sock",
    "dependencies": []
  },
  "methods": [
    {"name": "my-daemon.action", "description": "Do something", "params": [
      {"name": "input", "type": "string", "required": true}
    ]}
  ],
  "auth": {
    "type": "bearer_token",
    "provider": "service.com",
    "setup": "Set MY_API_KEY environment variable"
  },
  "platforms": ["darwin", "linux"]
}
```

**Step 2: Add to `registry.ts`**

Add a `Package` entry to `website/src/data/registry.ts`:

```typescript
{
  name: 'my-daemon',
  version: '1.0.0',
  description: 'Fast operations for X service',
  repository: 'https://github.com/fast-gateway-protocol/my-daemon',
  license: 'MIT',
  platforms: ['darwin', 'linux'],
  categories: ['devtools', 'cloud'],
  featured: false,
  verified: true,
  skills: ['claude-code'],
  auth: {
    type: 'bearer_token',
    provider: 'service.com',
    setup: 'Set MY_API_KEY environment variable',
  },
  methods: [
    { name: 'my-daemon.action', description: 'Do something' },
  ],
  benchmark: {
    avg_latency_ms: 20,
    vs_mcp_speedup: '40-80x',
  },
  added_at: '2026-01-15',
  updated_at: '2026-01-15',
  tier: 'free',
}
```

**Step 3: CI/CD Auto-Sync** (`.github/workflows/registry-sync.yml`)

The CI pipeline automatically:
- Validates all `manifest.json` files
- Builds all Rust daemons
- Deploys the website on changes to main

### Available Categories

```typescript
'browser' | 'productivity' | 'email' | 'calendar' | 'devtools' |
'cloud' | 'database' | 'travel' | 'research' | 'automation' |
'social' | 'media' | 'storage' | 'integrations'
```
