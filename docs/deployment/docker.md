# Deploying with Docker

Run FGP daemons in containers.

## Browser Daemon

### Dockerfile

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install Chrome
RUN apt-get update && apt-get install -y \
    chromium \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/browser-gateway /usr/local/bin/

ENV CHROME_PATH=/usr/bin/chromium

EXPOSE 9222
VOLUME /root/.fgp

CMD ["browser-gateway", "start"]
```

### Build and Run

```bash
docker build -t fgp-browser .
docker run -d \
  --name fgp-browser \
  -v ~/.fgp:/root/.fgp \
  fgp-browser
```

### Access Socket

Mount the socket directory:

```bash
docker run -d \
  --name fgp-browser \
  -v /tmp/fgp:/root/.fgp/services \
  fgp-browser
```

Then connect from host:
```bash
nc -U /tmp/fgp/browser/daemon.sock
```

## Docker Compose

```yaml
version: '3.8'

services:
  browser:
    build:
      context: ./browser
    volumes:
      - fgp-sockets:/root/.fgp/services
    restart: unless-stopped

  gmail:
    build:
      context: ./gmail
    volumes:
      - fgp-sockets:/root/.fgp/services
      - gmail-creds:/root/.fgp/services/gmail
    restart: unless-stopped

volumes:
  fgp-sockets:
  gmail-creds:
```

## Headless Chrome

For browser daemon, Chrome runs in headless mode by default.

To use with display (for debugging):

```bash
docker run -d \
  --name fgp-browser \
  -e DISPLAY=:0 \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  fgp-browser
```

## Health Checks

Add health check to Dockerfile:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s \
  CMD echo '{"id":"hc","v":1,"method":"health","params":{}}' | \
      nc -U /root/.fgp/services/browser/daemon.sock || exit 1
```

## Resource Limits

```yaml
services:
  browser:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '1.0'
```
