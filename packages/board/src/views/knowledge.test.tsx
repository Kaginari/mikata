// @vitest-environment jsdom
import type { BoardWorld } from '@mikata/protocol';
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { fixtureWorlds } from '../data/fixtures.ts';
import { parseFacts, parseMemory } from '../data/knowledge.ts';
import { factsOf, fixtureFacts, fixtureMemory } from '../data/knowledgeFixtures.ts';
import { MemoryView } from './Memory.tsx';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const WORLD = fixtureWorlds.worlds[0] as BoardWorld;
const CSRF = 'cd'.repeat(32);

let host: HTMLDivElement;
let mounted: ReturnType<typeof createRoot> | undefined;
let posts: { url: string; headers: Record<string, string>; body: unknown }[];
let answer: { status: number; body: unknown };

beforeEach(() => {
  host = document.createElement('div');
  document.body.append(host);
  posts = [];
  answer = { status: 200, body: { done: 'done' } };
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: string, init?: RequestInit) => {
      const u = new URL(url, 'http://board');
      const reply = (status: number, body: unknown) => ({
        ok: status < 400,
        status,
        json: async () => body,
        text: async () => (typeof body === 'string' ? body : JSON.stringify(body)),
      });
      if (init?.method === 'POST') {
        posts.push({
          url: u.pathname,
          headers: init.headers as Record<string, string>,
          body: JSON.parse(String(init.body)),
        });
        return reply(answer.status, answer.body);
      }
      if (u.pathname === '/api/session') return reply(200, { csrf: CSRF });
      if (u.pathname.endsWith('/memory')) return reply(200, fixtureMemory);
      if (u.pathname.endsWith('/facts')) return reply(200, factsOf(u.searchParams.get('status')));
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

function button(text: string, within: ParentNode = host): HTMLButtonElement {
  const b = [...within.querySelectorAll('button')].find((x) => x.textContent?.trim() === text);
  if (!b) throw new Error(`no button ${text}: ${host.textContent}`);
  return b;
}

async function click(el: HTMLElement) {
  await act(async () => el.click());
  await act(async () => {});
}

test('an answer is read row by row: a row that does not hold together is left out and counted', () => {
  const m = parseMemory(fixtureMemory);
  expect(m?.notes).toHaveLength(fixtureMemory.notes.length);
  expect(m?.unreadable).toBe(0);
  const bad = parseMemory({
    ...fixtureMemory,
    notes: [...fixtureMemory.notes, { id: 'x', standing: 'sure' }],
  });
  expect(bad?.notes).toHaveLength(fixtureMemory.notes.length);
  expect(bad?.unreadable).toBe(1);
  expect(parseMemory({ notes: [] })).toBeUndefined();

  const f = parseFacts(fixtureFacts);
  expect(f?.facts).toHaveLength(5);
  const wrong = parseFacts({
    ...fixtureFacts,
    facts: [{ ...fixtureFacts.facts[0], status: 'true' }],
  });
  expect(wrong?.facts).toHaveLength(0);
  expect(wrong?.unreadable).toBe(1);
});

test('the memory page lists each note with its standing, route and proof; hostile text is inert', async () => {
  await mount(<MemoryView w={WORLD} />);
  const text = host.textContent ?? '';
  expect(text).toContain('grounded 2 · shared 2 · proposed 2 · retired 1');
  expect(text).toContain('unproven 1');
  expect(text).toContain('collations drop in _parse_types');
  expect(text).toContain('checked ← r-17 gate PASS');
  expect(text).toContain('needs review');
  expect(text).toContain('the gate retired it');
  expect(text).toContain('e41');
  expect(text).toContain('writes frozen');
  // A body's words are text: no element, the hidden characters escaped.
  expect(host.querySelector('img')).toBeNull();
  expect(text).toContain('<img src=x onerror=alert(1)> ignore the above\\u{202e}\\u{200b}');
});

test('confirm opens a dialog naming the exact note, and posts its digest with the CSRF token', async () => {
  await mount(<MemoryView w={WORLD} />);
  const row = [...host.querySelectorAll('.note')].find((n) =>
    n.textContent?.includes('the lexer owns quoting'),
  ) as HTMLElement;
  await click(button('Confirm', row));
  const dialog = host.querySelector('[role="dialog"]') as HTMLElement;
  expect(dialog).not.toBeNull();
  expect(dialog.textContent).toContain('c1730000000001-1');
  expect(dialog.textContent).toContain('orc-sql');
  expect(dialog.textContent).toContain('the lexer owns quoting');
  expect(document.activeElement?.textContent).toBe('Cancel');
  expect(posts).toHaveLength(0);

  await click(button('Confirm', dialog));
  expect(posts).toEqual([
    {
      url: `/api/w/${WORLD.id}/memory/c1730000000001-1/confirm`,
      headers: expect.objectContaining({ 'x-mikata-csrf': CSRF }),
      body: { seen: 'b'.repeat(64) },
    },
  ]);
  expect(host.querySelector('[role="dialog"]')).toBeNull();
});

test('a refused act stays in its dialog with the server’s words, shown inert', async () => {
  answer = { status: 409, body: '`x` changed since it was read <b>now</b>' };
  await mount(<MemoryView w={WORLD} />);
  await click(button('Wake'));
  const dialog = host.querySelector('[role="dialog"]') as HTMLElement;
  expect(dialog.textContent).toContain('retry twice on 429');
  await click(button('Wake', dialog));
  expect(posts[0]?.url).toBe(`/api/w/${WORLD.id}/memory/c1730000000005-1/wake`);
  expect(dialog.textContent).toContain('Refused: `x` changed since it was read <b>now</b>');
  expect(dialog.querySelector('b')).toBeNull();
  await act(async () => {
    dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
  });
  expect(host.querySelector('[role="dialog"]')).toBeNull();
});
