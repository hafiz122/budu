# Wine and Steam compatibility maintenance

Budu uses Wine Staging 11.10 with one small, source-available macOS patch,
plus a separate SteamWebHelper launcher. Keeping both workarounds narrow makes
them auditable and independently replaceable.

## DXMT window integration

Wine's macOS driver hides the private Cocoa-window hooks that DXMT needs to put
a Metal layer inside a game window. DXMT 0.74 also consumes a private structure
layout used by FOSS CrossOver Wine, which differs from upstream Wine 11.10.

`wine/patches/0001-winemac-dxmt-compat.patch`:

1. Exports only the five window/view functions used by DXMT.
2. Gives each top-level window a persistent, client-sized Cocoa view before
   the first GDI presentation, which is when DXMT creates its first swap chain.
3. Exposes DXMT's narrow macdrv function table from `ntdll` and forwards each
   call through the exact, already-loaded `winemac.so` handle.
4. Leaves the graphics implementation in DXMT; it does not import CrossOver
   code or binaries.

The four affected Unix runtime modules are bundled under
`runtime/dist/wine-11.10` so Budu can patch a newly downloaded managed
Wine runtime before installing DXMT. Originals are retained beside each file
with a `.gamerunner-original` suffix. The same binaries can be reproduced from
WineHQ and Wine Staging sources with `wine/build.sh`.

## Steam CEF black window

Modern Steam renders its interface through Chromium processes. On macOS,
mainline Wine lacks the cross-process presentation path needed to display the
GPU process' surface in the browser window.

Budu's workaround:

1. Preserve Valve's current helper as
   `steamwebhelper.gamerunner-original.exe`.
2. Put Budu's open-source shim at Steam's expected helper path.
3. Start Steam with `-noverifyfiles`, preventing the startup verifier from
   immediately replacing the shim.
4. Have the shim launch the original helper with
   `--no-sandbox --in-process-gpu --disable-gpu`.
5. Detect a helper replaced by a Steam update, refresh the backup, and reapply
   the shim on the next Budu launch.

Source: `runtime/steamwebhelper-shim/steamwebhelper.c`

This is a rendering workaround only. Do not add account bypasses, Steam API
emulators, cracked DLLs, or DRM workarounds.

## Updating Steam compatibility

When Steam changes:

1. Restore or allow Steam to install its updated helper.
2. Launch once through Budu.
3. Confirm the backup hash matches Valve's new helper.
4. Confirm the active helper contains the legacy `GameRunner` marker. The
   marker remains stable so existing shim binaries can be detected after the
   Budu rename.
5. Inspect the actual child command line and verify the three compositor
   arguments are present.
6. Test both the sign-in/profile window and a real game launch.

If Valve removes support for the compositor arguments, the long-term fallback
is an LGPL DXMT/winemac IOSurface cross-process presentation implementation.
That should be developed as a separate, reviewable patch series and offered
upstream.
