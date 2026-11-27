/**
 * A chat end to end on the board's real server and bridge: the `board_e2e` example of
 * mikata-server serves this built page over one fresh world, and each chat's child is a real
 * core — the loop, the tools, the end gate — answering from a scripted model instead of a
 * provider. The page trades the printed key, opens a chat, sends two turns and sees them in the
 * TUI's grammar, under the server's own CSP; a reload replays without duplicating; the past run
 * reads from its journal once the chat ends.
 *
 * The binary is built by `cargo build -p mikata-server --example board_e2e` after the board
 * (`pnpm --filter @mikata/board build`): CI does both. Locally, the test is skipped without it.
 */
import { type ChildProcess, spawn } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, test } from '@playwright/test';

const repo = fileURLToPath(new URL('../../../', import.meta.url));
const BIN =
  process.env.MIKATA_BOARD_E2E ??
  join(process.env.CARGO_TARGET_DIR ?? join(repo, 'target'), 'debug/examples/board_e2e');

test.describe('the real bridge', () => {
  test.skip(!existsSync(BIN) && !process.env.CI, `no ${BIN}: build the board, then the example`);

  let root: string;
  let server: ChildProcess;
  let url: string;

  test.beforeEach(async () => {
    test.skip(test.info().project.name !== 'desktop', 'one browser is enough');
    root = mkdtempSync(join(tmpdir(), 'mikata-board-e2e-'));
    server = spawn(BIN, [root], { stdio: ['ignore', 'pipe', 'inherit'] });
    url = await new Promise<string>((resolve, reject) => {
      let out = '';
      server.stdout?.on('data', (d: Buffer) => {
        out += d.toString();
        const line = out.split('\n')[0];
        if (out.includes('\n') && line) resolve(line.trim());
      });
      server.on('exit', (code) => reject(new Error(`board_e2e exited with ${code}`)));
      setTimeout(() => reject(new Error('board_e2e printed no URL')), 30_000);
    });
  });

  test.afterEach(() => {
    server?.kill('SIGTERM');
    if (root) rmSync(root, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
  });

  test('a scripted session, two turns, through the real bridge', async ({ page }) => {
    test.setTimeout(90_000);
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await page.addInitScript(() => {
      const seen: string[] = [];
      (window as { __csp?: string[] }).__csp = seen;
      document.addEventListener('securitypolicyviolation', (e) =>
        seen.push(`${e.violatedDirective} ${e.blockedURI}`),
      );
    });
    await page.goto(url);
    await expect(page).toHaveURL(/#\/w\/[^/]+$/);
    // The rail's picker, a drawer's at phone width.
    const rail = async () => {
      const menu = page.getByRole('button', { name: 'Menu' });
      if (await menu.isVisible()) {
        if ((await menu.getAttribute('aria-expanded')) !== 'true') await menu.click();
        await expect(page.locator('#pf-rail')).toBeVisible();
      }
      return page.locator('#pf-rail');
    };
    await (await rail())
      .getByRole('combobox', { name: 'Workspace' })
      .selectOption({ label: 'world' });
    await expect(page.locator('.ss-empty')).toContainText('world');
    // A session is started in the Harness Builder, where its harness is chosen.
    await page.locator('.pf-top').getByRole('link', { name: 'Harness Builder' }).click();
    await page.getByRole('button', { name: 'Start session' }).click();
    await expect(page).toHaveURL(/#\/w\/[^/]+\/s\/[^/]+$/);
    await expect(page.locator('.ss-chips')).toContainText('scripted-model');
    const box = page.getByRole('textbox', { name: 'Ask Mikata' });
    await expect(box).toBeEnabled();
    await box.fill('what does notes.txt say?');
    await box.press('Enter');

    const talk = page.getByRole('region', { name: 'The conversation' });
    await expect(talk.locator('.ss-msg-user')).toContainText('what does notes.txt say?');
    await expect(talk.locator('.mapp-step h3', { hasText: 'edited notes.txt' })).toBeVisible({
      timeout: 30_000,
    });
    await expect(talk).toContainText('they say b now');
    await expect(talk.locator('.mapp-landed')).toBeVisible();
    // The model's hostile tail stays text.
    await expect(talk).toContainText('(link not followed)');
    expect(await page.locator('main img, main a[href^="javascript"]').count()).toBe(0);
    await expect(talk.locator('.mapp-working')).toHaveCount(0);
    await expect(page.getByRole('log', { name: 'Run log' })).toContainText('notes.txt');

    await box.fill('and again');
    await box.press('Enter');
    await expect(
      talk.locator('.mapp-answer', { hasText: 'The second turn, answered.' }),
    ).toBeVisible();
    await expect(talk.locator('.ss-msg-user')).toHaveCount(2);

    // A reload replays the session from the bridge: each message once.
    await page.reload();
    await expect(
      talk.locator('.mapp-answer', { hasText: 'The second turn, answered.' }),
    ).toBeVisible();
    await expect(talk.locator('.ss-msg-user')).toHaveCount(2);
    await expect(talk.locator('.mapp-step h3', { hasText: 'edited notes.txt' })).toHaveCount(1);

    // Ended, the session is a past run, read from its journal.
    await page.getByRole('button', { name: 'End session' }).click();
    await expect(box).toBeDisabled();
    const past = (await rail()).locator('a.pf-session[href*="/r/"]').first();
    await expect(past).toBeVisible({ timeout: 20_000 });
    await past.click();
    await expect(page).toHaveURL(/#\/w\/[^/]+\/r\/r\d+-\d+$/);
    await expect(page.locator('.msg-user')).toHaveCount(2);
    await expect(
      page.locator('.msg-body', { hasText: 'The second turn, answered.' }),
    ).toBeVisible();
    await expect(page.locator('.composer-closed')).toContainText('read-only');

    expect(errors).toEqual([]);
    expect(await page.evaluate(() => (window as { __csp?: string[] }).__csp)).toEqual([]);
  });
});
