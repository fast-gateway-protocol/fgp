# Windsurf Integration

This guide shows how to integrate FGP daemons with [Windsurf](https://windsurf.com), the agentic AI code editor.

## Overview

Windsurf's Cascade AI assistant uses rules to customize behavior. FGP integrates via:

1. **Windsurf Rules** - Custom rules that teach Cascade to use FGP commands
2. **Shell Integration** - Direct `fgp call` commands in terminal

The result: Cascade can use FGP daemons for fast browser automation, email, calendar, and more.

## Setup

### 1. Install FGP

```bash
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
```

Add to your PATH:
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

Verify with:
```bash
fgp status
```

### 3. Add FGP Rules to Windsurf

Create a rules file at `.windsurf/rules/fgp.md` in your project (or globally):

```markdown
# FGP Integration

When performing browser automation, email, calendar, or GitHub operations, use the Fast Gateway Protocol (FGP) CLI for optimal performance.

## Available FGP Commands

### Browser Automation
- `fgp call browser.open -p '{"url": "https://example.com"}'` - Navigate to URL
- `fgp call browser.snapshot` - Get page accessibility tree
- `fgp call browser.screenshot -p '{"path": "/tmp/screenshot.png"}'` - Take screenshot
- `fgp call browser.click -p '{"selector": "button#submit"}'` - Click element
- `fgp call browser.fill -p '{"selector": "input#search", "text": "query"}'` - Fill input

### Gmail
- `fgp call gmail.list` - List emails
- `fgp call gmail.read -p '{"id": "message_id"}'` - Read email
- `fgp call gmail.send -p '{"to": "user@example.com", "subject": "Hi", "body": "Hello"}'` - Send email

### Calendar
- `fgp call calendar.list` - List events
- `fgp call calendar.create -p '{"title": "Meeting", "start": "2024-01-15T10:00:00"}'` - Create event

### GitHub
- `fgp call github.issues -p '{"repo": "owner/repo"}'` - List issues
- `fgp call github.prs -p '{"repo": "owner/repo"}'` - List pull requests

## Usage Guidelines

1. **Prefer FGP over shell commands** for browser, email, and calendar tasks
2. **Check daemon status** with `fgp status` before running commands
3. **Auto-start** daemons with `fgp start <daemon>` if not running
4. **Output is JSON** - parse results as needed
```

### 4. Alternative: Global Rules

For global FGP integration across all projects, add the rules to Windsurf's global config.

Open Windsurf Settings → Customizations → Rules → Add new global rule with the content above.

## Using FGP in Windsurf

### Browser Automation

```
You: Navigate to GitHub and find issues for the FGP project

Cascade: I'll use FGP to navigate and find the issues.

$ fgp call browser.open -p '{"url": "https://github.com/fast-gateway-protocol/fgp/issues"}'
$ fgp call browser.snapshot

[Returns accessibility tree showing issue list]
```

### Email Operations

```
You: Check my unread emails

Cascade: Let me fetch your unread emails using FGP.

$ fgp call gmail.list -p '{"query": "is:unread"}'

[Returns list of unread messages]
```

### Calendar Management

```
You: What meetings do I have today?

Cascade: I'll check your calendar.

$ fgp call calendar.list -p '{"date": "today"}'

[Returns today's events]
```

## Performance Comparison

| Operation | Traditional MCP | FGP |
|-----------|-----------------|-----|
| Browser navigate | 2,300ms | 8ms |
| Gmail list | 2,400ms | 35ms |
| Calendar list | 2,300ms | 25ms |

## Troubleshooting

### Daemon not responding

```bash
# Check status
fgp status

# Restart daemon
fgp stop browser
fgp start browser
```

### Command not found

Ensure FGP is on your PATH:
```bash
export PATH="$HOME/.fgp/bin:$PATH"
```

### Check daemon health

```bash
fgp health browser
```

## Auto-Start with Monitor

For reliability, run the FGP monitor with auto-restart:

```bash
fgp monitor --auto-restart --interval 30
```

This watches all daemons and automatically restarts any that crash.

## Example Workflow

Here's a complete workflow using FGP in Windsurf:

1. **Start daemons**:
   ```bash
   fgp start browser gmail calendar
   ```

2. **Create project rules** (`.windsurfrules`):
   ```markdown
   Use FGP for all browser, email, and calendar operations.
   Prefer `fgp call` commands over other approaches.
   ```

3. **Use in Cascade**:
   - Ask Cascade to perform browser automation
   - Request email summaries
   - Schedule calendar events

## Next Steps

- [FGP CLI Reference](../reference/cli.md)
- [Browser Daemon](../daemons/browser.md)
- [Protocol Overview](../protocol/overview.md)
