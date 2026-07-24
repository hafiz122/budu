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
    <div className="flex flex-col h-full max-w-2xl">
      <h1 className="text-[13px] font-bold uppercase tracking-wider text-[#e0e0d0] text-shadow mb-4">Settings</h1>

      {message && (
        <div className={`mb-3 px-3 py-2 text-[11px] font-bold border-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] ${
          message.ok ? 'bg-[#2a3a1a] border-[#5a8f3c] text-[#8cc85c]' : 'bg-[#3a1a1a] border-[#aa3333] text-[#ff6666]'
        }`}>
          {message.text}
          <button className="ml-2 underline opacity-70 hover:opacity-100" onClick={() => setMessage(null)}>Dismiss</button>
        </div>
      )}

      <div className="space-y-4 overflow-auto flex-1">
        <Card>
          <CardHeader><h2 className="text-[11px] font-bold uppercase tracking-wider">Appearance</h2></CardHeader>
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
          <CardHeader><h2 className="text-[11px] font-bold uppercase tracking-wider">Wine Versions</h2></CardHeader>
          <CardContent>
            <div className="space-y-1.5">
              {wineVersions.length === 0 && (
                <p className="text-[11px] text-[#909080]">
                  No Wine found. Place at: <code className="px-1 py-0.5 bg-[#222] text-[10px] font-mono">~/.gamerunner/wine/9.14-staging/</code>
                </p>
              )}
              {wineVersions.map((wv) => (
                <div key={wv.version} className="flex items-center justify-between py-0.5">
                  <div>
                    <span className="text-[11px] text-[#e0e0d0] font-bold">{wv.version}</span>
                    {wv.is_default && <span className="ml-1.5 text-[10px] text-[#7c9c2e] font-bold uppercase">(default)</span>}
                  </div>
                  <span className="text-[10px] text-[#808070]">{wv.installed ? wv.arch : 'not installed'}</span>
                </div>
              ))}
              <Button variant="secondary" size="sm" disabled={installing !== null}
                onClick={() => handleInstallWine('9.14-staging')}>
                {installing ? '...' : 'Install 9.14-staging'}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader><h2 className="text-[11px] font-bold uppercase tracking-wider">Graphics Backends</h2></CardHeader>
          <CardContent>
            <div className="space-y-2.5">
              {backends.map((b) => (
                <div key={b.backend}>
                  <div className="flex items-center gap-2">
                    <span className={`w-1.5 h-1.5 border-2 ${b.installed ? 'bg-[#5a8f3c] border-[#7caf3c]' : 'bg-[#555] border-[#444]'}`} />
                    <span className="text-[11px] font-bold text-[#e0e0d0]">{b.name}</span>
                  </div>
                  <p className="text-[10px] text-[#808070] mt-0.5 ml-3.5">{b.description}</p>
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
    <div className="flex items-center justify-between">
      <span className="text-[11px] text-[#a0a090] font-bold uppercase tracking-wide">{label}</span>
      <div>{children}</div>
    </div>
  );
}
