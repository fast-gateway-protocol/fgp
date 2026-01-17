# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

FGP Manager is a **native macOS menu bar app** (SwiftUI) that manages FGP (Fast Gateway Protocol) daemons. It provides:
- Status monitoring for installed daemons
- Marketplace for discovering/installing new daemons
- MCP integration for AI coding assistants (Claude Code, Cursor, Claude Desktop)
- Autostart configuration via SMAppService

## Build & Run Commands

```bash
# Build with Swift Package Manager
swift build

# Build release
swift build -c release

# Run directly
swift run FGPManager

# Build with Xcode (in build/ directory)
xcodebuild -scheme FGPManager -configuration Debug build
```

The app runs as a menu bar accessory (`NSApp.setActivationPolicy(.accessory)`) - no dock icon.

## Architecture

### Service Layer (Domain Logic)
| Service | Purpose |
|---------|---------|
| `DaemonService` | Start/stop/list FGP daemons via UNIX sockets at `~/.fgp/services/<name>/daemon.sock` |
| `RegistryService` | Fetch package registry, install (binary or git clone + cargo build), uninstall |
| `AgentService` | Detect AI agents, register/unregister FGP MCP server in their configs |
| `AutostartService` | macOS login item via SMAppService |

### Protocol Layer
| File | Purpose |
|------|---------|
| `FGPProtocol.swift` | FGP request/response types, `FgpClient` for daemon RPC |
| `UnixSocketClient.swift` | Low-level UNIX socket I/O (Darwin/POSIX) |

Request format: `{"id": "uuid", "v": 1, "method": "service.action", "params": {...}}\n` (NDJSON)

### View Layer (SwiftUI)
| View | Purpose |
|------|---------|
| `PopoverView` | Main menu bar popover - daemon list with start/stop toggles |
| `MarketplaceView` | Browse/install daemons from registry |
| `SettingsView` | Agent integration, MCP config, autostart toggle |

### State Management
`AppState` (MainActor) owns three `ObservableObject` stores:
- `DaemonStore` - Polls daemon status every 5s
- `RegistryStore` - Package registry + install state
- `AgentsStore` - Detected agents + MCP registration state

## Key Paths

| Path | Purpose |
|------|---------|
| `~/.fgp/services/<daemon>/` | Installed daemon directory |
| `~/.fgp/services/<daemon>/daemon.sock` | Daemon UNIX socket |
| `~/.fgp/services/<daemon>/manifest.json` | Daemon metadata (version, entrypoint) |
| `~/.claude.json` | Claude Code MCP config |
| `~/.cursor/mcp.json` | Cursor MCP config |
| `~/Library/Application Support/Claude/claude_desktop_config.json` | Claude Desktop config |

## Bundled Resources

Located in `Sources/FGPManager/Resources/`:
- `registry.json` - Local package registry
- `mcp/fgp-mcp-server.py` - MCP server that agents connect to (bridges to daemons)
- `skill/skill.md` - Documentation for AI agent tool usage
- `tray-template.png` - Menu bar icon

## FGP Protocol Methods

All daemons respond to:
- `health` - Returns `{"status": "running", "version": "...", "uptime_seconds": ...}`
- `stop` - Graceful shutdown

Daemon-specific methods follow `<daemon>.<action>` pattern (e.g., `browser.open`, `gmail.list`).

## Agent Integration Flow

1. `AgentService.detectAgents()` finds installed AI agents
2. User clicks "Connect" in Settings
3. `AgentService.registerMcp()` either:
   - Runs `claude mcp add fgp -- python3 <mcp-server-path>` (Claude Code)
   - Edits JSON config to add MCP server entry (Cursor/Claude Desktop)
4. Agent now has `fgp_*` tools available via the bundled MCP server
