import type { SteamApp, CompatEntry } from '@/lib/types';
import { GameCard } from './GameCard';

interface GameGridProps {
  games: SteamApp[];
  compatMap: Record<string, CompatEntry>;
  runningGames: Set<string>;
  onPlay: (appId: string) => void;
  onSelect: (game: SteamApp) => void;
}

export function GameGrid({ games, compatMap, runningGames, onPlay, onSelect }: GameGridProps) {
  return (
    <div className="grid grid-cols-2 gap-4 p-1 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5">
      {games.map((game) => (
        <GameCard
          key={game.app_id || game.name}
          game={game}
          compat={compatMap[game.app_id]}
          isRunning={runningGames.has(game.app_id)}
          onPlay={() => onPlay(game.app_id)}
          onSelect={() => onSelect(game)}
        />
      ))}
    </div>
  );
}
