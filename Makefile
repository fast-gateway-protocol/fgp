# FGP - Fast Gateway Protocol
# Build orchestration for all components

.PHONY: all build test lint fmt clean release help
.PHONY: daemon browser cli services
.PHONY: gmail calendar github fly neon vercel

# Default target
all: build

# ============================================================================
# CORE TARGETS
# ============================================================================

build: daemon browser cli services ## Build all components
	@echo "Build complete"

test: ## Run all tests
	@echo "Testing daemon SDK..."
	cargo test --manifest-path daemon/Cargo.toml
	@echo "Testing browser daemon..."
	cargo test --manifest-path browser/Cargo.toml
	@echo "Testing CLI..."
	cargo test --manifest-path cli/Cargo.toml
	@echo "Testing service daemons..."
	@for d in gmail calendar github fly neon vercel; do \
		echo "Testing $$d..."; \
		cargo test --manifest-path $$d/Cargo.toml; \
	done

lint: ## Run clippy on all crates
	cargo clippy --manifest-path daemon/Cargo.toml -- -D warnings
	cargo clippy --manifest-path browser/Cargo.toml -- -D warnings
	cargo clippy --manifest-path cli/Cargo.toml -- -D warnings
	@for d in gmail calendar github fly neon vercel; do \
		cargo clippy --manifest-path $$d/Cargo.toml -- -D warnings; \
	done

fmt: ## Format all code
	cargo fmt --manifest-path daemon/Cargo.toml
	cargo fmt --manifest-path browser/Cargo.toml
	cargo fmt --manifest-path cli/Cargo.toml
	@for d in gmail calendar github fly neon vercel; do \
		cargo fmt --manifest-path $$d/Cargo.toml; \
	done

fmt-check: ## Check formatting without modifying
	cargo fmt --check --manifest-path daemon/Cargo.toml
	cargo fmt --check --manifest-path browser/Cargo.toml
	cargo fmt --check --manifest-path cli/Cargo.toml

clean: ## Clean all build artifacts
	@for d in daemon browser cli gmail calendar github fly neon vercel; do \
		cargo clean --manifest-path $$d/Cargo.toml 2>/dev/null || true; \
	done

# ============================================================================
# COMPONENT TARGETS
# ============================================================================

daemon: ## Build daemon SDK
	cargo build --manifest-path daemon/Cargo.toml --release

browser: daemon ## Build browser daemon
	cargo build --manifest-path browser/Cargo.toml --release

cli: daemon ## Build FGP CLI
	cargo build --manifest-path cli/Cargo.toml --release

services: gmail calendar github fly neon vercel ## Build all service daemons

# Individual service daemons
gmail: daemon
	cargo build --manifest-path gmail/Cargo.toml --release

calendar: daemon
	cargo build --manifest-path calendar/Cargo.toml --release

github: daemon
	cargo build --manifest-path github/Cargo.toml --release

fly: daemon
	cargo build --manifest-path fly/Cargo.toml --release

neon: daemon
	cargo build --manifest-path neon/Cargo.toml --release

vercel: daemon
	cargo build --manifest-path vercel/Cargo.toml --release

# ============================================================================
# RELEASE & INSTALL
# ============================================================================

release: ## Build optimized release binaries
	@echo "Building release binaries..."
	cargo build --manifest-path daemon/Cargo.toml --release
	cargo build --manifest-path browser/Cargo.toml --release
	cargo build --manifest-path cli/Cargo.toml --release
	@for d in gmail calendar github fly neon vercel; do \
		cargo build --manifest-path $$d/Cargo.toml --release; \
	done
	@echo "Release binaries in */target/release/"

install-browser: browser ## Install browser-gateway to ~/.local/bin
	@mkdir -p ~/.local/bin
	cp browser/target/release/browser-gateway ~/.local/bin/
	@echo "Installed browser-gateway to ~/.local/bin/"

install-cli: cli ## Install fgp CLI to ~/.local/bin
	@mkdir -p ~/.local/bin
	cp cli/target/release/fgp ~/.local/bin/
	@echo "Installed fgp to ~/.local/bin/"

# ============================================================================
# DEVELOPMENT
# ============================================================================

dev-browser: daemon ## Build and start browser daemon in foreground
	cargo build --manifest-path browser/Cargo.toml
	./browser/target/debug/browser-gateway start --foreground

check: lint fmt-check test ## Run all checks (CI equivalent)

# ============================================================================
# HELP
# ============================================================================

help: ## Show this help
	@echo "FGP Build System"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
