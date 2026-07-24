import { useState, useCallback, useEffect } from 'react';
import type { BottleInfo } from '@/lib/types';
import { listBottles, createBottle, deleteBottle } from '@/lib/tauri';

export function useBottles() {
  const [bottles, setBottles] = useState<BottleInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await listBottles();
      setBottles(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const create = useCallback(
    async (name: string, wineVersion: string, steamAppId?: string) => {
      const bottle = await createBottle(name, wineVersion, steamAppId);
      await refresh();
      return bottle;
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      await deleteBottle(id);
      await refresh();
    },
    [refresh],
  );

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { bottles, loading, error, refresh, create, remove };
}
