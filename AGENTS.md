# Repository Guidelines

## Project Structure & Module Organization
This workspace hosts multiple related repos, each with its own build/test setup. Key modules:
- `daemon/` (Rust SDK), `daemon-py/` (Python SDK), `protocol/` (spec)
- `cli/` (FGP CLI)
- Service daemons: `browser/`, `gmail/`, `calendar/`, `github/`, `fly/`, `neon/`, `vercel/`

Within each module, source lives under `src/`, with docs in that module’s `README.md`. Treat each directory as the root when building or testing it.

## Build, Test, and Development Commands
Run commands from the module you’re working on:
- Build a Rust daemon: `cd browser && cargo build --release`
- Run Rust tests (where available): `cd browser && cargo test`
- Install the Python SDK from source: `cd daemon-py && pip install -e .`
- Start a daemon binary (example): `browser-gateway start`

Check the module `README.md` for service-specific commands and required dependencies.

## Coding Style & Naming Conventions
- Rust: follow `rustfmt` defaults; prefer `snake_case` for functions/vars and `CamelCase` for types. Keep modules small and focused.
- Python (`daemon-py/`): Ruff linting and strict mypy are configured; line length is 100. Use `snake_case` and type annotations.

## Testing Guidelines
- Rust tests live in `#[cfg(test)]` modules or `tests/`. Run `cargo test` in the relevant crate.
- Python tests (if added) should use `pytest` with files named `test_*.py`.
- Add or update tests when changing protocol behavior or request/response shapes.

## Commit & Pull Request Guidelines
Git history currently uses Conventional Commit-style prefixes (e.g., `feat:`). Follow that format for new commits.

PRs should include:
- A concise description of the change and affected modules
- Linked issues (if any)
- Notes on local testing (commands + results)
- Screenshots only when user-facing CLI output changes

## Security & Configuration
Daemons communicate over UNIX sockets (e.g., `~/.fgp/services/<name>/daemon.sock`). Store API tokens in environment variables and never commit secrets. When adding a new service, document required config in its module `README.md`.
