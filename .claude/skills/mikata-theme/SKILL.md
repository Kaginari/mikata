---
name: mikata-theme
description: Mikata's design system — Bio Digital (dark) and Bio Pearl (light), biological intelligence meeting digital intelligence. Use when building or styling any Mikata surface: the board, the docs site, the website, the CLI's colours, cards, decks, banners or other images.
---

# The Mikata design system

**Bio Digital** (dark: immersive, technical) and **Bio Pearl** (light: clean, sophisticated): a
living, modular AI engineering environment, never a generic admin dashboard. Mikata's mark
(`assets/source/mikata-mark.png`: a living cell flowing into a digital one, hot pink through coral
to amber) is the reference look. They are the `neon` scheme, the default everywhere (the board
sets `data-scheme`, the docs site sets it before first paint). The races keep their **hex** frames
and their own colours in the experimental modes only.

The rules (the owner's, 2026-11-25):
- **Hot pink, coral, amber** are the brand. The pink-to-amber gradient is reserved for branding,
  primary actions, selected navigation and the runtime's important connections.
- **Lime** (yellow-green) is biological energy: active agents, successful execution, verification,
  connected plugins. Never a conventional emerald or turquoise. Words on lime are dark olive
  (`--mn-success-on`). The bio gradient is reserved for meaningful execution and verification.
- **Icons**: Lucide line icons, one stroke width (the board's `ui/Icon.tsx`: `Icon`, and `Glyph`
  for the status glyph names). Never an emoji as an interface icon.
- **Type**: Space Grotesk for the interface, JetBrains Mono for code, logs,
  configuration and technical metrics.
- The Harness Builder stays visibly connected: the runtime at the centre, edges to every unit and
  plugin (native pink, plugin amber, off dashed grey, live lime).

## The source of truth

- `packages/theme/src/tokens.css` — the web tokens (`--mn-*`), dark by default, light under
  `:root[data-theme="light"]`, and the components below. Import it as `@mikata/theme/tokens.css`.
- `packages/theme/src/index.ts` — the same palette for TypeScript (`dark`, `light`, `races`) and
  `contrast()`. The CLI's Ink colours come from here.
- Its test holds the two files equal and every text colour at WCAG AA on every surface. A new
  colour goes into both files and the test's lists.

Read colours from tokens, always: a hex literal in a component is a second source of truth.

## The vocabulary

Semantic roles, derived for every scheme in `:root` and set exactly for Bio Digital and Bio Pearl:

| Word | Token | Meaning |
|---|---|---|
| surface | `--mn-bg`, `--mn-panel`, `--mn-panel-raised`, `--mn-surface-hover` | page, container, a container on a container, hover |
| line | `--mn-line`, `--mn-line-strong` | borders, dividers |
| text | `--mn-text`, `--mn-text-muted`, `--mn-text-faint` | words, secondary words, metadata |
| brand fills | `--mn-hot`, `--mn-coral`, `--mn-amber` | fills, edges, gradients — not words on the light page |
| brand words | `--mn-holo` (hot), `--mn-gold` (amber) | the brand as text, AA on every surface |
| brand gradients | `--mn-gradient`, `--mn-gradient-full` | branding, primary actions, selection, connections |
| on the brand | `--mn-brand-surface` + `--mn-on-brand` | a bar, a primary button, a selected item, with its words |
| brand as words | `--mn-brand-ink` | a title clipped to the brand, AA on the page |
| **bio** | `--mn-success-fill`, `--mn-success-text`, `--mn-success-bg`, `--mn-success-border`, `--mn-success-on`, `--mn-gradient-bio` | alive, connected, verified |
| semantic | `--mn-warning(-bg)`, `--mn-error(-bg)`, `--mn-info(-bg)`, `--mn-alarm` | attention, failure, information |
| graph | `--mn-edge-native`, `--mn-edge-plugin`, `--mn-edge-active`, `--mn-edge-disabled`, `--mn-canvas-grid` | the Builder's wires and canvas |
| effects | `--mn-glow`, `--mn-glow-soft`, `--mn-glow-success`, `--mn-shadow` | glow and depth |
| race colour | `--mn-race-<race>` / `races.<race>` | anything that belongs to one race: its lane, prefix, card |
| panel | `.mn-panel` | rounded, thin holo border, inner glow |
| **hex** | `.mn-hex` | the frame of every race icon |
| title | `.mn-title` | the name and page titles only |
| data | `.mn-mono` | ids, logs, numbers, wire lines |
| interactive | `.mn-interactive` | hover: border brightens and pulses, one scan line; active: gold flash |
| stream | `.mn-stream` | faint scrolling code behind a page — decoration, never content |

Type: `--mn-font-ui` and `--mn-font-title` are Space Grotesk, `--mn-font-mono` JetBrains Mono; the board
bundles both (`@fontsource`, imported in `main.tsx`; the CSP allows no other font source).

## The assets

| What | Where |
|---|---|
| Mikata's logo, for dark and light pages | `assets/brand/mikata-logo-{dark,light}.png`, from `assets/source/mikata-logo-{dark,light}.svg` |
| Mikata's icon (the mark on its tile) | `assets/brand/mikata-icon{,@256,@128,@64,@32}.png`, `favicon.ico`, from `assets/source/mikata-icon.svg` |
| the link card | `assets/brand/social-preview.png` (1280×640) |
| Mnemorph's emblem, banner and deck (the races, experimental) | `docs/experimental/assets/mnemorph-*` |
| race icons, already cut to hexagons | `assets/portraits/<race>{,@64,@32}.png` |
| race cards | `assets/cards/<race>.png` — drawn by `python3 tools/cards.py` |
| fonts for images | `assets/fonts/` (Cinzel, Rajdhani; OFL) |

## A surface is done when

1. Every colour, font and radius is a token.
2. It reads in **both** modes: switch `data-theme` and look at each. Text keeps WCAG AA (4.5:1)
   on every surface **and on its own tint** (a tag's text on its tag's background).
3. Motion stops under `prefers-reduced-motion` — the tokens' components already do; new animation
   joins that block.
4. Focus is visible on every interactive element (`.mn-interactive` gives the holo ring).
5. Diagrams and other paper-drawn images keep their light sheet in dark mode (the site's
   `.mermaid` rule).
6. You have **seen** it: a screenshot of each mode, not only a passing build.
