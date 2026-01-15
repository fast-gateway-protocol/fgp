# FGP - Fast Gateway Protocol

[![CI](https://github.com/fast-gateway-protocol/fgp/actions/workflows/ci.yml/badge.svg)](https://github.com/fast-gateway-protocol/fgp/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://fast-gateway-protocol.github.io/fgp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Daemon-based architecture for AI agent tools. 19x faster than MCP stdio.**

FGP replaces slow MCP stdio servers with persistent UNIX socket daemons. Instead of spawning a new process for each tool call (~2.3s overhead), FGP keeps daemons warm and ready (~10-50ms latency).

> This repository hosts the **FGP docs, tooling, and companion apps**, plus the `fgp-travel` daemon. Core SDKs and most service daemons live in their own repos—see **Related Repositories** below.

## What lives in this repo

| Path | Description |
|------|-------------|
| `docs/` | MkDocs documentation site content (published to https://fast-gateway-protocol.github.io/fgp/) |
| `app/` | Tauri + SvelteKit desktop app for managing local FGP daemons |
| `website/` | Marketing site (React + Vite) |
| `travel/` | `fgp-travel` daemon for flight + hotel search via Kiwi/Skypicker and Xotelo |
| `benchmarks/` | Benchmark scripts and chart generation for performance claims |
| `install.sh` | Installer for the CLI + default daemons |
| `mkdocs.yml` | MkDocs configuration for the docs site |

## Performance

<p align="center">
  <img src="docs/assets/benchmark-browser.svg" alt="FGP vs MCP Browser Benchmark" width="700">
</p>

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

### iMessage Daemon (macOS)

Fast iMessage operations via direct SQLite queries to `chat.db`:

| Operation | FGP Daemon | MCP Stdio | Speedup |
|-----------|------------|-----------|---------|
| Recent messages | **8ms** | 2,300ms | **292x** |
| Unread messages | **10ms** | 2,300ms | **230x** |
| Analytics | **5ms** | 2,400ms | **480x** |

### Summary by Daemon

| Daemon | Avg Latency | Architecture | Speedup |
|--------|-------------|--------------|---------|
| **iMessage** | **8ms** | Native Rust + SQLite | **480x** |
| **Browser** | **16ms** | Native Rust + CDP | **292x** |
| **Calendar** | **233ms** | PyO3 + Google API | Beta |
| **GitHub** | **474ms** | Native Rust + gh CLI | **75x** |
| **Gmail** | **683ms** | PyO3 + Google API | **69x** |

**Key insight:** Latency is dominated by external API calls, not FGP overhead (~5-10ms). Local daemons (iMessage, Browser) are fastest. For MCP, add ~2.3s cold-start to every call.

## Quick paths for this repo

### Documentation site (MkDocs)

```bash
cd docs
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
mkdocs serve
```

### Desktop app (Tauri + SvelteKit)

```bash
cd app
pnpm install
pnpm dev
```

### Marketing website (React + Vite)

```bash
cd website
pnpm install
pnpm dev
```

### Travel daemon (`fgp-travel`)

```bash
cd travel
cargo build --release
./target/release/fgp-travel start
```

### Benchmarks

```bash
cd benchmarks
python3 browser_benchmark.py --iterations 5
python3 generate_charts.py
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

## Related repositories

Core SDKs and service daemons live in their own repos under the FGP GitHub org:

- [daemon](https://github.com/fast-gateway-protocol/daemon) - Core Rust SDK
- [daemon-py](https://github.com/fast-gateway-protocol/daemon-py) - Python SDK
- [cli](https://github.com/fast-gateway-protocol/cli) - `fgp` CLI
- [browser](https://github.com/fast-gateway-protocol/browser) - Browser daemon
- [gmail](https://github.com/fast-gateway-protocol/gmail) - Gmail daemon
- [calendar](https://github.com/fast-gateway-protocol/calendar) - Calendar daemon
- [github](https://github.com/fast-gateway-protocol/github) - GitHub daemon
- [imessage](https://github.com/fast-gateway-protocol/imessage) - iMessage daemon
- [neon](https://github.com/fast-gateway-protocol/neon) - Neon daemon
- [vercel](https://github.com/fast-gateway-protocol/vercel) - Vercel daemon

## License

MIT
