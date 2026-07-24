import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type { SteamApp, CompatEntry } from '@/lib/types';
import { GameGrid } from '@/components/GameGrid';
import { Button } from '@/components/ui/Button';
import { useSteamGames } from '@/hooks/useSteamGames';
import { useCompat } from '@/hooks/useCompat';
import { launchSteamGame, installSteamCmd, downloadGame, openSteamCmdTerminal, runExe, killWine } from '@/lib/tauri';

interface LibraryViewProps {
  onSelectGame: (game: SteamApp) => void;
}

export function LibraryView({ onSelectGame }: LibraryViewProps) {
  const { games, status, loading, refresh } = useSteamGames();
  const { results, search } = useCompat();
  const [runningGames] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');
  const [appIdInput, setAppIdInput] = useState('');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const [log, setLog] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);
  const [steamUser, setSteamUser] = useState('');
  const [steamPass, setSteamPass] = useState('');

  useEffect(() => {
    const unlisten = listen<string>('steam:install-progress', (event) => {
      setLog((prev) => [...prev, event.payload]);
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [log]);

  const addLog = (msg: string) => setLog((prev) => [...prev, msg]);

  const compatMap: Record<string, CompatEntry> = {};
  for (const e of results) compatMap[e.app_id] = e;

  const handlePlay = async (appId: string) => {
    if (!appId) return;
    setMessage(null);
    try {
      const result = await launchSteamGame(appId, 'steam');
      setMessage({ text: result, ok: true });
    } catch (err) {
      setMessage({ text: String(err), ok: false });
    }
  };

  const handleSetupSteamCmd = async () => {
    setBusy(true); setMessage(null); setLog([]);
    addLog('> Installing SteamCMD...');
    try {
      await installSteamCmd();
      addLog('> SteamCMD ready.');
      setMessage({ text: 'SteamCMD ready.', ok: true });
      refresh();
    } catch (err) {
      addLog(`! ${String(err)}`);
      setMessage({ text: String(err), ok: false });
    } finally { setBusy(false); }
  };

  const handleOpenTerminal = async () => {
    setMessage(null);
    addLog('> Opening SteamCMD terminal...');
    try {
      await openSteamCmdTerminal();
      addLog('> Terminal opened. Type: login YOUR_USERNAME');
    } catch (err) {
      setMessage({ text: String(err), ok: false });
    }
  };

  const handleRunExe = async () => {
    const selected = await open({
      filters: [{ name: 'Windows Executable', extensions: ['exe'] }],
      title: 'Select a Windows .exe to run',
    });
    if (!selected) return;
    setMessage(null);
    try {
      await runExe(selected as string);
      setMessage({ text: 'Launched.', ok: true });
    } catch (err) {
      setMessage({ text: String(err), ok: false });
    }
  };

  const handleDownload = async () => {
    const id = appIdInput.trim();
    if (!id) return;
    setBusy(true); setMessage(null); setLog([]);
    addLog(`> Downloading App ${id}...`);
    try {
      await downloadGame(id, steamUser || undefined, steamPass || undefined);
      addLog(`> App ${id} complete.`);
      setMessage({ text: `Downloaded.`, ok: true });
      setAppIdInput('');
      refresh();
    } catch (err) {
      addLog(`! ${String(err)}`);
      setMessage({ text: String(err), ok: false });
    } finally { setBusy(false); }
  };

  const steamcmdInstalled = status?.steamcmd_installed ?? false;
  const filteredGames = searchQuery
    ? games.filter((g) => g.name.toLowerCase().includes(searchQuery.toLowerCase()))
    : games;

  const inputClass = cn(
    'px-2 py-1 text-[11px]',
    'bg-[#222] border-2',
    'border-[#111] border-b-[#444] border-r-[#444]',
    'text-[#e0e0d0] placeholder:text-[#555]',
    'shadow-[inset_0_2px_3px_rgba(0,0,0,0.4)]',
    'focus:outline-none focus:border-[#7c9c2e]',
  );

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-4 drag-region">
        <div>
          <h1 className="text-[13px] font-bold uppercase tracking-wider text-[#e0e0d0] text-shadow">Library</h1>
          <p className="text-[10px] text-[#808070] mt-0.5">
            {games.length} game{games.length !== 1 ? 's' : ''} installed
          </p>
        </div>
        <div className="flex items-center gap-2 no-drag">
          <input type="text" placeholder="Search games..." value={searchQuery}
            onChange={(e) => { setSearchQuery(e.target.value); if (e.target.value) search(e.target.value); }}
            className={cn(inputClass, 'w-44')} />
          <Button variant="danger" size="sm" onClick={async () => { await killWine(); addLog('> Killed all Wine processes.'); }}>Kill All</Button>
          <Button variant="secondary" size="sm" onClick={handleRunExe}>Run .exe</Button>
          {!steamcmdInstalled ? (
            <Button onClick={handleSetupSteamCmd} disabled={busy}>
              {busy ? '...' : 'Setup SteamCMD'}
            </Button>
          ) : (
            <>
              <Button variant="secondary" size="sm" onClick={handleOpenTerminal}>Terminal</Button>
              <Button variant="secondary" size="sm" onClick={refresh}>Refresh</Button>
            </>
          )}
        </div>
      </div>

      {/* SteamCMD download bar */}
      {steamcmdInstalled && (
        <div className="mb-3 p-3 bg-[linear-gradient(180deg,#3e3e3e_0%,#333_100%)] border-2 border-[#4a4a4a] border-t-[#5a5a5a] shadow-[inset_0_1px_0_rgba(255,255,255,0.03),0_2px_3px_rgba(0,0,0,0.3)] space-y-2">
          <div className="flex gap-2">
            <input type="text" placeholder="Username" value={steamUser}
              onChange={(e) => setSteamUser(e.target.value)} className={cn(inputClass, 'w-40')} />
            <input type="password" placeholder="Password" value={steamPass}
              onChange={(e) => setSteamPass(e.target.value)} className={cn(inputClass, 'w-36')} />
          </div>
          <div className="flex items-center gap-2">
            <span className="text-[11px] text-[#a0a090] font-bold uppercase tracking-wide">App ID:</span>
            <input type="text" placeholder="e.g. 739630" value={appIdInput}
              onChange={(e) => setAppIdInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleDownload()}
              className={cn(inputClass, 'flex-1')} />
            <Button onClick={handleDownload} disabled={busy || !appIdInput.trim()}>
              {busy ? '...' : 'Download'}
            </Button>
          </div>
        </div>
      )}

      {/* Message */}
      {message && (
        <div className={`mb-3 px-3 py-2 text-[11px] font-bold border-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] ${
          message.ok
            ? 'bg-[#2a3a1a] border-[#5a8f3c] text-[#8cc85c]'
            : 'bg-[#3a1a1a] border-[#aa3333] text-[#ff6666]'
        }`}>
          {message.text}
          <button className="ml-2 underline opacity-70 hover:opacity-100" onClick={() => setMessage(null)}>Dismiss</button>
        </div>
      )}

      {/* Console log */}
      {log.length > 0 && (
        <div ref={logRef}
          className="mb-3 max-h-36 overflow-auto font-mono text-[10px] leading-relaxed
                     bg-[#0a0a0a] border-2 border-[#1a1a1a] p-2.5
                     shadow-[inset_0_2px_4px_rgba(0,0,0,0.5)]">
          {log.map((line, i) => (
            <div key={i} className={`whitespace-pre-wrap break-all ${line.startsWith('!') ? 'text-[#ff4444]' : line.startsWith('>') ? 'text-[#88cc44]' : 'text-[#33cc33]'}`}>
              {line}
            </div>
          ))}
        </div>
      )}

      {/* Game grid */}
      <div className="flex-1 overflow-auto -mx-1 px-1">
        {loading ? (
          <div className="flex items-center justify-center h-48 text-[#707060] text-[11px]">Loading...</div>
        ) : filteredGames.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 text-[#707060] gap-2">
            <p className="text-[11px] font-bold uppercase tracking-wider">No games found</p>
            <p className="text-[10px]">
              {!steamcmdInstalled ? 'Click Setup SteamCMD to get started.' : 'Enter an App ID above and click Download.'}
            </p>
          </div>
        ) : (
          <GameGrid games={filteredGames} compatMap={compatMap} runningGames={runningGames}
            onPlay={handlePlay} onSelect={onSelectGame} />
        )}
      </div>
    </div>
  );
}

function cn(...args: (string | undefined | false | null)[]): string {
  return args.filter(Boolean).join(' ');
}
