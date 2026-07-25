#!/bin/bash
# Reproducibly build the open-source Wine runtime used by GameRunner.
#
# Prerequisites:
#   brew install autoconf bison flex freetype gnutls mingw-w64 pkg-config
#
# Usage:
#   bash wine/build.sh [11.10]

set -euo pipefail

WINE_VERSION="${1:-11.10}"
SUPPORTED_VERSION="11.10"
WINE_SHA256="e4c35ebe26f4f8eef5f2143e24a1fb9fd103f1d46132ad4755479227d086b8e7"
STAGING_SHA256="102b9d401d9286a654eb437fdd68b02bd4c007532a203dea66367e55e7287e89"

if [[ "$WINE_VERSION" != "$SUPPORTED_VERSION" ]]; then
    echo "ERROR: No pinned source checksums for Wine $WINE_VERSION." >&2
    exit 1
fi

ROOT="$(cd "$(dirname "$0")" && pwd)"
WORK="${GAMERUNNER_WINE_WORK_DIR:-$ROOT/build}"
SOURCE="$WORK/wine-$WINE_VERSION"
STAGING="$WORK/wine-staging-$WINE_VERSION"
INSTALL="$WORK/install-$WINE_VERSION"
ARCHIVES="$WORK/archives"
WINE_ARCHIVE="$ARCHIVES/wine-$WINE_VERSION.tar.xz"
STAGING_ARCHIVE="$ARCHIVES/wine-staging-$WINE_VERSION.tar.gz"
WINE_SERIES="${WINE_VERSION%%.*}.x"

mkdir -p "$ARCHIVES"

download_verified() {
    local url="$1"
    local output="$2"
    local expected="$3"

    if [[ ! -f "$output" ]]; then
        curl -L --fail --retry 5 --output "$output" "$url"
    fi
    local actual
    actual="$(shasum -a 256 "$output" | awk '{print $1}')"
    if [[ "$actual" != "$expected" ]]; then
        echo "ERROR: checksum mismatch for $output" >&2
        echo "expected: $expected" >&2
        echo "actual:   $actual" >&2
        exit 1
    fi
}

download_verified \
    "https://dl.winehq.org/wine/source/$WINE_SERIES/wine-$WINE_VERSION.tar.xz" \
    "$WINE_ARCHIVE" \
    "$WINE_SHA256"
download_verified \
    "https://github.com/wine-staging/wine-staging/archive/refs/tags/v$WINE_VERSION.tar.gz" \
    "$STAGING_ARCHIVE" \
    "$STAGING_SHA256"

if [[ ! -d "$SOURCE" ]]; then
    tar -xJf "$WINE_ARCHIVE" -C "$WORK"
fi
if [[ ! -d "$STAGING" ]]; then
    tar -xzf "$STAGING_ARCHIVE" -C "$WORK"
fi

if [[ ! -f "$SOURCE/.gamerunner-staging-applied" ]]; then
    python3 "$STAGING/staging/patchinstall.py" \
        --destdir="$SOURCE" \
        --backend=patch \
        --all
    for patch_file in "$ROOT"/patches/*.patch; do
        [[ -f "$patch_file" ]] || continue
        patch -d "$SOURCE" -p1 < "$patch_file"
    done
    touch "$SOURCE/.gamerunner-staging-applied"
fi

rm -rf "$INSTALL"
mkdir -p "$INSTALL"

(
    cd "$SOURCE"
    arch -x86_64 ./configure \
        --prefix= \
        --enable-win64 \
        --disable-tests \
        --without-alsa \
        --without-capi \
        --without-cups \
        --without-dbus \
        --without-gphoto \
        --without-gssapi \
        --without-oss \
        --without-pulse \
        --without-sane \
        --without-sdl \
        --without-udev \
        --without-v4l2 \
        --without-vkd3d \
        --without-x \
        --with-coreaudio \
        --with-freetype \
        --with-gnutls \
        --with-metal
    arch -x86_64 make -j"$(sysctl -n hw.logicalcpu)"
    arch -x86_64 make install DESTDIR="$INSTALL"
)

cp "$SOURCE/COPYING.LIB" "$INSTALL/COPYING.LIB"
tar -cJf "$WORK/gamerunner-wine-$WINE_VERSION-macos-x86_64.tar.xz" \
    -C "$INSTALL" .

echo "Built $WORK/gamerunner-wine-$WINE_VERSION-macos-x86_64.tar.xz"
