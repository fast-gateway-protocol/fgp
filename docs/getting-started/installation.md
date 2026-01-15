# Installation

## Requirements

- **macOS or Linux** (UNIX sockets required)
- **Chrome** (for browser daemon)
- **Rust 1.70+** (only if building from source)

## Quick Install (Recommended)

Install FGP with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
```

This installs the FGP CLI and browser daemon to `~/.fgp/bin/`.

### Install Specific Daemons

```bash
# Install Gmail and Calendar daemons
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash -s -- gmail calendar

# Install all available daemons
curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash -s -- all
```

### Available Daemons

| Daemon | Description |
|--------|-------------|
| `cli` | FGP command-line interface |
| `browser` | Browser automation (Chrome DevTools) |
| `gmail` | Gmail API operations |
| `calendar` | Google Calendar |
| `github` | GitHub API |
| `fly` | Fly.io deployments |
| `neon` | Neon Postgres |
| `vercel` | Vercel deployments |

## Install via Cargo

If you prefer Cargo or need to build from source:

```bash
# Install the CLI
cargo install fgp

# Verify installation
fgp --version
```

## Install from Source

```bash
# Clone the repository
git clone https://github.com/fast-gateway-protocol/cli
cd cli

# Build and install
cargo install --path .
```

## Install Individual Daemons

Once the CLI is installed, use it to install daemons:

```bash
# Install browser daemon
fgp install browser

# Install Gmail daemon
fgp install gmail

# List installed daemons
fgp list
```

## Directory Structure

FGP stores its files in `~/.fgp/`:

```
~/.fgp/
├── services/           # Daemon binaries and sockets
│   ├── browser/
│   │   ├── daemon.sock # UNIX socket
│   │   └── browser-gateway
│   └── gmail/
│       ├── daemon.sock
│       └── gmail-daemon
└── config.toml         # Global configuration
```

## Verify Installation

```bash
# Check CLI
fgp --help

# Start a daemon
fgp start browser

# Check status
fgp status
```

## Troubleshooting

### Permission Denied

Ensure socket directories have correct permissions:

```bash
chmod 700 ~/.fgp/services/*/
```

### Daemon Won't Start

Check if port/socket is in use:

```bash
lsof ~/.fgp/services/browser/daemon.sock
```

### Chrome Not Found

Set Chrome path explicitly:

```bash
export CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

## Next Steps

- [Quick Start](quickstart.md) - Run your first workflow
- [Concepts](concepts.md) - Understand FGP architecture
