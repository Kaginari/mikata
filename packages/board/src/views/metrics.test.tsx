// @vitest-environment jsdom
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { fixtureInsights } from '../data/insightsFixtures.ts';
import { parseIntegrations } from '../data/integrations.ts';
import { dollars, parseMetrics, silent, totals } from '../data/metrics.ts';
import { fixtureIntegrations, fixtureMetrics } from '../data/metricsFixtures.ts';
import { IntegrationsView } from './Integrations.tsx';
import { MetricsView } from './Metrics.tsx';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let host: HTMLDivElement;
let mounted: ReturnType<typeof createRoot> | undefined;
let asked: string[];

beforeEach(() => {
  host = document.createElement('div');
  document.body.append(host);
  asked = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: string) => {
      const u = new URL(url, 'http://board');
      asked.push(`${u.pathname}${u.search}`);
      const reply = (status: number, body: unknown) => ({
        ok: status < 400,
        status,
        json: async () => body,
      });
      if (u.pathname === '/api/session') return reply(200, { csrf: 'ab'.repeat(32) });
      if (u.pathname === '/api/metrics') return reply(200, fixtureMetrics);
      if (u.pathname === '/api/insights') return reply(200, fixtureInsights);
      if (u.pathname === '/api/integrations') return reply(200, fixtureIntegrations);
      return reply(404, 'not found');
    }),
  );
});

afterEach(async () => {
  await act(async () => mounted?.unmount());
  mounted = undefined;
  host.remove();
  vi.unstubAllGlobals();
});

async function mount(view: React.ReactNode) {
  const root = createRoot(host);
  mounted = root;
  await act(async () => root.render(view));
  await act(async () => {});
}

test('unpriced never reads $0', () => {
  expect(dollars(null, 2)).toBe('unpriced (2)');
  expect(dollars(null, 0)).toBe('no calls');
  expect(dollars(3.12, 2)).toBe('$3.12 (2 unpriced)');
  expect(dollars(0, 0)).toBe('$0.00');
  expect(totals(fixtureMetrics.usage.value?.total ?? (undefined as never))).toBe(
    '$3.12 (2 unpriced) · 1.9M wt',
  );
  expect(silent('2026-10-20T08:00:00Z')).toBe('silent since 2026-10-20');
  expect(silent(null)).toBe('silent — no call booked yet');
});

test('the answers are read part by part: a part that does not hold reads unknown', () => {
  expect(parseMetrics(fixtureMetrics)).toEqual(fixtureMetrics);
  const broken = parseMetrics({
    ...fixtureMetrics,
    usage: { ...fixtureMetrics.usage, value: { total: 1 } },
  });
  expect(broken?.usage.value).toBeNull();
  expect(broken?.usage.why).toBe('this part of the answer did not read');
  expect(broken?.scores.value).toEqual(fixtureMetrics.scores.value);
  expect(parseMetrics({})).toBeUndefined();
  expect(parseIntegrations(fixtureIntegrations)).toEqual(fixtureIntegrations);
  expect(parseIntegrations({ board: {} })).toBeUndefined();
});

test('a part of the runs that does not hold reads unknown, the rest stands', () => {
  const broken = parseMetrics({
    ...fixtureMetrics,
    runs: { ...fixtureMetrics.runs, value: { rows: [{ run: 1 }] } },
  });
  expect(broken?.runs.value).toBeNull();
  expect(broken?.usage.value).toEqual(fixtureMetrics.usage.value);
});

test('the metrics page: every ask as numbers, charts with a legend, inert words', async () => {
  await mount(<MetricsView load={{ state: 'loading' }} />);
  const text = host.textContent ?? '';
  // The headline numbers.
  const kpis = host.querySelector('[aria-label="Headline numbers"]')?.textContent ?? '';
  expect(kpis).toContain('Asks13');
  expect(kpis).toContain('6 calls unpriced');
  expect(kpis).not.toContain('$0.000 a landed');
  // Every ask a bar, with how it ended said in a mark, and a legend.
  expect(host.querySelectorAll('.mx-hit')).toHaveLength(13);
  expect(host.querySelector('svg[aria-label^="Every ask"]')).not.toBeNull();
  expect(host.querySelector('.mx-legend')?.textContent).toContain('Model');
  // How asks end and why they stop.
  expect(text).toContain('Blocked by the end gate');
  for (const why of ['steps', 'clock', 'interrupted', 'error']) expect(text).toContain(why);
  // Models, harnesses and the Done when list, side by side.
  expect(text).toContain('qwen/qwen3.5-35b-a3b');
  expect(text).toContain('-reader');
  expect(text).toContain('default units');
  expect(text).toContain('with a Done when list');
  // The hidden characters of a model's name stay text, shown as escapes; nothing runs.
  expect(host.querySelector('.safe-esc')?.textContent).toBe('\\u{202e}');
  expect(host.querySelector('script')).toBeNull();
  // What the numbers do not cover.
  expect(text).toContain('1 run was not read');
});

test('a window tab asks for that window; a model narrows the page', async () => {
  await mount(<MetricsView load={{ state: 'loading' }} />);
  const tab = [...host.querySelectorAll('button')].find((b) => b.textContent === '28d');
  await act(async () => tab?.click());
  expect(asked.some((a) => a.startsWith('/api/insights?world=all&window=28d'))).toBe(true);
  const select = host.querySelector('select[aria-label="Model"]') as HTMLSelectElement;
  await act(async () => {
    select.value = 'qwen/qwen3.5-35b-a3b';
    select.dispatchEvent(new Event('change', { bubbles: true }));
  });
  expect(host.querySelectorAll('.mx-hit')).toHaveLength(4);
});

test('the integrations page: key sources never values, holes, MCP scope, origins, read-only', async () => {
  await mount(<IntegrationsView theme="auto" />);
  const text = host.textContent ?? '';
  expect(text).toContain('env OPENROUTER_API_KEY: set');
  expect(text).toContain('env ANTHROPIC_API_KEY: unset');
  expect(text).toContain('hidden: not a variable name');
  expect(text).toContain('file ~/.config/mikata/openrouter.key: readable (0600)');
  expect(text).toContain('file ~/open.key: refused, its mode is 0644');
  expect(text).toContain('providers.openrouter.api_key');
  expect(text).toContain('kijin');
  expect(text).toContain('read · submit = outward · network');
  expect(text).toContain('global file only');
  expect(text).toContain('7878');
  expect(text).toContain('How to add an MCP server');
  expect(host.querySelector('img')).toBeNull();
  expect(text).toContain('\\u{202e}');
  // Read-only: nothing to type into, no button but copying.
  expect(host.querySelectorAll('input, textarea, select')).toHaveLength(0);
  for (const b of host.querySelectorAll('button')) expect(b.textContent).toMatch(/^Cop/);
});
