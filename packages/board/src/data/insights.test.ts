import { expect, test } from 'vitest';
import {
  byDay,
  byList,
  byModel,
  byUnits,
  causes,
  duration,
  ends,
  median,
  money,
  parseInsights,
  signals,
  summarize,
} from './insights.ts';
import { fixtureInsights } from './insightsFixtures.ts';

const asks = fixtureInsights.asks;

test('an answer is read row by row: a row that does not hold is dropped and counted unread', () => {
  expect(parseInsights(fixtureInsights)).toEqual(fixtureInsights);
  const broken = parseInsights({
    ...fixtureInsights,
    asks: [...asks, { ...asks[0], outcome: 'won' }, { ...asks[0], turns: -1 }, 'x'],
  });
  expect(broken?.asks).toHaveLength(asks.length);
  expect(broken?.unread).toBe(fixtureInsights.unread + 3);
  expect(parseInsights({})).toBeUndefined();
  expect(parseInsights({ ...fixtureInsights, asks: 'all' })).toBeUndefined();
});

test('the headline numbers: landed of the finished asks, dollars never a made-up zero', () => {
  const s = summarize(asks);
  expect(s.asks).toBe(13);
  // One ask runs still: twelve finished, six of them landed (asks 0, 1, 3, 6, 8, 11).
  expect(s.landed).toBe(6);
  expect(s.landedShare).toBeCloseTo(6 / 12);
  expect(s.unpriced).toBe(6);
  expect(s.usd).toBeGreaterThan(0);
  expect(s.perLanded).toBeCloseTo((s.usd ?? 0) / 6);
  expect(s.cacheHit).toBeCloseTo(0.75);
  expect(summarize([]).usd).toBeUndefined();
  expect(money(undefined)).toBe('—');
  expect(money(0.0123)).toBe('$0.012');
  expect(money(3.456)).toBe('$3.46');
});

test('how asks end, and why they stopped', () => {
  expect(ends(asks)).toEqual([
    ['landed', 6],
    ['blocked', 1],
    ['answered', 1],
    ['stopped', 3],
    ['failed', 1],
    ['open', 1],
  ]);
  expect(causes(asks)).toEqual([
    ['clock', 1],
    ['error', 1],
    ['interrupted', 1],
    ['steps', 1],
  ]);
});

test('models, harnesses and the Done when list, side by side', () => {
  expect(byModel(asks).map((g) => [g.key, g.asks])).toEqual([
    ['deepseek/deepseek-v4-flash', 8],
    ['qwen/qwen3.5-35b-a3b', 4],
    ['evil/‮model<script>', 1],
  ]);
  expect(byUnits(asks).map((g) => g.key)).toEqual(['default units', '-reader']);
  const list = byList(asks);
  expect(list.map((g) => g.key)).toEqual(['with a Done when list', 'without a list']);
});

test('asks per day by outcome, oldest first; the signals of careful work', () => {
  const days = byDay(asks);
  expect(days.map(([d]) => d)).toEqual(['2026-11-20', '2026-11-21', '2026-11-22', '2026-11-23']);
  expect(days.reduce((n, [, row]) => n + row.reduce((a, b) => a + b, 0), 0)).toBe(13);
  const s = signals(asks);
  expect([s.tested, s.wrote]).toEqual([10, 11]);
  expect(s.repaired).toBe(1);
  expect(median([3, 1, 2])).toBe(2);
  expect(median([1, 2, 3, 4])).toBe(2.5);
  expect(duration(42)).toBe('42s');
  expect(duration(185)).toBe('3m 05s');
});
