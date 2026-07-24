import { useState, useCallback, useEffect, useRef } from 'react';
import type { GameProcess } from '@/lib/types';
import { listProcesses, signalProcess } from '@/lib/tauri';

/** Polls the process supervisor for active game processes. */
export function useGameProcess(bottleId?: string) {
  const [processes, setProcesses] = useState<GameProcess[]>([]);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const refresh = useCallback(async () => {
    try {
      const all = await listProcesses();
      setProcesses(
        bottleId ? all.filter((p) => p.bottle_id === bottleId) : all,
      );
    } catch {
      // Process listing may fail if daemon is not ready; silent ignore.
    }
  }, [bottleId]);

  const kill = useCallback(async (pid: number) => {
    await signalProcess(pid, 'term');
    await refresh();
  }, [refresh]);

  useEffect(() => {
    refresh();
    intervalRef.current = setInterval(refresh, 3000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [refresh]);

  return { processes, refresh, kill };
}
