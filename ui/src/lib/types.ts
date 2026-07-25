// Shared types mirroring the Rust backend structs.

export interface BottleInfo {
  id: string;
  name: string;
  wine_version: string;
  prefix_path: string;
  config_path: string;
  created_at: string;
  steam_app_id: string | null;
  status: BottleStatus;
}

export type BottleStatus = 'idle' | 'running' | 'crashed' | 'configuring';

export interface BottleConfig {
  bottle: { name: string; wine_version: string };
  graphics: {
    backend: GraphicsBackend;
    dxgi_override_native: boolean;
    d3d10_override_native: boolean;
    d3d11_override_native: boolean;
    d3d12_override_native: boolean;
  };
  windows: { version: string; virtual_desktop: string | null };
  environment: Record<string, string>;
  steam: { app_id: string | null; launch_options: string | null };
  dependencies: { vcrun: string[]; native_dlls: string[] };
}

export type GraphicsBackend = 'd3dmetal' | 'dxmt' | 'dxvk' | 'wine3d';

export interface GraphicsBackendInfo {
  backend: GraphicsBackend;
  name: string;
  description: string;
  installed: boolean;
  dll_paths: string[];
}

export interface SteamApp {
  app_id: string;
  name: string;
  install_dir: string;
  size_bytes: number | null;
  installed: boolean;
}

export interface SteamStatus {
  bottle_created: boolean;
  steam_installed: boolean;
  steamcmd_installed: boolean;
  steam_running: boolean;
  logged_in_user: string | null;
}

export interface GameProcess {
  pid: number;
  bottle_id: string;
  steam_app_id: string | null;
  command: string;
  started_at: string;
  status: ProcessStatus;
  exit_code: number | null;
}

export type ProcessStatus = 'running' | 'exited' | 'crashed' | 'killed';

export interface CompatEntry {
  app_id: string;
  name: string;
  rating: CompatRating;
  last_tested_wine: string | null;
  last_tested_date: string | null;
  graphics: { backend: string; notes: string | null } | null;
  fixes: CompatFix[];
  known_issues: string[];
}

export type CompatRating = 'platinum' | 'gold' | 'silver' | 'bronze' | 'borked' | 'unknown';

export interface CompatReport {
  app_id: string;
  app_name: string;
  rating: CompatRating;
  wine_version: string;
  graphics_backend: string;
  mac_model: string;
  macos_version: string;
  notes: string;
  submitted_at: string;
}

export interface CompatFix {
  type: 'dll_override' | 'env' | 'launch_option' | 'registry' | 'winetricks';
  dll?: string;
  mode?: string;
  key?: string;
  value?: string;
  verb?: string;
}

export interface WineVersion {
  version: string;
  path: string;
  wine_bin: string;
  wineserver_bin: string;
  is_default: boolean;
  arch: string;
  installed: boolean;
  source: string;
}

export interface DllOverrideConfig {
  dxgi: boolean;
  d3d9: boolean;
  d3d10: boolean;
  d3d11: boolean;
  d3d12: boolean;
}
