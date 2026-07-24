# Building GameRunner

## Prerequisites

- Rust 1.77+ (https://rustup.rs)
- Node.js 20+ (https://nodejs.org)
- macOS 14+ with Xcode Command Line Tools
- Apple Silicon Mac (the app builds as universal binary, but only runs on Apple Silicon)

Optional for building Wine from source:
- Homebrew packages: `brew install bison flex mingw-w64 pkg-config`

## Development Build

```bash
# First-time setup
make bootstrap

# Start with hot module reload (UI changes instant)
make dev
```

The `make dev` command:
1. Starts the Vite dev server on port 5173 (HMR for React)
2. Launches the Tauri app connecting to the dev server
3. The Rust backend recompiles on changes

## Production Build

```bash
make build
```

This produces:
- `ui/dist/` -- compiled static frontend
- `src-tauri/target/release/bundle/macos/GameRunner.app` -- signed .app bundle

## Building Wine

Wine must be built as x86-64 Mach-O binaries (Rosetta 2 handles the ARM64 translation at runtime):

```bash
bash wine/build.sh 9.14
```

The build script:
1. Downloads Wine + Wine-Staging source
2. Applies staging and macOS-specific patches
3. Configures with Metal, CoreAudio, GStreamer support
4. Builds and installs to `wine/build/install/`

Copy the install directory to `~/.gamerunner/wine/<version>/` for the app to discover it.

## Architecture-specific notes

- **x86-64 Wine on ARM64**: Wine runs as x86-64 via Rosetta 2. No ARM64 port is needed. The `wine64` binary is a Mach-O fat binary with x86-64 slice.
- **Graphics**: D3DMetal (from Apple's GPTK) is the preferred D3D11/12 path. DXVK bridges D3D9/10/11 via Vulkan-to-Metal.
- **esync/fsync**: macOS lacks `eventfd()`. The esync patch replaces it with Mach semaphores.
