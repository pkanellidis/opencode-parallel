.PHONY: help build test lint fmt clean install run dev release check

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build

release: ## Build optimized release binary
	cargo build --release
	@echo "Binary available at: target/release/opencode-parallel"

test: ## Run tests
	cargo test

test-verbose: ## Run tests with output
	cargo test -- --nocapture

lint: ## Run clippy linter
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

check: ## Check code without building
	cargo check

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

install: release ## Install to local system
	cp target/release/opencode-parallel /usr/local/bin/
	@echo "Installed to /usr/local/bin/opencode-parallel"

uninstall: ## Uninstall from local system
	rm -f /usr/local/bin/opencode-parallel
	@echo "Uninstalled opencode-parallel"

run: ## Run the TUI with default settings
	cargo run

run-web: ## Run the web interface
	cargo run -- web

run-batch: ## Run batch mode with example config
	cargo run -- run --config tasks.example.json --parallel 4

dev: ## Run in development mode with logging
	RUST_LOG=debug cargo run

watch: ## Run with auto-reload on changes (requires cargo-watch)
	cargo watch -x run

doc: ## Generate and open documentation
	cargo doc --open

audit: ## Run security audit
	cargo audit

update: ## Update dependencies
	cargo update

ci: fmt lint test ## Run all CI checks locally

setup: ## Setup development environment
	./setup.sh

.DEFAULT_GOAL := help
