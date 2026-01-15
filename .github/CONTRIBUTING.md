# Contributing to FGP

Thank you for your interest in contributing to the Fast Gateway Protocol project!

## Getting Started

1. Fork the repository you want to contribute to
2. Clone your fork locally
3. Create a new branch for your changes

## Development Setup

### Rust Daemons

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Python Components

```bash
# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -e ".[dev]"

# Run tests
pytest
```

## Making Changes

1. Create a descriptive branch name (e.g., `feature/add-scroll-support` or `fix/session-cleanup`)
2. Make your changes with clear, atomic commits
3. Add tests for new functionality
4. Ensure all tests pass
5. Update documentation if needed

## Commit Messages

We use conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Adding or updating tests
- `refactor:` Code refactoring
- `chore:` Maintenance tasks

Example: `feat(browser): add scroll support for elements`

## Pull Requests

1. Push your branch to your fork
2. Open a PR against the `main` branch
3. Fill out the PR template
4. Wait for review

### PR Guidelines

- Keep PRs focused on a single change
- Include tests for new features
- Update relevant documentation
- Respond to review feedback promptly

## Reporting Issues

- Search existing issues before creating a new one
- Use issue templates when available
- Include reproduction steps for bugs
- Provide system information (OS, Rust version, etc.)

## Code Style

### Rust

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings

### Python

- Follow PEP 8
- Use type hints
- Run `ruff` for linting

## Questions?

Open a discussion or issue if you have questions about contributing.
