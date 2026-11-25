import { cpSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitepress';
import { withMermaid } from 'vitepress-plugin-mermaid';

// The book lives in docs/ and stays readable on GitHub as it is; the site only renders it.
// Its pages sit outside this package, so `vue` is a root devDependency: resolution from docs/
// reaches the root node_modules under pnpm's isolated layout.
export default withMermaid(
  defineConfig({
    title: 'Mikata',
    description:
      'Mikata, the coding agent on your side: one body that writes down what done means and checks every line of it.',
    srcDir: '../docs',
    // The front page is the landing (index.md); the book's own front page is at /book.
    rewrites: { 'README.md': 'book.md', 'experimental/README.md': 'experimental/index.md' },
    // Local-only notes (ignored by git) never reach the site.
    srcExclude: ['study-maki.md'],
    cleanUrls: true,
    // Formulas are written as ```math fences: GitHub renders those untouched (a `$$` block is
    // parsed as Markdown first, so `\,` loses its backslash and a line starting with `+` becomes a
    // list). Here each fence goes to the same MathJax renderer `$$` uses.
    markdown: {
      math: true,
      config(md) {
        const fence = md.renderer.rules.fence;
        const mathBlock = md.renderer.rules.math_block;
        md.renderer.rules.fence = (tokens, idx, options, env, self) =>
          tokens[idx].info.trim() === 'math' && mathBlock
            ? mathBlock(tokens, idx, options, env, self)
            : fence
              ? fence(tokens, idx, options, env, self)
              : self.renderToken(tokens, idx, options);
      },
    },
    // The brand's files are the site's public files: the icon is the logo and the favicon.
    vite: { publicDir: fileURLToPath(new URL('../../assets/brand', import.meta.url)) },
    // A canvas links to its SVG at full size: the SVGs are served beside the pages, at /canvas/.
    buildEnd(site) {
      cpSync(join(site.srcDir, 'canvas'), join(site.outDir, 'canvas'), {
        recursive: true,
        filter: (path) => path.endsWith('.svg') || statSync(path).isDirectory(),
      });
    },
    head: [
      ['link', { rel: 'icon', href: '/favicon.ico', sizes: 'any' }],
      ['link', { rel: 'icon', type: 'image/png', href: '/mikata-icon@64.png' }],
      ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
      [
        'link',
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Cinzel:wght@600;700&family=Rajdhani:wght@600;700&family=JetBrains+Mono:wght@400;600&display=swap',
        },
      ],
    ],
    // Dark first, like the logo; the toggle still offers light.
    appearance: 'dark',
    lastUpdated: true,
    themeConfig: {
      logo: '/mikata-icon@64.png',
      nav: [
        { text: 'Book', link: '/book' },
        { text: 'Bench', link: '/bench' },
        { text: 'Roadmap', link: '/roadmap' },
        { text: 'Experimental', link: '/experimental/' },
        { text: 'GitHub', link: 'https://github.com/Kaginari/mikata' },
      ],
      sidebar: [
        {
          text: 'Mikata',
          items: [
            { text: 'The book', link: '/book' },
            { text: 'Overview', link: '/01-overview' },
            { text: 'The loop', link: '/02-loop' },
            { text: 'Architecture', link: '/03-architecture' },
            { text: 'The client protocol', link: '/04-protocol' },
            { text: 'Consumption', link: '/05-consumption' },
            { text: 'MCP', link: '/06-mcp' },
            { text: 'Memory', link: '/07-memory' },
            { text: 'The bench', link: '/bench' },
            { text: 'Roadmap', link: '/roadmap' },
          ],
        },
        {
          text: 'Reference',
          items: [
            { text: 'Lessons', link: '/lessons' },
            { text: 'Releasing', link: '/releasing' },
          ],
        },
        {
          text: 'Experimental: the lab',
          collapsed: true,
          items: [
            { text: 'What the lab is', link: '/experimental/' },
            { text: 'The experimental modes', link: '/experimental/modes' },
            { text: 'Races', link: '/experimental/races' },
            { text: 'Communication between bodies', link: '/experimental/communication' },
            { text: 'Ontology', link: '/experimental/ontology' },
            { text: 'How it was built: M0 to M14', link: '/experimental/roadmap-m0-m14' },
            { text: 'Race cards brief', link: '/experimental/race-cards-prompt' },
            { text: 'Research: DIKWP-MESH', link: '/experimental/research/dikwp-mesh' },
          ],
        },
      ],
      outline: { level: [2, 3] },
      search: { provider: 'local' },
    },
    mermaid: { look: 'handDrawn', theme: 'neutral' },
  }),
);
