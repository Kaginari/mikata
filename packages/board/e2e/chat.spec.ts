/**
 * A session, on the built page under the board server's CSP, against a fake bridge: its timeline
 * of steps beside the Live run (screenshots at phone and desktop, both themes, contrast and
 * labels), hostile text inert, a reconnect that replays without duplicating, an observer that
 * cannot send, Enter queueing and Steer steering, Esc, the queue; and a past run's transcript, read-only and resumed,
 * what changed drawn under its gate, and 2,000 messages scrolled in a window.
 */
import { fileURLToPath } from 'node:url';
import { expect, type Page, test } from '@playwright/test';
import { Bridge, CHAT, hello, LIVE, openChat, PAST, richTurn, violations } from './fake.ts';

const screen = (file: string) =>
  fileURLToPath(new URL(`../test-results/screens/${file}.png`, import.meta.url));

/** Text nodes below WCAG AA on what is behind them (as board.spec.ts measures). */
function lowContrast(page: Page) {
  return page.evaluate(() => {
    type C = [number, number, number, number];
    const parse = (s: string): C | undefined => {
      const n = (s.match(/-?[\d.]+(?:e-?\d+)?/g) ?? []).map(Number);
      const [r = 0, g = 0, b = 0, a = 1] = n;
      if (s === 'transparent') return [0, 0, 0, 0];
      if (s.startsWith('color')) return [r * 255, g * 255, b * 255, a];
      if (s.startsWith('rgb') && n.length >= 3) return [r, g, b, a];
      return undefined;
    };
    const over = (top: C, under: C): C => {
      const a = top[3] + under[3] * (1 - top[3]);
      if (a === 0) return [0, 0, 0, 0];
      const m = (i: 0 | 1 | 2) => (top[i] * top[3] + under[i] * under[3] * (1 - top[3])) / a;
      return [m(0), m(1), m(2), a];
    };
    const lum = (c: C) => {
      const ch = (v: number) => {
        const x = v / 255;
        return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4;
      };
      return 0.2126 * ch(c[0]) + 0.7152 * ch(c[1]) + 0.0722 * ch(c[2]);
    };
    const backdrop = (el: Element | null): C => {
      const layers: C[] = [];
      for (let e = el; e; e = e.parentElement) {
        const c = parse(getComputedStyle(e).backgroundColor);
        if (c && c[3] > 0) layers.push(c);
        if (c && c[3] >= 1) break;
      }
      let acc: C = [255, 255, 255, 1];
      for (const l of layers.reverse()) acc = over(l, acc);
      return acc;
    };
    const out: string[] = [];
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      const el = n.parentElement;
      if (!el || !n.textContent?.trim()) continue;
      const r = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      if (r.width === 0 || r.height === 0 || cs.visibility === 'hidden') continue;
      if (el.closest('[hidden]')) continue;
      const fg = parse(cs.color);
      if (!fg || fg[3] === 0) continue;
      const bg = backdrop(el);
      const shown = over(fg, bg);
      const [hi, lo] = [lum(shown), lum(bg)].sort((a, b) => b - a) as [number, number];
      const ratio = (hi + 0.05) / (lo + 0.05);
      const px = Number.parseFloat(cs.fontSize);
      const large = px >= 24 || (px >= 18.66 && Number(cs.fontWeight) >= 700);
      const disabled = el.closest(':disabled, [aria-disabled="true"]') !== null;
      if (!disabled && ratio < (large ? 3 : 4.5)) {
        out.push(`${n.textContent.trim().slice(0, 40)}: ${ratio.toFixed(2)}`);
      }
    }
    return out;
  });
}

function unlabelled(page: Page) {
  return page.$$eval('[data-tone], [data-race]', (els) =>
    els
      .filter((e) => !(e.querySelector('.mark-label')?.textContent ?? '').trim())
      .map((e) => `${e.tagName} ${e.className}`),
  );
}

const transcript = (page: Page) => page.getByRole('region', { name: 'Transcript' });
const conversation = (page: Page) => page.getByRole('region', { name: 'The conversation' });

