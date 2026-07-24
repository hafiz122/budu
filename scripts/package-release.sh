#!/bin/bash
# Package GameRunner for macOS distribution.
#
# Prerequisites:
#   - Apple Developer ID certificate in keychain
#   - App-specific password for notarization in NOTARIZE_PASSWORD env var
#
# Usage: bash scripts/package-release.sh [VERSION]

set -euo pipefail

VERSION="${1:-0.1.0}"
APP_NAME="GameRunner"
DMG_NAME="GameRunner-${VERSION}.dmg"
DEVELOPER_ID="Developer ID Application: Your Name (TEAMID)"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="$ROOT/target/release"

echo "==> Building UI..."
cd "$ROOT/ui" && npm run build

echo "==> Building Rust backend (universal)..."
cd "$ROOT"
cargo build --release

echo "==> Creating .app bundle..."
# Tauri's build command handles the .app creation.
# This is a manual packaging script for custom workflows.
BUNDLE_DIR="$ROOT/target/release/bundle/macos"
mkdir -p "$BUNDLE_DIR"

echo "==> Signing..."
codesign --force --sign "$DEVELOPER_ID" \
    --options runtime \
    --entitlements "$ROOT/scripts/entitlements.plist" \
    "$BUNDLE_DIR/$APP_NAME.app"

echo "==> Creating DMG..."
hdiutil create -volname "$APP_NAME" \
    -srcfolder "$BUNDLE_DIR/$APP_NAME.app" \
    -ov -format UDZO \
    "$ROOT/target/$DMG_NAME"

echo "==> Notarizing..."
xcrun notarytool submit "$ROOT/target/$DMG_NAME" \
    --apple-id "your@email.com" \
    --team-id "TEAMID" \
    --password "${NOTARIZE_PASSWORD:?NOTARIZE_PASSWORD not set}" \
    --wait

echo "==> Stapling..."
xcrun stapler staple "$ROOT/target/$DMG_NAME"

echo "==> Release ${VERSION} packaged: target/${DMG_NAME}"
