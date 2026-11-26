/**
 * Light, dark or auto. `@mikata/theme`'s tokens.css is dark by default and light under
 * `data-theme="light"`; auto follows the system and moves when it does. The choice is kept per
 * browser.
 */
import { DEFAULT_SCHEME, nativeLook, PALETTES, type PaletteName, palettes } from '@mikata/theme';
import { useCallback, useEffect, useState } from 'react';

export type Choice = 'auto' | 'light' | 'dark';
export type Theme = 'light' | 'dark';
export const CHOICES: readonly Choice[] = ['auto', 'light', 'dark'];

const KEY = 'mikata.theme';

/** The theme a choice gives, the system preferring light or not. */
export function resolve(choice: Choice, systemLight: boolean): Theme {
  if (choice === 'auto') return systemLight ? 'light' : 'dark';
  return choice;
}

/** The stored choice; auto when none, or when storage cannot be read. */
export function stored(): Choice {
  try {
    const v = window.localStorage.getItem(KEY);
    return v === 'light' || v === 'dark' ? v : 'auto';
  } catch {
    return 'auto';
  }
}

function store(choice: Choice) {
  try {
    if (choice === 'auto') window.localStorage.removeItem(KEY);
    else window.localStorage.setItem(KEY, choice);
  } catch {
    // A private window: the choice lasts this page only.
  }
}

const query = () => window.matchMedia('(prefers-color-scheme: light)');

/** Sets the resolved theme on `<html>`; called once before the first render, then by the hook. */
export function apply(choice: Choice) {
  const root = document.documentElement;
  root.dataset.theme = resolve(choice, query().matches);
  root.dataset.themeChoice = choice;
}

/**
 * The palette, chosen per browser: the native look (Bio Digital, Bio Pearl — `data-scheme="neon"`)
 * or one of the owner's gradient systems over it (`data-palette`). The terminal app keeps its own
 * schemes: nothing here moves them.
 */
export type PaletteChoice = 'native' | PaletteName;
export const PALETTE_CHOICES: readonly PaletteChoice[] = ['native', ...PALETTES];
export { nativeLook, palettes };

const PALETTE_KEY = 'mikata.palette';

/** The stored palette; native when none, an unknown one, or storage cannot be read. */
export function storedPalette(): PaletteChoice {
  try {
    const v = window.localStorage.getItem(PALETTE_KEY);
    return (PALETTE_CHOICES as readonly string[]).includes(v ?? '')
      ? (v as PaletteChoice)
      : 'native';
  } catch {
    return 'native';
  }
}

/** Sets the palette on `<html>`; called once before the first render, then by the hook. */
export function applyPalette(p: PaletteChoice) {
  const root = document.documentElement;
  root.dataset.scheme = DEFAULT_SCHEME;
  if (p === 'native') delete root.dataset.palette;
  else root.dataset.palette = p;
}

/** The palette and its setter. */
export function usePalette(): [PaletteChoice, (p: PaletteChoice) => void] {
  const [palette, setPalette] = useState<PaletteChoice>(storedPalette);
  useEffect(() => applyPalette(palette), [palette]);
  const set = useCallback((p: PaletteChoice) => {
    try {
      if (p === 'native') window.localStorage.removeItem(PALETTE_KEY);
      else window.localStorage.setItem(PALETTE_KEY, p);
    } catch {
      // A private window: the palette lasts this page only.
    }
    setPalette(p);
  }, []);
  return [palette, set];
}

/** The theme choice and its setter; auto follows the system while the page is open. */
export function useTheme(): [Choice, (c: Choice) => void] {
  const [choice, setChoice] = useState<Choice>(stored);
  useEffect(() => {
    apply(choice);
    if (choice !== 'auto') return;
    const mq = query();
    const on = () => apply('auto');
    mq.addEventListener('change', on);
    return () => mq.removeEventListener('change', on);
  }, [choice]);
  const set = useCallback((c: Choice) => {
    store(c);
    setChoice(c);
  }, []);
  return [choice, set];
}
