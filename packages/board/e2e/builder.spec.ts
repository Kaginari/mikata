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

/** The sessions endpoint, answering a new session and recording what it was asked. */
async function sessions(p: import('@playwright/test').Page, started: unknown[]) {
  await p.route(`**/api/w/${LIVE.id}/sessions`, async (r) => {
    if (r.request().method() !== 'POST') return r.fallback();
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
}

test('the builder composes the harness new sessions open with, and keeps it', async ({
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
    await sessions(p, started);
  });
  // The builder as the picture draws it: the graph, its legend, the TOML; no run of its own.
  await expect(page.getByRole('heading', { name: 'Harness Builder', level: 1 })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Legend' })).toContainText('External plugin');
  await expect(page.getByRole('button', { name: /Start run/ })).toHaveCount(0);
  await expect(page.getByRole('switch', { name: 'Memory: on' })).toBeVisible();
  // A switch flipped: the composition says it, and the rail's status counts it.
  await page.getByRole('switch', { name: 'Challenge: on' }).click();
  await expect(page.getByRole('switch', { name: 'Challenge: off' })).toBeVisible();
  await expect(page.locator('.bapp-units')).toContainText('-challenge');
  await page.screenshot({ path: shot(`builder-${info.project.name}`), fullPage: true });
  // A new session opens with it, and runs on the session's own page.
  // A new session is started in the Harness Builder, where its harness is chosen.
  await page.locator('.pf-top').getByRole('link', { name: 'New session' }).click();
  await page.getByRole('button', { name: 'Start session' }).click();
  await expect(page).toHaveURL((u) => u.hash === `#/w/${LIVE.id}/s/${CHAT}`);
  expect(started).toEqual([{ units: '-challenge' }]);
  const box = page.getByRole('textbox', { name: 'Ask Mikata' });
  await box.fill('Support 1h30m duration parsing. Keep numeric parsing unchanged.');
  await box.press('Enter');
  await expect(page.locator('.ss-head .ss-badge')).toHaveText('VERIFIED');
  await expect(page.getByRole('log', { name: 'Run log' })).toContainText('end gate: pass');
  // The composition is kept for the workspace: a reload finds the switch as it was left.
  await page.goto(`/#/w/${LIVE.id}/builder`);
  await expect(page.getByRole('switch', { name: 'Challenge: off' })).toBeVisible();
});

test('a profile starts the switches from its own composition, and New session takes it', async ({
  page,
}, info) => {
  const bridge = new Bridge();
  const started: unknown[] = [];
  await openChat(page, bridge, `#/w/${LIVE.id}/builder`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(units) }),
    );
    await sessions(p, started);
  });
  await expect(page.getByRole('heading', { name: 'Harness Builder', level: 1 })).toBeVisible();
  await page.getByLabel('Profile').selectOption('small-fix');
  await expect(page.getByRole('switch', { name: 'Plan: off' })).toBeVisible();
  // Nothing flipped over the profile: it goes as it is.
  await expect(page.locator('.bapp-units')).toContainText('as the profile is');
  if (info.project.name === 'desktop') {
    await expect(page.getByRole('list', { name: 'Runtime status' })).toContainText('small-fix');
  }
  // A flip over it, then back to the profile.
  await page.getByRole('switch', { name: 'End gate: on' }).click();
  await expect(page.locator('.bapp-units')).toContainText('-gate');
  await page.getByRole('button', { name: 'Reset to the profile' }).click();
  await expect(page.getByRole('switch', { name: 'End gate: on' })).toBeVisible();
  // A new session is started in the Harness Builder, where its harness is chosen.
  await page.locator('.pf-top').getByRole('link', { name: 'New session' }).click();
  await page.getByRole('button', { name: 'Start session' }).click();
  await expect.poll(() => started).toEqual([{ profile: 'small-fix' }]);
});

test('a guard switched off holds for the next session only, said on that session', async ({
  page,
}) => {
  const bridge = new Bridge();
  const started: unknown[] = [];
  await openChat(page, bridge, `#/w/${LIVE.id}/builder`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(units) }),
    );
    await sessions(p, started);
  });
  await page.getByRole('switch', { name: 'End gate: on' }).click();
  await expect(page.getByRole('status')).toContainText('next new session only');
  // The session, opened with -gate, says so in its hello, as the core does.
  bridge.hello = { ...hello, guardsOff: ['gate'] };
  // A new session is started in the Harness Builder, where its harness is chosen.
  await page.locator('.pf-top').getByRole('link', { name: 'New session' }).click();
  await page.getByRole('button', { name: 'Start session' }).click();
  await expect.poll(() => started).toEqual([{ units: '-gate' }]);
  // That session says so, in the alarm's tone.
  const chip = page.getByRole('list', { name: 'This session' }).locator('li[data-status="alarm"]');
  await expect(chip).toContainText('guards off: gate');
  // And it is the session that says it: another tab, which started nothing, says it too.
  const other = await page.context().newPage();
  await openChat(other, bridge, `#/w/${LIVE.id}/s/${CHAT}`, async (p) => {
    await p.route(`**/api/w/${LIVE.id}/units`, (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify(units) }),
    );
  });
  await expect(
    other.getByRole('list', { name: 'This session' }).locator('li[data-status="alarm"]'),
  ).toContainText('guards off: gate');
  await other.close();
  // Back on the builder the guard is on again, and the next session opens with it.
  await page.goto(`/#/w/${LIVE.id}/builder`);
  await expect(page.getByRole('switch', { name: 'End gate: on' })).toBeVisible();
  // A new session is started in the Harness Builder, where its harness is chosen.
  await page.locator('.pf-top').getByRole('link', { name: 'New session' }).click();
  await page.getByRole('button', { name: 'Start session' }).click();
  await expect.poll(() => started).toEqual([{ units: '-gate' }, {}]);
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
    await expect(page.locator('.bapp-graph')).toBeVisible();
    await page.mouse.move(0, 999);
    await page.screenshot({ path: docs(`builder-${theme}`) });
    // A run, from a new session's first ask.
    // A new session is started in the Harness Builder, where its harness is chosen.
    await page.locator('.pf-top').getByRole('link', { name: 'New session' }).click();
    await page.getByRole('button', { name: 'Start session' }).click();
    const box = page.getByRole('textbox', { name: 'Ask Mikata' });
    await box.fill('Support 1h30m duration parsing. Keep numeric parsing unchanged.');
    await box.press('Enter');
    await expect(page.locator('.ss-head .ss-badge')).toHaveText('VERIFIED');
    // Each page at its own address; the session last, where the run's card opens.
    for (const [name, at] of [
      ['plugins', `#/w/${LIVE.id}/plugins`],
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
