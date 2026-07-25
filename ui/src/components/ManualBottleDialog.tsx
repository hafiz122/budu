import type { BottleInfo } from '@/lib/types';
import { Button } from '@/components/ui/Button';

interface ManualBottleDialogProps {
  executablePath: string;
  bottles: BottleInfo[];
  selectedBottleId: string;
  newBottleName: string;
  busy: boolean;
  onSelect: (bottleId: string) => void;
  onNameChange: (name: string) => void;
  onCreate: () => void;
  onLaunch: () => void;
  onCancel: () => void;
}

export function ManualBottleDialog({
  executablePath,
  bottles,
  selectedBottleId,
  newBottleName,
  busy,
  onSelect,
  onNameChange,
  onCreate,
  onLaunch,
  onCancel,
}: ManualBottleDialogProps) {
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="manual-bottle-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-[#0c0c0e] p-4"
    >
      <div className="mac-panel w-full max-w-md space-y-5 p-5 shadow-2xl">
        <div>
          <h2 id="manual-bottle-title" className="text-[17px] font-semibold tracking-tight text-white">
            Select a bottle
          </h2>
          <p className="mt-1 truncate text-[11px] text-white/35" title={executablePath}>
            {executablePath}
          </p>
        </div>

        <label className="mac-label block space-y-1.5">
          Existing bottle
          <select
            aria-label="Existing bottle"
            value={selectedBottleId}
            onChange={(event) => onSelect(event.target.value)}
            className="mac-select w-full text-[12px]"
          >
            <option value="">Choose a bottle</option>
            {bottles.map((bottle) => (
              <option key={bottle.id} value={bottle.id}>
                {bottle.name || bottle.id}, {bottle.wine_version || 'default Wine'}
              </option>
            ))}
          </select>
        </label>

        <div className="space-y-1">
          <label htmlFor="new-bottle-name" className="mac-label">
            Or create a bottle
          </label>
          <div className="flex gap-2">
            <input
              id="new-bottle-name"
              value={newBottleName}
              onChange={(event) => onNameChange(event.target.value)}
              placeholder="Bottle name"
              className="mac-input min-w-0 flex-1 text-[12px]"
            />
            <Button
              variant="secondary"
              onClick={onCreate}
              disabled={busy || !newBottleName.trim()}
            >
              Create
            </Button>
          </div>
        </div>

        <div className="flex justify-end gap-2 border-t border-[#38383c] pt-4">
          <Button variant="ghost" onClick={onCancel} disabled={busy}>Cancel</Button>
          <Button onClick={onLaunch} disabled={busy || !selectedBottleId}>
            {busy ? 'Working...' : 'Launch'}
          </Button>
        </div>
      </div>
    </div>
  );
}
