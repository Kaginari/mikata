/**
 * The metrics page's numbers: every ask of the workspace's session runs, read by the server from
 * their journals and the usage ledger (`GET /api/insights`, `board::insights`), and what the page
 * folds from them. The page checks the shape; a row that does not read is dropped, never guessed.
 */
import type { BoardInsights, InsightAsk, InsightTool, Outcome } from '@mikata/protocol';
import { useCallback, useEffect, useState } from 'react';
import { type Change, usePush } from './push.ts';
import { establish, signedOut } from './session.ts';

export type { BoardInsights, InsightAsk, InsightTool, Outcome };

type Obj = Record<string, unknown>;
const obj = (v: unknown): Obj | undefined =>
  typeof v === 'object' && v !== null && !Array.isArray(v) ? (v as Obj) : undefined;
const str = (v: unknown): v is string => typeof v === 'string';
const num = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v) && v >= 0;

export const OUTCOMES: readonly Outcome[] = [
  'landed',
  'blocked',
  'answered',
  'stopped',
  'failed',
  'open',
];

const NUMBERS = [
  'n',
  'listed',
  'turns',
  'modelMs',
  'calls',
  'toolMs',
  'failed',
  'verdicts',
  'writes',
  'added',
  'removed',
  'reads',
  'rereads',
  'input',
  'output',
  'cacheRead',
  'unpriced',
] as const;

/** One ask, or `undefined` when it does not hold together. */
export function askOf(v: unknown): InsightAsk | undefined {
  const o = obj(v);
  if (!o || !str(o.world) || !str(o.run) || !str(o.at)) return undefined;
  if (!OUTCOMES.includes(o.outcome as Outcome)) return undefined;
  if (!NUMBERS.every((k) => num(o[k]))) return undefined;
  const opt = (x: unknown) => x === null || x === undefined || str(x);
  if (!opt(o.model) || !opt(o.mode) || !opt(o.cause)) return undefined;
  if (!Array.isArray(o.units) || !o.units.every(str)) return undefined;
  if (!(o.usd === null || o.usd === undefined || num(o.usd))) return undefined;
  if (!(o.tested === null || o.tested === undefined || typeof o.tested === 'boolean')) {
    return undefined;
  }
  return {
    ...(o as unknown as InsightAsk),
    model: (o.model as string | null) ?? null,
    mode: (o.mode as string | null) ?? null,
    cause: (o.cause as string | null) ?? null,
    usd: (o.usd as number | null) ?? null,
    tested: (o.tested as boolean | null) ?? null,
  };
}

/** The insights an answer holds; `undefined` when it is not insights at all. */
export function parseInsights(body: unknown): BoardInsights | undefined {
  const o = obj(body);
  if (!o || !str(o.world) || !str(o.window) || !num(o.unread)) return undefined;
  if (!Array.isArray(o.asks) || !Array.isArray(o.tools) || !Array.isArray(o.notes)) {
    return undefined;
  }
  const asks = o.asks.map(askOf).filter((a): a is InsightAsk => a !== undefined);
  const tools = o.tools.flatMap((t): InsightTool[] => {
    const x = obj(t);
    return x && str(x.name) && num(x.calls) && num(x.failed) && num(x.ms)
      ? [{ name: x.name, calls: x.calls, failed: x.failed, ms: x.ms }]
      : [];
  });
  return {
    world: o.world,
    window: o.window,
    asks,
    tools,
    unread: o.unread + (o.asks.length - asks.length),
    notes: o.notes.filter(str),
  };
}

export type InsightsLoad =
  | { state: 'loading' }
  | { state: 'ready'; data: BoardInsights }
  | { state: 'failed'; reason: string };

/**
 * The insights of `world` (`all` for every workspace) over `window`, read again when the push
 * channel says a world changed: a running session's asks move the page without a reload, and a
 * reread keeps what is shown until its answer comes.
 */
