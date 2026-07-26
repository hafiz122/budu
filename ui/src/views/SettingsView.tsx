import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader } from '@/components/ui/Card';
import { setTheme, setColorScheme, getAvailableThemes } from '@/lib/theme';
import type { WineVersion, GraphicsBackendInfo } from '@/lib/types';
import { listWineVersions, detectBackends, installWineVersion } from '@/lib/tauri';

export function SettingsView() {
  const [wineVersions, setWineVersions] = useState<WineVersion[]>([]);
  const [backends, setBackends] = useState<GraphicsBackendInfo[]>([]);
  const [installing, setInstalling] = useState<string | null>(null);
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const themes = getAvailableThemes();

  useEffect(() => {
    listWineVersions().then(setWineVersions).catch(() => {});
    detectBackends().then(setBackends).catch(() => {});
  }, []);

  const handleInstallWine = async (version: string) => {
    setInstalling(version); setMessage(null);
    try {
      await installWineVersion(version);
      setMessage({ text: `Wine ${version} installed.`, ok: true });
      const updated = await listWineVersions();
      setWineVersions(updated);
    } catch (err) {
      setMessage({ text: String(err), ok: false });
    } finally { setInstalling(null); }
  };

  return (
    <div className="flex h-full max-w-3xl flex-col">
      <div className="mb-5">
        <h1 className="text-[26px] font-bold tracking-[-0.035em] text-white">Settings</h1>
        <p className="mt-1 text-[12px] text-white/40">Manage appearance and game runtimes.</p>
      </div>

      {message && (
        <div className={`mac-notice mb-4 ${!message.ok ? 'selectable-diagnostic ' : ''}${
          message.ok ? 'mac-notice-success' : 'mac-notice-danger'
        }`}>
          {message.text}
          <button className="ml-2 font-semibold opacity-70 hover:opacity-100" onClick={() => setMessage(null)}>Dismiss</button>
        </div>
      )}

      <div className="flex-1 space-y-4 overflow-auto pb-4">
        <Card>
          <CardHeader><h2>Appearance</h2></CardHeader>
          <CardContent className="space-y-3">
            <Row label="Theme">
              <div className="flex gap-1.5">
                {themes.map((t) => (
                  <Button key={t.id} variant="secondary" size="sm" onClick={() => setTheme(t.id)}>{t.label}</Button>
                ))}
              </div>
            </Row>
            <Row label="Color Scheme">
              <div className="flex gap-1.5">
                {(['dark', 'light', 'auto'] as const).map((s) => (
                  <Button key={s} variant="secondary" size="sm" onClick={() => setColorScheme(s)}>
                    {s.charAt(0).toUpperCase() + s.slice(1)}
                  </Button>
                ))}
              </div>
            </Row>
          </CardContent>
        </Card>

        <Card>
          <CardHeader><h2>Wine Versions</h2></CardHeader>
          <CardContent>
            <div className="space-y-1.5">
              {wineVersions.length === 0 && (
                <p className="text-[12px] text-white/45">
                  No managed Wine runtime installed yet. Use Install Wine below.
                </p>
              )}
              {wineVersions.map((wv) => (
                <div key={wv.version} className="flex items-center justify-between rounded-[2px] px-1 py-1.5">
                  <div>
                    <span className="text-[12px] font-medium text-white/85">{wv.version}</span>
                    {wv.is_default && <span className="ml-2 text-[10px] font-semibold text-[#69b4ff]">Default</span>}
                  </div>
                  <span className="text-[11px] text-white/35">{wv.installed ? wv.arch : 'Not installed'}</span>
                </div>
              ))}
              <Button variant="secondary" size="sm" disabled={installing !== null}
                onClick={() => handleInstallWine('11.10-staging')}>
                {installing ? '...' : 'Install Wine'}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader><h2>Graphics Backends</h2></CardHeader>
          <CardContent>
            <div className="space-y-2.5">
              {backends.map((b) => (
                <div key={b.backend} className="rounded-[3px] bg-[#18181a] px-3 py-2.5">
                  <div className="flex items-center gap-2">
                    <span className={`h-2 w-2 rounded-full ${b.installed ? 'bg-[#30d158]' : 'bg-[#55555a]'}`} />
                    <span className="text-[12px] font-medium text-white/85">{b.name}</span>
                  </div>
                  <p className="ml-4 mt-1 text-[11px] leading-relaxed text-white/35">{b.description}</p>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex min-h-10 items-center justify-between border-b border-[#38383c] py-2 last:border-0">
      <span className="text-[12px] font-medium text-white/60">{label}</span>
      <div>{children}</div>
    </div>
  );
}
