import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const stylesheet = readFileSync(new URL('./globals.css', import.meta.url), 'utf8');

describe('global text selection policy', () => {
  it('makes only diagnostic text selectable', () => {
    const bodyRule = stylesheet.match(/body \{([\s\S]*?)\n\}/)?.[1] ?? '';

    expect(bodyRule).toContain('user-select: none;');
    expect(bodyRule).toContain('-webkit-user-select: none;');
    expect(stylesheet).toMatch(/\.selectable-diagnostic \{[\s\S]*?user-select: text;/);
  });

  it('keeps controls and window drag regions non-selectable', () => {
    expect(stylesheet).toMatch(/button,\s*\nselect \{[\s\S]*?user-select: none;/);
    expect(stylesheet).toMatch(/\.drag-region \{[\s\S]*?user-select: none;/);
    expect(stylesheet).toMatch(/\[data-tauri-drag-region\] \{[\s\S]*?user-select: none;/);
  });
});
