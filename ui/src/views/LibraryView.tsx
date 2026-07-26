import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type { BottleInfo, SteamApp, CompatEntry } from '@/lib/types';
import { GameGrid } from '@/components/GameGrid';
import { ManualBottleDialog } from '@/components/ManualBottleDialog';
import { Button } from '@/components/ui/Button';
import { useSteamGames } from '@/hooks/useSteamGames';
import { useCompat } from '@/hooks/useCompat';
import { useGameProcess } from '@/hooks/useGameProcess';
import {
  downloadGame,
  getDefaultWineVersion,
  getOrCreateSteamBottle,
  installSteamCmd,
  killWine,
  launchSteamGame,
  openSteamCmdTerminal,
  runExe,
} from '@/lib/tauri';

interface LibraryViewProps {
  onSelectGame: (game: SteamApp) => void;
  bottles: BottleInfo[];
  onCreateBottle: (name: string, wineVersion: string) => Promise<BottleInfo>;
  onBottlesChanged: () => Promise<void>;
}

export function LibraryView({
  onSelectGame,
  bottles,
  onCreateBottle,
  onBottlesChanged,
}: LibraryViewProps) {
  const { games, status, loading, refresh } = useSteamGames();
  const { results, search } = useCompat();
  const { processes } = useGameProcess();
  const [searchQuery, setSearchQuery] = useState('');
  const [appIdInput, setAppIdInput] = useState('');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const [log, setLog] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);
  const [pendingExe, setPendingExe] = useState<string | null>(null);
  const [selectedBottleId, setSelectedBottleId] = useState('');
  const [newBottleName, setNewBottleName] = useState('');

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
  const runningGames = new Set(
    processes
      .filter((process) => process.status === 'running' && process.steam_app_id)
      .map((process) => process.steam_app_id as string),
  );

  const handlePlay = async (appId: string) => {
    if (!appId) return;
    setMessage(null);
    try {
      const game = games.find((candidate) => candidate.app_id === appId);
      if (!game) throw new Error(`Steam App ${appId} is not in the library`);
      const wineVersion = await getDefaultWineVersion();
      const bottle = await getOrCreateSteamBottle(appId, game.name, wineVersion);
      await onBottlesChanged();
      const result = await launchSteamGame(appId, bottle.id);
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
    const id = appIdInput.trim();
    if (!id) return;
    setMessage(null);
    addLog(`> Opening SteamCMD Terminal for App ${id}...`);
    try {
      const instructions = await openSteamCmdTerminal(id);
      addLog(`> ${instructions}`);
      setMessage({ text: instructions, ok: true });
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
    setPendingExe(selected as string);
    setSelectedBottleId(bottles[0]?.id ?? '');
    setNewBottleName('');
  };

  const handleCreateManualBottle = async () => {
    const name = newBottleName.trim();
    if (!name) return;
    setBusy(true);
    try {
      const wineVersion = await getDefaultWineVersion();
      const bottle = await onCreateBottle(name, wineVersion);
      setSelectedBottleId(bottle.id);
      setNewBottleName('');
    } catch (err) {
      setMessage({ text: String(err), ok: false });
    } finally {
      setBusy(false);
    }
  };

  const handleLaunchManualExe = async () => {
    if (!pendingExe || !selectedBottleId) return;
    setBusy(true);
    try {
      await runExe(pendingExe, selectedBottleId);
      setMessage({ text: 'Launched.', ok: true });
      setPendingExe(null);
    } catch (err) {
      setMessage({ text: String(err), ok: false });
    } finally {
      setBusy(false);
    }
  };

  const handleDownload = async () => {
    const id = appIdInput.trim();
    if (!id) return;
    setBusy(true); setMessage(null); setLog([]);
    addLog(`> Downloading App ${id}...`);
    try {
      await downloadGame(id);
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

  const inputClass = 'mac-input text-[12px]';

  return (
    <div className="flex h-full flex-col">
      {pendingExe && (
        <ManualBottleDialog
          executablePath={pendingExe}
          bottles={bottles}
          selectedBottleId={selectedBottleId}
          newBottleName={newBottleName}
          busy={busy}
          onSelect={setSelectedBottleId}
          onNameChange={setNewBottleName}
          onCreate={handleCreateManualBottle}
          onLaunch={handleLaunchManualExe}
          onCancel={() => setPendingExe(null)}
        />
      )}
      {/* Header */}
      <div className="drag-region mb-5 flex items-end justify-between gap-6">
        <div>
          <h1 className="text-[26px] font-bold tracking-[-0.035em] text-white">Library</h1>
          <p className="mt-1 text-[12px] text-white/40">
            {games.length} installed game{games.length !== 1 ? 's' : ''}
          </p>
        </div>
        <div className="no-drag flex flex-wrap items-center justify-end gap-2">
          <input type="text" placeholder="Search games..." value={searchQuery}
            onChange={(e) => { setSearchQuery(e.target.value); if (e.target.value) search(e.target.value); }}
            className={cn(inputClass, 'w-48')} />
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
          <Button variant="danger" size="sm" onClick={async () => { await killWine(); addLog('> Killed all Wine processes.'); }}>Stop All</Button>
        </div>
      </div>

      {/* SteamCMD download bar */}
      {steamcmdInstalled && (
        <div className="mac-panel mb-4 p-4">
          <div className="mb-3 flex items-center justify-between">
            <div>
              <h2 className="text-[13px] font-semibold text-white/90">Download from Steam</h2>
              <p className="mt-0.5 text-[11px] text-white/35">Anonymous downloads run in Budu. Owned games sign in directly through SteamCMD Terminal.</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <input type="text" placeholder="Steam App ID, e.g. 739630" value={appIdInput}
              onChange={(e) => setAppIdInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleDownload()}
              className={cn(inputClass, 'flex-1')} />
            <Button onClick={handleDownload} disabled={busy || !appIdInput.trim()}>
              {busy ? 'Downloading…' : 'Anonymous Download'}
            </Button>
            <Button variant="secondary" onClick={handleOpenTerminal} disabled={busy || !appIdInput.trim()}>
              Download in Terminal
            </Button>
          </div>
        </div>
      )}

      {/* Message */}
      {message && (
        <div className={`mac-notice mb-4 ${!message.ok ? 'selectable-diagnostic ' : ''}${
          message.ok
            ? 'mac-notice-success'
            : 'mac-notice-danger'
        }`}>
          {message.text}
          <button className="ml-2 font-semibold opacity-70 hover:opacity-100" onClick={() => setMessage(null)}>Dismiss</button>
        </div>
      )}

      {/* Console log */}
      {log.length > 0 && (
        <div ref={logRef}
          className="mb-4 max-h-36 overflow-auto rounded-[3px] border border-[#38383c] bg-[#0d0d0f] p-3
                     font-mono text-[10px] leading-relaxed shadow-inner selectable-diagnostic">
          {log.map((line, i) => (
            <div key={i} className={`whitespace-pre-wrap break-all ${line.startsWith('!') ? 'text-[#ff6961]' : line.startsWith('>') ? 'text-[#7ce997]' : 'text-[#69b4ff]'}`}>
              {line}
            </div>
          ))}
        </div>
      )}

      {/* Game grid */}
      <div className="-mx-1 flex-1 overflow-auto px-1 pb-3">
        {loading ? (
          <div className="flex h-48 items-center justify-center text-[12px] text-white/35">Loading…</div>
        ) : filteredGames.length === 0 ? (
          <div className="mac-panel flex h-52 flex-col items-center justify-center gap-2 text-white/35">
            <p className="text-[14px] font-semibold text-white/65">No games found</p>
            <p className="text-[12px]">
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
