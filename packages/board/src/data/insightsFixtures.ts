/**
 * Insights as the board server answers them, for the metrics page's tests: asks over a week, two
 * models (one named with hidden characters), two compositions, every way an ask ends.
 */
import type { BoardInsights, InsightAsk } from '@mikata/protocol';

/** A model name a body could have written: it must stay text. */
export const HOSTILE_MODEL = 'evil/‮model<script>';

const base: InsightAsk = {
  world: 'w-live',
  run: 'r1795600000000-100',
  n: 1,
  at: '2026-11-20T10:00:00Z',
  model: 'deepseek/deepseek-v4-flash',
  mode: 'light-coding',
  units: [],
  outcome: 'landed',
  cause: null,
  listed: 2,
  turns: 8,
  modelMs: 18_000,
  calls: 9,
  toolMs: 6_000,
  failed: 0,
  verdicts: 1,
  writes: 1,
  added: 12,
  removed: 3,
  tested: true,
  reads: 4,
  rereads: 1,
  input: 30_000,
  output: 2_000,
  cacheRead: 90_000,
  usd: 0.012,
  unpriced: 0,
};

const ask = (n: number, over: Partial<InsightAsk>): InsightAsk => ({
  ...base,
  run: `r17956${String(n).padStart(5, '0')}0000-${100 + n}`,
  at: `2026-11-${String(20 + Math.floor(n / 4)).padStart(2, '0')}T${String(9 + (n % 8)).padStart(2, '0')}:00:00Z`,
  ...over,
});

export const fixtureInsights: BoardInsights = {
  world: 'all',
  window: '7d',
  asks: [
    ask(0, {}),
    ask(1, { modelMs: 42_000, toolMs: 31_000, turns: 14, added: 40, removed: 8 }),
    ask(2, {
      outcome: 'answered',
      verdicts: 0,
      writes: 0,
      added: 0,
      removed: 0,
      tested: null,
      listed: 0,
    }),
    ask(3, { model: 'qwen/qwen3.5-35b-a3b', usd: 0.004, modelMs: 9_000, toolMs: 2_000 }),
    ask(4, {
      model: 'qwen/qwen3.5-35b-a3b',
      outcome: 'blocked',
      verdicts: 3,
      failed: 2,
      usd: 0.02,
    }),
    ask(5, {
      outcome: 'stopped',
      cause: 'steps',
      verdicts: 1,
      turns: 30,
      modelMs: 120_000,
      toolMs: 300_000,
    }),
    ask(6, { units: ['-reader'], listed: 0 }),
    ask(7, {
      units: ['-reader'],
      outcome: 'stopped',
      cause: 'interrupted',
      verdicts: 0,
      tested: false,
    }),
    ask(8, { model: HOSTILE_MODEL, usd: null, unpriced: 6 }),
    ask(9, { model: 'qwen/qwen3.5-35b-a3b', outcome: 'stopped', cause: 'clock', verdicts: 0 }),
    ask(10, { outcome: 'failed', cause: 'error', verdicts: 0, tested: null, writes: 0 }),
    ask(11, { model: 'qwen/qwen3.5-35b-a3b', usd: 0.006 }),
    ask(12, { outcome: 'open', verdicts: 0 }),
  ],
  tools: [
    { name: 'bash', calls: 197, failed: 32, ms: 1_500_000 },
    { name: 'read', calls: 67, failed: 0, ms: 4_000 },
    { name: 'edit', calls: 64, failed: 1, ms: 9_000 },
    { name: 'check', calls: 26, failed: 0, ms: 3_000 },
  ],
  unread: 1,
  notes: ['durations: the usage ledger was read from its last 64 MiB'],
};
