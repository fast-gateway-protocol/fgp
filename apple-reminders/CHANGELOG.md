# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-15

### Added
- Initial release
- Native EventKit bindings via objc2-event-kit
- CLI with commands: lists, all, incomplete, completed, due-today, overdue, search, create, complete, auth
- FGP daemon with socket at `~/.fgp/services/apple-reminders/daemon.sock`
- Async reminder fetching with 30-second timeout
- Methods: `apple-reminders.lists`, `apple-reminders.all`, `apple-reminders.incomplete`, `apple-reminders.completed`, `apple-reminders.due_today`, `apple-reminders.overdue`, `apple-reminders.search`, `apple-reminders.create`, `apple-reminders.complete`, `apple-reminders.auth`

### Performance
- Reminder list: ~12-15ms (vs ~2.3s MCP cold start)
- Create reminder: ~8ms
