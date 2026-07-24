# Customizing the UI

The entire GameRunner UI is a standard web application (React + TypeScript + Tailwind CSS). You can change every pixel without touching the Rust backend.

## Quick Wins

### Change Colors and Fonts

Edit `ui/tailwind.config.ts` to override the design tokens:

```ts
// Example: change accent color to green
theme: {
  extend: {
    colors: {
      accent: {
        DEFAULT: '#10b981',
        hover: '#34d399',
        muted: '#10b98133',
      },
      // ... other tokens
    },
  },
},
```

### Add a Theme

1. Create a new CSS file in `ui/src/styles/themes/my-theme.css`:

```css
[data-theme='my-theme'] {
  --color-surface: #1a1a2e;
  --color-surface-alt: #16213e;
  --color-surface-hover: #0f3460;
  --color-accent: #e94560;
  --color-accent-hover: #ff6b81;
  --color-accent-muted: #e9456033;
  --color-text-primary: #eee;
  --color-text-secondary: #aaa;
  --color-text-muted: #666;
  --color-border: #2a2a4a;
  --color-border-focus: #e94560;
  --color-success: #2ecc71;
  --color-warning: #f39c12;
  --color-danger: #e74c3c;
  --radius: 0.5rem;
  --font-sans: 'SF Pro', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
}
```

2. Import it in `ui/src/main.tsx`:

```tsx
import './styles/themes/my-theme.css';
```

3. Register it in `ui/src/lib/theme.ts`:

```ts
export function getAvailableThemes() {
  return [
    { id: 'default', label: 'Default' },
    { id: 'steam', label: 'Steam' },
    { id: 'minimal', label: 'Minimal' },
    { id: 'my-theme', label: 'My Theme' },  // added
  ];
}
```

### Replace a Component

Every component in `ui/src/components/` is self-contained. Open any file, edit the JSX and Tailwind classes, and the changes appear instantly with HMR.

### Replace the Entire Frontend Framework

The UI communicates with the Rust backend exclusively through typed Tauri `invoke()` calls. Any framework that can call these commands works.

Example: to swap React for Svelte:

1. Delete `ui/` directory
2. Create a new SvelteKit project in `ui/`
3. Install `@tauri-apps/api`
4. Call the same Tauri commands:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';

  let bottles = [];

  async function loadBottles() {
    bottles = await invoke('bottle_list');
  }
</script>
```

The Tauri command API is the stable contract. See `src-tauri/src/commands/` for all available commands.

## Architecture

```
UI (React/Svelte/Vue/etc.)
  │
  │  invoke('bottle_list')        emit('game:started')
  ▼                               ▲
Tauri WebKit Bridge (JSON IPC)
  │                               │
  ▼                               ▲
Rust Backend (#[tauri::command])
  │
  ▼
Wine / Steam / Graphics layers
```

## Development Workflow

```bash
# Start with hot module reload
make dev

# Or manually:
cd ui && npm run dev     # terminal 1: Vite dev server
cargo tauri dev          # terminal 2: Tauri + Rust backend
```

UI edits are instant. Rust changes trigger a recompile.
