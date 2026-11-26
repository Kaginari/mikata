/**
 * The composition the Harness Builder sets for a workspace — a profile, the switches flipped over
 * it, and the `--units` they make — which new sessions in that workspace open with
 * (`POST …/sessions` with `profile` and `units`). The global file is never written.
 *
 * Two kinds of switch:
 * - a guard switched off ([`GUARDS`]: the secret files, the approval card, the end gate, the
 *   hold) is never saved: it holds for the next new session from this page only, then the Builder
 *   shows it on again. A session started later always opens with its guards;
 * - any other switch is saved in this browser, per workspace.
 *
 * What each session was opened with is kept for this tab, so the session's page can say it.
 */
import { useSyncExternalStore } from 'react';

export interface Composition {
  /** The profile to start from; `''` is the configuration as it is. */
  profile: string;
  /** Units switched against the profile and kept: id → on. */
  flips: Readonly<Record<string, boolean>>;
  /** The kept changes as `--units` reads them: `-plan,+plan_lite`. */
  units: string;
}

/** Guards switched off for the next new session only. */
export interface Once {
  /** The guards: id → false. */
  flips: Readonly<Record<string, boolean>>;
  /** Every change for that one session, kept and once, as `--units` reads them. */
  units: string;
}

/** What a new session in a workspace opens with: the kept composition, and what holds once. */
export interface Composed extends Composition {
  once: Once | undefined;
}

export const NONE: Composition = { profile: '', flips: {}, units: '' };

/** The units that protect the human — the secret files, the approval card, the end gate, the hold
 * on outward acts —: switched off in the Builder, they hold for one session and are never kept. */
export const GUARDS: ReadonlySet<string> = new Set(['secrets', 'approval', 'gate', 'hold']);

const KEY = 'mikata.compose';
const OPENED = 'mikata.opened';
const WORD = /^[A-Za-z0-9_:.-]{1,64}$/;
const UNITS = /^[A-Za-z0-9_:.,+-]{0,2000}$/;

/** One stored composition, or `undefined` when it does not hold together. */
export function compositionOf(v: unknown): Composition | undefined {
  if (typeof v !== 'object' || v === null) return undefined;
  const { profile, flips, units } = v as Record<string, unknown>;
  if (typeof profile !== 'string' || (profile !== '' && !WORD.test(profile))) return undefined;
  if (typeof units !== 'string' || !UNITS.test(units)) return undefined;
  if (typeof flips !== 'object' || flips === null) return undefined;
  const kept: Record<string, boolean> = {};
  for (const [id, on] of Object.entries(flips)) {
    if (!WORD.test(id) || typeof on !== 'boolean') return undefined;
    kept[id] = on;
  }
  return { profile, flips: kept, units };
}

function load(): Record<string, Composition> {
  try {
    const raw: unknown = JSON.parse(window.localStorage.getItem(KEY) ?? '{}');
    if (typeof raw !== 'object' || raw === null) return {};
    const out: Record<string, Composition> = {};
    for (const [world, v] of Object.entries(raw)) {
      const c = compositionOf(v);
      if (c) out[world] = c;
    }
    return out;
  } catch {
    return {};
  }
}

let saved: Record<string, Composition> | undefined;
const once = new Map<string, Once>();
const views = new Map<string, Composed>();
const listeners = new Set<() => void>();

function changed(world: string) {
  views.delete(world);
  for (const l of listeners) l();
}

/** The kept composition of `world`. */
export function composition(world: string): Composition {
  saved ??= load();
  return saved[world] ?? NONE;
}

/** Keep `world`'s composition: only switches safe to keep, as [`Composition`] says. */
export function setComposition(world: string, c: Composition) {
  saved = { ...(saved ?? load()), [world]: c };
  try {
    window.localStorage.setItem(KEY, JSON.stringify(saved));
  } catch {
    // kept for this page only
  }
  changed(world);
}

/** Set, or clear, the guards switched off for `world`'s next new session. */
export function setOnce(world: string, o: Once | undefined) {
  if (o && Object.keys(o.flips).length > 0) once.set(world, o);
  else once.delete(world);
  changed(world);
}

function composed(world: string): Composed {
  let v = views.get(world);
  if (!v) {
    v = { ...composition(world), once: once.get(world) };
    views.set(world, v);
  }
  return v;
}

function subscribe(l: () => void) {
  listeners.add(l);
  return () => listeners.delete(l);
}

/** `world`'s composition, kept and once, following its changes. */
export function useComposition(world: string): Composed {
  return useSyncExternalStore(subscribe, () => composed(world));
}

/** What `startChat` sends for a new session in `world`: its composition, the guards switched off
 * once included — which are then spent. */
export function sessionOptions(world: string): { profile?: string; units?: string } {
  const c = composition(world);
  const o = once.get(world);
  if (o) setOnce(world, undefined);
  const units = o ? o.units : c.units;
  return { ...(c.profile ? { profile: c.profile } : {}), ...(units ? { units } : {}) };
}

/** What a session started here was opened with: `profile` and `units`, as sent. */
export type Opened = { profile?: string; units?: string };

/** Note what session `id` was opened with, for its page. */
export function noteOpened(id: string, with_: Opened) {
  try {
    const all = JSON.parse(window.sessionStorage.getItem(OPENED) ?? '{}') as Record<string, Opened>;
    all[id] = with_;
    window.sessionStorage.setItem(OPENED, JSON.stringify(all));
  } catch {
    // said for this page only
  }
}

/** What session `id` was opened with, when this tab opened it. */
export function openedWith(id: string): Opened | undefined {
  try {
    const all: unknown = JSON.parse(window.sessionStorage.getItem(OPENED) ?? '{}');
    const v = (all as Record<string, unknown>)[id];
    if (typeof v !== 'object' || v === null) return undefined;
    const { profile, units } = v as Record<string, unknown>;
    if (profile !== undefined && (typeof profile !== 'string' || !WORD.test(profile))) {
      return undefined;
    }
    if (units !== undefined && (typeof units !== 'string' || !UNITS.test(units))) return undefined;
    return { ...(profile ? { profile } : {}), ...(units ? { units } : {}) };
  } catch {
    return undefined;
  }
}