/** Opens the past run `PAST`, its journal holding `b`'s events: the transcript, read-only. */
async function openPast(page: Page, b: Bridge) {
  await openChat(page, b, `#/w/${LIVE.id}/r/${PAST}`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/runs/${PAST}`, (r) =>
      r.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          run: PAST,
          state: 'stopped',
          dropped: 0,
          skipped: [],
          events: b.events.map((e) => ({ ...e, params: { ...e.params, run: PAST, body: '' } })),
        }),
      }),
    );
  });
}

for (const theme of ['dark', 'light'] as const) {
  test.describe(`${theme} theme`, () => {
    test.use({ colorScheme: theme });

    test('a session reads as a timeline of steps beside its Live run: screenshot, contrast, labels', async ({
      page,
    }, info) => {
      await page.emulateMedia({ reducedMotion: 'reduce' });
      const b = new Bridge();
      richTurn(b);
      b.emit('turnStarted', { text: 'and the docs?' });
      b.emit('toolCall', {
        id: 't9',
        name: 'bash',
        input: { command: 'pnpm --filter site build' },
      });
      await openChat(page, b);
      const talk = conversation(page);
      await expect(talk.locator('.mapp-ask').first()).toContainText('fix the collate parse');
      await expect(talk).toContainText('edited sqlglot/parser.py');
      await expect(talk.locator('.mapp-landed')).toContainText('Landed:');
      await expect(talk.locator('.mapp-working')).toContainText('working…');
      const run = page.getByRole('complementary', { name: 'Live run' });
      await expect(run.getByRole('log', { name: 'Run log' })).toContainText(
        'edit: sqlglot/parser.py',
      );
      await expect(run.locator('.ss-tiles')).toContainText('$0.040');
      await expect(run).toContainText('Context');
      // A replayed verdict is a step, never a toast.
      await expect(page.locator('.toast')).toHaveCount(0);
      if (info.project.name === 'desktop') {
        await expect(page.locator('.pf-session[data-live="true"]')).toContainText(
          'fix the collate parse',
        );
      }
      await page.screenshot({ path: screen(`${info.project.name}-${theme}-chat`) });
      expect(await lowContrast(page)).toEqual([]);
      expect(await unlabelled(page)).toEqual([]);
      expect(await violations(page)).toEqual([]);
    });
  });
}

test('hostile text from a model or a tool stays text', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(e.message));
  const evil =
    '<img src=x onerror="window.__pwned=1"><script>window.__pwned=2</script> ‮gnp.exe \u001b[31mred\u001b[0m';
  const b = new Bridge();
  b.emit('turnStarted', { text: evil });
  b.emit('assistant', {
    text: `${evil}\n\n[click](javascript:window.__pwned=3) [data](data:text/html,<b>x</b>) ![beacon](https://evil.example/b.png) <a href="javascript:window.__pwned=4">a</a>\n\n\`\`\`html\n<script>window.__pwned=5</script>\n\`\`\``,
    stop: 'end_turn',
  });
  b.emit('toolCall', { id: 't1', name: 'bash', input: { command: '<b onmouseover=x>ls</b>' } });
  b.emit('toolResult', {
    id: 't1',
    content: `line1\nline2\n${evil}`,
    isError: false,
    cut: 0,
  });
  b.emit('note', { text: evil });
  b.emit('turnEnded', { ending: 'answered' });
  await openChat(page, b);
  const talk = conversation(page);
  await expect(talk.locator('.mapp-ask')).toContainText('<img src=x onerror=');
  await expect(talk.locator('.mapp-ask .safe-esc').first()).toHaveText('\\u{202e}');
  await expect(talk.locator('.mapp-answer')).toContainText('<img src=x onerror=');
  await expect(talk.locator('.mapp-answer')).toContainText('(link not followed)');
  await expect(talk.locator('.mapp-answer')).toContainText('image not loaded');
  await expect(talk.locator('.mapp-out')).toContainText('<script>');
  await expect(talk.locator('.mapp-note')).toContainText('<img src=x');
  await expect(talk).toContainText('<b onmouseover=x>ls</b>');
  const main = page.locator('main');
  expect(await main.locator('img, script, iframe, [onerror], [onmouseover]').count()).toBe(0);
  expect(await main.locator('a[href^="javascript"], a[href^="data"]').count()).toBe(0);
  expect(await page.evaluate(() => (window as { __pwned?: number }).__pwned)).toBeUndefined();
  expect(errors).toEqual([]);
  expect(await violations(page)).toEqual([]);
});

