// @vitest-environment jsdom
import { expect, it } from 'vitest';
import { applyPalette, resolve, storedPalette } from './theme.ts';
import { ago } from './time.ts';

it('resolves a theme choice', () => {
  expect(resolve('auto', true)).toBe('light');
  expect(resolve('auto', false)).toBe('dark');
  expect(resolve('dark', true)).toBe('dark');
  expect(resolve('light', false)).toBe('light');
});

it('keeps the palette per browser, over the native look; native unless another is chosen', () => {
  const root = document.documentElement;
  window.localStorage.removeItem('mikata.palette');
  expect(storedPalette()).toBe('native');
  applyPalette(storedPalette());
  expect(root.dataset.scheme).toBe('neon');
  expect(root.dataset.palette).toBeUndefined();
  window.localStorage.setItem('mikata.palette', 'aurora');
  applyPalette(storedPalette());
  expect(root.dataset.scheme).toBe('neon');
  expect(root.dataset.palette).toBe('aurora');
  // An unknown one reads as native, and takes the palette away.
  window.localStorage.setItem('mikata.palette', 'pink');
  applyPalette(storedPalette());
  expect(root.dataset.palette).toBeUndefined();
  window.localStorage.removeItem('mikata.palette');
});

it('says how long ago, short', () => {
  const now = Date.parse('2026-10-30T12:00:00Z');
  expect(ago('2026-10-30T11:59:30Z', now)).toBe('now');
  expect(ago('2026-10-30T11:55:00Z', now)).toBe('5m');
  expect(ago('2026-10-30T10:00:00Z', now)).toBe('2h');
  expect(ago('2026-10-27T12:00:00Z', now)).toBe('3d');
  expect(ago('2026-07-01T16:20:00Z', now)).toBe('2026-07-01');
  expect(ago('2026-10-30T13:00:00Z', now)).toBe('now');
  expect(ago('not a date', now)).toBe('unknown');
});
