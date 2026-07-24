# GameRunner

Run your Windows Steam games on Apple Silicon Macs.

## How It Works

GameRunner wraps [Wine](https://www.winehq.org/) (Windows API translation) and graphics translation layers (D3DMetal, DXVK, MoltenVK) behind a clean macOS interface. It creates isolated "bottles" for each game, manages your Steam library, and applies the optimal configuration automatically.

- **Wine** translates Windows system calls to POSIX/macOS
- **D3DMetal** (Apple's Game Porting Toolkit) translates DirectX 11/12 to Metal
- **DXVK + MoltenVK** translates DirectX 9/10/11 to Vulkan, then Vulkan to Metal
- **Rosetta 2** (built into macOS) translates x86-64 instructions to ARM64

## Requirements

- macOS 14 (Sonoma) or later
- Apple Silicon Mac (M1/M2/M3/M4)
- [Apple Game Porting Toolkit](https://developer.apple.com/download/all/) (for D3DMetal, recommended)

## Quick Start

```bash
git clone https://github.com/user/gamerunner.git
cd gamerunner
make bootstrap
make dev
```

## Project Structure

```
gamerunner/
├── src-tauri/          # Rust backend (Tauri app)
│   └── src/
│       ├── commands/   # Tauri command handlers (IPC boundary)
│       ├── bottle.rs   # Wine bottle management
│       ├── compat.rs   # Compatibility database engine
│       ├── config.rs   # Application configuration
│       ├── graphics.rs # Graphics backend management
│       ├── process_supervisor.rs  # Game process lifecycle
│       ├── runtime.rs  # VC++/DirectX/.NET runtime installers
│       ├── steam_bridge.rs  # Steam client integration
│       └── wine_manager.rs  # Wine version management
├── ui/                 # Web frontend (React + TypeScript + Tailwind)
│   └── src/
│       ├── components/ # Reusable UI components
│       ├── views/      # Page-level views
│       ├── hooks/      # React hooks (Tauri invoke wrappers)
│       ├── lib/        # Utilities, types, theme engine
│       └── styles/     # CSS themes and global styles
├── wine/               # Wine build scripts and macOS patches
├── runtimes/           # Runtime dependency installers
├── compat-db/          # Game compatibility database
├── docs/               # Documentation
└── scripts/            # Build and release scripts
```

## Customizing the UI

The entire UI is a standard web application (React + Tailwind CSS). You can customize every aspect without touching the Rust backend. See [docs/customizing-ui.md](docs/customizing-ui.md) for details.

Quick examples:
- **Change colors/fonts:** Edit `ui/tailwind.config.ts`
- **Add a theme:** Create a CSS file in `ui/src/styles/themes/`
- **Replace components:** Edit files in `ui/src/components/`
- **Swap frameworks:** Replace the `ui/` directory with a Svelte/Vue/Solid project using the same Tauri commands

## License

MIT License. See [LICENSE](LICENSE) for details.
