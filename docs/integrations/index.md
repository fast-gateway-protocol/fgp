# IDE & Agent Integrations

FGP integrates with AI coding assistants and IDEs to provide fast tool access.

## Supported Platforms

| Platform | Status | Guide |
|----------|--------|-------|
| [Claude Code](./claude-code.md) | Full support | MCP bridge + skills |
| [Cursor](./cursor.md) | Full support | MCP bridge |
| [Windsurf](./windsurf.md) | Full support | Rules + shell |
| VS Code | Planned | Extension |

## How It Works

FGP provides an **MCP bridge** that translates the Model Context Protocol to FGP's native UNIX socket protocol. This gives you:

1. **Compatibility** - Works with any MCP-compatible client
2. **Performance** - Sub-10ms responses instead of 2+ second cold-starts
3. **Reliability** - Daemons stay warm and ready

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Claude Code   │────▶│   MCP Bridge    │────▶│   FGP Daemons   │
│   or Cursor     │ MCP │   (fgp mcp-     │ FGP │   (browser,     │
│                 │◀────│    bridge)      │◀────│    gmail, etc)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
      ~0ms                    ~1ms                   ~5-50ms
```

## Quick Setup

1. **Install FGP**
   ```bash
   curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
   ```

2. **Start daemons**
   ```bash
   fgp start browser
   ```

3. **Configure your IDE** - See platform-specific guide above

## Performance Comparison

Multi-step browser workflow (navigate → snapshot → click → snapshot):

| Platform + Backend | Total Time |
|--------------------|------------|
| Claude Code + FGP | **585ms** |
| Claude Code + Playwright MCP | 11,211ms |
| Cursor + FGP | **585ms** |
| Cursor + Playwright MCP | 11,211ms |

**FGP is 19x faster for real workflows.**
