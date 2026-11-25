/**
 * The Harness Builder: the world's units as cards, a switch flipped, an ask started with exactly
 * that composition, and the run followed live to the end gate's word.
 */
import { fileURLToPath } from 'node:url';
import { expect, test } from '@playwright/test';
import { Bridge, CHAT, hello, LIVE, openChat, RUN } from './fake.ts';

const shot = (file: string) =>
  fileURLToPath(new URL(`../test-results/screens/${file}.png`, import.meta.url));

const units = {
  units: [
    ['returns', '1'],
    ['memory', '1'],
    ['what_changed', '1'],
    ['reader', '1'],
    ['stress', '1'],
    ['plan', '1'],
    ['gate', '1'],
    ['hold', '1'],
    ['fold', '1'],
    ['challenge', '1'],
  ].map(([id, version]) => ({
    id,
    kind: 'native',
    version,
    on: true,
    frame: id === 'plan',
    tightens: id === 'gate' || id === 'hold',
  })),
  profiles: [{ name: 'small-fix', units: '-plan', from: 'build' }],
  holes: [],
};

/** A run in the TUI's grammar: the plan written, an edit, the verify line, the gate, the answer. */
function run(b: Bridge, text: string) {
  b.emit('turnStarted', { text });
  b.emit('toolCall', {
    id: 'p',
    name: 'write',
    input: { path: '.mikata/tmp/plan.md', content: '## Done when\n- D1 parse("1h30m") == 5400\n' },
  });
  b.emit('toolResult', { id: 'p', content: 'written', isError: false, cut: 0 });
  b.emit('toolCall', {
    id: 'e',
    name: 'edit',
    input: { path: 'src/time.rs', old_string: 'a', new_string: 'b' },
  });
  b.emit('toolResult', { id: 'e', content: 'edited', isError: false, cut: 0 });
  b.emit('toolCall', { id: 'v', name: 'bash', input: { command: 'cargo test' } });
  b.emit('toolResult', {
    id: 'v',
    content: 'running 42 tests\ntest result: ok. 42 passed; 0 failed',
    isError: false,
    cut: 0,
  });
  b.emit('usage', {
    input: 14_000,
    output: 900,
    cacheRead: 3_000,
    cacheWrite: 0,
    usd: 0.038,
    wt: 18_000,
    spentWt: 18_000,
    occupancy: 18_400,
    stressLine: 128_000,
  });
  b.emit('assistant', { text: 'Parsed 1h30m as 90 minutes; "90" is unchanged.', stop: 'end_turn' });
  b.emit('gateVerdict', {
    passed: true,
    verdict: 'PASS',
    changed: ['src/time.rs'],
    failures: [],
    unverified: [],
    unchecked: false,
    verifiedBy: ['cargo test'],
  });
  b.emit('turnEnded', { ending: 'answered' });
}

test('the builder starts a run with the composition its switches make, and follows it', async ({
  page,
}, info) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  if (info.project.name === 'desktop') await page.setViewportSize({ width: 1720, height: 1080 });
  const bridge = new Bridge();
  bridge.onPrompt = (text) => run(bridge, text);
  const started: unknown[] = [];
  await openChat(page, bridge, `#/w/${LIVE.id}/builder`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(units) }),
    );
    await p.route(`**/api/w/${LIVE.id}/sessions`, async (r) => {
      started.push(r.request().postDataJSON());
      await r.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          session: {
            id: CHAT,
            world: LIVE.id,
            session: 's-1',
            run: RUN,
            controlled: true,
            tabs: 1,
            asks: 0,
          },
          hello,
        }),
      });
    });
  });
  await expect(page.getByRole('heading', { name: 'Harness Builder' })).toBeVisible();
  if (info.project.name === 'desktop') {
    await page.getByRole('button', { name: 'Hide the side pane' }).click();
  }
  await expect(page.getByRole('switch', { name: 'Memory: on' })).toBeVisible();
  // A switch flipped: the composition says it.
  await page.getByRole('switch', { name: 'Challenge: on' }).click();
  await expect(page.getByRole('switch', { name: 'Challenge: off' })).toBeVisible();
  await expect(page.locator('.config-note')).toContainText('-challenge');
  await page.getByLabel('Ask').fill('fix the time parser to support 1h30m');
  await page.getByRole('button', { name: /Start run/ }).click();
  await expect(page.getByRole('heading', { name: 'Verified' })).toBeVisible();
  expect(started).toEqual([{ units: '-challenge' }]);
  const log = page.getByRole('log', { name: 'Timeline' });
  await expect(log).toContainText('bash: cargo test');
  await expect(log).toContainText('end gate: pass');
  await expect(page.locator('.stage[data-state="done"]')).toHaveCount(4);
  await page.screenshot({ path: shot(`builder-${info.project.name}`), fullPage: true });
});

test('a profile starts the switches from its own composition', async ({ page }) => {
  const bridge = new Bridge();
  await openChat(page, bridge, `#/w/${LIVE.id}/builder`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(units) }),
    );
  });
  await page.getByLabel('Profile').selectOption('small-fix');
  await expect(page.getByRole('switch', { name: 'Plan: off' })).toBeVisible();
  // Nothing flipped over the profile: it goes as it is.
  await expect(page.locator('.config-note')).toContainText('the profile as it is');
});
