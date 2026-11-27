import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import {
  brand,
  contrast,
  DEFAULT_SCHEME,
  dark,
  failure,
  light,
  PALETTES,
  type Palette,
  palettes,
  races,
  SCHEMES,
  type SchemePalette,
  schemes,
  states,
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

describe("a run's states and failure, in Bio Digital and Bio Pearl", () => {
  const sides = [
    ['dark', ':root[data-scheme="neon"]:not([data-theme="light"])', schemes.neon.dark],
    ['light', ':root[data-scheme="neon"][data-theme="light"]', schemes.neon.light],
  ] as const;

  it.each(sides)('%s: the css and the ts agree', (side, selector) => {
    const decl = block(selector);
    for (const [k, v] of Object.entries(states[side])) expect(decl[`state-${k}`], k).toBe(v);
    for (const [k, v] of Object.entries(failure[side])) expect(decl[`error-${k}`], k).toBe(v);
  });

  it.each(sides)(
    '%s: every state reads as a mark on its panels (3:1), but disabled',
    (side, _s, p) => {
      for (const [k, v] of Object.entries(states[side])) {
        if (k === 'disabled') continue;
        for (const bg of [p.bg, p.panel, p.panelRaised])
          expect(contrast(v, bg), `${k} on ${bg}`).toBeGreaterThanOrEqual(3);
      }
    },
  );

  it.each(sides)(
    '%s: failure text reads on its own tint and on the panel (AA, 4.5:1)',
    (side, _s, p) => {
      const f = failure[side];
      for (const bg of [f.bg, p.panel])
        expect(contrast(f.text, bg), bg).toBeGreaterThanOrEqual(4.5);
    },
  );
});
