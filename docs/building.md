# Building Budu

## Prerequisites

- Apple Silicon Mac running macOS 14 or newer
- Xcode Command Line Tools
- Rust 1.77 or newer
- Node.js 20 or newer
- Homebrew `mingw-w64` to reproduce the Windows Steam shim

```bash
brew install mingw-w64
make bootstrap
```

## Development

```bash
make dev
```

Tauri builds the Steam shim, starts Vite, compiles the Rust backend, and opens
Budu. The runtime itself is installed through Budu's Settings page
and stored below `~/.gamerunner/`.

## Verification

```bash
make test-all
make lint
```

The full release check is:

```bash
cd src-tauri
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cd ../ui
npm run build
npm run lint
npm run test
```

## Steam shim

The checked-in PE binary is reproducible from its MIT-licensed C source:

```bash
bash scripts/build-steam-shim.sh
shasum -a 256 runtime/dist/steamwebhelper-shim.exe
```

Expected SHA-256:

```text
44d85630c95d3f666fee35cb64d06c7fcf3ed493817bedd107d3e851922eb6e7
```

## Production bundle

```bash
make build
```

Output:

```text
src-tauri/target/release/bundle/macos/Budu.app
src-tauri/target/release/bundle/dmg/Budu_0.1.0_aarch64.dmg
```

The bundle contains the compatibility database, Steam shim, and four small
Wine Unix modules implementing Budu's DXMT window bridge. The rest of
Wine and DXMT are checksum-verified downloads so the app remains reasonably
sized.

## Rebuilding Wine from source

The normal app downloads the pinned Gcenx build. Maintainers can independently
rebuild Wine 11.10 and its staging patch set with:

```bash
brew install autoconf bison flex freetype gnutls mingw-w64 pkg-config
bash wine/build.sh 11.10
```

The script downloads checksum-pinned upstream sources into the ignored
`wine/build/` directory, applies Wine Staging plus any patches in
`wine/patches/`, and produces a redistributable archive with `COPYING.LIB`.
The affected modules checked into `runtime/dist/wine-11.10` come from this
source recipe and are overlaid onto the pinned managed Wine build at runtime.

Local builds are ad-hoc signed but not notarized. For public distribution,
release maintainers should use a Developer ID and the notarization variables
described in `scripts/package-release.sh`.
