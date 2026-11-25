# Mikata — the book

Mikata is a coding agent: a Rust core and terminal app, `mikata`, a server, `mikatad`, for the
TypeScript SDK and the web board, and one body that works over your repository, held to an end
gate and to every line of "done" you write down. This book says exactly what the code does.

Every page reads the same way:

1. **In one picture** — a diagram you can understand in a minute.
2. **In plain words** — the idea, without jargon.
3. **Technical detail** — the exact rules, types and numbers the code implements.

## In one picture

```mermaid
---
config:
  theme: base
  themeVariables:
    darkMode: true
    background: "#0d0b10"
    fontFamily: "Inter, Segoe UI, system-ui, -apple-system, Helvetica, Arial, sans-serif"
    fontSize: "15px"
    primaryColor: "#211824"
    primaryTextColor: "#f5eff7"
    primaryBorderColor: "#ff168a"
    secondaryColor: "#2c2030"
    secondaryTextColor: "#f5eff7"
    secondaryBorderColor: "#ff655c"
    tertiaryColor: "#16111b"
    tertiaryTextColor: "#f5eff7"
    tertiaryBorderColor: "#59405d"
    lineColor: "#ff168a"
    textColor: "#f5eff7"
    titleColor: "#f5eff7"
    edgeLabelBackground: "#16111b"
    clusterBkg: "#16111b"
    clusterBorder: "#59405d"
    noteBkgColor: "#332711"
    noteTextColor: "#ffe1a8"
    noteBorderColor: "#ffb020"
    actorBkg: "#211824"
    actorBorder: "#ff168a"
    actorTextColor: "#f5eff7"
    actorLineColor: "#59405d"
    signalColor: "#ff655c"
    signalTextColor: "#f5eff7"
    labelBoxBkgColor: "#211824"
    labelBoxBorderColor: "#ff168a"
    labelTextColor: "#f5eff7"
    loopTextColor: "#f5eff7"
    activationBkgColor: "#3a0f26"
    activationBorderColor: "#ff168a"
    sequenceNumberColor: "#0d0b10"
    transitionColor: "#ff168a"
    transitionLabelColor: "#f5eff7"
    stateLabelColor: "#f5eff7"
    stateBkg: "#211824"
    labelBackgroundColor: "#16111b"
    compositeBackground: "#16111b"
    compositeTitleBackground: "#211824"
    altBackground: "#16111b"
    specialStateColor: "#b6f542"
    innerEndBackground: "#b6f542"
  flowchart:
    wrappingWidth: 240
---
flowchart TD
    O["Overview<br/>what Mikata does with an ask"] --> LP["The loop<br/>one turn, the end gate"]
    LP --> A["Architecture<br/>crates, TUI, board, sandbox"]
    A --> P["Client protocol"]
    A --> C["Consumption"]
    A --> MC["MCP"]
    A --> ME["Memory"]
    B[("The bench<br/>every screen, measured")]
    R["Roadmap"]
    LS["Lessons"]
    RL["Releasing"]
    X[("The lab<br/>races, chains, plugins")]
    LP -.-> B
    B -.-> R
    classDef core fill:#3a0f26,stroke:#ff168a,stroke-width:3px,color:#ffffff
    class O,LP core
    classDef page fill:#211824,stroke:#ff168a,stroke-width:1.5px,color:#f5eff7
    class A,P,C,MC,ME page
    classDef proof fill:#202c13,stroke:#9be22d,stroke-width:2px,color:#d9f95c
    class B proof
    classDef ref fill:#2a1820,stroke:#ff655c,stroke-width:1.5px,color:#ffe3e0
    class R,LS,RL ref
    classDef lab fill:#16111b,stroke:#95849c,stroke-width:1.5px,color:#d8cfdc
    class X lab
```

## Diagrams

Two layers, one picture each:

- **Canvases** — the overview, the loop, the architecture, the sandbox, the client protocol and
  consumption (and, in the lab, the races and one order down the chain) are drawn on tldraw's
  canvas: one document per page, `docs/canvas/<page>.tldr` (a tldraw page per sketch; a lab page's
  is `experimental-<page>.tldr`), and each sketch
  exported as `docs/canvas/<sketch>-light.svg` and `-dark.svg`, which the page shows in the
  reader's theme. A `.tldr` opens in [tldraw offline](https://github.com/tldraw/tldraw-offline),
  tldraw.com and tldraw's VS Code extension.
