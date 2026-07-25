import { invoke } from '@tauri-apps/api/core';
import type {
  BottleInfo,
  BottleConfig,
  CompatEntry,
  CompatReport,
  DllOverrideConfig,
  GameProcess,
  GraphicsBackendInfo,
  SteamApp,
  SteamStatus,
  WineVersion,
} from './types';

// ── Bottle commands ──────────────────────────────────────────

export async function listBottles(): Promise<BottleInfo[]> {
  return invoke('bottle_list');
}

export async function createBottle(
  name: string,
  wineVersion: string,
  steamAppId?: string,
): Promise<BottleInfo> {
  return invoke('bottle_create', { name, wineVersion, steamAppId });
}

export async function getOrCreateSteamBottle(
  appId: string,
  appName: string,
  wineVersion: string,
): Promise<BottleInfo> {
  return invoke('bottle_get_or_create_for_steam_app', { appId, appName, wineVersion });
}

export async function deleteBottle(id: string): Promise<void> {
  return invoke('bottle_delete', { id });
}

export async function getBottleConfig(id: string): Promise<BottleConfig> {
  return invoke('bottle_get_config', { id });
}

export async function saveBottleConfig(id: string, config: BottleConfig): Promise<void> {
  return invoke('bottle_save_config', { id, config });
}

export async function installBottleRuntime(bottleId: string, runtimeId: string): Promise<void> {
  return invoke('bottle_install_runtime', { bottleId, runtimeId });
}

// ── Graphics commands ────────────────────────────────────────

export async function detectBackends(): Promise<GraphicsBackendInfo[]> {
  return invoke('graphics_detect_backends');
}

export async function buildDllOverrides(
  backend: string,
  overrides: DllOverrideConfig,
): Promise<string> {
  return invoke('graphics_build_dll_overrides', { backend, overrides });
}

// ── Steam commands ───────────────────────────────────────────

export async function steamStatus(): Promise<SteamStatus> {
  return invoke('steam_status');
}

export async function listSteamGames(): Promise<SteamApp[]> {
  return invoke('steam_list_games');
}

export async function installSteam(setupPath: string): Promise<void> {
  return invoke('steam_install', { setupPath });
}

export async function installSteamCmd(): Promise<void> {
  return invoke('steamcmd_install');
}

export async function openSteamCmdTerminal(): Promise<void> {
  return invoke('steamcmd_open_terminal');
}

export async function downloadGame(
  appId: string,
  username?: string,
  password?: string,
): Promise<void> {
  return invoke('steamcmd_download', { appId, username, password });
}

export async function launchSteamClient(): Promise<void> {
  return invoke('steam_run_client');
}

export async function launchSteamGame(appId: string, bottleId: string): Promise<string> {
  return invoke('steam_launch_game', { appId, bottleId });
}

export async function runExe(path: string, bottleId: string): Promise<void> {
  return invoke('run_exe', { path, bottleId });
}

export async function killWine(): Promise<void> {
  return invoke('kill_wine');
}

// ── Wine commands ────────────────────────────────────────────

export async function listWineVersions(): Promise<WineVersion[]> {
  return invoke('wine_list_versions');
}

export async function installWineVersion(version: string): Promise<void> {
  return invoke('wine_install_version', { version });
}

export async function getDefaultWineVersion(): Promise<string> {
  return invoke('wine_get_default_version');
}

// ── Process commands ─────────────────────────────────────────

export async function listProcesses(): Promise<GameProcess[]> {
  return invoke('process_list');
}

export async function signalProcess(pid: number, signal: 'term' | 'kill'): Promise<void> {
  return invoke('process_signal', { pid, signal });
}

// ── Compat commands ──────────────────────────────────────────

export async function compatLookup(appId: string): Promise<CompatEntry | null> {
  return invoke('compat_lookup', { appId });
}

export async function compatSearch(query: string): Promise<CompatEntry[]> {
  return invoke('compat_search', { query });
}

export async function submitCompatReport(report: CompatReport): Promise<void> {
  return invoke('compat_submit_report', { report });
}
