# Claude Code Integration

This guide shows how to integrate FGP daemons with [Claude Code](https://claude.ai/code), Anthropic's CLI for Claude.

## Why FGP with Claude Code?

Claude Code uses MCP (Model Context Protocol) for tool integrations. Standard MCP stdio servers have ~2.3 seconds of cold-start overhead per call. For multi-step browser workflows, this compounds:

| Workflow | MCP Stdio | FGP Daemon | Improvement |
|----------|-----------|------------|-------------|
| 5-step login | 11.5s overhead | 0.05s overhead | **230x faster** |
| 10-step form | 23s overhead | 0.1s overhead | **230x faster** |

FGP daemons stay warm and respond in milliseconds.

## Setup

### 1. Install FGP

```bash
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
```

This installs the `fgp` CLI and browser daemon to `~/.fgp/bin/`.

### 2. Start the Browser Daemon

```bash
# Start in headless mode (default)
fgp start browser

# Or with visible browser for debugging
fgp start browser --no-headless
```

### 3. Configure Claude Code MCP Bridge

Add to your Claude Code settings (`~/.claude.json` or project `.claude/settings.json`):

```json
{
  "mcpServers": {
    "fgp": {
      "command": "fgp",
      "args": ["mcp-bridge"],
      "env": {}
    }
  }
}
```

The MCP bridge translates MCP protocol to FGP protocol, so Claude Code can use FGP daemons like any MCP server.

## Usage

Once configured, Claude Code can use FGP tools directly:

### Browser Automation

```
User: Navigate to GitHub and search for "rust async"

Claude: I'll use the FGP browser daemon to do this.

[Calls fgp__browser__open with url="https://github.com"]
[Calls fgp__browser__fill with selector="input[name=q]" value="rust async"]
[Calls fgp__browser__press with key="Enter"]
[Calls fgp__browser__snapshot to see results]
```

### Available Tools

| Tool | Description |
|------|-------------|
| `fgp__browser__open` | Navigate to URL |
| `fgp__browser__snapshot` | Get ARIA accessibility tree |
| `fgp__browser__screenshot` | Capture PNG |
| `fgp__browser__click` | Click element |
| `fgp__browser__fill` | Fill input field |
| `fgp__browser__press` | Press key |
| `fgp__browser__select` | Select dropdown option |
| `fgp__gmail__inbox` | List inbox messages |
| `fgp__gmail__search` | Search emails |
| `fgp__gmail__send` | Send email |
| `fgp__calendar__today` | Today's events |
| `fgp__calendar__upcoming` | Upcoming events |
| `fgp__calendar__free_slots` | Find available times |
| `fgp__github__repos` | List repositories |
| `fgp__github__issues` | List issues |
| `fgp__github__notifications` | Get notifications |

## Skill-Based Integration

For more control, create a Claude Code skill that wraps FGP commands.

### Create the Skill

```bash
mkdir -p ~/.claude/skills/fgp-browser
```

Create `~/.claude/skills/fgp-browser/skill.md`:

```markdown
---
name: fgp-browser
description: Fast browser automation via FGP daemon
tools: [Bash]
---

# FGP Browser Skill

Use the FGP browser daemon for fast browser automation. The daemon must be running.

## Starting the daemon

\`\`\`bash
fgp start browser
\`\`\`

## Available commands

\`\`\`bash
# Navigate
fgp call browser open "https://example.com"

# Get page structure (returns ARIA tree with element refs like @e1, @e2)
fgp call browser snapshot

# Click element (by CSS selector or @ref)
fgp call browser click "button.submit"
fgp call browser click "@e5"

# Fill form field
fgp call browser fill "input#email" "user@example.com"

# Take screenshot
fgp call browser screenshot /tmp/page.png

# Press key
fgp call browser press "Enter"
\`\`\`

## Performance

FGP browser is 292x faster than Playwright MCP for navigation (8ms vs 2,328ms).
```

### Use the Skill

```
User: /fgp-browser Navigate to HN and screenshot the front page

Claude: [Invokes skill, runs fgp commands]
```

## Direct Socket Communication

For advanced use cases, communicate directly with FGP sockets:

```bash
# Send NDJSON request
echo '{"id":"1","v":1,"method":"browser.open","params":{"url":"https://example.com"}}' | \
  nc -U ~/.fgp/services/browser/daemon.sock
```

Response:
```json
{"id":"1","ok":true,"result":{"title":"Example Domain"},"meta":{"server_ms":8.2}}
```

## Multi-Session Workflows

FGP supports multiple isolated browser sessions for parallel workflows:

```bash
# Create sessions
fgp call browser session.new --name "gmail"
fgp call browser session.new --name "github"

# Work in parallel
fgp call browser open "https://gmail.com" --session "gmail" &
fgp call browser open "https://github.com" --session "github" &
wait

# Each session has isolated cookies, localStorage, etc.
```

## Troubleshooting

### Daemon not responding

```bash
# Check status
fgp status

# View logs
tail -f ~/.fgp/services/browser/daemon.log

# Restart daemon
fgp stop browser && fgp start browser
```

### Chrome not found

FGP looks for Chrome/Chromium in standard locations. Set explicitly:

```bash
export CHROME_PATH="/path/to/chrome"
fgp start browser
```

### Permission denied on socket

```bash
# Check socket permissions
ls -la ~/.fgp/services/browser/daemon.sock

# Remove stale socket
rm ~/.fgp/services/browser/daemon.sock
fgp start browser
```

## Performance Comparison

Real-world benchmark (4-step workflow: navigate, snapshot, click, snapshot):

| Tool | Total Time | Cold Start | Per-Call |
|------|------------|------------|----------|
| **FGP Browser** | **585ms** | 0ms | ~8ms |
| Playwright MCP | 11,211ms | ~2.3s | ~500ms |

FGP is **19x faster** for multi-step workflows.

## Next Steps

- [Gmail Integration](./gmail.md) - Email automation
- [Calendar Integration](./calendar.md) - Schedule management
- [GitHub Integration](./github.md) - Repository operations
- [Protocol Reference](../protocol/overview.md) - Wire format details
