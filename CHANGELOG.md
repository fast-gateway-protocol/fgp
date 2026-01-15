# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial FGP architecture with daemon SDK
- Browser Gateway with Chrome DevTools Protocol integration
- Gmail, Calendar, GitHub daemons
- CLI for daemon management
- CI/CD workflows
- Community health files (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY)
- Issue and PR templates

## [0.1.0] - 2025-01-14

### Added
- Initial release
- FGP protocol specification (NDJSON over UNIX sockets)
- Rust daemon SDK (`fgp-daemon`)
- Python daemon SDK (`daemon-py`)
- Browser Gateway with 292x speedup over Playwright MCP
- Gmail daemon with Google API integration
- Calendar daemon
- GitHub daemon
- Fly.io, Neon, Vercel daemons
- MCP bridge for compatibility
