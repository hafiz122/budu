#!/bin/bash
# Reproducibly build the open-source Wine runtime used by GameRunner.
#
# Prerequisites (run from an Intel/Rosetta Homebrew shell):
#   arch -x86_64 /usr/local/bin/brew install autoconf bison flex
#
# This builds only Budu's x86_64 DXMT bridge overlay. It is installed over the
# official Gcenx Wine Staging archive at runtime, so it does not replace or
# redistribute the full Wine build.
#
# Usage:
#   arch -x86_64 bash wine/build.sh [11.13]

set -euo pipefail

WINE_VERSION="${1:-11.13}"
SUPPORTED_VERSION="11.13"
WINE_SHA256="9548390c5042126b6ecf0af2cba477b75aa3e99be9797ea70d5026374c5074f1"
STAGING_SHA256="59738ad7f2ca72f4b21a34a1a0edbfb7e60f2d8fd50e337391cf5fdb1cc8d4f0"

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
        --build=x86_64-apple-darwin \
        --enable-win64 \
        --enable-archs=none \
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
        --without-freetype \
        --without-gnutls \
        --without-mingw \
        --with-coreaudio
    arch -x86_64 make -j"$(sysctl -n hw.logicalcpu)" \
        loader/wine tools/wine/wine dlls/ntdll/ntdll.so dlls/winemac.drv/winemac.so
    arch -x86_64 make install DESTDIR="$INSTALL"
)

file "$INSTALL/bin/wine" "$INSTALL/lib/wine/x86_64-unix/ntdll.so" | grep -q x86_64
tar -cJf "$WORK/budu-wine-$WINE_VERSION-dxmt-bridge-macos-x86_64.tar.xz" -C "$INSTALL" .

echo "Built $WORK/budu-wine-$WINE_VERSION-dxmt-bridge-macos-x86_64.tar.xz"
