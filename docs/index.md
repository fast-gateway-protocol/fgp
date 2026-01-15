# FGP - Fast Gateway Protocol

**Daemon-based architecture for AI agent tools. Up to 480x faster than MCP stdio.**

FGP replaces slow MCP stdio servers with persistent UNIX socket daemons. Instead of spawning a new process for each tool call (~2.3s overhead), FGP keeps daemons warm and ready (~10-50ms latency).

## Performance

### iMessage (macOS) - Fastest Local Daemon

| Operation | FGP iMessage | MCP Stdio | Speedup |
|-----------|--------------|-----------|---------|
| Analytics | **5ms**      | 2,400ms   | **480x** |
| Recent    | **8ms**      | 2,300ms   | **292x** |
| Unread    | **10ms**     | 2,300ms   | **230x** |

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

| Daemon | Description | Speedup | Status |
|--------|-------------|---------|--------|
| [imessage](daemons/imessage.md) | macOS iMessage via SQLite | **480x** | Stable |
| [browser](daemons/browser.md) | Chrome automation via DevTools Protocol | **292x** | Stable |
| [github](daemons/github.md) | GitHub GraphQL + REST | **75x** | Beta |
| [gmail](daemons/gmail.md) | Gmail API operations | **69x** | Beta |
| [calendar](daemons/calendar.md) | Google Calendar integration | - | Beta |
| fly | Fly.io deployments | - | Alpha |
| neon | Neon Postgres | - | Alpha |
| vercel | Vercel deployments | - | Alpha |

## IDE & Agent Integrations

| Platform | Guide | Status |
|----------|-------|--------|
| Claude Code | [Integration Guide](integrations/claude-code.md) | Full support |
| Cursor | [Integration Guide](integrations/cursor.md) | Full support |
| Windsurf | Coming soon | Planned |

## Next Steps

- [Installation Guide](getting-started/installation.md) - Set up FGP
- [Quick Start](getting-started/quickstart.md) - Build your first workflow
- [Claude Code Integration](integrations/claude-code.md) - Use with Claude Code
- [Cursor Integration](integrations/cursor.md) - Use with Cursor
- [Protocol Reference](protocol/overview.md) - Understand the wire format
- [Building Daemons](development/building-daemons.md) - Create custom integrations
