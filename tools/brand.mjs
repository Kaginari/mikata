// Renders Mikata's brand files from assets/source (the mark, mikata-mark.png, is the owner's
// fusion-cell art, cleaned of its cut-out fringe): the icon and its sizes, the logos for dark
// and light pages, the social card and the favicon — then copies the icon where the TUI, the board
// and the site read it.
// usage: node tools/brand.mjs [<a woff2 holding 味方>]
// The wordmark's two kanji need a Japanese font: pass one (a Noto Sans JP subset is enough), or
// the system's is used.

import { copyFileSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(join(repo, 'packages', 'board', 'package.json'));
const { chromium } = require('@playwright/test');
const src = (f) => join(repo, 'assets', 'source', f);
const out = (f) => join(repo, 'assets', 'brand', f);
/** A font file as a data: URL: a page set from a string may not read files. */
const inline = (path) => `data:font/woff2;base64,${readFileSync(path).toString('base64')}`;
const font = (pkg, file) =>
  inline(join(repo, 'packages', 'board', 'node_modules', '@fontsource', pkg, 'files', file));

const jp = process.argv[2];
const faces = [
  `@font-face{font-family:'JetBrains Mono';font-weight:700;src:url(${font('jetbrains-mono', 'jetbrains-mono-latin-700-normal.woff2')})}`,
  `@font-face{font-family:'JetBrains Mono';font-weight:400;src:url(${font('jetbrains-mono', 'jetbrains-mono-latin-400-normal.woff2')})}`,
  `@font-face{font-family:'Manrope';font-weight:600;src:url(${font('manrope', 'manrope-latin-600-normal.woff2')})}`,
  jp ? `@font-face{font-family:'Noto Sans JP';font-weight:700;src:url(${inline(jp)})}` : '',
].join('');

const browser = await chromium.launch();
const page = await browser.newPage();

/** The SVG `file`, inline (so its text takes the faces above), as a PNG `w`×`h`. */
async function render(file, w, h) {
  const mark = `data:image/png;base64,${readFileSync(src('mikata-mark.png')).toString('base64')}`;
  const svg = readFileSync(src(file), 'utf8')
    .replace(/<!--[\s\S]*?-->/g, '')
    .replaceAll('href="mikata-mark.png"', `href="${mark}"`);
  await page.setViewportSize({ width: w, height: h });
  await page.setContent(
    `<style>${faces}html,body{margin:0;background:transparent}svg{display:block;width:${w}px;height:${h}px}</style>${svg}`,
  );
  await page.evaluate(async () => {
    await document.fonts.ready;
    await Promise.all(
      [...document.querySelectorAll('image')].map(
        (i) =>
          new Promise((done) => (i.complete !== false ? done() : i.addEventListener('load', done))),
      ),
    );
  });
  await page.waitForTimeout(150);
  return page.screenshot({ omitBackground: true, clip: { x: 0, y: 0, width: w, height: h } });
}

const icon = await render('mikata-icon.svg', 512, 512);
writeFileSync(out('mikata-icon.png'), icon);
for (const n of [256, 128, 64])
  writeFileSync(out(`mikata-icon@${n}.png`), await render('mikata-icon.svg', n, n));
const i32 = await render('mikata-icon-small.svg', 32, 32);
writeFileSync(out('mikata-icon@32.png'), i32);
writeFileSync(out('mikata-logo-dark.png'), await render('mikata-logo-dark.svg', 512, 512));
writeFileSync(out('mikata-logo-light.png'), await render('mikata-logo-light.svg', 512, 512));
writeFileSync(out('social-preview.png'), await render('social-preview.svg', 1280, 640));
const i64 = await render('mikata-icon-small.svg', 64, 64);
// The mark alone, 600 px wide, for the board's header and core.
await page.setViewportSize({ width: 600, height: 217 });
await page.setContent(
  `<style>html,body{margin:0;background:transparent}img{display:block;width:600px;height:217px}</style><img src="data:image/png;base64,${readFileSync(src('mikata-mark.png')).toString('base64')}">`,
);
await page.evaluate(() => document.querySelector('img').decode());
writeFileSync(
  join(repo, 'packages', 'board', 'public', 'mikata-mark.png'),
  await page.screenshot({ omitBackground: true, clip: { x: 0, y: 0, width: 600, height: 217 } }),
);
await browser.close();

// The favicon: an ICO holding two PNGs, 32 and 64 px.
const images = [
  [32, i32],
  [64, i64],
];
const head = Buffer.alloc(6 + 16 * images.length);
head.writeUInt16LE(0, 0);
head.writeUInt16LE(1, 2);
head.writeUInt16LE(images.length, 4);
let offset = head.length;
images.forEach(([size, png], i) => {
  const e = 6 + 16 * i;
  head.writeUInt8(size, e);
  head.writeUInt8(size, e + 1);
  head.writeUInt16LE(1, e + 4);
  head.writeUInt16LE(32, e + 6);
  head.writeUInt32LE(png.length, e + 8);
  head.writeUInt32LE(offset, e + 12);
  offset += png.length;
});
writeFileSync(out('favicon.ico'), Buffer.concat([head, ...images.map(([, png]) => png)]));

// The copies the other parts read: the TUI embeds its own, the board serves its own.
copyFileSync(
  out('mikata-icon@256.png'),
  join(repo, 'crates', 'mikata-tui', 'assets', 'brand', 'mikata-icon.png'),
);
copyFileSync(
  out('mikata-icon@64.png'),
  join(repo, 'packages', 'board', 'public', 'mikata-icon.png'),
);
console.log('brand files written');
