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
    platinum: { label: 'Platinum', color: 'text-[#66aacc]' },
    gold: { label: 'Gold', color: 'text-[#ccaa44]' },
    silver: { label: 'Silver', color: 'text-[#aaaaaa]' },
    bronze: { label: 'Bronze', color: 'text-[#cc8844]' },
    borked: { label: 'Borked', color: 'text-[#cc4444]' },
  };
  return map[rating] ?? { label: 'Unknown', color: 'text-[#808070]' };
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
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-4 mb-4 drag-region">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-[#2a2a2a] border-2 border-[#4a4a4a] border-t-[#5a5a5a] flex items-center justify-center
            shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
            <Gamepad2 size={20} className="text-[#707060]" />
          </div>
          <div>
            <h1 className="text-[13px] font-bold uppercase tracking-wider text-[#e0e0d0] text-shadow">{game.name}</h1>
            <div className="flex items-center gap-2 mt-0.5">
              {compatRating && (
                <span className={`text-[10px] font-bold uppercase tracking-wide ${compatRating.color}`}>{compatRating.label}</span>
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
                  <div className="space-y-4 p-1">
                    <SettingsRow label="Wine Version">
                      <span className="text-[11px] text-[#a0a090]">
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
                        className="bg-[#222] border-2 border-[#111] border-b-[#444] border-r-[#444] px-2 py-1
                                   text-[11px] text-[#e0e0d0] shadow-[inset_0_2px_3px_rgba(0,0,0,0.4)]
                                   focus:outline-none focus:border-[#7c9c2e]"
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
                        className="bg-[#222] border-2 border-[#111] border-b-[#444] border-r-[#444] px-2 py-1
                                   text-[11px] text-[#e0e0d0] w-48
                                   shadow-[inset_0_2px_3px_rgba(0,0,0,0.4)]
                                   focus:outline-none focus:border-[#7c9c2e]"
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
                  <div className="space-y-4 p-1">
                    <SettingsRow label="Graphics Backend">
                      <select
                        value={config.graphics.backend}
                        onChange={(e) =>
                          setConfig({
                            ...config,
                            graphics: {
                              ...config.graphics,
                              backend: e.target.value as 'd3dmetal' | 'dxvk' | 'wine3d',
                            },
                          })
                        }
                        className="bg-[#222] border-2 border-[#111] border-b-[#444] border-r-[#444] px-2 py-1
                                   text-[11px] text-[#e0e0d0] shadow-[inset_0_2px_3px_rgba(0,0,0,0.4)]
                                   focus:outline-none focus:border-[#7c9c2e]"
                      >
                        <option value="d3dmetal">D3DMetal (Best)</option>
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
                        <label className="flex items-center gap-2 text-[11px] text-[#a0a090] cursor-default">
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
                            className="rounded"
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
                  <div className="space-y-4 p-1">
                    {compat ? (
                      <>
                        <Card>
                          <CardHeader>
                            <span className={`text-sm font-medium ${compatRating?.color}`}>
                              {compatRating?.label}
                            </span>
                          </CardHeader>
                          <CardContent>
                            <p className="text-[11px] text-[#a0a090]">
                              Last tested: {compat.last_tested_wine ?? 'Unknown'} on{' '}
                              {compat.last_tested_date ?? 'Unknown'}
                            </p>
                          </CardContent>
                        </Card>

                        {compat.fixes.length > 0 && (
                          <div>
                            <h3 className="text-sm font-medium text-text-primary mb-2">
                              Known Fixes
                            </h3>
                            <div className="space-y-1">
                              {compat.fixes.map((fix, i) => (
                                <div
                                  key={i}
                                  className="text-[10px] text-[#808070] font-mono bg-[#2a2a2a]
                                             rounded px-3 py-1.5 border border-border"
                                >
                                  {fix.type}: {fix.dll ?? fix.key ?? fix.value ?? fix.verb ?? 'unknown'}
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                        {compat.known_issues.length > 0 && (
                          <div>
                            <h3 className="text-sm font-medium text-text-primary mb-2">
                              Known Issues
                            </h3>
                            <ul className="space-y-1">
                              {compat.known_issues.map((issue, i) => (
                                <li key={i} className="text-[10px] text-[#a0a090] list-disc list-inside">
                                  {issue}
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </>
                    ) : (
                      <div className="text-sm text-text-muted">
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
    <div className="flex items-center justify-between py-1.5 border-b-2 border-[#2a2a2a] last:border-0">
      <span className="text-[11px] text-[#a0a090] font-bold uppercase tracking-wide">{label}</span>
      <div>{children}</div>
    </div>
  );
}
