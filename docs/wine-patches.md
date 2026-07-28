# Wine and Steam compatibility maintenance

Budu uses Wine Staging 11.10 as its stable default, with a private Wine
Staging 11.13 test runtime available only for Raft. Both use one small,
source-available macOS patch plus a separate SteamWebHelper launcher.

## DXMT window integration

Wine's macOS driver hides the private Cocoa-window hooks that DXMT needs to put
a Metal layer inside a game window. DXMT 0.74 also consumes a private structure
layout used by FOSS CrossOver Wine. The local patch is ported and verified
against the Wine Staging 11.13 source used by the Raft test runtime.

`wine/patches/0001-winemac-dxmt-compat.patch`:

1. Exports only the five window/view functions used by DXMT.
2. Gives each top-level window a persistent, client-sized Cocoa view before
   the first GDI presentation, which is when DXMT creates its first swap chain.
3. Exposes DXMT's narrow macdrv function table from `ntdll` and forwards each
   call through the exact, already-loaded `winemac.so` handle.
4. Leaves the graphics implementation in DXMT; it does not import CrossOver
   code or binaries.

The four affected Unix runtime modules are bundled under
`runtime/dist/wine-11.10` and `runtime/dist/wine-11.13`. Budu selects the
matching bridge for the downloaded runtime and refuses to overlay the 11.10
bridge onto 11.13. Originals are retained beside each file with a
`.gamerunner-original` suffix. The 11.13 overlay can be reproduced from WineHQ
and Wine Staging sources with `wine/build.sh`.

## Raft-only Wine 11.13 test

This is deliberately not the default runtime. Settings can install Wine
Staging 11.13 and assign it only to Raft's `steam-648800` bottle. Before each
Raft launch, Budu runs Wine `ipconfig /all` in that bottle and requires a
non-loopback adapter with both IPv4 and a default gateway. If detection fails,
the game does not launch and the exact runtime check is shown instead of
silently falling back to Wine 11.10.

The test also warns when `/Library/Frameworks/GStreamer.framework` is absent.
Budu does not bundle or install GStreamer.

To roll back: close Raft and Steam, open Settings, select **Roll Back Raft to
Wine 11.10**, then relaunch. No other bottle is changed.

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
