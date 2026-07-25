#!/bin/bash
# Bootstrap the development environment for Budu.
#
# Usage: bash scripts/dev-bootstrap.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Budu Dev Bootstrap"

# ── Check prerequisites ──────────────────────────────────

command -v rustc >/dev/null 2>&1 || {
    echo "==> Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
}

command -v node >/dev/null 2>&1 || {
    echo "ERROR: Node.js is required. Install from https://nodejs.org"
    exit 1
}

# ── macOS-specific tooling ───────────────────────────────

if [[ "$(uname)" == "Darwin" ]]; then
    if ! xcode-select -p >/dev/null 2>&1; then
        echo "==> Installing Xcode Command Line Tools..."
        xcode-select --install
        echo "    Re-run this script after CLT installation completes."
        exit 0
    fi

    # Check for Homebrew (needed for Wine build dependencies)
    if ! command -v brew >/dev/null 2>&1; then
        echo "WARNING: Homebrew not found. Wine cannot be built from source."
        echo "  Install from https://brew.sh if you plan to build Wine."
    fi
fi

# ── Project setup ────────────────────────────────────────

echo "==> Installing Rust toolchain..."
rustup update stable
rustup target add x86_64-apple-darwin aarch64-apple-darwin

echo "==> Installing frontend dependencies..."
cd "$ROOT/ui"
npm install

echo "==> Creating data directories..."
mkdir -p "$HOME/.gamerunner"/{wine,bottles,runtimes,logs,compat}

echo ""
echo "==> Bootstrap complete!"
echo "    Run 'make dev' to start the development server."
echo ""
echo "    To run Windows games, you need a Wine build."
echo "    Place it at: ~/.gamerunner/wine/<version>/"
echo "    Or download one from the Budu releases page."