export function useInsights(world: string, window: string): InsightsLoad {
  const [load, setLoad] = useState<InsightsLoad>({ state: 'loading' });
  const [tick, setTick] = useState(0);
  const on = useCallback(
    (c: Change) => {
      if (c.kind === 'worlds' || world === 'all' || c.id === world) setTick((t) => t + 1);
    },
    [world],
  );
  usePush(on);
  // A new world or window starts from nothing; a push rereads in place.
  // biome-ignore lint/correctness/useExhaustiveDependencies: a new filter, not a reread, clears
  useEffect(() => setLoad({ state: 'loading' }), [world, window]);
  // biome-ignore lint/correctness/useExhaustiveDependencies: `tick` asks for a reread
  useEffect(() => {
    const ctl = new AbortController();
    const q = `?world=${encodeURIComponent(world)}&window=${encodeURIComponent(window)}`;
    establish()
      .then(() =>
        fetch(`/api/insights${q}`, {
          signal: ctl.signal,
          credentials: 'same-origin',
          headers: { accept: 'application/json' },
        }),
      )
      .then(async (res) => {
        if (res.status === 401) {
          signedOut();
          throw new Error('this browser holds no board session');
        }
        if (res.status === 404) throw new Error('the board lists no world with this id');
        if (!res.ok) throw new Error(`the board server answered ${res.status}`);
        const i = parseInsights(await res.json());
        if (!i) throw new Error('the answer is not insights');
        setLoad({ state: 'ready', data: i });
      })
      .catch((e: unknown) => {
        if (ctl.signal.aborted) return;
        setLoad({ state: 'failed', reason: e instanceof Error ? e.message : String(e) });
      });
    return () => ctl.abort();
  }, [world, window, tick]);
  return load;
}

// ── what the page folds ─────────────────────────────────────────────────────────────────────────

/** An ask's time in the model and in tools, in seconds. */
export const secs = (a: InsightAsk) => (a.modelMs + a.toolMs) / 1000;

export function median(xs: readonly number[]): number {
  if (xs.length === 0) return 0;
  const s = [...xs].sort((a, b) => a - b);
  const m = Math.floor(s.length / 2);
  return s.length % 2 ? (s[m] ?? 0) : ((s[m - 1] ?? 0) + (s[m] ?? 0)) / 2;
}

/** What a set of asks did, in the page's headline numbers. */
export interface Summary {
  asks: number;
  landed: number;
  /** Landed of those the end gate judged or that finished: open asks are not counted. */
  landedShare: number | undefined;
  usd: number | undefined;
  perLanded: number | undefined;
  unpriced: number;
  medianSecs: number;
  medianTurns: number;
  cacheHit: number | undefined;
  added: number;
  removed: number;
}

export function summarize(asks: readonly InsightAsk[]): Summary {
  const done = asks.filter((a) => a.outcome !== 'open');
  const landed = asks.filter((a) => a.outcome === 'landed').length;
  const priced = asks.filter((a) => a.usd !== null);
  const usd = priced.length ? priced.reduce((s, a) => s + (a.usd ?? 0), 0) : undefined;
  const input = asks.reduce((s, a) => s + a.input, 0);
  const cached = asks.reduce((s, a) => s + a.cacheRead, 0);
  return {
    asks: asks.length,
    landed,
    landedShare: done.length ? landed / done.length : undefined,
    usd,
    perLanded: usd !== undefined && landed > 0 ? usd / landed : undefined,
    unpriced: asks.reduce((s, a) => s + a.unpriced, 0),
    medianSecs: median(asks.map(secs)),
    medianTurns: median(asks.map((a) => a.turns)),
    cacheHit: input + cached > 0 ? cached / (input + cached) : undefined,
    added: asks.reduce((s, a) => s + a.added, 0),
    removed: asks.reduce((s, a) => s + a.removed, 0),
  };
}

