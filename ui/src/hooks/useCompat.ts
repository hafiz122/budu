import { useState, useCallback } from 'react';
import type { CompatEntry } from '@/lib/types';
import { compatLookup, compatSearch } from '@/lib/tauri';

export function useCompat() {
  const [entry, setEntry] = useState<CompatEntry | null>(null);
  const [results, setResults] = useState<CompatEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const lookup = useCallback(async (appId: string) => {
    setLoading(true);
    try {
      const e = await compatLookup(appId);
      setEntry(e);
      return e;
    } finally {
      setLoading(false);
    }
  }, []);

  const search = useCallback(async (query: string) => {
    setLoading(true);
    try {
      const r = await compatSearch(query);
      setResults(r);
      return r;
    } finally {
      setLoading(false);
    }
  }, []);

  return { entry, results, loading, lookup, search };
}
