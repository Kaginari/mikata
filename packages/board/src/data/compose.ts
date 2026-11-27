/**
 * The composition the Harness Builder sets for a workspace — a profile, the switches flipped over
 * it, the `--units` they make, and the model the session rides — which new sessions in that
 * workspace open with (`POST …/sessions` with `profile`, `units` and `model`). The global file is
 * never written.
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
  /** The model a new session rides, as the configuration names one; none: the configured one. */
  model?: string;
  /** What each ask of a new session may spend; a limit not named is the configuration's. */
  limits?: Limits;
}

/** A session's limits as Start session asks them (`POST …/sessions`). */
export interface Limits {
  /** The steps an ask may take. */
  steps?: number;
  /** An ask's clock, in minutes. */
  minutes?: number;
  /** The context stress line, in tokens. */
  context?: number;
}

/** Each limit's bounds, as the server takes them (`board::limits_asked`). */
export const LIMITS: Readonly<Record<keyof Limits, readonly [number, number]>> = {
  steps: [10, 5000],
  minutes: [5, 1440],
  context: [8000, 10_000_000],
};

/** `v` as limits, or `undefined` when one does not hold. */
export function limitsOf(v: unknown): Limits | undefined {
  if (typeof v !== 'object' || v === null) return undefined;
  const out: Limits = {};
  for (const [k, n] of Object.entries(v)) {
    const range = LIMITS[k as keyof Limits];
    if (!range || typeof n !== 'number' || !Number.isInteger(n)) return undefined;
    if (n < range[0] || n > range[1]) return undefined;
    out[k as keyof Limits] = n;
  }
  return out;
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
/** A model's name as the server takes it (`board::model_name`). */
export const MODEL = /^[A-Za-z0-9_.:/@+-]{1,200}$/;

/** One stored composition, or `undefined` when it does not hold together. */
export function compositionOf(v: unknown): Composition | undefined {
  if (typeof v !== 'object' || v === null) return undefined;
  const { profile, flips, units, model, limits } = v as Record<string, unknown>;
  if (typeof profile !== 'string' || (profile !== '' && !WORD.test(profile))) return undefined;
  if (typeof units !== 'string' || !UNITS.test(units)) return undefined;
  if (model !== undefined && (typeof model !== 'string' || !MODEL.test(model))) return undefined;
  const bounded = limits === undefined ? {} : limitsOf(limits);
  if (!bounded) return undefined;
  if (typeof flips !== 'object' || flips === null) return undefined;
  const kept: Record<string, boolean> = {};
  for (const [id, on] of Object.entries(flips)) {
    if (!WORD.test(id) || typeof on !== 'boolean') return undefined;
    kept[id] = on;
  }
  return {
    profile,
    flips: kept,
    units,
    ...(model ? { model } : {}),
    ...(Object.keys(bounded).length > 0 ? { limits: bounded } : {}),
  };
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
  const kept: Composition = {
    profile: c.profile,
    flips: c.flips,
    units: c.units,
    ...(c.model ? { model: c.model } : {}),
    ...(c.limits && Object.keys(c.limits).length > 0 ? { limits: c.limits } : {}),
  };
  saved = { ...(saved ?? load()), [world]: kept };
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
export function sessionOptions(world: string): Opened {
  const c = composition(world);
  const o = once.get(world);
  if (o) setOnce(world, undefined);
  const units = o ? o.units : c.units;
  return {
    ...(c.profile ? { profile: c.profile } : {}),
    ...(units ? { units } : {}),
    ...(c.model ? { model: c.model } : {}),
    ...(c.limits ?? {}),
  };
}

/** What a session started here was opened with, as sent. */
export type Opened = { profile?: string; units?: string; model?: string } & Limits;

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
    const { profile, units, model } = v as Record<string, unknown>;
    if (profile !== undefined && (typeof profile !== 'string' || !WORD.test(profile))) {
      return undefined;
    }
    if (units !== undefined && (typeof units !== 'string' || !UNITS.test(units))) return undefined;
    if (model !== undefined && (typeof model !== 'string' || !MODEL.test(model))) return undefined;
    const { steps, minutes, context } = v as Record<string, unknown>;
    const bounded = limitsOf(
      Object.fromEntries(
        Object.entries({ steps, minutes, context }).filter(([, n]) => n !== undefined),
      ),
    );
    if (!bounded) return undefined;
    return {
      ...(profile ? { profile } : {}),
      ...(units ? { units } : {}),
      ...(model ? { model } : {}),
      ...bounded,
    };
  } catch {
    return undefined;
  }
}
