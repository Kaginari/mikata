import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import {
  brand,
  contrast,
  DEFAULT_SCHEME,
  dark,
  light,
  PALETTES,
  type Palette,
  palettes,
  races,
  SCHEMES,
  type SchemePalette,
  schemes,
} from './index.js';

const css = readFileSync(new URL('./tokens.css', import.meta.url), 'utf8');

/** The `--mn-*` declarations of the first block opened by `selector`. */
function block(selector: string): Record<string, string> {
  const start = css.indexOf(`${selector} {`);
  const body = css.slice(start, css.indexOf('}', start));
  return Object.fromEntries(
    [...body.matchAll(/--mn-([a-z-]+):\s*([^;]+);/g)].map((m) => [m[1], (m[2] ?? '').trim()]),
  );
}

const kebab = (k: string) => k.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

describe('the css and the ts agree', () => {
  it.each([
    [':root', dark],
    [':root[data-theme="light"]', light],
  ] as const)('%s', (selector, palette: Palette) => {
    const decl = block(selector);
    for (const [k, v] of Object.entries(palette)) expect(decl[kebab(k)], k).toBe(v);
  });

  it('the races', () => {
    const decl = block(':root');
    for (const [k, v] of Object.entries(races)) expect(decl[`race-${kebab(k)}`], k).toBe(v);
  });

  it('the brand', () => {
    const decl = block(':root');
    for (const [k, v] of Object.entries(brand)) expect(decl[`brand-${kebab(k)}`], k).toBe(v);
  });

  it("the brand's inks read on the light page (WCAG AA for large text, 3:1)", () => {
    for (const ink of [brand.youInk, brand.allyInk])
      for (const bg of [light.bg, light.panel]) expect(contrast(ink, bg)).toBeGreaterThanOrEqual(3);
  });
});

describe('every scheme: the css and the ts agree', () => {
  for (const s of SCHEMES.filter((s) => s !== 'hud')) {
    it.each([
      [`:root[data-scheme="${s}"]:not([data-theme="light"])`, schemes[s].dark],
      [`:root[data-scheme="${s}"][data-theme="light"]`, schemes[s].light],
    ] as const)('%s', (selector, palette: SchemePalette) => {
      const decl = block(selector);
      for (const [k, v] of Object.entries(palette)) {
        const name = k === 'player' ? 'race-player' : kebab(k);
        expect(decl[name], k).toBe(v);
      }
    });
  }
  it('neon is the default, and every scheme is listed', () => {
    expect(DEFAULT_SCHEME).toBe('neon');
    expect(Object.keys(schemes).sort()).toEqual([...SCHEMES].sort());
  });
});

describe('every text colour reads on every surface (WCAG AA, 4.5:1)', () => {
  for (const [name, p] of [
    ['dark', dark],
    ['light', light],
    ...SCHEMES.flatMap((s) => [
      [`${s} dark`, schemes[s].dark],
      [`${s} light`, schemes[s].light],
    ]),
  ] as [string, Palette][]) {
    for (const fg of ['text', 'textMuted', 'holo', 'alarm', 'gold'] as const) {
      for (const bg of ['bg', 'panel', 'panelRaised'] as const) {
        it(`${name}: ${fg} on ${bg}`, () =>
          expect(contrast(p[fg], p[bg])).toBeGreaterThanOrEqual(4.5));
      }
    }
  }
});

describe('the board palettes: the css and the ts agree, and every text role reads', () => {
  for (const name of PALETTES) {
    for (const mode of ['dark', 'light'] as const) {
      const p = palettes[name][mode];
      const sel =
        mode === 'dark'
          ? `:root[data-palette="${name}"]:not([data-theme="light"])`
          : `:root[data-palette="${name}"][data-theme="light"]`;
      it(`${name} ${mode}: the css says the ts`, () => {
        const decl = block(sel);
        for (const [k, v] of Object.entries(p)) {
          const key = kebab(k);
          expect(decl[key], `${sel} --mn-${key}`).toBe(v);
        }
      });
      for (const fg of [
        'text',
        'textMuted',
        'holo',
        'alarm',
        'gold',
        'successText',
        'error',
        'info',
        'warning',
      ] as const) {
        for (const bg of ['bg', 'panel', 'panelRaised'] as const) {
          it(`${name} ${mode}: ${fg} on ${bg}`, () =>
            expect(contrast(p[fg], p[bg])).toBeGreaterThanOrEqual(4.5));
        }
      }
      for (const stop of ['hot', 'coral', 'amber'] as const) {
        it(`${name} ${mode}: the words on the gradient read on its ${stop} stop`, () =>
          expect(contrast(p.onBrand, p[stop])).toBeGreaterThanOrEqual(4.5));
      }
    }
  }
});
