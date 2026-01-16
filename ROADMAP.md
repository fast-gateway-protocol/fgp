# FGP Roadmap

This document outlines the planned development direction for the Fast Gateway Protocol project.

## Repositories

Core:
- [daemon](https://github.com/fast-gateway-protocol/daemon)
- [daemon-py](https://github.com/fast-gateway-protocol/daemon-py)
- [protocol](https://github.com/fast-gateway-protocol/protocol)
- [cli](https://github.com/fast-gateway-protocol/cli)

Daemons:
- [browser](https://github.com/fast-gateway-protocol/browser)
- [gmail](https://github.com/fast-gateway-protocol/gmail)
- [calendar](https://github.com/fast-gateway-protocol/calendar)
- [github](https://github.com/fast-gateway-protocol/github)
- [imessage](https://github.com/fast-gateway-protocol/imessage)
- [fly](https://github.com/fast-gateway-protocol/fly)
- [neon](https://github.com/fast-gateway-protocol/neon)
- [vercel](https://github.com/fast-gateway-protocol/vercel)
- [slack](https://github.com/fast-gateway-protocol/slack)

Tooling:
- [dashboard](https://github.com/fast-gateway-protocol/dashboard)
- [workflow](https://github.com/fast-gateway-protocol/workflow)
- [homebrew-fgp](https://github.com/fast-gateway-protocol/homebrew-fgp)

## Vision

Make FGP the standard protocol for AI agent tool integrations - replacing slow stdio-based approaches with persistent, low-latency daemons.

---

## Current Status (v0.1.x)

### Stable
- Core daemon SDK (Rust)
- Browser Gateway (Chrome DevTools Protocol)
- iMessage daemon (macOS)
- NDJSON-over-UNIX-socket protocol

### Beta
- Gmail, Calendar, GitHub daemons
- Python daemon SDK
- MCP bridge compatibility layer

### Alpha
- Fly.io, Neon, Vercel daemons
- Slack daemon
- CLI (`fgp` command)
- Workflow composition library (`fgp-workflow`)
- Dashboard UI (`fgp-dashboard`)

---

## Near-Term Priorities

### Documentation & Developer Experience
- [x] Hosted documentation site (MkDocs on GitHub Pages)
- [x] Protocol reference documentation
- [x] Deployment guide (systemd, Docker, launchd)
- [ ] Troubleshooting guide (dedicated doc)

### Distribution
- [x] Pre-built binary releases (GitHub Releases)
- [x] One-line installer (`curl | sh`)
- [x] Docker images for each daemon
- [x] Homebrew formula (tap repo)

### Ecosystem
- [x] Integration examples for Claude Code
- [x] Integration examples for Cursor
- [x] Integration examples for Windsurf
- [ ] Migration guide from MCP stdio servers
- [ ] Plugin/extension system for custom daemons (scaffolding + manifest install exist)

---

## Medium-Term Goals

### New Daemons
- [x] Slack daemon
- [x] Linear daemon
- [x] Notion daemon
- [x] Postgres daemon (direct, not via Neon)

### Performance & Reliability
- [ ] Connection pooling
- [x] Health monitoring dashboard
- [ ] Automatic daemon restart on failure
- [ ] Metrics/telemetry (opt-in)

### Security
- [ ] Optional authentication layer
- [ ] Encrypted socket communication
- [ ] Audit logging

---

## Long-Term Vision

### Protocol Evolution
- Multi-daemon orchestration (workflow library exists; protocol support TBD)
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
