# FGP - Fast Gateway Protocol

[![CI](https://github.com/fast-gateway-protocol/fgp/actions/workflows/ci.yml/badge.svg)](https://github.com/fast-gateway-protocol/fgp/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://fast-gateway-protocol.github.io/fgp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Daemon-based architecture for AI agent tools. Eliminates cold-start latency.**

FGP replaces slow MCP stdio servers with persistent UNIX socket daemons. Daemons stay warm across sessions, providing consistent sub-10ms latency.

This repo is the umbrella docs/meta repo. Service implementations and tools live in separate repositories under the Fast Gateway Protocol org.

## Performance Summary

| Scenario | FGP vs MCP/CLI | Why |
|----------|----------------|-----|
| **Cold start** (new session) | 10-20x faster | No process spawn, no init |
| **Warm calls** (same session) | 3-12x faster | Lower protocol overhead |
| **Local daemons** (SQLite) | 50-100x faster | No subprocess spawn |

## What is FGP?

FGP is a **protocol and architecture** for building fast AI agent tools. The same pattern works across any domain:

| Daemon | Use Case | Cold Start | Warm |
|--------|----------|------------|------|
| **Browser** | Web automation | 17x faster | 3-12x faster |
| **Screen Time** | App usage analytics | 50x faster | N/A (local) |
| **iMessage** | Message search | 50x+ faster | N/A (local) |
| **Gmail** | Email access | 10x faster | ~same (API-bound) |
| **Calendar** | Schedule queries | 10x faster | ~same (API-bound) |
| **GitHub** | Repos, issues | 5x faster | ~same (API-bound) |
| **Travel** | Flights & hotels | 10x faster | ~same (API-bound) |
| **+ more** | Fly, Neon, Vercel | Alpha | Alpha |

All daemons share the same NDJSON-over-UNIX-socket protocol. Build once, use everywhere.

## Performance

### Understanding the Benchmarks

FGP speedups depend on context:

| Context | What's Compared | Typical Speedup |
|---------|-----------------|-----------------|
| **Cold start** | Fresh MCP spawn vs warm FGP daemon | 10-100x |
| **Warm calls** | Already-running MCP vs FGP daemon | 3-12x |
| **Local ops** | Python subprocess vs FGP daemon | 50-100x |

> **Note:** Claude Code keeps MCP servers running within a session. The cold-start speedup applies to first tool use and new sessions. Warm speedups apply to subsequent calls.

### Browser Automation

**Cold start comparison** (first call, MCP spawns new process):

| Operation | FGP Browser | Playwright MCP (cold) | Speedup |
|-----------|-------------|----------------------|---------|
| Navigate  | **8ms**     | ~1,900ms             | **~17x** |
| Snapshot  | **9ms**     | ~1,000ms             | **~11x** |

**Warm comparison** (MCP server already running):

| Operation | FGP Browser | Playwright MCP (warm) | Speedup |
|-----------|-------------|----------------------|---------|
| Navigate  | **2.5ms**   | 29ms                 | **12x** |
| Snapshot  | **0.7ms**   | 2.2ms                | **3x** |

### Multi-Step Workflow Benchmark

4-step workflow: Navigate → Snapshot → Click → Snapshot

| Tool | Total Time | vs Cold MCP |
|------|------------|-------------|
| **FGP Browser** | **585ms** | **19x faster** |
| Playwright MCP (cold) | 11,211ms | baseline |

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

### Local Daemons (macOS)

Local daemons (iMessage, Screen Time, Keychain) show the largest speedups because they compare FGP's warm daemon against Python subprocess spawn overhead (~50-80ms just to start Python).

**Screen Time Daemon** (vs Python subprocess with SQLite):

| Operation | FGP Daemon | Python Subprocess | Speedup |
|-----------|------------|-------------------|---------|
| daily_total | **1.2ms** | ~60ms | **50x** |
| weekly_summary | **5ms** | ~80ms | **16x** |
| most_used | **2.1ms** | ~60ms | **29x** |

**iMessage Daemon** (vs Python subprocess with SQLite):

| Operation | FGP Daemon | Python Subprocess | Speedup |
|-----------|------------|-------------------|---------|
| Recent messages | **8ms** | ~80ms | **10x** |
| Unread messages | **10ms** | ~80ms | **8x** |
| Analytics | **5ms** | ~100ms | **20x** |

> **Note:** The "480x" claim in earlier versions compared FGP against MCP stdio cold-start. Against warm Python subprocesses, the speedup is 10-50x (still significant).

### Summary by Daemon

| Daemon | Avg Latency | Cold Speedup | Warm Speedup | Notes |
|--------|-------------|--------------|--------------|-------|
| **Screen Time** | **1-5ms** | 50x | N/A | Local SQLite |
| **iMessage** | **5-10ms** | 50x | N/A | Local SQLite |
| **Browser** | **1-8ms** | 17x | 3-12x | CDP protocol |
| **Calendar** | **233ms** | 10x | ~1x | API-bound |
| **GitHub** | **474ms** | 5x | ~1x | API-bound |
| **Gmail** | **683ms** | 10x | ~1x | API-bound |

**Key insights:**
- **Local daemons** (iMessage, Screen Time): Huge speedup because FGP eliminates subprocess spawn overhead entirely
- **Browser**: Significant speedup from persistent Chrome connection and optimized CDP calls
- **API daemons** (Gmail, Calendar, GitHub): Speedup mainly from cold-start elimination; warm calls are API-latency bound

Additional alpha daemons (Fly, Neon, Vercel, Slack) are available; see the Status section for current performance ranges.

## Why FGP?

### The Real Value Proposition

1. **Eliminates cold-start delays** - No 1-2 second pause on first tool use
2. **Cross-session persistence** - Daemons stay warm across Claude Code sessions
3. **Consistent latency** - No surprise delays mid-conversation
4. **Local operations become instant** - SQLite queries in 1-5ms instead of 50-80ms

### When FGP Helps Most

| Scenario | Without FGP | With FGP | Impact |
|----------|------------|----------|--------|
| First tool call in session | ~2s delay | Instant | High |
| Switching between services | Cold start each | All warm | High |
| Local data (messages, files) | Subprocess spawn | Direct access | Very High |
| API calls (Gmail, GitHub) | Cold start + API | Just API | Medium |

### Workflow Impact

For multi-step workflows, cold-start overhead compounds:

| Agent Workflow | Tool Calls | Cold MCP Overhead | FGP Overhead | Savings |
|----------------|------------|-------------------|--------------|---------|
| Check email | 2 | ~2s (first call) | 0s | **2s** |
| Browse + fill form | 5 | ~2s (first call) | 0s | **2s** |
| Multi-service task | 10 | ~6s (if 3 services) | 0s | **6s** |

> **Note:** Within a Claude Code session, MCP servers stay warm after first use. The savings above apply to session startup and switching services.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                           AI Agent / Claude                              │
├──────────────────────────────────────────────────────────────────────────┤
│                          FGP UNIX Sockets                                │
│   ~/.fgp/services/{browser,gmail,calendar,github,imessage,travel,...}   │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬───────┤
│ Browser  │  Gmail   │ Calendar │  GitHub  │ iMessage │  Travel  │  ...  │
│ Daemon   │  Daemon  │  Daemon  │  Daemon  │  Daemon  │  Daemon  │       │
│ (Rust)   │  (PyO3)  │  (PyO3)  │  (Rust)  │  (Rust)  │  (Rust)  │       │
├──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴───────┤
│    Chrome    │    Google APIs    │  gh CLI  │ chat.db  │ Kiwi/Xotelo    │
└──────────────────────────────────────────────────────────────────────────┘
```

**Key design decisions:**
- **UNIX sockets** - Zero network overhead, file-based permissions
- **NDJSON protocol** - Human-readable, streaming-friendly
- **Per-service daemons** - Independent scaling, fault isolation
- **Rust core** - Sub-millisecond latency, low memory (~10MB)

## Installation

### One-liner (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
```

This installs the FGP CLI and browser daemon to `~/.fgp/bin/`.

### Install specific daemons

```bash
# Install Gmail and Calendar daemons
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash -s -- gmail calendar

# Install all daemons
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash -s -- all
```

### From source

```bash
git clone https://github.com/fast-gateway-protocol/browser
cd browser && cargo build --release
```

## Quick Start

### Browser Daemon

```bash
# Start daemon
fgp start browser

# Or if installed from source:
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

### iMessage Daemon (macOS)

```bash
cd imessage && cargo build --release

# Start daemon (requires Full Disk Access)
./target/release/fgp-imessage-daemon start

# Use it
fgp call imessage.recent '{"limit": 10}'
fgp call imessage.unread
fgp call imessage.analytics '{"days": 30}'
fgp call imessage.bundle '{"include": "unread_count,recent,analytics"}'
```

### Travel Daemon (Flights & Hotels)

```bash
cd travel && cargo build --release

# Start daemon
./target/release/fgp-travel start

# Use it - Location search (instant, local DB)
fgp call travel.find_location '{"term": "SFO"}'

# Flight search
fgp call travel.search_flights '{"origin": "SFO", "destination": "BER", "departure_from": "2026-02-15"}'

# Ultra-light price check (~55 tokens, 10x more efficient)
fgp call travel.price_check '{"origin": "SFO", "destination": "LAX", "date": "2026-02-15"}'

# Find cheapest day in a month (parallel search)
fgp call travel.search_cheapest_day '{"origin": "SFO", "destination": "BER", "date_from": "2026-02-01", "date_to": "2026-02-28"}'

# Hotel search
fgp call travel.search_hotels '{"location": "Berlin", "limit": 5}'
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

## Ecosystem Repositories

Core:
- [daemon](https://github.com/fast-gateway-protocol/daemon) - Rust SDK
- [daemon-py](https://github.com/fast-gateway-protocol/daemon-py) - Python SDK
- [protocol](https://github.com/fast-gateway-protocol/protocol) - Protocol spec
- [cli](https://github.com/fast-gateway-protocol/cli) - `fgp` CLI

Daemons:
- [browser](https://github.com/fast-gateway-protocol/browser)
- [gmail](https://github.com/fast-gateway-protocol/gmail)
- [calendar](https://github.com/fast-gateway-protocol/calendar)
- [github](https://github.com/fast-gateway-protocol/github)
- [imessage](https://github.com/fast-gateway-protocol/imessage)
- [travel](https://github.com/fast-gateway-protocol/travel) - Flight & hotel search (Kiwi/Xotelo APIs)
- [fly](https://github.com/fast-gateway-protocol/fly)
- [neon](https://github.com/fast-gateway-protocol/neon)
- [vercel](https://github.com/fast-gateway-protocol/vercel)
- [slack](https://github.com/fast-gateway-protocol/slack)

Tooling:
- [dashboard](https://github.com/fast-gateway-protocol/dashboard) - Local monitoring UI
- [workflow](https://github.com/fast-gateway-protocol/workflow) - Workflow composition library
- [Homebrew tap](https://github.com/fast-gateway-protocol/homebrew-fgp) - Formulae for `brew install`

## Status

| Component | Status | Performance | Speedup |
|-----------|--------|-------------|---------|
| screen-time | **Production** | 1-5ms queries | 50x vs subprocess |
| imessage | **Production** | 5-10ms queries | 10-50x vs subprocess |
| browser | **Production** | 1-8ms operations | 3-12x warm, 17x cold |
| gmail | Beta | 116ms thread, 881ms inbox | 10x cold |
| calendar | Beta | 177ms search, 233ms avg | 10x cold |
| github | Beta | 390ms issues, 474ms avg | 5x cold |
| travel | Beta | 1-10ms location, 400-600ms flights | 10x cold |
| fly | Alpha | 140ms user, 191ms avg | Alpha |
| neon | Alpha | 86ms user, 120ms avg | Alpha |
| vercel | Alpha | 55ms deployments, 82ms avg | Alpha |
| slack | Alpha | Not benchmarked yet | Alpha |
| daemon SDK | Stable | Core library | - |
| daemon-py SDK | Beta | Python daemon SDK | - |
| mcp-bridge | Beta | MCP compatibility | - |
| cli | WIP | Daemon management | - |

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
- [browser](https://github.com/fast-gateway-protocol/browser) - Browser daemon (3-17x faster)
- [imessage](https://github.com/fast-gateway-protocol/imessage) - iMessage daemon (10-50x faster, macOS)
- [travel](https://github.com/fast-gateway-protocol/travel) - Flight & hotel search with token-optimized methods
