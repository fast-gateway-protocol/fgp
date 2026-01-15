# Installation

## Requirements

- **Rust 1.70+** (for building from source)
- **macOS or Linux** (UNIX sockets required)
- **Chrome** (for browser daemon)

## Install via Cargo

The recommended way to install FGP:

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
