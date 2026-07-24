import type { SteamApp, CompatEntry } from '@/lib/types';
import { Badge } from './ui/Badge';
import { Gamepad2 } from 'lucide-react';

interface GameCardProps {
  game: SteamApp;
  compat?: CompatEntry | null;
  isRunning?: boolean;
  onPlay?: () => void;
  onSelect?: () => void;
  className?: string;
}

function ratingColors(rating: string): string {
  const m: Record<string, string> = {
    platinum: 'bg-[linear-gradient(180deg,#4488aa_0%,#336688_100%)] border-[#225577] text-white',
    gold: 'bg-[linear-gradient(180deg,#ccaa44_0%,#aa8822_100%)] border-[#886611] text-white',
    silver: 'bg-[linear-gradient(180deg,#888_0%,#666_100%)] border-[#555] text-white',
    bronze: 'bg-[linear-gradient(180deg,#aa7744_0%,#885522_100%)] border-[#664411] text-white',
    borked: 'bg-[linear-gradient(180deg,#cc4444_0%,#aa2222_100%)] border-[#881111] text-white',
  };
  return m[rating] ?? 'bg-[linear-gradient(180deg,#555_0%,#3a3a3a_100%)] border-[#444] text-[#999]';
}

export function GameCard({ game, compat, isRunning, onPlay, onSelect, className }: GameCardProps) {
  return (
    <div
      onClick={onSelect}
      className={`group relative flex flex-col border-2 border-[#4a4a4a] border-t-[#5a5a5a]
        bg-[linear-gradient(180deg,#3e3e3e_0%,#333_100%)]
        shadow-[inset_0_1px_0_rgba(255,255,255,0.03),0_2px_3px_rgba(0,0,0,0.3)]
        hover:bg-[linear-gradient(180deg,#484848_0%,#383838_100%)] hover:border-[#666]
        cursor-default overflow-hidden transition-none ${className ?? ''}`}
    >
      <div className="relative aspect-[16/10] bg-[#2a2a2a] flex items-center justify-center
        border-b-2 border-[#2a2a2a]">
        <Gamepad2 size={40} className="opacity-10" />

        <div className="absolute top-1.5 right-1.5 flex gap-1">
          {isRunning && <Badge variant="success">ON</Badge>}
          {compat && (
            <span className={`text-[9px] px-1.5 py-0.5 border font-bold uppercase tracking-wide shadow-[inset_0_1px_0_rgba(255,255,255,0.1)] ${ratingColors(compat.rating)}`}>
              {compat.rating}
            </span>
          )}
        </div>

        {onPlay && game.installed && (
          <button
            onClick={(e) => { e.stopPropagation(); onPlay(); }}
            className="absolute inset-0 flex items-center justify-center
              bg-black/70 opacity-0 group-hover:opacity-100 transition-none"
          >
            <span className="px-4 py-1.5 bg-[linear-gradient(180deg,#8db834_0%,#6a8c1e_100%)] text-white text-[10px] font-bold uppercase tracking-wider
              border-2 border-[#4a6c0e] border-t-[#a0cc40] shadow-[inset_0_1px_0_rgba(255,255,255,0.2),0_2px_3px_rgba(0,0,0,0.4)]">
              {isRunning ? 'Stop' : 'Play'}
            </span>
          </button>
        )}
      </div>

      <div className="p-2.5 flex flex-col gap-1">
        <h3 className="text-[11px] font-bold text-[#e0e0d0] truncate text-shadow">{game.name}</h3>
        <div className="flex items-center gap-1.5 text-[9px] text-[#909080]">
          {game.installed ? <Badge variant="success">Installed</Badge> : <Badge>Not installed</Badge>}
          {game.size_bytes != null && <span>{fmt(game.size_bytes)}</span>}
        </div>
      </div>
    </div>
  );
}

function fmt(bytes: number): string {
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let s = bytes; let i = 0;
  while (s >= 1024 && i < u.length - 1) { s /= 1024; i++; }
  return `${s.toFixed(i === 0 ? 0 : 1)} ${u[i]}`;
}
