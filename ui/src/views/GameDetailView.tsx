import { useEffect, useState } from 'react';
import type { BottleConfig, SteamApp } from '@/lib/types';
import { Tabs } from '@/components/ui/Tabs';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Card, CardContent, CardHeader } from '@/components/ui/Card';
import { ConsoleView } from '@/components/ConsoleView';
import { useCompat } from '@/hooks/useCompat';
import { useGameProcess } from '@/hooks/useGameProcess';
import { getBottleConfig, saveBottleConfig } from '@/lib/tauri';
import { Gamepad2 } from 'lucide-react';

interface GameDetailViewProps {
  game: SteamApp;
  bottleId?: string;
}

function ratingLabel(rating: string): { label: string; color: string } {
  const map: Record<string, { label: string; color: string }> = {
    platinum: { label: 'Platinum', color: 'text-[#64d2ff]' },
    gold: { label: 'Gold', color: 'text-[#ffd60a]' },
    silver: { label: 'Silver', color: 'text-[#d1d1d6]' },
    bronze: { label: 'Bronze', color: 'text-[#ff9f0a]' },
    borked: { label: 'Borked', color: 'text-[#ff6961]' },
  };
  return map[rating] ?? { label: 'Unknown', color: 'text-white/40' };
}

export function GameDetailView({ game, bottleId }: GameDetailViewProps) {
  const { entry: compat, lookup } = useCompat();
  const { processes } = useGameProcess(bottleId);
  const [config, setConfig] = useState<BottleConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [logLines] = useState<string[]>([]);

  useEffect(() => {
    if (bottleId) {
      getBottleConfig(bottleId)
        .then(setConfig)
        .catch(() => {});
    }
  }, [bottleId]);

  useEffect(() => {
    if (game.app_id) {
      lookup(game.app_id);
    }
  }, [game.app_id, lookup]);

  const handleSaveConfig = async () => {
    if (!bottleId || !config) return;
    setSaving(true);
    try {
      await saveBottleConfig(bottleId, config);
    } catch (err) {
      console.error('Failed to save config:', err);
    } finally {
      setSaving(false);
    }
  };

  const runningProcess = processes.find((p) => p.status === 'running');
  const compatRating = compat ? ratingLabel(compat.rating) : null;

  return (
    <div className="flex h-full flex-col">
      <div className="drag-region mb-6 flex items-center gap-4">
        <div className="flex items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-[3px] border border-[#38383c] bg-[#202023]">
            <Gamepad2 size={23} strokeWidth={1.5} className="text-white/35" />
          </div>
          <div>
            <h1 className="text-[24px] font-bold tracking-[-0.035em] text-white">{game.name}</h1>
            <div className="mt-1 flex items-center gap-2">
              {compatRating && (
                <span className={`text-[11px] font-semibold ${compatRating.color}`}>{compatRating.label}</span>
              )}
              {runningProcess && <Badge variant="success">PID {runningProcess.pid}</Badge>}
            </div>
          </div>
        </div>
      </div>

      <div className="flex-1 min-h-0">
        {config && (
          <Tabs
            tabs={[
              {
                id: 'general',
                label: 'General',
                content: (
                  <div className="mac-panel max-w-3xl space-y-1 p-5">
                    <SettingsRow label="Wine Version">
                      <span className="text-[12px] text-white/50">
                        {config.bottle.wine_version}
                      </span>
                    </SettingsRow>
                    <SettingsRow label="Windows Version">
                      <select
                        value={config.windows.version}
                        onChange={(e) =>
                          setConfig({
                            ...config,
                            windows: { ...config.windows, version: e.target.value },
                          })
                        }
                        className="mac-select text-[12px]"
                      >
                        {['win10', 'win11', 'win8', 'win7', 'winxp'].map((v) => (
                          <option key={v} value={v}>{v}</option>
                        ))}
                      </select>
                    </SettingsRow>
                    <SettingsRow label="Virtual Desktop">
                      <input
                        type="text"
                        placeholder="e.g. 1920x1080 or leave empty"
                        value={config.windows.virtual_desktop ?? ''}
                        onChange={(e) =>
                          setConfig({
                            ...config,
                            windows: {
                              ...config.windows,
                              virtual_desktop: e.target.value || null,
                            },
                          })
                        }
                        className="mac-input w-52 text-[12px]"
                      />
                    </SettingsRow>
                    <Button onClick={handleSaveConfig} disabled={saving}>
                      {saving ? 'Saving...' : 'Save Configuration'}
                    </Button>
                  </div>
                ),
              },
              {
                id: 'graphics',
                label: 'Graphics',
                content: (
                  <div className="mac-panel max-w-3xl space-y-1 p-5">
                    <SettingsRow label="Graphics Backend">
                      <select
                        value={config.graphics.backend}
                        onChange={(e) =>
                          setConfig({
                            ...config,
                            graphics: {
                              ...config.graphics,
                              backend: e.target.value as 'd3dmetal' | 'dxmt' | 'dxvk' | 'wine3d',
                            },
                          })
                        }
                        className="mac-select text-[12px]"
                      >
                        <option value="dxmt">DXMT (Open-source Metal)</option>
                        <option value="d3dmetal">D3DMetal (User-supplied)</option>
                        <option value="dxvk">DXVK + MoltenVK</option>
                        <option value="wine3d">WineD3D (Fallback)</option>
                      </select>
                    </SettingsRow>

                    {([
                      ['d3d12', 'DirectX 12'],
                      ['d3d11', 'DirectX 11'],
                      ['d3d10', 'DirectX 10'],
                      ['dxgi', 'DXGI'],
                    ] as [string, string][]).map(([key, label]) => (
                      <SettingsRow key={key} label={label}>
                        <label className="flex cursor-default items-center gap-2 text-[12px] text-white/50">
                          <input
                            type="checkbox"
                            checked={
                              key === 'd3d12' ? config.graphics.d3d12_override_native :
                              key === 'd3d11' ? config.graphics.d3d11_override_native :
                              key === 'd3d10' ? config.graphics.d3d10_override_native :
                              config.graphics.dxgi_override_native
                            }
                            onChange={(e) =>
                              setConfig({
                                ...config,
                                graphics: {
                                  ...config.graphics,
                                  [key === 'd3d12' ? 'd3d12_override_native' :
                                   key === 'd3d11' ? 'd3d11_override_native' :
                                   key === 'd3d10' ? 'd3d10_override_native' :
                                   'dxgi_override_native']: e.target.checked,
                                },
                              })
                            }
                            className="h-4 w-4 rounded-[2px] accent-[#0a84ff]"
                          />
                          Use native override
                        </label>
                      </SettingsRow>
                    ))}

                    <Button onClick={handleSaveConfig} disabled={saving}>
                      {saving ? 'Saving...' : 'Save Configuration'}
                    </Button>
                  </div>
                ),
              },
              {
                id: 'console',
                label: 'Console',
                content: <ConsoleView lines={logLines} />,
              },
              {
                id: 'compat',
                label: 'Compatibility',
                content: (
                  <div className="max-w-3xl space-y-4">
                    {compat ? (
                      <>
                        <Card>
                          <CardHeader>
                            <span className={`text-[13px] font-semibold ${compatRating?.color}`}>
                              {compatRating?.label}
                            </span>
                          </CardHeader>
                          <CardContent>
                            <p className="text-[12px] text-white/45">
                              Last tested: {compat.last_tested_wine ?? 'Unknown'} on{' '}
                              {compat.last_tested_date ?? 'Unknown'}
                            </p>
                          </CardContent>
                        </Card>

                        {compat.fixes.length > 0 && (
                          <div>
                            <h3 className="mb-2 text-[13px] font-semibold text-text-primary">
                              Known Fixes
                            </h3>
                            <div className="space-y-1">
                              {compat.fixes.map((fix, i) => (
                                <div
                                  key={i}
                                  className="rounded-[3px] border border-[#38383c] bg-[#151517] px-3 py-2
                                             font-mono text-[10px] text-white/45"
                                >
                                  {fix.type}: {fix.dll ?? fix.key ?? fix.value ?? fix.verb ?? 'unknown'}
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                        {compat.known_issues.length > 0 && (
                          <div>
                            <h3 className="mb-2 text-[13px] font-semibold text-text-primary">
                              Known Issues
                            </h3>
                            <ul className="space-y-1">
                              {compat.known_issues.map((issue, i) => (
                                <li key={i} className="list-inside list-disc text-[11px] text-white/50">
                                  {issue}
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </>
                    ) : (
                      <div className="mac-panel p-6 text-sm text-text-muted">
                        No compatibility data available for this game.
                      </div>
                    )}
                  </div>
                ),
              },
            ]}
          />
        )}
      </div>
    </div>
  );
}

function SettingsRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-11 items-center justify-between border-b border-[#38383c] py-2 last:border-0">
      <span className="text-[12px] font-medium text-white/60">{label}</span>
      <div>{children}</div>
    </div>
  );
}
