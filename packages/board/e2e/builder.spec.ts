/**
 * The Harness Builder: the world's units as cards, a switch flipped, an ask started with exactly
 * that composition, and the run followed live to the end gate's word.
 */
import { fileURLToPath } from 'node:url';
import { expect, test } from '@playwright/test';
import { askLine, Bridge, CHAT, hello, LIVE, openChat, RUN } from './fake.ts';

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

/** The README's composition: the native units and two plugins, one connected, one off. */
const shown = {
  ...units,
  units: [
    ...units.units,
    ...['license-guard', 'pr-notes'].map((id, i) => ({
      id: `plugin:${id}`,
      kind: 'plugin',
      version: '',
      on: i === 0,
      frame: false,
      tightens: false,
    })),
  ],
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
  if (info.project.name === 'desktop') await page.setViewportSize({ width: 1600, height: 1000 });
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
  // It opens on the builder itself.
  await expect(page.getByRole('heading', { name: 'Harness Builder', level: 1 })).toBeVisible();
  await expect(page.getByRole('switch', { name: 'Memory: on' })).toBeVisible();
  // A switch flipped: the composition says it.
  await page.getByRole('switch', { name: 'Challenge: on' }).click();
  await expect(page.getByRole('switch', { name: 'Challenge: off' })).toBeVisible();
  await expect(page.locator('.bapp-units')).toContainText('-challenge');
  await page
    .getByLabel('Your ask')
    .fill('Support 1h30m duration parsing. Keep numeric parsing unchanged.');
  await page.getByRole('button', { name: /Start run/ }).click();
  await expect(page.getByRole('heading', { name: 'Verified', exact: true })).toBeVisible();
  expect(started).toEqual([{ units: '-challenge' }]);
  const log = page.getByRole('log', { name: 'Run log' });
  await expect(log).toContainText('bash: cargo test');
  await expect(log).toContainText('end gate: pass');
  await expect(page.locator('.bapp-steps li[data-state="done"]')).toHaveCount(4);
  await expect(page.locator('.bapp-state')).toHaveText('VERIFIED');
  await page.screenshot({ path: shot(`builder-run-${info.project.name}`), fullPage: true });
  // The session's own page shows the run's conversation, and a composer to go on.
  await page.goto(`/#/w/${LIVE.id}/s/${CHAT}`);
  await expect(page.getByRole('region', { name: 'The conversation' })).toContainText(
    'Parsed 1h30m as 90 minutes',
  );
  await expect(page.getByRole('textbox', { name: 'Ask Mikata' })).toBeEnabled();
  await page.screenshot({ path: shot(`builder-${info.project.name}`), fullPage: true });
});

test('a profile starts the switches from its own composition', async ({ page }) => {
  const bridge = new Bridge();
  await openChat(page, bridge, `#/w/${LIVE.id}/builder`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(units) }),
    );
  });
  await expect(page.getByRole('heading', { name: 'Harness Builder', level: 1 })).toBeVisible();
  await page.getByLabel('Profile').selectOption('small-fix');
  await expect(page.getByRole('switch', { name: 'Plan: off' })).toBeVisible();
  // Nothing flipped over the profile: it goes as it is.
  await expect(page.locator('.bapp-units')).toContainText('as the profile is');
  // The profiles page lists them, each to apply.
  await page.getByRole('link', { name: 'Profiles', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Profiles', level: 1 })).toBeVisible();
});

/** Where the README's screenshots of the board go. */
const docs = (file: string) =>
  fileURLToPath(new URL(`../test-results/readme/${file}.png`, import.meta.url));

// The board's pages over a fake bridge, photographed on demand (README_SHOTS=1) into
// test-results/readme/. The README's and the book's pictures are the real board's, taken on a
// running `mikatad board` with a real world and a real run, never these.
for (const theme of ['dark', 'light'] as const) {
  test(`the README's pictures of the board, ${theme}`, async ({ page }, info) => {
    test.skip(
      !process.env.README_SHOTS || info.project.name !== 'desktop',
      'README_SHOTS=1, desktop',
    );
    // Settled pictures: no transition caught half-way.
    await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
    await page.setViewportSize({ width: 1600, height: 1000 });
    const bridge = new Bridge();
    bridge.onPrompt = (text) => run(bridge, text);
    await openChat(page, bridge, `#/w/${LIVE.id}/builder`, async (p) => {
      await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
        r.fulfill({ contentType: 'application/json', body: JSON.stringify(shown) }),
      );
      await p.route(`**/api/w/${LIVE.id}/sessions`, (r) =>
        r.fulfill({
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
        }),
      );
    });
    await page
      .getByLabel('Your ask')
      .fill('Support 1h30m duration parsing. Keep numeric parsing unchanged.');
    await page.getByRole('button', { name: /Start run/ }).click();
    await expect(page.getByRole('heading', { name: 'Verified', exact: true })).toBeVisible();
    await page.mouse.move(0, 999);
    await page.screenshot({ path: docs(`builder-${theme}`) });
    // Each page at its own address; the session last, where the run's card opens.
    for (const [name, at] of [
      ['plugins', `#/w/${LIVE.id}/plugins`],
      ['profiles', `#/w/${LIVE.id}/profiles`],
      ['runs', `#/w/${LIVE.id}/runs`],
      ['sessions', `#/w/${LIVE.id}/s/${CHAT}`],
    ] as const) {
      await page.goto(`/${at}`);
      await expect(page.locator('.pf-page')).toBeVisible();
      await page.mouse.move(0, 999);
      await page.screenshot({ path: docs(`${name}-${theme}`) });
    }
    // An act the run asks the human for: the card over the page.
    bridge.say(
      askLine('gate-1', {
        body: 'mikata',
        race: 'summoner',
        class: 'outward',
        act: 'git push origin fix/duration-parsing',
        why: ['pushes a branch to a remote'],
        outside: [],
        plan: 'plan 3f9a1c2e4b5d · head 1a2b3c4d',
      }),
    );
    await expect(page.getByRole('alertdialog')).toBeVisible();
    // Armed: the answers take keys now, as the human would see it a moment after it opens.
    await expect(page.getByRole('alertdialog')).not.toContainText('Arming');
    await page.screenshot({ path: docs(`approval-${theme}`) });
  });
}
