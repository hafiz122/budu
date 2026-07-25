# Budu runtime components

Budu's application code is MIT-licensed. Its downloadable compatibility
runtime is composed only of redistributable open-source components:

- Wine: LGPL-2.1-or-later
- DXMT: LGPL-2.1-or-later
- SteamWebHelper compatibility shim: MIT

The shim is built from
[`steamwebhelper-shim/steamwebhelper.c`](steamwebhelper-shim/steamwebhelper.c).
It does not bypass Steam authentication or DRM. It only injects Chromium
software-compositing arguments into `steamwebhelper.exe`, working around
Wine/macOS's missing cross-process presentation path. Budu preserves the
original Valve executable beside the shim and refreshes the backup whenever
Steam updates it.

Build the shim with:

```bash
brew install mingw-w64
bash scripts/build-steam-shim.sh
```

The distributable binary lives in `runtime/dist/`. It is checked in so a normal
Rust build does not require a cross-compiler; maintainers can reproduce it from
the adjacent source with the command above.

`runtime/dist/wine-11.10/` contains the four small Wine Unix modules affected
by Budu's DXMT window patch. They are built only from WineHQ, Wine
Staging, and [`../wine/patches/0001-winemac-dxmt-compat.patch`](../wine/patches/0001-winemac-dxmt-compat.patch).
Rebuild them—or the full runtime—with `bash wine/build.sh 11.10`.
