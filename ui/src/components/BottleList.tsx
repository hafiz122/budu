import type { BottleInfo } from '@/lib/types';
import { Badge } from './ui/Badge';
import { Button } from './ui/Button';
import { Card } from './ui/Card';

interface BottleListProps {
  bottles: BottleInfo[];
  onDelete: (id: string) => void;
  onSelect: (bottle: BottleInfo) => void;
}

const statusVariant: Record<string, 'success' | 'warning' | 'danger' | 'default'> = {
  idle: 'default',
  running: 'success',
  crashed: 'danger',
  configuring: 'warning',
};

export function BottleList({ bottles, onDelete, onSelect }: BottleListProps) {
  if (bottles.length === 0) {
    return (
      <div className="text-center text-text-muted py-12 text-sm">
        No bottles created yet. Add a game to create one.
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {bottles.map((bottle) => (
        <Card
          key={bottle.id}
          hover
          onClick={() => onSelect(bottle)}
          className="flex items-center justify-between"
        >
          <div className="flex items-center gap-3">
            <div>
              <div className="text-sm font-medium text-text-primary">
                {bottle.name}
              </div>
              <div className="text-xs text-text-muted mt-0.5">
                Wine {bottle.wine_version}
                {bottle.steam_app_id && (
                  <span className="ml-2">App {bottle.steam_app_id}</span>
                )}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Badge variant={statusVariant[bottle.status] ?? 'default'}>
              {bottle.status}
            </Badge>
            <Button
              variant="ghost"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(bottle.id);
              }}
            >
              Delete
            </Button>
          </div>
        </Card>
      ))}
    </div>
  );
}
