import { afterEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  getOrCreateSteamBottle,
  downloadGame,
  launchSteamGame,
  runExe,
} from './tauri';

describe('Tauri command wrappers', () => {
  afterEach(() => {
    invokeMock.mockReset();
  });

  it('passes the Steam app identity when resolving a bottle', async () => {
    invokeMock.mockResolvedValue({ id: 'steam-730' });

    await getOrCreateSteamBottle('730', 'Counter-Strike 2', '9.14-staging');

    expect(invokeMock).toHaveBeenCalledWith(
      'bottle_get_or_create_for_steam_app',
      {
        appId: '730',
        appName: 'Counter-Strike 2',
        wineVersion: '9.14-staging',
      },
    );
  });

  it('always passes a bottle ID for manual and Steam launches', async () => {
    invokeMock.mockResolvedValue(undefined);

    await runExe('/Games/example.exe', 'manual-bottle');
    await launchSteamGame('730', 'steam-730');

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'run_exe', {
      path: '/Games/example.exe',
      bottleId: 'manual-bottle',
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'steam_launch_game', {
      appId: '730',
      bottleId: 'steam-730',
    });
  });

  it('downloads anonymously with only the Steam App ID', async () => {
    invokeMock.mockResolvedValue(undefined);

    await downloadGame('480');

    expect(invokeMock).toHaveBeenCalledWith('steamcmd_download', { appId: '480' });
  });
});
