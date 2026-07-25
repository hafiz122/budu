// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LibraryView } from './LibraryView';

const downloadGameMock = vi.hoisted(() => vi.fn());
const openTerminalMock = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

vi.mock('@/hooks/useSteamGames', () => ({
  useSteamGames: () => ({
    games: [],
    status: { steamcmd_installed: true },
    loading: false,
    refresh: vi.fn(),
  }),
}));

vi.mock('@/hooks/useCompat', () => ({
  useCompat: () => ({ results: [], search: vi.fn() }),
}));

vi.mock('@/hooks/useGameProcess', () => ({
  useGameProcess: () => ({ processes: [] }),
}));

vi.mock('@/lib/tauri', () => ({
  downloadGame: downloadGameMock,
  getDefaultWineVersion: vi.fn(),
  getOrCreateSteamBottle: vi.fn(),
  installSteamCmd: vi.fn(),
  killWine: vi.fn(),
  launchSteamGame: vi.fn(),
  openSteamCmdTerminal: openTerminalMock,
  runExe: vi.fn(),
}));

afterEach(() => {
  cleanup();
  downloadGameMock.mockReset();
  openTerminalMock.mockReset();
});

function renderLibrary() {
  render(
    <LibraryView
      bottles={[]}
      onSelectGame={vi.fn()}
      onCreateBottle={vi.fn()}
      onBottlesChanged={vi.fn()}
    />,
  );
}

describe('LibraryView SteamCMD download controls', () => {
  it('does not render a Steam password field', () => {
    renderLibrary();

    expect(screen.queryByPlaceholderText('Password')).toBeNull();
    expect(screen.queryByPlaceholderText('Username')).toBeNull();
  });

  it('sends only an App ID for anonymous downloads', async () => {
    downloadGameMock.mockResolvedValue(undefined);
    renderLibrary();

    fireEvent.change(screen.getByPlaceholderText('Steam App ID, e.g. 739630'), {
      target: { value: '480' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Anonymous Download' }));

    await waitFor(() => expect(downloadGameMock).toHaveBeenCalledWith('480'));
  });
});