/** How many asks ended each way, in [`OUTCOMES`] order, the empty ones left out. */
export function ends(asks: readonly InsightAsk[]): [Outcome, number][] {
  return OUTCOMES.map((o): [Outcome, number] => [
    o,
    asks.filter((a) => a.outcome === o).length,
  ]).filter(([, n]) => n > 0);
}

/** Why asks stopped or failed: cause → count, most first. */
export function causes(asks: readonly InsightAsk[]): [string, number][] {
  const m = new Map<string, number>();
  for (const a of asks) if (a.cause) m.set(a.cause, (m.get(a.cause) ?? 0) + 1);
  return [...m].sort((x, y) => y[1] - x[1] || x[0].localeCompare(y[0]));
}

/** A group of asks and its summary. */
export interface Group {
  key: string;
  asks: number;
  s: Summary;
}

function groups(asks: readonly InsightAsk[], key: (a: InsightAsk) => string): Group[] {
  const m = new Map<string, InsightAsk[]>();
  for (const a of asks) {
    const k = key(a);
    m.set(k, [...(m.get(k) ?? []), a]);
  }
  return [...m]
    .map(([k, xs]) => ({ key: k, asks: xs.length, s: summarize(xs) }))
    .sort((x, y) => y.asks - x.asks || x.key.localeCompare(y.key));
}

/** Asks by the model they rode. */
export const byModel = (asks: readonly InsightAsk[]) => groups(asks, (a) => a.model ?? 'unknown');

/** Asks by the run's units changed from the default: the harness each ran with. */
export const byUnits = (asks: readonly InsightAsk[]) =>
  groups(asks, (a) => (a.units.length ? a.units.join(' ') : 'default units'));

/** Asks that carried a `Done when` list, and asks that carried none. */
export const byList = (asks: readonly InsightAsk[]) =>
  groups(asks, (a) => (a.listed > 0 ? 'with a Done when list' : 'without a list'));

/** Asks per day, by outcome: `[day, counts in OUTCOMES order]`, oldest first. */
export function byDay(asks: readonly InsightAsk[]): [string, number[]][] {
  const m = new Map<string, number[]>();
  for (const a of asks) {
    const d = a.at.slice(0, 10);
    const row = m.get(d) ?? OUTCOMES.map(() => 0);
    const i = OUTCOMES.indexOf(a.outcome);
    row[i] = (row[i] ?? 0) + 1;
    m.set(d, row);
  }
  return [...m].sort((x, y) => x[0].localeCompare(y[0]));
}

/** How carefully the asks worked. */
export function signals(asks: readonly InsightAsk[]) {
  const wrote = asks.filter((a) => a.tested !== null);
  const calls = asks.reduce((s, a) => s + a.calls, 0);
  const reads = asks.reduce((s, a) => s + a.reads, 0);
  return {
    tested: wrote.filter((a) => a.tested).length,
    wrote: wrote.length,
    calls,
    failedShare: calls ? asks.reduce((s, a) => s + a.failed, 0) / calls : undefined,
    rereadShare: reads ? asks.reduce((s, a) => s + a.rereads, 0) / reads : undefined,
    gated: asks.filter((a) => a.verdicts > 0).length,
    repaired: asks.filter((a) => a.verdicts > 1).length,
  };
}

/** Dollars, never a made-up zero: none reads as an em dash. */
export function money(usd: number | undefined): string {
  if (usd === undefined) return '—';
  return usd < 1 ? `$${usd.toFixed(3)}` : `$${usd.toFixed(2)}`;
}

/** Seconds as `12s` or `3m 04s`. */
export function duration(s: number): string {
  const t = Math.round(s);
  return t >= 60 ? `${Math.floor(t / 60)}m ${String(t % 60).padStart(2, '0')}s` : `${t}s`;
}

export const pct = (x: number | undefined) => (x === undefined ? '—' : `${Math.round(x * 100)}%`);