test('a dropped socket reconnects and replays, and nothing shows twice', async ({ page }) => {
  const b = new Bridge();
  b.emit('turnStarted', { text: 'first' });
  b.emit('assistant', { text: 'one', stop: 'end_turn' });
  b.emit('turnEnded', { ending: 'answered' });
  await openChat(page, b);
  const asks = page.locator('.ss-msg-user');
  await expect(asks).toHaveCount(1);
  // The socket drops; while it is down the chat goes on.
  for (const s of b.sockets) await s.close({ code: 4000, reason: 'drop' });
  b.sockets = [];
  b.emit('turnStarted', { text: 'second' });
  b.emit('assistant', { text: 'two', stop: 'end_turn' });
  b.emit('turnEnded', { ending: 'answered' });
  await expect(asks).toHaveCount(2);
  const since = b.sent.filter((m) => m.method === 'session/events').map((m) => m.params?.since);
  expect(since.at(-1)).toBe(3);
  // A live event the replay already carried arrives again: it is dropped by its seq.
  b.say(b.events[4]);
  b.emit('turnStarted', { text: 'third' });
  await expect(asks).toHaveCount(3);
  await expect(page.locator('.mapp-answer', { hasText: 'two' })).toHaveCount(1);
  await expect(page.locator('.ss-msg-user', { hasText: 'second' })).toHaveCount(1);
  // A reload replays from the start: still once each.
  await page.reload();
  await expect(asks).toHaveCount(3);
  await expect(page.locator('.mapp-answer', { hasText: 'two' })).toHaveCount(1);
});

test('an observer cannot send; taking over lets it', async ({ page }) => {
  const b = new Bridge();
  b.role = 'observer';
  await openChat(page, b);
  const box = page.getByRole('textbox', { name: 'Ask Mikata' });
  await expect(page.locator('.ss-observer')).toContainText('This tab observes');
  await expect(box).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Send' })).toBeDisabled();
  expect(b.sent.filter((m) => m.method === 'session/prompt')).toEqual([]);
  await page.getByRole('button', { name: 'Take over' }).click();
  await expect(box).toBeEnabled();
  await expect(page.locator('.ss-observer')).toHaveCount(0);
  await box.fill('hello');
  await box.press('Enter');
  await expect(page.locator('.mapp-answer')).toContainText('Answered: hello');
  expect(b.sent.filter((m) => m.method === 'board/takeOver')).toHaveLength(1);
});

test('while a turn runs, Enter queues, Steer and /send steer, Stop and Esc interrupt', async ({
  page,
}) => {
  const b = new Bridge();
  // The core holds a prompt sent mid-turn for after it: nothing answers here.
  b.onPrompt = () => {};
  b.emit('turnStarted', { text: 'long job' });
  b.emit('toolCall', { id: 't1', name: 'bash', input: { command: 'sleep 60' } });
  await openChat(page, b);
  await expect(page.locator('.mapp-working')).toContainText('working…');
  const box = page.getByRole('textbox', { name: 'Ask Mikata' });
  // Enter queues, as the terminal app's does: a separate ask is never folded into this one.
  await box.fill('then write the changelog');
  await box.press('Enter');
  await expect
    .poll(() => b.sent.filter((m) => m.method === 'session/prompt').map((m) => m.params?.text))
    .toEqual(['then write the changelog']);
  // Steering is its own act: the Steer button, or /send.
  await box.fill('look at the lexer too');
  await page.locator('.ss-composer').getByRole('button', { name: 'Steer' }).click();
  await box.fill('/send and the parser');
  await box.press('Enter');
  await expect
    .poll(() => b.sent.filter((m) => m.method === 'session/send').map((m) => m.params?.text))
    .toEqual(['look at the lexer too', 'and the parser']);
  expect(b.sent.filter((m) => m.method === 'session/prompt')).toHaveLength(1);
  // A line the core holds for the next turn is shown as queued.
  b.emit('queued', { text: 'next' });
  await expect(page.getByRole('list', { name: 'Queued' })).toContainText('queued: next');
  await box.press('Escape');
  await expect.poll(() => b.sent.filter((m) => m.method === 'session/interrupt').length).toBe(1);
  await page.locator('.ss-composer').getByRole('button', { name: 'Stop' }).click();
  await expect.poll(() => b.sent.filter((m) => m.method === 'session/interrupt').length).toBe(2);
  b.emit('turnEnded', { ending: 'interrupted' });
  await expect(page.locator('.mapp-note')).toContainText('interrupted');
  await expect(page.locator('.mapp-working')).toHaveCount(0);
  b.emit('gateVerdict', {
    passed: false,
    verdict: 'FAIL',
    changed: ['a.rs'],
    failures: ['cargo test'],
    unverified: [],
  });
  await expect(page.locator('.mapp-landed[data-ok="false"]')).toContainText(
    'Not landed: cargo test',
  );
  await expect(page.locator('.ss-head .ss-badge')).toHaveText('NOT LANDED');
});

