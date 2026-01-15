# Contributing

Thank you for your interest in contributing to FGP!

## Quick Links

- [GitHub Repository](https://github.com/fast-gateway-protocol)
- [Issue Tracker](https://github.com/fast-gateway-protocol/fgp/issues)
- [Code of Conduct](https://github.com/fast-gateway-protocol/fgp/blob/master/.github/CODE_OF_CONDUCT.md)

## Ways to Contribute

### Documentation

- Fix typos and improve clarity
- Add examples and tutorials
- Translate documentation

### Code

- Fix bugs
- Add new daemon methods
- Improve performance
- Add tests

### New Daemons

Build integrations for new services! See [Building Daemons](building-daemons.md).

## Development Setup

### Rust Components

```bash
# Clone
git clone https://github.com/fast-gateway-protocol/daemon
cd daemon

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy
cargo fmt --check
```

### Python Components

```bash
cd daemon-py
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pytest
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Submit PR

### PR Guidelines

- Keep PRs focused
- Include tests
- Update docs if needed
- Follow commit conventions

## Commit Messages

We use conventional commits:

```
feat(browser): add scroll method
fix(gmail): handle empty inbox
docs: update installation guide
test: add protocol tests
```

## Code Style

### Rust

- Follow standard Rust conventions
- Run `cargo fmt`
- Address `cargo clippy` warnings
- Use meaningful variable names

### Documentation

- Use clear, simple language
- Include code examples
- Keep explanations concise

## Questions?

- Open an issue for bugs
- Start a discussion for questions
- Check existing issues first
