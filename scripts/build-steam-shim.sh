#!/bin/bash
# Build the auditable Windows helper used to render Steam's CEF interface.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE="$ROOT/runtime/steamwebhelper-shim/steamwebhelper.c"
OUTPUT_DIR="$ROOT/runtime/dist"
OUTPUT="$OUTPUT_DIR/steamwebhelper-shim.exe"
COMPILER="${MINGW_CC:-x86_64-w64-mingw32-gcc}"

if ! command -v "$COMPILER" >/dev/null 2>&1; then
    echo "ERROR: $COMPILER is required to build the Steam compatibility shim." >&2
    echo "Install it with: brew install mingw-w64" >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR"
"$COMPILER" \
    -std=c11 \
    -Os \
    -s \
    -municode \
    -mwindows \
    -frandom-seed=gamerunner-steam-shim \
    -Wl,--no-insert-timestamp \
    -Wall \
    -Wextra \
    -Werror \
    "$SOURCE" \
    -lshell32 \
    -o "$OUTPUT"

echo "Built $OUTPUT"