- **Mermaid sketches** — every page also keeps its diagrams as Mermaid in the text. They render
  on GitHub, diff in review, and are the source a canvas is drawn from: under a canvas, its sketch
  sits folded in a "The sketch in Mermaid" section.
- **Charts** — numbers the book reports are charted as Mermaid `xychart-beta`, one measure a
  chart (never two scales on one), its table beside it as the table view.

Every page has at least one. All of them wear Mikata's design system: a Mermaid block carries
the **Bio Digital** palette in its front matter (`theme: base`: night panels, hot-pink lines, lime
for what checks and passes, amber for you, coral for what is kept, muted grey for what is
outside), so GitHub draws it as the book does; a canvas is exported in **Bio Pearl** for a light
page and **Bio Digital** for a dark one (`tools/canvas/page.js` repaints tldraw's palette, and
arrows are drawn hot). No emoji in a diagram: words say what a node is.

A canvas is drawn, never edited by hand: `pnpm --filter @mikata/canvas draw` lays each sketch
out with tldraw's own Mermaid converter (`@tldraw/mermaid`) in headless Chromium and writes the
`.tldr` and both SVGs; the same sketches always make the same files. Its labels are written as
plain SVG text in tldraw's hand-drawn font, cut down to the characters they use, so an SVG holds
no HTML and fetches nothing. A canvas must read in the page's column: no wider than 2:1, its
labels at 10 px or more at 900 px wide, or the draw fails; a long sketch is re-laid for its canvas
only (the loop and the sandbox top-down, the architecture by ELK, one order's messages wrapped and
its rows spaced), its Mermaid text kept as written. A node its sketch dashes keeps the dashed
border. Each canvas links to its SVG at full size. The list of sketches and their layouts is
`tools/canvas/draw.mjs`'s; it needs the Chromium `@playwright/test` pins once (`pnpm --filter
@mikata/canvas exec playwright install chromium`).

When the code changes a flow, the sketch and its canvas change in the same change (Vitality).
Canvases are reviewed like code: a tldraw document or an SVG can carry script, and tldraw
offline's own `.tldraw` archive can embed a document script that runs when it is opened, so the
book keeps `.tldr` and SVG only. `node tools/check-canvas.mjs` (in `just docs` and CI) fails on
anything else in `docs/canvas/`, and on any script, event handler, `javascript:` URL, link or
source outside the file, `<foreignObject>`, embed or media there.

## Reading order

| # | Page | Read it to learn |
|---|---|---|
| 1 | [Overview](01-overview.md) | what Mikata does with an ask, the split by project, the processes |
| 2 | [The loop](02-loop.md) | one turn, budgets, the stress instruments, resume, the verify pins, the end gate |
| 3 | [Architecture](03-architecture.md) | crates, the terminal UI, the board, storage, sandbox, configuration |
| 4 | [The client protocol](04-protocol.md) | how the SDK and the board speak to `mikatad` |
| 5 | [Consumption](05-consumption.md) | how every token is budgeted, metered and stopped |
| 6 | [MCP](06-mcp.md) | MCP servers from your config, scoped, ranked, priced and learned; only the needed specs out |
| 7 | [Memory](07-memory.md) | the archive and the traces, recall with a seat for dissent, why only the gate writes memory |
| — | [The bench](bench.md) | what we measured against OpenCode and Maki, how, and every screen since |
| — | [Roadmap](roadmap.md) | what comes next |
| — | [Lessons](lessons.md) | what isekai and our own bench taught us, each one a test or a rule |
| — | [Releasing](releasing.md) | how a version is tagged, built, smoke-tested, drafted and published |

## The lab

[Experimental](experimental/) holds what Mikata was as **Mnemorph**: a kingdom of races, a chain of
bodies that hand work down (structured-coding), and a collective of equals (collective-coding).
They still run, behind `--mode`, and are frozen: one body a project did better on every screen.
Their pages are there: [the modes](experimental/modes.md), [the races](experimental/races.md),
[communication between bodies](experimental/communication.md), [the ontology](experimental/ontology.md),
[plugins: every part a unit](experimental/plugins.md),
and [how it was all built, M0 to M14](experimental/roadmap-m0-m14.md).

## Three words

- **World** — the repository Mikata works in, and its folder `.mikata/`: the journal `log.md`, the
  ledgers, memory, your config. Sessions forget; the world remembers.
- **Body** — one running context: the model, its prompt and its tools, for one ask. It dies with
  the ask. Only what it wrote to the world survives it.
- **Gate** — what stands between a body and your repository: the sandbox on every command, your
  word on an act a guard rule says to ask about, and the end gate on every change.
