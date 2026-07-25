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

VERSION="${1:-0.1.0}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
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
