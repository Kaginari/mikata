/**
 * The run journal end to end, at phone and desktop in both themes: a screenshot, contrast and
 * labels, its search and contents, hostile text staying text.
 */
import { fileURLToPath } from 'node:url';
import { expect, type Page, test } from '@playwright/test';
import { journalOf } from '../src/data/bodiesFixtures.ts';
import { FIXTURE_NOW, fixtureOverviews, fixtureWorlds } from '../src/data/fixtures.ts';
import { lowContrast, unlabelled, violations } from './checks.ts';

const screen = (file: string) =>
  fileURLToPath(new URL(`../test-results/screens/${file}.png`, import.meta.url));

const W = fixtureWorlds.worlds.find((w) => w.name === 'sqlglot');
if (!W) throw new Error('the fixtures hold sqlglot');

async function open(page: Page, hash: string) {
  await page.addInitScript(() => {
    const seen: string[] = [];
    (window as { __csp?: string[] }).__csp = seen;
    document.addEventListener('securitypolicyviolation', (e) =>
      seen.push(`${e.violatedDirective} ${e.blockedURI}`),
    );
  });
  await page.clock.setFixedTime(FIXTURE_NOW);
  const json = (body: unknown) => ({ contentType: 'application/json', body: JSON.stringify(body) });
  await page.route('**/api/worlds', (r) => r.fulfill(json(fixtureWorlds)));
  await page.route('**/api/session', (r) => r.fulfill(json({ csrf: 'ab'.repeat(32) })));
  await page.route('**/api/w/**', (r) => {
    const url = new URL(r.request().url());
    const [, , , id, part] = url.pathname.split('/');
    const q = (k: string) => url.searchParams.get(k);
    if (part === 'journal') return r.fulfill(json(journalOf(q('q'))));
    const o = fixtureOverviews[decodeURIComponent(id ?? '')];
    return o ? r.fulfill(json(o)) : r.fulfill({ status: 404, body: 'no listed world has this id' });
  });
  await page.routeWebSocket('**/ws/board', () => {});
  await page.goto(`/${hash}`);
  await expect(page.locator('.shell')).toBeVisible();
}

const base = `#/w/${W.id}`;

async function checks(page: Page) {
  expect(await lowContrast(page)).toEqual([]);
  expect(await unlabelled(page)).toEqual([]);
  expect(await violations(page)).toEqual([]);
}

for (const theme of ['dark', 'light'] as const) {
  test.describe(`${theme} theme`, () => {
    test.use({ colorScheme: theme });

    test('journal: screenshot, contrast, labels', async ({ page }, info) => {
      await open(page, `${base}/journal`);
      await expect(page.getByText('the dialect landing').first()).toBeVisible();
      await page.screenshot({
        path: screen(`${info.project.name}-${theme}-journal`),
        fullPage: true,
      });
      await checks(page);
    });
  });
}

test('the journal searches, and its contents jump to an entry', async ({ page }) => {
  await open(page, `${base}/journal`);
  await page.getByRole('searchbox').fill('quoting');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page.locator('.journal-entry')).toHaveCount(1);
  await page.getByRole('button', { name: 'Clear' }).click();
  await expect(page.locator('.journal-entry')).toHaveCount(3);
  await page.locator('.journal-toc button').last().click();
  await expect(page.locator('.journal-entry').last()).toBeFocused();
});

test('hostile text in the journal stays text', async ({ page }) => {
  await open(page, `${base}/journal`);
  await expect(page.getByText('<script>alert(1)</script>', { exact: false })).toBeVisible();
  await expect(page.locator('.journal-entry img')).toHaveCount(0);
  expect(await violations(page)).toEqual([]);
});
