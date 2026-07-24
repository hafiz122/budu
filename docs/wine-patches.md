# macOS Wine Patches

Wine's Linux focus means several features need patching to work well on macOS. This document tracks the patches maintained in `wine/patches/`.

## Active Patches

### 0001-esync-mach-sem.patch

**Problem:** Wine's `esync` (eventfd-based synchronization) uses Linux's `eventfd()` system call, which does not exist on macOS. Without esync, many games experience severe performance degradation due to slow synchronization primitives.

**Solution:** Replace `eventfd()` calls with Mach semaphores or Grand Central Dispatch semaphores (`dispatch_semaphore_t`). The patch modifies `dlls/ntdll/unix/esync.c` to:

- Use `dispatch_semaphore_create()` instead of `eventfd()`
- Use `dispatch_semaphore_signal()` instead of `write()` to the eventfd
- Use `dispatch_semaphore_wait()` instead of `read()` from the eventfd

**Status:** Required for acceptable game performance. Must be updated for each Wine release.

**Test:** Games that use many synchronization primitives (most D3D11/D3D12 titles) should show stable frame pacing.

### 0002-metal-window-interop.patch

**Problem:** Wine's Mac driver (`winemac.drv`) handles window creation and GDI rendering, but does not coordinate with Metal-based rendering from D3DMetal. This can cause the D3DMetal output to appear in the wrong window or not at all when multiple Wine windows exist.

**Solution:** Patch `winemac.drv` to expose a `CALayer`/`CAMetalLayer` handle that D3DMetal can render into. Coordinate window resize and focus events between the two rendering paths.

**Status:** Needed for D3DMetal integration. May be upstreamed to Wine's Mac driver.

### Future Patches

- **Wine server QoS tuning:** Map Windows thread priorities to macOS QoS classes (`QOS_CLASS_USER_INTERACTIVE`, `QOS_CLASS_USER_INITIATED`, etc.)
- **GPU detection:** Report Apple Silicon GPU capabilities correctly to games that query adapter features
- **HID device passthrough:** Better gamepad support via IOKit HID

## Patch Development

Each patch targets a specific Wine version. When updating Wine:

1. Apply patches incrementally (0001, then 0002, etc.)
2. Fix conflicts manually
3. Rebuild and run the integration test suite
4. Update this document with any changes

## Upstreaming Goal

The long-term goal is to upstream as many patches as possible to Wine proper. Patches that are macOS-specific and non-controversial (esync replacement) are the best candidates. Patches that change Wine's internal APIs (Metal window interop) may need to stay downstream.
