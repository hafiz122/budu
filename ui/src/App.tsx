import { useState, useCallback } from 'react';
import type { SteamApp } from '@/lib/types';
import { LibraryView } from '@/views/LibraryView';
import { GameDetailView } from '@/views/GameDetailView';
import { SettingsView } from '@/views/SettingsView';
import { OnboardingView } from '@/views/OnboardingView';
import { Button } from '@/components/ui/Button';
import { useBottles } from '@/hooks/useBottles';
import { ArrowLeft } from 'lucide-react';
import buduLogo from '@/assets/budu-logo.svg';
import libraryIcon from '@/assets/library-icon.png';
import settingsIcon from '@/assets/settings-icon.png';

type NavPage = 'library' | 'settings';

export default function App() {
  const [page, setPage] = useState<NavPage>('library');
  const [selectedGame, setSelectedGame] = useState<SteamApp | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(() => {
    return localStorage.getItem('gamerunner-onboarded') !== 'true';
  });

  const { bottles, create: createBottle, refresh: refreshBottles } = useBottles();

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
    <div className="flex h-screen flex-col overflow-hidden bg-[#141416] text-[#f5f5f7]">
      <div
        data-tauri-drag-region
        className="relative flex h-10 shrink-0 items-center justify-center border-b border-[#303034] bg-[#151517]"
      >
        <span
          data-tauri-drag-region
          className="text-[12px] font-semibold text-white/55"
        >
          Budu
        </span>
      </div>

      <div className="flex flex-1 min-h-0">
        {/* Sidebar */}
        <aside className="flex w-48 shrink-0 flex-col border-r border-[#303034] bg-[#1a1a1c] px-3 py-4">
          <img src={buduLogo} alt="Budu" className="mb-5 ml-2 h-auto w-28" />
          <div className="mb-2 px-2 text-[10px] font-semibold uppercase tracking-[0.08em] text-white/35">
            Browse
          </div>
          <NavIcon
            label="Library"
            active={page === 'library' && !selectedGame}
            onClick={() => {
              setPage('library');
              setSelectedGame(null);
            }}
          >
            <span className="flex h-6 w-6 shrink-0 items-center justify-center">
              <img
                src={libraryIcon}
                alt=""
                aria-hidden="true"
                className="h-8 w-8 max-w-none object-contain"
              />
            </span>
          </NavIcon>
          <NavIcon
            label="Settings"
            active={page === 'settings'}
            onClick={() => {
              setPage('settings');
              setSelectedGame(null);
            }}
          >
            <span className="flex h-6 w-6 shrink-0 items-center justify-center">
              <img
                src={settingsIcon}
                alt=""
                aria-hidden="true"
                className="h-6 w-6 object-contain"
              />
            </span>
          </NavIcon>

          <div className="flex-1" />

          <div className="mx-1 rounded-[3px] border border-[#353539] bg-[#202023] px-3 py-2.5">
            <div className="text-[10px] font-medium text-white/35">Bottles</div>
            <div className="mt-0.5 text-[13px] font-semibold text-white/75">
              {bottles.length} configured
            </div>
          </div>
        </aside>

        {/* Main content */}
        <main className="min-w-0 flex-1 overflow-auto p-6">
          {selectedGame && (
            <div className="mb-4">
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
            <LibraryView
              onSelectGame={handleSelectGame}
              bottles={bottles}
              onCreateBottle={createBottle}
              onBottlesChanged={refreshBottles}
            />
          ) : (
            <SettingsView />
          )}
        </main>
      </div>

      {/* Status bar */}
      <footer className="flex h-7 shrink-0 items-center gap-3 border-t border-[#303034] bg-[#101012] px-3 text-[10px] text-white/35">
        <span className="font-medium">Budu 0.1</span>
        <span className="flex-1" />
        <span>
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
      className={`mb-1 flex h-10 w-full items-center gap-2.5 rounded-[2px] border px-3 text-[13px] font-medium transition duration-150 cursor-default
        ${active
          ? 'border-[#2e5f8d] bg-[#203d59] text-[#8bc7ff]'
          : 'border-transparent bg-[#1a1a1c] text-white/50 hover:border-[#37373b] hover:bg-[#252528] hover:text-white/80'
        }`}
    >
      {children}
      <span>{label}</span>
    </button>
  );
}