test('a past run reads from its journal, read-only, and resumes', async ({ page }) => {
  const b = new Bridge();
  const transcript = {
    run: PAST,
    state: 'stopped',
    dropped: 0,
    skipped: [],
    events: [
      {
        jsonrpc: '2.0',
        method: 'event/turnStarted',
        params: { session: '', run: PAST, body: '', seq: 1, text: 'wire refactor' },
      },
      {
        jsonrpc: '2.0',
        method: 'event/assistant',
        params: { session: '', run: PAST, body: '', seq: 2, text: 'Half done.', stop: 'end_turn' },
      },
      {
        jsonrpc: '2.0',
        method: 'event/turnEnded',
        params: { session: '', run: PAST, body: '', seq: 3, ending: 'interrupted' },
      },
    ],
  };
  let resumed: unknown;
  await openChat(page, b, `#/w/${LIVE.id}/r/${PAST}`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/runs/${PAST}`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(transcript) }),
    );
    await p.route(`**/api/w/${LIVE.id}/sessions`, async (r) => {
      resumed = r.request().postDataJSON();
      expect(r.request().headers()['x-mikata-csrf']).toBe('a'.repeat(64));
      await r.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          session: {
            id: CHAT,
            world: LIVE.id,
            session: 's-1',
            run: PAST,
            controlled: true,
            tabs: 0,
            asks: 0,
          },
          hello: { ...hello, run: PAST },
        }),
      });
    });
  });
  await expect(page.locator('.msg-user')).toContainText('wire refactor');
  await expect(page.locator('.line-end')).toContainText('interrupted');
  await expect(page.locator('.composer-closed')).toContainText('read-only');
  await expect(page.getByRole('textbox')).toHaveCount(0);
  await page.getByRole('button', { name: 'Resume' }).click();
  await expect(page).toHaveURL((u) => u.hash === `#/w/${LIVE.id}/s/${CHAT}`);
  expect(resumed).toEqual({ resume: PAST });
  await expect(page.getByRole('textbox', { name: 'Ask Mikata' })).toBeEnabled();
});

test('a scroll up is kept when a redraw comes before its scroll event', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'one measure is enough');
  const b = new Bridge();
  for (let i = 0; i < 200; i++) {
    b.emit('turnStarted', { text: `question ${i}` });
    b.emit('assistant', { text: `Answer ${i}: a few words of prose`, stop: 'end_turn' });
    b.emit('turnEnded', { ending: 'answered' });
  }
  await openPast(page, b);
  const list = transcript(page);
  await expect(list).toHaveAttribute('data-total', '400');
  await expect(page.locator('.msg-body').last()).toContainText('Answer 199');
  // The list follows its end from a flag only a scroll event refreshes. A redraw that runs after
  // the reader has scrolled and before that scroll's event must not take the flag's word: here
  // the order is fixed by the frames themselves, where a slow machine only makes it likely.
  const left = await list.evaluate(async (el) => {
    const frame = () => new Promise((r) => requestAnimationFrame(() => r(null)));
    const kept: number[] = [];
    // Several rounds: the order below is likely, not certain, on any one of them.
    for (let round = 0; round < 8; round++) {
      el.scrollTop = el.scrollHeight;
      for (let i = 0; i < 3; i++) await frame();
      // One pixel, still at the end: its scroll event comes with the next frame and asks for a
      // redraw, which runs after this script's own turn in that frame.
      el.scrollTop = el.scrollHeight - el.clientHeight - 1;
      await frame();
      // The reader scrolls up, in the turn before that redraw.
      el.scrollTop -= 3000;
      for (let i = 0; i < 4; i++) await frame();
      kept.push(el.scrollHeight - el.scrollTop - el.clientHeight);
    }
    return kept;
  });
  expect(
    Math.min(...left),
    `how far from the end each round was left: ${left.join(', ')}`,
  ).toBeGreaterThan(2000);
  await expect(page.getByRole('button', { name: '↓ Latest' })).toBeVisible();
});

