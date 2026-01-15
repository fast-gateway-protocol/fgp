# CLI Reference

The `fgp` command-line tool for managing FGP daemons.

## Installation

```bash
cargo install fgp
```

## Global Options

```
fgp [OPTIONS] <COMMAND>

Options:
  -v, --verbose    Enable verbose output
  -q, --quiet      Suppress output
  -h, --help       Print help
  -V, --version    Print version
```

## Commands

### install

Install a daemon.

```bash
fgp install <daemon>
fgp install browser
fgp install gmail
```

### list

List installed daemons.

```bash
fgp list
```

Output:
```
Installed daemons:
  browser    v0.1.0    ~/.fgp/services/browser/
  gmail      v0.1.0    ~/.fgp/services/gmail/
```

### start

Start a daemon.

```bash
fgp start <daemon>
fgp start browser
```

Options:
- `--foreground`: Run in foreground (don't daemonize)
- `--log-level <LEVEL>`: Set log level (debug, info, warn, error)

### stop

Stop a running daemon.

```bash
fgp stop <daemon>
fgp stop browser
```

### status

Check daemon status.

```bash
fgp status [daemon]
fgp status          # All daemons
fgp status browser  # Specific daemon
```

Output:
```
Daemon Status:
  browser    running    pid=12345    uptime=2h 15m
  gmail      stopped    -            -
```

### call

Call a daemon method.

```bash
fgp call <daemon> <method> [params...]
```

Examples:
```bash
# Simple call
fgp call browser health

# With parameters
fgp call browser open "https://example.com"

# Named parameters
fgp call browser fill --selector "#email" --value "test@example.com"

# JSON parameters
fgp call browser fill '{"selector": "#email", "value": "test@example.com"}'
```

### logs

View daemon logs.

```bash
fgp logs <daemon>
fgp logs browser
fgp logs browser --follow
fgp logs browser --lines 100
```

Options:
- `-f, --follow`: Follow log output
- `-n, --lines <N>`: Number of lines to show

### update

Update a daemon to the latest version.

```bash
fgp update <daemon>
fgp update browser
fgp update --all
```

### uninstall

Remove a daemon.

```bash
fgp uninstall <daemon>
fgp uninstall browser
```

## Configuration

Configuration file: `~/.fgp/config.toml`

```toml
[defaults]
log_level = "info"

[browser]
chrome_path = "/usr/bin/google-chrome"

[gmail]
credentials_path = "~/.fgp/gmail-credentials.json"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `FGP_HOME` | FGP directory (default: `~/.fgp`) |
| `FGP_LOG_LEVEL` | Default log level |
| `CHROME_PATH` | Chrome executable path |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Daemon not found |
| 3 | Daemon not running |
| 4 | Connection failed |
