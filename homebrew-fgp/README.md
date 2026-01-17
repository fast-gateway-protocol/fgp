# Homebrew Tap for FGP

## Doctrine

See [DOCTRINE.md](./DOCTRINE.md).


This is the official Homebrew tap for [FGP (Fast Gateway Protocol)](https://github.com/fast-gateway-protocol/fgp).

## Installation

```bash
# Add the tap
brew tap fast-gateway-protocol/fgp

# Install the CLI
brew install fgp

# Install individual daemons
brew install fgp-browser
brew install fgp-gmail
brew install fgp-calendar
brew install fgp-github

# Or install everything
brew install fgp fgp-browser fgp-slack fgp-gmail fgp-calendar fgp-github fgp-travel fgp-fly fgp-neon fgp-vercel
```

## Available Formulas

| Formula | Description | Status |
|---------|-------------|--------|
| `fgp` | FGP command-line interface | Stable |
| `fgp-browser` | Browser automation daemon (292x faster than Playwright MCP) | Production |
| `fgp-slack` | Slack daemon - channels, messages, search | Beta |
| `fgp-gmail` | Gmail API daemon | Beta |
| `fgp-calendar` | Google Calendar daemon | Beta |
| `fgp-github` | GitHub API daemon | Beta |
| `fgp-travel` | Flight & hotel search daemon (45-230x faster) | Production |
| `fgp-fly` | Fly.io deployment daemon | Alpha |
| `fgp-neon` | Neon Postgres daemon | Alpha |
| `fgp-vercel` | Vercel deployment daemon | Alpha |

## Quick Start

```bash
# Start browser daemon
brew services start fgp-browser

# Or manually
fgp start browser

# Use it
fgp call browser open "https://example.com"
fgp call browser snapshot
```

## Running as Services

All daemons support `brew services`:

```bash
# Start at login
brew services start fgp-browser
brew services start fgp-gmail

# Check status
brew services list

# Stop
brew services stop fgp-browser
```

## Updating

```bash
brew update
brew upgrade fgp fgp-browser fgp-gmail fgp-calendar fgp-github fgp-travel
```

## Uninstalling

```bash
brew uninstall fgp-browser fgp-gmail fgp-calendar fgp-github fgp-travel fgp
brew untap fast-gateway-protocol/fgp
```

## Documentation

- [FGP Documentation](https://fast-gateway-protocol.github.io/fgp/)
- [GitHub Organization](https://github.com/fast-gateway-protocol)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to add new formulas.

## License

MIT
