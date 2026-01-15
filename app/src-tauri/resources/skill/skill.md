# FGP Gateway Skill

Fast Gateway Protocol (FGP) provides 10-100x faster tool execution compared to MCP stdio servers by using persistent Unix socket daemons.

## Available Daemons

| Daemon | Description | Key Methods |
|--------|-------------|-------------|
| browser | Browser automation via Chrome DevTools | `browser.open`, `browser.snapshot`, `browser.click`, `browser.fill` |
| gmail | Gmail operations via Google API | `gmail.list`, `gmail.read`, `gmail.send`, `gmail.search` |
| calendar | Google Calendar integration | `calendar.list`, `calendar.create`, `calendar.update` |
| github | GitHub via GraphQL/REST | `github.issues`, `github.prs`, `github.repos` |
| fly | Fly.io deployment | `fly.apps`, `fly.deploy`, `fly.logs` |
| neon | Neon Postgres | `neon.query`, `neon.branches`, `neon.tables` |
| vercel | Vercel deployment | `vercel.deploy`, `vercel.domains`, `vercel.logs` |

## Usage Pattern

### 1. Check Daemon Status
```
Use fgp_list_daemons to see which daemons are installed and running.
```

### 2. Start Required Daemon
```
If the daemon you need is stopped, use fgp_start_daemon with the daemon name.
```

### 3. Call Daemon Methods
```
Call daemon methods using the fgp_<daemon>_<method> tools.
Example: fgp_browser_open, fgp_gmail_list, fgp_github_issues
```

## Examples

### Browser Automation
```
1. fgp_start_daemon(name="browser")  # Start if not running
2. fgp_browser_open(url="https://example.com")
3. fgp_browser_snapshot()  # Get ARIA accessibility tree
4. fgp_browser_click(selector="button#submit")
5. fgp_browser_fill(selector="input#email", value="test@example.com")
```

### Email Operations
```
1. fgp_start_daemon(name="gmail")
2. fgp_gmail_list(max_results=10)  # List recent emails
3. fgp_gmail_read(id="<message_id>")  # Read specific email
4. fgp_gmail_send(to="user@example.com", subject="Hello", body="...")
```

### GitHub Operations
```
1. fgp_start_daemon(name="github")
2. fgp_github_issues(repo="owner/repo", state="open")
3. fgp_github_prs(repo="owner/repo")
```

## Performance Comparison

| Operation | MCP Stdio | FGP Daemon | Speedup |
|-----------|-----------|------------|---------|
| Browser navigate | 2,300ms | 8ms | **292x** |
| Gmail list | 2,400ms | 35ms | **69x** |
| GitHub issues | 2,100ms | 28ms | **75x** |

## Troubleshooting

### Daemon Not Running
If you get "daemon not running" errors:
1. Use `fgp_list_daemons` to check status
2. Use `fgp_start_daemon(name="<daemon>")` to start it

### Connection Errors
If you get socket connection errors:
1. The daemon may have crashed - restart with `fgp_start_daemon`
2. Check `~/.fgp/services/<daemon>/` for logs

### Missing Daemons
If a daemon isn't listed:
1. Open FGP Manager app from menu bar
2. Go to Marketplace
3. Install the required daemon

## File Locations

- Daemons: `~/.fgp/services/<daemon>/`
- Sockets: `~/.fgp/services/<daemon>/daemon.sock`
- Manifests: `~/.fgp/services/<daemon>/manifest.json`
