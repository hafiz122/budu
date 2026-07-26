<p align="center">
  <img src="ui/src/assets/budu-logo.svg" alt="Budu" width="560">
</p>

<p align="center">
  A native macOS launcher for playing your Windows Steam games on Apple Silicon.
</p>

<p align="center">
  <a href="https://github.com/hafiz122/budu/actions/workflows/ci.yml"><img src="https://github.com/hafiz122/budu/actions/workflows/ci.yml/badge.svg" alt="Build status"></a>
</p>

## What is Budu?

Budu helps you install and launch Windows-only Steam games on an Apple Silicon
Mac. It takes care of the less glamorous parts, including downloading Windows
game files through SteamCMD, setting up a Wine bottle for each game, and
configuring the graphics stack used to run it.

The goal is simple: get from your Steam library to a game that launches without
turning every setup step into a terminal project. Budu is open source and does
not require CrossOver.

## What you need

- An Apple Silicon Mac running macOS 14 or later
- Rosetta 2
- A Steam account that owns the game you want to play

Game compatibility will vary. A game may depend on unsupported anti-cheat,
launchers, codecs, or graphics features, so it is worth checking a game before
expecting it to work perfectly.

## Getting started

1. Download Budu from the project's [GitHub Releases page](https://github.com/hafiz122/budu/releases).
2. Open Budu and go to **Settings**.
3. Select **Install Wine** and let Budu prepare the runtime.
4. Install or import a Windows Steam game.
5. Select the game, or choose its `.exe`, then launch it with its managed
   bottle.

SteamCMD downloads are kept in shared storage, while every game gets its own
managed bottle and configuration. This keeps game-specific settings from
spilling into the rest of your library.

## Under the hood

The default runtime combines these open components:

- Wine Staging 11.10
- DXMT 0.74 for Direct3D 10 and 11 translation
- Rosetta 2 for x86-64 execution on Apple Silicon
- A small GPL-3.0-only SteamWebHelper shim that addresses Steam's black-window
  issue under Wine on macOS

Wine and DXMT are downloaded from their public releases when needed, and Budu
checks them against pinned SHA-256 hashes. The app also applies its bundled,
source-reproducible Wine and DXMT window bridge. CrossOver and Apple's
proprietary D3DMetal are not included.

The SteamWebHelper shim does not bypass Steam sign-in, ownership checks, or
DRM. It preserves Valve's executable and starts it with software compositing
to work around a missing Wine/macOS presentation path. Steam updates can
replace the shim, and Budu reinstalls it on the next launch.

## Building from source

For local development, install the following first:

- Rust 1.77 or newer
- Node.js 20 or newer
- Xcode Command Line Tools
- Homebrew `mingw-w64` if you need to rebuild the Steam shim

```bash
git clone https://github.com/hafiz122/budu.git
cd budu
brew install mingw-w64
make bootstrap
make dev
```

Useful commands:

```bash
make test-all  # Run Rust and UI tests
make lint      # Run Rust and UI linters
make build     # Create a production app bundle
```

The built application is available at:

```text
src-tauri/target/release/bundle/macos/Budu.app
```

For a deeper look at local builds, runtime packaging, and rebuilding Wine, see
[docs/building.md](docs/building.md).

## Data and upgrades

Budu stores its data in `~/.gamerunner/`. Updates preserve that directory and
the existing `gamerunner-*` preferences, so bottles made before the project
rename continue to work.

Back up your game saves before trying a pre-release, especially if a game keeps
saves inside its Wine bottle.

## Security and releases

Pre-release builds are experimental and currently unsigned. Only download them
from the official [GitHub Releases page](https://github.com/hafiz122/budu/releases).
Budu is not affiliated with Valve or Steam.

Stable releases are planned to be signed with a Developer ID certificate,
notarized by Apple, and distributed with the notarization ticket attached.

To report a security issue privately, use GitHub's vulnerability reporting
feature. See [SECURITY.md](SECURITY.md) for the reporting policy.

## Project layout

```text
src-tauri/   Rust and Tauri backend
ui/          React and TypeScript frontend
runtime/     Runtime helpers and reproducible Steam shim source
compat-db/   Bundled game compatibility entries
wine/        Wine source-build tooling
scripts/     Development and release scripts
docs/        Architecture and contributor documentation
```

## Contributing

Contributions are welcome. Start with [docs/contributing.md](docs/contributing.md).
For compatibility fixes, please keep the change focused, test it against a
named Wine and Steam version, and document enough detail for the next person to
reproduce it.

## License

Budu is licensed under the [GNU General Public License v3.0 only](LICENSE).
Downloaded runtime components keep their own licenses. See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for details.
