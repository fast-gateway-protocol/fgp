# Quick Start

This guide walks you through your first FGP workflow in under 5 minutes.

## Start the Browser Daemon

```bash
# Start the daemon
fgp start browser

# Verify it's running
fgp status
```

## Basic Navigation

```bash
# Open a webpage
fgp call browser open "https://news.ycombinator.com"

# Get the page structure (ARIA tree)
fgp call browser snapshot

# Take a screenshot
fgp call browser screenshot /tmp/hn.png
```

## Interact with Elements

```bash
# Click an element
fgp call browser click "a.storylink"

# Fill a form field
fgp call browser fill "input[name=q]" "hello world"

# Press a key
fgp call browser press "Enter"
```

## Multi-Step Workflow Example

Here's a complete workflow that logs into a site:

```bash
#!/bin/bash

# Navigate to login page
fgp call browser open "https://example.com/login"

# Fill credentials
fgp call browser fill "#email" "user@example.com"
fgp call browser fill "#password" "secret"

# Submit form
fgp call browser click "button[type=submit]"

# Wait and verify
sleep 2
fgp call browser snapshot | grep "Dashboard"
```

## Using Sessions

Sessions provide isolated browser contexts:

```bash
# Create a new session
fgp call browser session.new --name "my-session"

# Work in the session
fgp call browser open "https://example.com" --session "my-session"

# List sessions
fgp call browser session.list

# Close when done
fgp call browser session.close --session "my-session"
```

## Direct Protocol Access

You can also communicate directly with the daemon:

```bash
# Send raw request
echo '{"id":"1","v":1,"method":"browser.open","params":{"url":"https://example.com"}}' | \
  nc -U ~/.fgp/services/browser/daemon.sock
```

Response:
```json
{"id":"1","ok":true,"result":{"title":"Example Domain"},"meta":{"server_ms":8.2}}
```

## Next Steps

- [Browser Daemon Reference](../daemons/browser.md) - All browser methods
- [Protocol Overview](../protocol/overview.md) - Wire format details
- [Building Daemons](../development/building-daemons.md) - Create your own
