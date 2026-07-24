import { useState, useCallback } from 'react';
import type { SteamApp } from '@/lib/types';
import { LibraryView } from '@/views/LibraryView';
import { GameDetailView } from '@/views/GameDetailView';
import { SettingsView } from '@/views/SettingsView';
import { OnboardingView } from '@/views/OnboardingView';
import { Button } from '@/components/ui/Button';
import { useBottles } from '@/hooks/useBottles';
import { Gamepad2, Settings, ArrowLeft } from 'lucide-react';

type NavPage = 'library' | 'settings';

export default function App() {
  const [page, setPage] = useState<NavPage>('library');
  const [selectedGame, setSelectedGame] = useState<SteamApp | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(() => {
    return localStorage.getItem('gamerunner-onboarded') !== 'true';
  });

  const { bottles } = useBottles();

  const handleCompleteOnboarding = useCallback(() => {
    localStorage.setItem('gamerunner-onboarded', 'true');
    setShowOnboarding(false);
  }, []);

  const handleSelectGame = useCallback((game: SteamApp) => {
    setSelectedGame(game);
  }, []);

  const handleBack = useCallback(() => {
    setSelectedGame(null);
  }, []);

  const gameBottleId = selectedGame
    ? bottles.find((b) => b.steam_app_id === selectedGame.app_id)?.id
    : undefined;

  return (
    <div className="flex flex-col h-screen bg-[linear-gradient(180deg,#3c3c3c_0%,#303030_100%)] text-[#e0e0d0]">
      <div className="flex flex-1 min-h-0">
        {/* Sidebar */}
        <aside className="w-14 flex flex-col items-center py-3 gap-1.5 border-r-2 border-[#2a2a2a] bg-[linear-gradient(90deg,#333_0%,#2d2d2d_100%)]">
          <NavIcon
            label="Library"
            active={page === 'library' && !selectedGame}
            onClick={() => {
              setPage('library');
              setSelectedGame(null);
            }}
          >
            <Gamepad2 size={18} />
          </NavIcon>
          <NavIcon
            label="Settings"
            active={page === 'settings'}
            onClick={() => {
              setPage('settings');
              setSelectedGame(null);
            }}
          >
            <Settings size={18} />
          </NavIcon>

          <div className="flex-1" />

          <div className="text-[10px] text-[#808070] text-center leading-tight">
            <div className="font-bold">{bottles.length}</div>
            <div>btls</div>
          </div>
        </aside>

        {/* Main content */}
        <main className="flex-1 p-4 overflow-auto min-w-0">
          {selectedGame && (
            <div className="mb-3">
              <Button variant="ghost" size="sm" onClick={handleBack}>
                <ArrowLeft size={14} />
                <span>Back</span>
              </Button>
            </div>
          )}

          {showOnboarding ? (
            <OnboardingView onComplete={handleCompleteOnboarding} />
          ) : selectedGame ? (
            <GameDetailView game={selectedGame} bottleId={gameBottleId} />
          ) : page === 'library' ? (
            <LibraryView onSelectGame={handleSelectGame} />
          ) : (
            <SettingsView />
          )}
        </main>
      </div>

      {/* Status bar */}
      <footer className="h-7 flex items-center px-3 text-[10px] text-[#a0a090] bg-[linear-gradient(180deg,#2a2a2a_0%,#252525_100%)] border-t-2 border-[#3a3a3a] shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] gap-3">
        <span className="font-bold uppercase tracking-wider">GameRunner v0.1</span>
        <span className="flex-1" />
        <span className="border-l-2 border-[#3a3a3a] pl-3">
          {bottles.length} bottle{bottles.length !== 1 ? 's' : ''}
        </span>
      </footer>
    </div>
  );
}

function NavIcon({
  label,
  active,
  onClick,
  children,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      title={label}
      onClick={onClick}
      className={`w-9 h-9 flex items-center justify-center transition-none cursor-default
        ${active
          ? 'bg-[linear-gradient(180deg,#5a5a5a_0%,#444_100%)] text-[#b8d860] border-2 border-[#555] border-t-[#6a6a6a] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'
          : 'text-[#888] border-2 border-transparent hover:bg-[#3a3a3a] hover:text-[#ccc]'
        }`}
    >
      {children}
    </button>
  );
}
