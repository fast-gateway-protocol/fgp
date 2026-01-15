# Cursor Integration

This guide shows how to integrate FGP daemons with [Cursor](https://cursor.com), the AI-powered code editor.

## Overview

Cursor supports MCP servers for tool integrations. FGP provides an MCP bridge that exposes FGP daemons as MCP tools, giving you sub-10ms response times instead of the 2+ second cold-start of traditional MCP stdio servers.

## Setup

### 1. Install FGP

```bash
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
```

Add to your PATH (the installer prompts for this):
```bash
export PATH="$HOME/.fgp/bin:$PATH"
```

### 2. Start Daemons

```bash
# Start browser daemon
fgp start browser

# Start other daemons as needed
fgp start gmail
fgp start calendar
fgp start github
```

### 3. Configure Cursor

Open Cursor Settings (`Cmd+,` on macOS, `Ctrl+,` on Windows/Linux) and navigate to **Features > MCP Servers**.

Add the FGP MCP bridge:

```json
{
  "fgp": {
    "command": "fgp",
    "args": ["mcp-bridge"],
    "env": {}
  }
}
```

Or edit `~/.cursor/mcp.json` directly:

```json
{
  "mcpServers": {
    "fgp": {
      "command": "/Users/yourname/.fgp/bin/fgp",
      "args": ["mcp-bridge"]
    }
  }
}
```

Restart Cursor to load the new MCP server.

## Using FGP Tools in Cursor

Once configured, Cursor's AI can use FGP tools in chat:

### Browser Automation

```
You: Open the React docs and find the useEffect documentation

Cursor: I'll navigate to the React documentation and find the useEffect section.

[Uses fgp__browser__open to navigate to reactjs.org]
[Uses fgp__browser__snapshot to get the page structure]
[Uses fgp__browser__click to navigate to Hooks section]
[Uses fgp__browser__snapshot to find useEffect content]
```

### Email Operations

```
You: Check my inbox for any messages from the team

Cursor: I'll check your Gmail inbox for team messages.

[Uses fgp__gmail__search with query="from:team@company.com"]
```

### Calendar Management

```
You: What meetings do I have today?

Cursor: Let me check your calendar.

[Uses fgp__calendar__today to get today's events]
```

## Available Tools

### Browser (`fgp__browser__*`)

| Tool | Description | Example |
|------|-------------|---------|
| `open` | Navigate to URL | `{"url": "https://example.com"}` |
| `snapshot` | Get ARIA tree | `{}` |
| `screenshot` | Capture PNG | `{"path": "/tmp/page.png"}` |
| `click` | Click element | `{"selector": "button.submit"}` |
| `fill` | Fill input | `{"selector": "#email", "value": "user@example.com"}` |
| `press` | Press key | `{"key": "Enter"}` |
| `select` | Select option | `{"selector": "#country", "value": "US"}` |
| `scroll` | Scroll page | `{"y": 500}` |

### Gmail (`fgp__gmail__*`)

| Tool | Description | Example |
|------|-------------|---------|
| `inbox` | List inbox | `{"limit": 10}` |
| `search` | Search emails | `{"query": "from:boss@company.com"}` |
| `thread` | Get thread | `{"id": "thread_123"}` |
| `send` | Send email | `{"to": "...", "subject": "...", "body": "..."}` |

### Calendar (`fgp__calendar__*`)

| Tool | Description | Example |
|------|-------------|---------|
| `today` | Today's events | `{}` |
| `upcoming` | Next N days | `{"days": 7}` |
| `search` | Find events | `{"query": "standup"}` |
| `free_slots` | Available times | `{"duration_minutes": 30}` |
| `create` | Create event | `{"title": "...", "start": "...", "end": "..."}` |

### GitHub (`fgp__github__*`)

| Tool | Description | Example |
|------|-------------|---------|
| `repos` | List repos | `{"limit": 10}` |
| `issues` | List issues | `{"repo": "owner/repo"}` |
| `notifications` | Get notifications | `{}` |
| `user` | User info | `{}` |

## Performance Benefits

Cursor + FGP vs Cursor + Standard MCP:

| Scenario | Standard MCP | FGP | Improvement |
|----------|--------------|-----|-------------|
| Single browser action | 2.3s | 8ms | **287x** |
| 5-step form fill | 11.5s | 40ms | **287x** |
| Email check | 2.5s | 120ms | **21x** |
| Calendar query | 2.4s | 180ms | **13x** |

The improvement is most dramatic for browser operations because FGP eliminates both the MCP cold-start AND the Playwright overhead.

## Composer Integration

FGP works seamlessly with Cursor's Composer feature. When Composer needs to:

- **Research a topic**: Uses browser to navigate and extract content
- **Check documentation**: Fast ARIA snapshots for structured content
- **Verify deployments**: Screenshot verification in seconds
- **Manage tasks**: Email and calendar operations

## Troubleshooting

### Tools not appearing

1. Verify daemons are running:
   ```bash
   fgp status
   ```

2. Check MCP bridge works:
   ```bash
   fgp mcp-bridge --test
   ```

3. Restart Cursor after config changes

### "Daemon not found" errors

Ensure the full path in your config:
```json
{
  "fgp": {
    "command": "/Users/yourname/.fgp/bin/fgp",
    "args": ["mcp-bridge"]
  }
}
```

### Slow responses

If responses are slow, check if daemons are running:
```bash
fgp status

# Restart if needed
fgp restart browser
```

### Socket permission errors

```bash
# Clear stale sockets
rm -f ~/.fgp/services/*/daemon.sock

# Restart all daemons
fgp restart all
```

## Tips for Best Performance

1. **Keep daemons running** - Start daemons at login with launchd (macOS) or systemd (Linux)

2. **Use sessions for parallel work** - Multiple browser sessions avoid context switching:
   ```bash
   fgp call browser session.new --name "docs"
   fgp call browser session.new --name "testing"
   ```

3. **Use element refs** - After `snapshot`, use `@e5` style refs instead of CSS selectors for faster clicks

4. **Batch operations** - Chain related operations in a single Composer request

## Auto-Start on Login

### macOS (launchd)

Create `~/Library/LaunchAgents/com.fgp.browser.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.fgp.browser</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/yourname/.fgp/bin/fgp</string>
        <string>start</string>
        <string>browser</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load with:
```bash
launchctl load ~/Library/LaunchAgents/com.fgp.browser.plist
```

### Linux (systemd)

See [systemd deployment guide](../deployment/systemd.md).

## Next Steps

- [Browser Daemon Reference](../daemons/browser.md) - All browser methods
- [Gmail Integration](./gmail.md) - Email automation
- [Building Custom Daemons](../development/building-daemons.md) - Create your own
