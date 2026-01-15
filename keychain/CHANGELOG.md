# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-15

### Added
- Initial release
- Native Security framework bindings via security-framework crate
- CLI with commands: find-generic, set-generic, delete, exists, auth
- FGP daemon with socket at `~/.fgp/services/keychain/daemon.sock`
- Methods: `keychain.find_generic`, `keychain.set_generic`, `keychain.delete`, `keychain.exists`, `keychain.auth`

### Security
- Code signing required for Keychain API access
- Passwords never logged
- No bulk export operations by design

### Performance
- Find password: ~3ms (vs ~150ms `security` CLI)
- Set password: ~5ms
- 40-50x faster than spawning `security` subprocess
