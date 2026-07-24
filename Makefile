# GameRunner Build Orchestration

.PHONY: bootstrap dev build clean test lint release

SHELL := /bin/bash

# ── Development ──────────────────────────────────────────

bootstrap:
	@echo "==> Checking Rust toolchain..."
	rustup show
	@echo "==> Checking Node.js..."
	node --version
	@echo "==> Installing frontend dependencies..."
	cd ui && npm install
	@echo "==> Creating data directories..."
	mkdir -p ~/.gamerunner/{wine,bottles,runtimes,logs,compat}
	@echo "==> Bootstrap complete. Run 'make dev' to start."

dev:
	cd src-tauri && cargo tauri dev

# ── Building ─────────────────────────────────────────────

build: ui-build
	cd src-tauri && cargo tauri build

ui-build:
	cd ui && npm run build

ui-lint:
	cd ui && npm run lint

# ── Testing ──────────────────────────────────────────────

test:
	cd src-tauri && cargo test

test-ui:
	cd ui && npm run test

test-all: test test-ui

lint:
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
	cd ui && npm run lint

# ── Wine ─────────────────────────────────────────────────

wine-build:
	bash wine/build.sh

# ── Packaging ────────────────────────────────────────────

release: test lint build
	bash scripts/package-release.sh

# ── Cleanup ──────────────────────────────────────────────

clean:
	cd src-tauri && cargo clean
	cd ui && rm -rf dist node_modules

distclean: clean
	rm -rf ~/.gamerunner

# ── Help ─────────────────────────────────────────────────

help:
	@echo "GameRunner build targets:"
	@echo "  bootstrap    - Set up development environment"
	@echo "  dev          - Start development server (HMR)"
	@echo "  build        - Build for production"
	@echo "  test         - Run Rust tests"
	@echo "  test-all     - Run all tests (Rust + UI)"
	@echo "  lint         - Run all linters"
	@echo "  release      - Full release build pipeline"
	@echo "  clean        - Remove build artifacts"
	@echo "  distclean    - Remove everything including user data"
