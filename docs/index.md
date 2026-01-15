# FGP - Fast Gateway Protocol

**Daemon-based architecture for AI agent tools. 19x faster than MCP stdio.**

FGP replaces slow MCP stdio servers with persistent UNIX socket daemons. Instead of spawning a new process for each tool call (~2.3s overhead), FGP keeps daemons warm and ready (~10-50ms latency).

## Performance

### Browser Automation (vs Playwright MCP)

| Operation | FGP Browser | Playwright MCP | Speedup |
|-----------|-------------|----------------|---------|
| Navigate  | **8ms**     | 2,328ms        | **292x** |
| Snapshot  | **9ms**     | 2,484ms        | **276x** |
| Screenshot| **30ms**    | 1,635ms        | **54x** |

### Multi-Step Workflow

4-step workflow: Navigate → Snapshot → Click → Snapshot

| Tool | Total Time | vs MCP |
|------|------------|--------|
| **FGP Browser** | **585ms** | **19x faster** |
| Vercel agent-browser | 733ms | 15x faster |
| Playwright MCP | 11,211ms | baseline |

## Quick Start

```bash
# Install FGP CLI
cargo install fgp

# Install browser daemon
fgp install browser

# Start the daemon
fgp start browser

# Use it
fgp call browser open "https://example.com"
fgp call browser snapshot
```

## Why FGP?

### The Problem with MCP stdio

Every MCP tool call:

1. Spawns a new process (~500ms)
2. Initializes runtime (~1s)
3. Establishes connections (~500ms)
4. **Finally** executes your request

This adds ~2.3 seconds to every single operation. For a 5-step workflow, that's 11+ seconds of overhead.

### The FGP Solution

FGP daemons:

1. Start once, stay warm
2. Accept requests via UNIX socket (~1ms)
3. Execute immediately
4. Return results in milliseconds

The daemon handles connection pooling, state management, and resource caching.

## Architecture

```
┌─────────────────┐     UNIX Socket      ┌─────────────────┐
│   AI Agent      │ ──────────────────── │   FGP Daemon    │
│ (Claude Code)   │    ~10-50ms RTT      │  (browser, etc) │
└─────────────────┘                      └─────────────────┘
                                                  │
                                                  ▼
                                         ┌─────────────────┐
                                         │  External API   │
                                         │ (Chrome, Gmail) │
                                         └─────────────────┘
```

## Available Daemons

| Daemon | Description | Status |
|--------|-------------|--------|
| [browser](daemons/browser.md) | Chrome automation via DevTools Protocol | Stable |
| [gmail](daemons/gmail.md) | Gmail API operations | Beta |
| [calendar](daemons/calendar.md) | Google Calendar integration | Beta |
| [github](daemons/github.md) | GitHub GraphQL + REST | Beta |
| fly | Fly.io deployments | Alpha |
| neon | Neon Postgres | Alpha |
| vercel | Vercel deployments | Alpha |

## Next Steps

- [Installation Guide](getting-started/installation.md) - Set up FGP
- [Quick Start](getting-started/quickstart.md) - Build your first workflow
- [Protocol Reference](protocol/overview.md) - Understand the wire format
- [Building Daemons](development/building-daemons.md) - Create custom integrations
