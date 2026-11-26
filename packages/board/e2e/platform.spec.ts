/**
 * The platform: one frame for every page — the top bar's tabs, the rail's workspace, pages,
 * sessions and runtime, and the page itself. The session page with its Live run, the Harness
 * Builder and the runs inside the frame; screenshots at phone and desktop in both themes.
 */
import { fileURLToPath } from 'node:url';
import { expect, test } from '@playwright/test';
import { Bridge, CHAT, LIVE, openChat, richTurn, violations } from './fake.ts';

const shot = (file: string) =>
  fileURLToPath(new URL(`../test-results/screens/${file}.png`, import.meta.url));

const units = {
  units: ['memory', 'what_changed', 'reader', 'stress', 'plan', 'gate', 'hold', 'secrets'].map(
    (id) => ({
      id,
      kind: 'native',
      version: '1',
      on: id !== 'hold',
      frame: false,
      tightens: false,
    }),
  ),
  profiles: [{ name: 'small-fix', units: '-plan', from: 'build' }],
  holes: [],
};

const desk = {
  verify: ['cargo test'],
  done: ['parse("1h30m") == 5400', 'numeric parsing unchanged'],
  branch: 'main',
};

const routes = async (page: import('@playwright/test').Page) => {
  const json = (body: unknown) => ({ contentType: 'application/json', body: JSON.stringify(body) });
  await page.route(`**/api/w/${LIVE.id}/units`, (r) => r.fulfill(json(units)));
  await page.route(`**/api/w/${LIVE.id}/desk`, (r) => r.fulfill(json(desk)));
};

for (const theme of ['dark', 'light'] as const) {
  test(`a session in the platform, mid-run (${theme})`, async ({ page }, info) => {
    const b = new Bridge();
    await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
    await openChat(page, b, `#/w/${LIVE.id}/s/${CHAT}`, routes);
    await expect(page.locator('.pf-top')).toBeVisible();
    richTurn(b);
    await expect(page.getByRole('region', { name: 'The conversation' })).toContainText('collate');
    await expect(page.getByRole('complementary', { name: 'Live run' })).toBeVisible();
    await expect(page.locator('a.pf-tab[aria-current="page"]')).toContainText('Sessions');
    await page.screenshot({ path: shot(`platform-session-${theme}-${info.project.name}`) });
    expect(await violations(page)).toEqual([]);
  });

  test(`the builder, the plugins and the runs in the frame (${theme})`, async ({ page }, info) => {
    const b = new Bridge();
    await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
    await openChat(page, b, `#/w/${LIVE.id}/builder`, routes);
    await expect(page.locator('.bapp-graph')).toBeVisible();
    await expect(page.locator('.bapp-head')).toHaveCount(0);
    await page.screenshot({ path: shot(`platform-builder-${theme}-${info.project.name}`) });
    // The top bar's tabs on a desktop; at phone width the rail, a drawer the menu opens.
    const go = async (name: string) => {
      if (info.project.name === 'desktop') {
        await page.locator('a.pf-tab', { hasText: name }).click();
        return;
      }
      await page.getByRole('button', { name: 'Menu' }).click();
      await page
        .getByRole('navigation', { name: 'Pages', exact: true })
        .getByRole('link', { name, exact: true })
        .click();
    };
    await go('Plugins');
    await expect(page.getByRole('heading', { name: 'Plugin inspector' })).toBeVisible();
    await go('Runs');
    await expect(page.getByRole('heading', { name: 'Agent runs' })).toBeVisible();
    await expect(page.locator('.pg-run')).toHaveCount(3);
    await page.screenshot({ path: shot(`platform-runs-${theme}-${info.project.name}`) });
    expect(await violations(page)).toEqual([]);
  });
}

test('a workspace with no live session opens on an empty session, the composer ready', async ({
  page,
}, info) => {
  const b = new Bridge();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await openChat(page, b, `#/w/${LIVE.id}`, async (p) => {
    await routes(p);
    await p.route('**/api/sessions', (r) =>
      r.fulfill({ contentType: 'application/json', body: JSON.stringify({ sessions: [] }) }),
    );
  });
  await expect(page.getByRole('textbox', { name: 'Ask Mikata' })).toBeEnabled();
  await expect(page.locator('.ss-badge').first()).toHaveText('READY');
  await page.screenshot({ path: shot(`platform-empty-${info.project.name}`) });
});

test('the front page opens the workspace worked in last', async ({ page }) => {
  const b = new Bridge();
  await openChat(page, b, '#/', routes);
  await expect(page).toHaveURL((u) => u.hash === `#/w/${LIVE.id}`);
});

test('at phone width the rail is a drawer the menu opens', async ({ page }, info) => {
  test.skip(info.project.name !== 'phone', 'the drawer is the phone layout');
  const b = new Bridge();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await openChat(page, b, `#/w/${LIVE.id}/runs`, routes);
  const rail = page.locator('#pf-rail');
  await expect(rail).toBeHidden();
  await page.getByRole('button', { name: 'Menu' }).click();
  await expect(rail).toBeVisible();
  await page.screenshot({ path: shot('platform-drawer-phone') });
  await rail.getByRole('link', { name: 'Memory' }).click();
  await expect(page).toHaveURL(/\/memory$/);
  await expect(rail).toBeHidden();
});
