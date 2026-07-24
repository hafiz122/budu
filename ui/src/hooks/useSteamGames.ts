import { useState, useCallback, useEffect } from 'react';
import type { SteamApp, SteamStatus } from '@/lib/types';
import { listSteamGames, steamStatus } from '@/lib/tauri';

export function useSteamGames() {
  const [games, setGames] = useState<SteamApp[]>([]);
  const [status, setStatus] = useState<SteamStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const [s, g] = await Promise.all([steamStatus(), listSteamGames()]);
      setStatus(s);
      setGames(g);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { games, status, loading, error, refresh };
}
