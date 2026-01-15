# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-15

### Added
- Initial release
- Native EventKit bindings via objc2-event-kit
- CLI with commands: calendars, today, events, range, search, upcoming, auth
- FGP daemon with socket at `~/.fgp/services/apple-calendar/daemon.sock`
- Methods: `apple-calendar.calendars`, `apple-calendar.today`, `apple-calendar.events`, `apple-calendar.range`, `apple-calendar.search`, `apple-calendar.upcoming`, `apple-calendar.auth`

### Performance
- Calendar list: ~5ms (vs ~2.3s MCP cold start)
- Event queries: ~8-12ms
