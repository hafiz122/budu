// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { BottleInfo } from '@/lib/types';
import { ManualBottleDialog } from './ManualBottleDialog';

const bottle: BottleInfo = {
  id: 'manual-bottle',
  name: 'Manual games',
  wine_version: '9.14-staging',
  prefix_path: '/tmp/bottles/manual-bottle',
  config_path: '/tmp/bottles/manual-bottle/bottle.toml',
  created_at: '2026-07-25T00:00:00Z',
  steam_app_id: null,
  status: 'idle',
};

afterEach(cleanup);

function renderDialog(
  overrides: Partial<React.ComponentProps<typeof ManualBottleDialog>> = {},
) {
  const props: React.ComponentProps<typeof ManualBottleDialog> = {
    executablePath: '/Games/example.exe',
    bottles: [bottle],
    selectedBottleId: '',
    newBottleName: '',
    busy: false,
    onSelect: vi.fn(),
    onNameChange: vi.fn(),
    onCreate: vi.fn(),
    onLaunch: vi.fn(),
    onCancel: vi.fn(),
    ...overrides,
  };
  render(<ManualBottleDialog {...props} />);
  return props;
}

describe('ManualBottleDialog', () => {
  it('requires a bottle before launching', () => {
    renderDialog();

    expect(
      (screen.getByRole('button', { name: 'Launch' }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it('reports the selected bottle and launches it', () => {
    const props = renderDialog({ selectedBottleId: bottle.id });

    fireEvent.change(screen.getByLabelText('Existing bottle'), {
      target: { value: bottle.id },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Launch' }));

    expect(props.onSelect).toHaveBeenCalledWith(bottle.id);
    expect(props.onLaunch).toHaveBeenCalledOnce();
  });

  it('allows creation only after a name is entered', () => {
    const props = renderDialog({ newBottleName: 'New bottle' });

    fireEvent.change(screen.getByLabelText('Or create a bottle'), {
      target: { value: 'Updated name' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    expect(props.onNameChange).toHaveBeenCalledWith('Updated name');
    expect(props.onCreate).toHaveBeenCalledOnce();
  });
});
