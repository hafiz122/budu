#!/bin/bash
# Build Wine for macOS (x86-64) with staging patches.
# This script is intended to run in CI or on a macOS build machine.
#
# Prerequisites:
#   - Xcode Command Line Tools
#   - Homebrew: bison, flex, mingw-w64, pkg-config
#
# Usage: bash wine/build.sh [WINE_VERSION]

set -euo pipefail

WINE_VERSION="${1:-9.14}"
BUILD_DIR="$(cd "$(dirname "$0")" && pwd)"
WORK_DIR="$BUILD_DIR/build"
SRC_DIR="$WORK_DIR/wine-${WINE_VERSION}"
INSTALL_DIR="$WORK_DIR/install"
PATCHES_DIR="$BUILD_DIR/patches"

echo "==> Building Wine ${WINE_VERSION} (staging) for macOS x86-64"

mkdir -p "$WORK_DIR"

# Download source if not present
if [ ! -d "$SRC_DIR" ]; then
    echo "==> Downloading Wine ${WINE_VERSION} source..."
    curl -fsSL "https://dl.winehq.org/wine/source/9.x/wine-${WINE_VERSION}.tar.xz" \
        | tar -xJ -C "$WORK_DIR"

    echo "==> Downloading Wine-Staging ${WINE_VERSION} patches..."
    curl -fsSL "https://github.com/wine-staging/wine-staging/archive/refs/tags/v${WINE_VERSION}.tar.gz" \
        | tar -xz -C "$WORK_DIR"

    # Apply staging patches
    cd "$SRC_DIR"
    "../wine-staging-${WINE_VERSION}/patches/patchinstall.sh" DESTDIR="$SRC_DIR" --all

    # Apply macOS-specific patches
    if [ -d "$PATCHES_DIR" ]; then
        echo "==> Applying macOS-specific patches..."
        for patch in "$PATCHES_DIR"/*.patch; do
            [ -f "$patch" ] || continue
            echo "    Applying $(basename "$patch")..."
            patch -p1 < "$patch"
        done
    fi
fi

# Configure and build
cd "$SRC_DIR"

echo "==> Configuring Wine..."
./configure \
    --prefix="$INSTALL_DIR" \
    --enable-win64 \
    --disable-tests \
    --without-alsa \
    --without-capi \
    --without-cups \
    --without-dbus \
    --without-gphoto \
    --without-gsm \
    --without-ldap \
    --without-oss \
    --without-pulse \
    --without-sane \
    --without-sdl \
    --without-udev \
    --without-v4l2 \
    --without-vkd3d \
    --with-coreaudio \
    --with-gstreamer \
    --with-metal \
    --with-opengl \
    --with-x

echo "==> Building Wine (this takes a while)..."
make -j"$(sysctl -n hw.logicalcpu)"

echo "==> Installing to ${INSTALL_DIR}..."
make install

echo "==> Wine ${WINE_VERSION} built successfully"
echo "    Install location: ${INSTALL_DIR}"
