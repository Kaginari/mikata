// Photographs a frame of the terminal UI: the cells `cargo run -p mikata-tui --example frame` writes,
// laid on a fixed grid, each with its own colours, in a monospace face.
// usage: node tools/tui-shot.mjs <frame.json> <out.png>

import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(join(here, 'canvas', 'package.json'));
const { chromium } = require('@playwright/test');
const [input, out] = process.argv.slice(2);
const f = JSON.parse(readFileSync(input, 'utf8'));
const CW = 9.6,
  CH = 19;
const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
let html = '';
const pageBg = f.rows[0]?.[0]?.bg || '#000';
for (let y = 0; y < f.h; y++) {
  for (let x = 0; x < f.w; x++) {
    const c = f.rows[y][x];
    const st = [`left:${(x * CW).toFixed(1)}px`, `top:${y * CH}px`];
    if (c.bg?.startsWith('#')) st.push(`background:${c.bg}`);
    if (c.fg?.startsWith('#')) st.push(`color:${c.fg}`);
    if (c.b) st.push('font-weight:700');
    if (c.i) st.push('font-style:italic');
    if (c.d) st.push('opacity:.6');
    if (c.u) st.push('text-decoration:underline');
    const s = c.s === ' ' || c.s === '' ? '&nbsp;' : esc(c.s);
    html += `<span style="${st.join(';')}">${s}</span>`;
  }
}
const W = Math.ceil(f.w * CW),
  H = f.h * CH;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: W, height: H }, deviceScaleFactor: 2 });
await page.setContent(`<!doctype html><style>
body{margin:0;background:${pageBg}}
span{position:absolute;width:${CW}px;height:${CH}px;line-height:${CH}px;text-align:center;
font-family:'Liberation Mono','DejaVu Sans Mono',monospace;font-size:15.5px;color:#ddd;overflow:visible;white-space:pre}
</style><body>${html}</body>`);
await page.screenshot({ path: out });
await browser.close();
console.log(out, W, H);