test('2,000 messages scroll in a window, smoothly', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'one measure is enough');
  const b = new Bridge();
  for (let i = 0; i < 1000; i++) {
    b.emit('turnStarted', { text: `question ${i}` });
    b.emit('assistant', {
      text: `Answer ${i}: ${'a few words of prose to wrap '.repeat((i % 5) + 1)}`,
      stop: 'end_turn',
    });
    b.emit('turnEnded', { ending: 'answered' });
  }
  await openPast(page, b);
  const list = transcript(page);
  await expect(list).toHaveAttribute('data-total', '2000');
  await expect(page.locator('.msg-body').last()).toContainText('Answer 999');
  // Only a window of rows is in the DOM.
  const rows = Number(await list.getAttribute('data-rows'));
  expect(rows).toBeGreaterThan(0);
  expect(rows).toBeLessThan(120);
  // Scrolled from the end to the start in steps, frame by frame: no frame stalls.
  const frames = await list.evaluate(async (el) => {
    const times: number[] = [];
    let last = performance.now();
    const steps = 60;
    const from = el.scrollTop;
    for (let i = 1; i <= steps; i++) {
      el.scrollTop = from * (1 - i / steps);
      await new Promise((r) => requestAnimationFrame(() => r(null)));
      const now = performance.now();
      times.push(now - last);
      last = now;
    }
    // Let measurement settle at the top.
    for (let i = 0; i < 10; i++) {
      el.scrollTop = 0;
      await new Promise((r) => requestAnimationFrame(() => r(null)));
    }
    return times.sort((a, b) => a - b);
  });
  const p90 = frames[Math.floor(frames.length * 0.9)] ?? Infinity;
  expect(p90, `frame times ${frames.map((t) => t.toFixed(0)).join(',')}`).toBeLessThan(50);
  expect(Math.max(...frames)).toBeLessThan(250);
  await expect(page.locator('.msg-user', { hasText: /^youquestion 0$/ })).toBeVisible();
  expect(Number(await list.getAttribute('data-rows'))).toBeLessThan(120);
});

test('a long live session draws its newest steps, scrolls smoothly and keeps up', async ({
  page,
}, info) => {
  test.skip(info.project.name !== 'desktop', 'one measure is enough');
  const b = new Bridge();
  for (let i = 0; i < 1000; i++) {
    b.emit('turnStarted', { text: `question ${i}` });
    b.emit('toolCall', { id: `c${i}`, name: 'bash', input: { command: `make step-${i}` } });
    b.emit('toolResult', { id: `c${i}`, content: `ok ${i}`, isError: false, cut: 0 });
    b.emit('assistant', {
      text: `Answer ${i}: ${'a few words '.repeat((i % 5) + 1)}`,
      stop: 'end_turn',
    });
    b.emit('turnEnded', { ending: 'answered' });
  }
  await openChat(page, b);
  // By its class, not its role: a role locator reads the accessible text of the whole region on
  // every poll, which on a session this long is the test's cost, not the page's.
  const convo = page.locator('.ss-convo');
  await expect(convo).toContainText('Answer 999');
  // Only the newest window is in the page; the rest waits behind a button.
  const earlier = convo.locator('.ss-earlier');
  await expect(earlier).toBeVisible();
  const drawn = await convo.locator('.mapp-step').count();
  expect(drawn).toBeLessThanOrEqual(200);
  expect(drawn).toBeGreaterThan(50);
  // Scrolled from the end to the start in steps, frame by frame: no frame stalls.
  const frames = await convo.evaluate(async (el) => {
    const times: number[] = [];
    let last = performance.now();
    const from = el.scrollTop;
    for (let i = 1; i <= 60; i++) {
      el.scrollTop = from * (1 - i / 60);
      await new Promise((r) => requestAnimationFrame(() => r(null)));
      const now = performance.now();
      times.push(now - last);
      last = now;
    }
    return times.sort((a, z) => a - z);
  });
  const p90 = frames[Math.floor(frames.length * 0.9)] ?? Infinity;
  expect(p90, `frame times ${frames.map((t) => t.toFixed(0)).join(',')}`).toBeLessThan(50);
  // More, on demand.
  await earlier.click();
  expect(await convo.locator('.mapp-step').count()).toBeGreaterThan(drawn);
  // A new event still lands at once, the session this long: timed in the page, from before the
  // events leave to the frame that shows them (a locator's polling reads the whole region, which
  // is the test's cost, not the page's).
  await convo.evaluate((el) => {
    const w = window as { __landed?: Promise<number> };
    const t0 = performance.now();
    w.__landed = new Promise((done) => {
      const seen = () => (el.textContent ?? '').includes('Answer at the end');
      const watch = new MutationObserver(() => {
        if (seen()) {
          watch.disconnect();
          done(performance.now() - t0);
        }
      });
      watch.observe(el, { childList: true, subtree: true, characterData: true });
    });
  });
  b.emit('turnStarted', { text: 'one more' });
  b.emit('assistant', { text: 'Answer at the end', stop: 'end_turn' });
  const took = await page.evaluate(() => (window as { __landed?: Promise<number> }).__landed);
  expect(took).toBeLessThan(500);
});

