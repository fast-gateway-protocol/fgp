# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-15

### Added
- Initial release
- SQLite queries to knowledgeC.db for Screen Time data
- CLI with commands: daily-total, app-usage, weekly-summary, most-used, timeline, auth
- FGP daemon with socket at `~/.fgp/services/screen-time/daemon.sock`
- Mac Absolute Time conversion (2001-01-01 epoch)
- Methods: `screen-time.daily_total`, `screen-time.app_usage`, `screen-time.weekly_summary`, `screen-time.most_used`, `screen-time.usage_timeline`, `screen-time.auth`

### Technical
- Uses `/app/usage` stream from ZOBJECT table
- Requires Full Disk Access permission
- Read-only access (no write operations)

### Performance
- Daily total: ~8ms
- Weekly summary: ~50ms
- All queries return JSON with formatted duration strings
