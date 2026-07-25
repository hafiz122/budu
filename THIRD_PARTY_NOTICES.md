# Third-party notices

Budu's source code and SteamWebHelper compatibility shim are distributed
under the GNU General Public License, version 3.0 only.

Budu can download the following independent runtime components. They are
not relicensed under GPL-3.0-only.

## Patched Wine Staging 11.10

- Project: https://www.winehq.org/
- macOS build: https://github.com/Gcenx/macOS_Wine_builds/releases/tag/11.10
- License: GNU Lesser General Public License, version 2.1 or later
- Pinned archive SHA-256:
  `940bdd1a177872020be01c5c33917cb8eecc1cc3193ad554914fb6efd90d7889`

Wine's license text and corresponding source are available from the upstream
Wine repository and inside Wine source distributions. Budu's macOS DXMT
compatibility patch is included at
`wine/patches/0001-winemac-dxmt-compat.patch` and is distributed under Wine's
LGPL-2.1-or-later terms. The corresponding four patched Unix modules are
bundled under `runtime/dist/wine-11.10` and can be reproduced with
`wine/build.sh`. They contain no CrossOver application code or binaries.

## DXMT 0.74

- Project and source: https://github.com/3Shain/dxmt
- Release: https://github.com/3Shain/dxmt/releases/tag/v0.74
- License: GNU Lesser General Public License, version 2.1 or later
- Pinned archive SHA-256:
  `2598981a8b725653773e277470a95dda4253b8a14d36e0dc96dce0e3800f0ceb`

## Steam and SteamCMD

Steam and SteamCMD are Valve software and are not part of Budu. Users
install or download them separately and remain responsible for Valve's terms
and each game's license. Budu does not include a Steam API emulator or
modify game ownership checks.

## Optional backends

The UI can detect other locally installed graphics backends. Budu does
not distribute Apple's D3DMetal or the CrossOver application.
