/**
 * Theme engine for GameRunner.
 *
 * Themes are defined as CSS custom property sets. This module provides
 * the glue to hot-swap themes at runtime by toggling classes on <html>.
 */

export type ThemeId = 'default' | 'steam' | 'minimal' | string;

const THEME_ATTR = 'data-theme';

export function getCurrentTheme(): ThemeId {
  return document.documentElement.getAttribute(THEME_ATTR) ?? 'default';
}

export function setTheme(theme: ThemeId): void {
  document.documentElement.setAttribute(THEME_ATTR, theme);
  localStorage.setItem('gamerunner-theme', theme);
}

export function restoreTheme(): void {
  const saved = localStorage.getItem('gamerunner-theme') ?? 'default';
  setTheme(saved);
}

export function getAvailableThemes(): { id: ThemeId; label: string }[] {
  return [
    { id: 'default', label: 'Default' },
    { id: 'steam', label: 'Steam' },
    { id: 'minimal', label: 'Minimal' },
  ];
}

/** Toggle between light and dark mode. */
export function setColorScheme(scheme: 'light' | 'dark' | 'auto'): void {
  if (scheme === 'auto') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.classList.toggle('dark', prefersDark);
  } else {
    document.documentElement.classList.toggle('dark', scheme === 'dark');
  }
  localStorage.setItem('gamerunner-color-scheme', scheme);
}

export function restoreColorScheme(): void {
  const saved = localStorage.getItem('gamerunner-color-scheme') as
    | 'light'
    | 'dark'
    | 'auto'
    | null;
  setColorScheme(saved ?? 'dark');
}
