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

The checked-in PE binary is reproducible from its GPL-3.0-only C source:

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

## Rebuilding the Raft Wine 11.13 bridge

The normal app downloads a checksum-pinned Gcenx Wine archive. The Raft test
runtime additionally needs a source-built x86_64 bridge whose Wine and Staging
sources are pinned in `wine/build.sh`. Use an Intel/Rosetta shell and Intel
Homebrew dependencies:

```bash
arch -x86_64 /usr/local/bin/brew install autoconf bison flex
arch -x86_64 bash wine/build.sh 11.13
```

The script downloads checksum-pinned upstream sources into the ignored
`wine/build/` directory, applies Wine Staging and Budu's patch, and produces
only the four x86_64 overlay modules. Copy those files to
`runtime/dist/wine-11.13` after verifying their architecture and symbols. The
full Wine runtime remains the checksum-verified upstream archive.

Wine 11.10 remains the default. The 11.13 runtime is a private Raft test only:
do not publish it or claim a multiplayer fix until a real friend-world join has
succeeded.

Local builds are ad-hoc signed but not notarized. For public distribution,
release maintainers should use a Developer ID and the notarization variables
described in `scripts/package-release.sh`.
