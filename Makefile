.PHONY: help db-up db-down db-logs run test bench fmt clippy check lint

COMPOSE ?= docker compose -f compose.yaml

# Helper to load .env and export variables for shell commands
WITH_ENV = @set -a; \
	if [ -f .env ]; then . ./.env; fi; \
	set +a;

help:
	@echo "Common commands:"
	@echo "  make db-up     # Start PostgreSQL (Docker)"
	@echo "  make db-down   # Stop PostgreSQL (Docker)"
	@echo "  make db-logs   # Follow PostgreSQL logs"
	@echo "  make run       # Run the API"
	@echo "  make test      # Run tests"
	@echo "  make bench     # Run benchmarks"
	@echo "  make fmt       # Format code"
	@echo "  make clippy    # Run clippy (warnings as errors)"
	@echo "  make check     # Cargo check"
	@echo "  make lint      # fmt + clippy"

db-up:
	$(COMPOSE) up -d db

db-down:
	$(COMPOSE) down

db-logs:
	$(COMPOSE) logs -f db

run:
	$(WITH_ENV) cargo run

test:
	$(WITH_ENV) cargo test

bench:
	$(WITH_ENV) cargo bench

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check

lint: fmt clippy