/** A turn that landed, followed by what changed: `last` is its text's last line. */
function landedWithChanges(b: Bridge, last: string) {
  b.emit('turnStarted', { text: 'keep the one-string form of a fractional interval' });
  b.emit('assistant', { text: 'Fixed.', stop: 'end_turn' });
  b.emit('gateVerdict', {
    passed: true,
    verdict: 'PASS',
    changed: ['sqlglot/generators/duckdb.py'],
    failures: [],
    unverified: [],
  });
  b.emit('changes', {
    text: [
      "what changed: of Mikata's last 12 checks, 5 print differently than on the code you gave it. The last of them:",
      '',
      '  Fractional two-string:',
      "    Input:  SELECT INTERVAL '1.5' DAY",
      "-   Output: SELECT INTERVAL '1.5' DAY",
      "+   Output: SELECT INTERVAL '1.5 DAY'",
      '',
      last,
    ].join('\n'),
  });
  b.emit('turnEnded', { ending: 'answered' });
}

for (const theme of ['dark', 'light'] as const) {
  test(`what changed is drawn under the gate, a check's lines kept as printed (${theme})`, async ({
    page,
  }, info) => {
    await page.emulateMedia({ colorScheme: theme });
    const b = new Bridge();
    landedWithChanges(b, '4 more checks: .mikata/tmp/what-changed.md');
    await openPast(page, b);
    const block = page.locator('.line-changes');
    await expect(block).toContainText("of Mikata's last 12 checks, 5 print differently");
    await expect(block.locator('.changes-gone')).toHaveText(
      "-   Output: SELECT INTERVAL '1.5' DAY",
    );
    await expect(block.locator('.changes-now')).toHaveText("+   Output: SELECT INTERVAL '1.5 DAY'");
    await expect(block).toContainText('4 more checks: .mikata/tmp/what-changed.md');
    await page.screenshot({ path: screen(`${info.project.name}-${theme}-changes`) });
    expect(await lowContrast(page)).toEqual([]);
    expect(await violations(page)).toEqual([]);
  });
}

test("a check's output is the code's own words: text, never markup", async ({ page }) => {
  const b = new Bridge();
  landedWithChanges(b, '+ <img src=x onerror=alert(1)>');
  await openPast(page, b);
  const block = page.locator('.line-changes');
  await expect(block).toContainText('<img src=x onerror=alert(1)>');
  await expect(block.locator('img')).toHaveCount(0);
  // The session's timeline draws the same lines, as text.
  await page.goto(`/#/w/${LIVE.id}/s/${CHAT}`);
  const step = conversation(page).locator('.mapp-step', { hasText: "of Mikata's last 12 checks" });
  await expect(step).toContainText("+   Output: SELECT INTERVAL '1.5 DAY'");
  await expect(step).toContainText('<img src=x onerror=alert(1)>');
  await expect(step.locator('img')).toHaveCount(0);
});
