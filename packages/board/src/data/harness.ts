/**
 * The Harness Builder's data: the units a session in the world would open with
 * (`GET /api/w/<id>/units`), a profile's changes over them, the switches the human flips, and the
 * composition that goes back as `POST …/sessions`'s `profile` and `units` — the same words
 * `mikata run --profile` and `--units` take.
 */
import type { BoardProfile, BoardUnit, BoardUnits } from '@mikata/protocol';
import { useEffect, useState } from 'react';
import { getJson } from './chats.ts';

export type HarnessLoad =
  | { state: 'loading' }
  | { state: 'ready'; data: BoardUnits }
  | { state: 'failed'; why: string };

/** The server's answer as a harness, or `undefined` when it does not hold together. */
export function harnessOf(d: unknown): BoardUnits | undefined {
  if (typeof d !== 'object' || d === null) return undefined;
  const { units, profiles, holes } = d as Record<string, unknown>;
  if (!Array.isArray(units) || !Array.isArray(profiles) || !Array.isArray(holes)) return undefined;
  const unit = (u: unknown) =>
    typeof u === 'object' &&
    u !== null &&
    typeof (u as BoardUnit).id === 'string' &&
    typeof (u as BoardUnit).on === 'boolean';
  if (!units.every(unit)) return undefined;
  if (!holes.every((h) => typeof h === 'string')) return undefined;
  return { units, profiles, holes } as BoardUnits;
}

/** The world's harness, read once. */
export function useHarness(world: string): HarnessLoad {
  const [load, setLoad] = useState<HarnessLoad>({ state: 'loading' });
  useEffect(() => {
    const ctl = new AbortController();
    getJson(`/api/w/${encodeURIComponent(world)}/units`, ctl.signal)
      .then((d) => {
        const units = harnessOf(d);
        setLoad(
          units
            ? { state: 'ready', data: units }
            : { state: 'failed', why: 'the harness answer did not hold together' },
        );
      })
      .catch((e: unknown) => {
        if (!ctl.signal.aborted) {
          setLoad({ state: 'failed', why: e instanceof Error ? e.message : String(e) });
        }
      });
    return () => ctl.abort();
  }, [world]);
  return load;
}

/** A composition's changes, as `--units` reads them: `-plan,+plan_lite` → plan off, plan_lite on. */
export function changes(list: string): Map<string, boolean> {
  const out = new Map<string, boolean>();
  for (const item of list.split(',')) {
    const t = item.trim();
    if (t.length < 2) continue;
    if (t[0] === '+') out.set(t.slice(1), true);
    else if (t[0] === '-') out.set(t.slice(1), false);
  }
  return out;
}

/** Each unit on or off as a session would open: the configuration's, then the profile's. */
export function opening(units: readonly BoardUnit[], profile?: BoardProfile): Map<string, boolean> {
  const out = new Map(units.map((u) => [u.id, u.on]));
  if (profile) {
    for (const [id, on] of changes(profile.units)) if (out.has(id)) out.set(id, on);
  }
  return out;
}

/** The switches the human flipped over `base`, as `--units` writes them, in the units' order. */
export function composed(
  units: readonly BoardUnit[],
  base: ReadonlyMap<string, boolean>,
  want: ReadonlyMap<string, boolean>,
): string {
  return units
    .filter((u) => want.get(u.id) !== base.get(u.id))
    .map((u) => `${want.get(u.id) ? '+' : '-'}${u.id}`)
    .join(',');
}

/** A TOML string, quoted. */
const q = (s: string) => `"${s.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;

/**
 * The composition as a global file's profile would hold it: paste it into `config.toml` and
 * `mikata run --profile <name>` opens the same harness.
 */
export function toml(
  name: string,
  units: readonly BoardUnit[],
  want: ReadonlyMap<string, boolean>,
): string {
  const off = units.filter((u) => !want.get(u.id));
  const on = units.filter((u) => want.get(u.id) && !u.on);
  const list = [...off.map((u) => `-${u.id}`), ...on.map((u) => `+${u.id}`)].join(',');
  const lines = [`[profiles.${name}]`, `units = ${q(list)}`];
  const named = (us: readonly BoardUnit[]) => us.map((u) => u.id).join(', ') || 'none';
  lines.push('', `# on:  ${named(units.filter((u) => want.get(u.id)))}`, `# off: ${named(off)}`);
  return lines.join('\n');
}
