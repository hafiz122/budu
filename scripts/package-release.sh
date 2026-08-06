#!/bin/bash
# Build a distributable Budu macOS application.
#
# A local ad-hoc-signed bundle only needs:
#   bash scripts/package-release.sh
#
# Optional signing:
#   SIGN_IDENTITY="Developer ID Application: Name (TEAMID)" \
#     bash scripts/package-release.sh
#
# Optional notarization additionally requires APPLE_ID, APPLE_TEAM_ID, and
# NOTARIZE_PASSWORD.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_VERSION="$(sed -nE 's/^version = "([^"]+)"/\1/p' "$ROOT/src-tauri/Cargo.toml" | head -n 1)"
TAURI_VERSION="$(node -p "JSON.parse(require('node:fs').readFileSync('$ROOT/src-tauri/tauri.conf.json', 'utf8')).version")"
UI_VERSION="$(node -p "JSON.parse(require('node:fs').readFileSync('$ROOT/ui/package.json', 'utf8')).version")"

if [[ -z "$CARGO_VERSION" || "$CARGO_VERSION" != "$TAURI_VERSION" || "$TAURI_VERSION" != "$UI_VERSION" ]]; then
    echo "Release version mismatch: Cargo=${CARGO_VERSION:-missing}, Tauri=$TAURI_VERSION, UI=$UI_VERSION" >&2
    exit 1
fi

VERSION="${1:-$TAURI_VERSION}"
if [[ "$VERSION" != "$TAURI_VERSION" ]]; then
    echo "Release version $VERSION does not match the app version $TAURI_VERSION" >&2
    exit 1
fi

APP="$ROOT/src-tauri/target/release/bundle/macos/Budu.app"

echo "==> Building open-source Steam compatibility shim..."
bash "$ROOT/scripts/build-steam-shim.sh"

echo "==> Running release checks..."
(
    cd "$ROOT/src-tauri"
    cargo test --all-targets
    cargo clippy --all-targets --all-features -- -D warnings
)
(
    cd "$ROOT/ui"
    npm run lint
    npm run test
)

echo "==> Building Budu ${VERSION}..."
(
    cd "$ROOT/src-tauri"
    cargo tauri build --bundles app
)

BUNDLE_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
if [[ "$BUNDLE_VERSION" != "$VERSION" ]]; then
    echo "Built app version $BUNDLE_VERSION does not match expected version $VERSION" >&2
    exit 1
fi

if [[ -n "${SIGN_IDENTITY:-}" ]]; then
    echo "==> Signing..."
    codesign --force --deep --sign "$SIGN_IDENTITY" --options runtime "$APP"
else
    echo "==> Verifying ad-hoc signature..."
    codesign --verify --deep --strict --verbose=2 "$APP"
fi

if [[ -n "${APPLE_ID:-}" || -n "${APPLE_TEAM_ID:-}" || -n "${NOTARIZE_PASSWORD:-}" ]]; then
    : "${SIGN_IDENTITY:?SIGN_IDENTITY is required for notarization}"
    : "${APPLE_ID:?APPLE_ID is required for notarization}"
    : "${APPLE_TEAM_ID:?APPLE_TEAM_ID is required for notarization}"
    : "${NOTARIZE_PASSWORD:?NOTARIZE_PASSWORD is required for notarization}"

    archive="$ROOT/src-tauri/target/release/bundle/macos/Budu-${VERSION}.zip"
    ditto -c -k --keepParent "$APP" "$archive"
    xcrun notarytool submit "$archive" \
        --apple-id "$APPLE_ID" \
        --team-id "$APPLE_TEAM_ID" \
        --password "$NOTARIZE_PASSWORD" \
        --wait
    xcrun stapler staple "$APP"
fi

echo "==> Built $APP"
