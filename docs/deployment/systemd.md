# Deploying with systemd

Run FGP daemons as systemd services on Linux.

## Create Service File

Create `/etc/systemd/system/fgp-browser.service`:

```ini
[Unit]
Description=FGP Browser Daemon
After=network.target

[Service]
Type=simple
User=your-username
ExecStart=/home/your-username/.cargo/bin/browser-gateway start
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=CHROME_PATH=/usr/bin/google-chrome

[Install]
WantedBy=multi-user.target
```

## Enable and Start

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable on boot
sudo systemctl enable fgp-browser

# Start now
sudo systemctl start fgp-browser

# Check status
sudo systemctl status fgp-browser
```

## View Logs

```bash
# Recent logs
journalctl -u fgp-browser -f

# All logs
journalctl -u fgp-browser --no-pager
```

## Multiple Daemons

Create a service file for each daemon:

- `/etc/systemd/system/fgp-browser.service`
- `/etc/systemd/system/fgp-gmail.service`
- `/etc/systemd/system/fgp-calendar.service`

Or use a template:

```ini
# /etc/systemd/system/fgp@.service
[Unit]
Description=FGP %i Daemon

[Service]
Type=simple
User=your-username
ExecStart=/home/your-username/.fgp/services/%i/%i-daemon start
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo systemctl enable fgp@browser
sudo systemctl start fgp@browser
```

## Socket Permissions

Ensure the socket directory is accessible:

```bash
chmod 700 ~/.fgp/services/browser/
```

## Troubleshooting

### Daemon Won't Start

Check logs:
```bash
journalctl -u fgp-browser -n 50
```

### Permission Denied

Ensure user has access to Chrome and socket directories.

### Chrome Not Found

Set `CHROME_PATH` in the service file's `Environment` directive.
