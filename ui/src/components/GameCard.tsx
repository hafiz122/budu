import { useState } from 'react';
import type { SteamApp, CompatEntry } from '@/lib/types';
import { Badge } from './ui/Badge';
import { Gamepad2, Play } from 'lucide-react';

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
    platinum: 'border-[#4faac7] bg-[#24687d] text-white',
    gold: 'border-[#d1a900] bg-[#937900] text-white',
    silver: 'border-[#73737a] bg-[#515157] text-white',
    bronze: 'border-[#c97912] bg-[#8f5208] text-white',
    borked: 'border-[#d84b43] bg-[#a62f29] text-white',
  };
  return m[rating] ?? 'border-[#515157] bg-[#38383d] text-white';
}

export function GameCard({ game, compat, isRunning, onPlay, onSelect, className }: GameCardProps) {
  const [artworkFailed, setArtworkFailed] = useState(false);
  const artworkUrl = /^\d+$/.test(game.app_id)
    ? `https://cdn.akamai.steamstatic.com/steam/apps/${game.app_id}/header.jpg`
    : null;

  return (
    <div
      onClick={onSelect}
      className={`group relative flex cursor-default flex-col overflow-hidden rounded-[3px] border border-[#38383c]
        bg-[#202023] shadow-[0_12px_28px_rgba(0,0,0,0.14)]
        transition duration-200 hover:-translate-y-0.5 hover:border-[#505055]
        hover:bg-[#27272a] hover:shadow-[0_18px_38px_rgba(0,0,0,0.24)] ${className ?? ''}`}
    >
      <div className="relative flex aspect-[460/215] items-center justify-center overflow-hidden border-b border-[#38383c] bg-[#202024]">
        {artworkUrl && !artworkFailed ? (
          <img
            src={artworkUrl}
            alt={`${game.name} artwork`}
            loading="lazy"
            draggable={false}
            onError={() => setArtworkFailed(true)}
            className="h-full w-full object-cover"
          />
        ) : (
          <Gamepad2 size={42} strokeWidth={1.3} className="text-white/10" />
        )}

        <div className="absolute right-2 top-2 flex gap-1">
          {isRunning && <Badge variant="success">Running</Badge>}
          {compat && (
            <span className={`rounded-[2px] border px-2 py-0.5 text-[9px] font-semibold tracking-wide ${ratingColors(compat.rating)}`}>
              {compat.rating}
            </span>
          )}
        </div>

        {onPlay && game.installed && (
          <button
            onClick={(e) => { e.stopPropagation(); onPlay(); }}
            className="absolute inset-0 flex items-center justify-center opacity-0 transition duration-200 group-hover:opacity-100"
          >
            <span className="flex items-center gap-1.5 rounded-[2px] border border-[#d1d1d6] bg-white px-4 py-2
              text-[12px] font-semibold text-black shadow-xl">
              <Play size={12} fill="currentColor" />
              {isRunning ? 'Stop' : 'Play'}
            </span>
          </button>
        )}
      </div>

      <div className="flex flex-col gap-2 p-3.5">
        <h3 className="truncate text-[13px] font-semibold text-white/90">{game.name}</h3>
        <div className="flex items-center gap-2 text-[10px] text-white/35">
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
