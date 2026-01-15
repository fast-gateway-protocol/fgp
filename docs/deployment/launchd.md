# Deploying with launchd (macOS)

Run FGP daemons as launchd services on macOS.

## Create Plist File

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
        <string>/Users/YOUR_USERNAME/.cargo/bin/browser-gateway</string>
        <string>start</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/Users/YOUR_USERNAME/.fgp/logs/browser.log</string>

    <key>StandardErrorPath</key>
    <string>/Users/YOUR_USERNAME/.fgp/logs/browser.error.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
</dict>
</plist>
```

Replace `YOUR_USERNAME` with your actual username.

## Load and Start

```bash
# Create logs directory
mkdir -p ~/.fgp/logs

# Load the service
launchctl load ~/Library/LaunchAgents/com.fgp.browser.plist

# Check if running
launchctl list | grep fgp
```

## Manage Service

```bash
# Stop
launchctl stop com.fgp.browser

# Start
launchctl start com.fgp.browser

# Unload (disable)
launchctl unload ~/Library/LaunchAgents/com.fgp.browser.plist

# Reload after changes
launchctl unload ~/Library/LaunchAgents/com.fgp.browser.plist
launchctl load ~/Library/LaunchAgents/com.fgp.browser.plist
```

## View Logs

```bash
tail -f ~/.fgp/logs/browser.log
tail -f ~/.fgp/logs/browser.error.log
```

## Multiple Daemons

Create separate plist files:

- `com.fgp.browser.plist`
- `com.fgp.gmail.plist`
- `com.fgp.calendar.plist`

## Troubleshooting

### Service Won't Start

Check for errors:
```bash
launchctl error com.fgp.browser
```

### Permission Issues

Ensure correct file permissions:
```bash
chmod 644 ~/Library/LaunchAgents/com.fgp.browser.plist
```

### Binary Not Found

Use absolute paths in ProgramArguments.

### Environment Variables

Add them to the EnvironmentVariables dict in the plist.

## Example: All Daemons

Script to install all daemon plists:

```bash
#!/bin/bash

DAEMONS="browser gmail calendar github"

for daemon in $DAEMONS; do
    cat > ~/Library/LaunchAgents/com.fgp.$daemon.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.fgp.$daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$HOME/.fgp/services/$daemon/${daemon}-daemon</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF
    launchctl load ~/Library/LaunchAgents/com.fgp.$daemon.plist
done
```
