# Docker Deployment

Run FGP daemons in Docker containers for isolated, reproducible deployments.

## Prerequisites

- Docker 20.10+
- Docker Compose v2.0+

## Quick Start

```bash
# Clone the repo
git clone https://github.com/fast-gateway-protocol/fgp
cd fgp

# Start all daemons
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

## Available Images

Each daemon has a Dockerfile in its directory:

| Daemon | Dockerfile | Dependencies |
|--------|------------|--------------|
| Browser | `browser/Dockerfile` | Chromium, ~150MB |
| GitHub | `github/Dockerfile` | CA certs only, ~50MB |
| Gmail | `gmail/Dockerfile` | Python + Google API, ~200MB |
| Calendar | `calendar/Dockerfile` | Python + Google API, ~200MB |

## Single Daemon

Start individual daemons:

```bash
# Browser only
docker-compose up -d browser

# GitHub only (requires token)
GITHUB_TOKEN=ghp_xxx docker-compose up -d github

# Gmail/Calendar (requires Google credentials)
docker-compose up -d gmail calendar
```

## Building Images

Build individual daemon images:

```bash
# Build browser daemon
docker build -t fgp-browser ./browser

# Build all daemons
docker-compose build
```

## Configuration

### Environment Variables

| Variable | Description | Daemon |
|----------|-------------|--------|
| `GITHUB_TOKEN` | GitHub personal access token | github |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to Google credentials | gmail, calendar |
| `FGP_SOCKET_DIR` | Socket directory on host | all |
| `CHROME_PATH` | Chrome binary path | browser |

### Volumes

The `fgp-sockets` volume is shared between all daemons for UNIX socket communication:

```yaml
volumes:
  fgp-sockets:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: ${HOME}/.fgp/services
```

### Google Credentials

For Gmail and Calendar daemons, mount your OAuth credentials:

```yaml
volumes:
  - ${HOME}/.config/google:/home/fgp/.config/google:ro
```

## Browser Daemon

The browser daemon requires additional configuration for Chrome.

### Headless Mode (Default)

Works out of the box. Chrome runs without display.

### Headful Mode (X11 Forwarding)

For visual debugging, enable X11 forwarding:

```yaml
browser:
  network_mode: host
  environment:
    - DISPLAY=${DISPLAY}
  volumes:
    - /tmp/.X11-unix:/tmp/.X11-unix
```

### Shared Memory

Chrome needs `/dev/shm` for optimal performance:

```yaml
browser:
  volumes:
    - /dev/shm:/dev/shm
```

Or use `--shm-size`:

```bash
docker run --shm-size=2g fgp-browser
```

## Health Checks

All daemon images include health checks (30s interval, 5s timeout).

Check container health:

```bash
docker inspect --format='{{.State.Health.Status}}' fgp-browser
```

## Connecting from Host

FGP clients on the host connect via the shared socket directory:

```bash
# Using fgp CLI
fgp status

# Direct socket connection
echo '{"id":"1","v":1,"method":"health","params":{}}' | \
  nc -U ~/.fgp/services/browser/daemon.sock
```

## Production Deployment

### Resource Limits

```yaml
browser:
  deploy:
    resources:
      limits:
        cpus: '2'
        memory: 2G
      reservations:
        memory: 512M
```

### Logging

Configure log rotation:

```yaml
browser:
  logging:
    driver: "json-file"
    options:
      max-size: "10m"
      max-file: "3"
```

### Auto-Restart

All services use `restart: unless-stopped`. For production:

```yaml
browser:
  restart: always
```

## Claude Code Integration

Use containerized FGP with Claude Code's MCP bridge:

```json
{
  "mcpServers": {
    "fgp": {
      "command": "fgp",
      "args": ["mcp-bridge"]
    }
  }
}
```

The MCP bridge connects to sockets in `~/.fgp/services/`, shared with Docker containers.

## Troubleshooting

### Browser: Chrome crashes

Ensure `/dev/shm` is mounted with enough space:

```bash
docker run --shm-size=2g fgp-browser
```

### Socket permission denied

Check host socket directory permissions:

```bash
chmod 755 ~/.fgp/services
```

### Container can't connect

Verify socket volume is mounted:

```bash
docker exec fgp-browser ls -la /home/fgp/.fgp/services/
```
