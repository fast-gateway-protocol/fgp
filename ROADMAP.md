# FGP Roadmap

This document outlines the planned development direction for the Fast Gateway Protocol project.

## Vision

Make FGP the standard protocol for AI agent tool integrations - replacing slow stdio-based approaches with persistent, low-latency daemons.

---

## Current Status (v0.1.x)

### Stable
- Core daemon SDK (Rust)
- Browser Gateway (Chrome DevTools Protocol)
- NDJSON-over-UNIX-socket protocol

### Beta
- Gmail, Calendar, GitHub daemons
- Python daemon SDK
- MCP bridge compatibility layer

### Alpha
- Fly.io, Neon, Vercel daemons
- CLI (`fgp` command)

---

## Near-Term Priorities

### Documentation & Developer Experience
- [ ] Hosted documentation site (MkDocs on GitHub Pages)
- [ ] Protocol reference documentation
- [ ] Deployment guide (systemd, Docker, launchd)
- [ ] Troubleshooting guide

### Distribution
- [ ] Pre-built binary releases (GitHub Releases)
- [ ] One-line installer (`curl | sh`)
- [ ] Docker images for each daemon
- [ ] Homebrew formula

### Ecosystem
- [ ] Integration examples for Claude Code, Cursor, Windsurf
- [ ] Migration guide from MCP stdio servers
- [ ] Plugin/extension system for custom daemons

---

## Medium-Term Goals

### New Daemons
- [ ] Slack daemon
- [ ] Linear daemon
- [ ] Notion daemon
- [ ] Postgres daemon (direct, not via Neon)

### Performance & Reliability
- [ ] Connection pooling
- [ ] Health monitoring dashboard
- [ ] Automatic daemon restart on failure
- [ ] Metrics/telemetry (opt-in)

### Security
- [ ] Optional authentication layer
- [ ] Encrypted socket communication
- [ ] Audit logging

---

## Long-Term Vision

### Protocol Evolution
- Multi-daemon orchestration
- Streaming responses for long-running operations
- Cross-machine daemon communication (TCP option)

### Ecosystem Growth
- Community daemon registry
- Visual daemon manager GUI
- IDE integrations (VS Code extension)

---

## How to Contribute

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines. Priority areas:

1. **Documentation** - Help improve docs and examples
2. **New daemons** - Build integrations for your favorite services
3. **Testing** - Add tests and report bugs
4. **Performance** - Benchmark and optimize

---

*This roadmap is subject to change based on community feedback and priorities.*
