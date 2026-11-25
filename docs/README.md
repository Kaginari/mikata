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
    primaryColor: "#3d2b40"
    primaryTextColor: "#f5eff7"
    primaryBorderColor: "#3d2b40"
    secondaryColor: "#59405d"
    secondaryTextColor: "#f5eff7"
    secondaryBorderColor: "#59405d"
    tertiaryColor: "#16111b"
    tertiaryTextColor: "#f5eff7"
    tertiaryBorderColor: "#3d2b40"
    lineColor: "#ff168a"
    textColor: "#f5eff7"
    titleColor: "#f5eff7"
    edgeLabelBackground: "#0d0b10"
    clusterBkg: "#16111b"
    clusterBorder: "#3d2b40"
    noteBkgColor: "#ffb020"
    noteTextColor: "#1a1205"
    noteBorderColor: "#ffb020"
    actorBkg: "#3d2b40"
    actorBorder: "#3d2b40"
    actorTextColor: "#f5eff7"
    actorLineColor: "#59405d"
    signalColor: "#ff655c"
    signalTextColor: "#f5eff7"
    labelBoxBkgColor: "#3d2b40"
    labelBoxBorderColor: "#3d2b40"
    labelTextColor: "#f5eff7"
    loopTextColor: "#f5eff7"
    activationBkgColor: "#ff168a"
    activationBorderColor: "#ff168a"
    sequenceNumberColor: "#0d0b10"
    transitionColor: "#ff168a"
    transitionLabelColor: "#f5eff7"
    stateLabelColor: "#f5eff7"
    stateBkg: "#3d2b40"
    labelBackgroundColor: "#0d0b10"
    compositeBackground: "#16111b"
    compositeTitleBackground: "#3d2b40"
    altBackground: "#16111b"
    specialStateColor: "#b6f542"
    innerEndBackground: "#b6f542"
  flowchart:
    wrappingWidth: 240
---
flowchart TD
    subgraph MIKATA_BG[" "]
      direction TB
      O("Overview<br/>what Mikata does with an ask") --> LP("The loop<br/>one turn, the end gate")
      LP --> A("Architecture<br/>crates, TUI, board, sandbox")
      A --> P("Client protocol")
      A --> C("Consumption")
      A --> MC("MCP")
      A --> ME("Memory")
      B[("The bench<br/>every screen, measured")]
      R("Roadmap")
      LS("Lessons")
      RL("Releasing")
      X[("The lab<br/>races, chains, plugins")]
      LP -.-> B
      B -.-> R
    end
    classDef core fill:#ff168a,stroke:#ff168a,stroke-width:2px,color:#0d0b10
    class O,LP core
    classDef page fill:#3d2b40,stroke:#3d2b40,stroke-width:1.5px,color:#f5eff7
    class A,P,C,MC,ME page
    classDef proof fill:#b6f542,stroke:#b6f542,stroke-width:2px,color:#203008
    class B proof
    classDef ref fill:#ff655c,stroke:#ff655c,stroke-width:1.5px,color:#1a0a0a
    class R,LS,RL ref
    classDef lab fill:#59405d,stroke:#59405d,stroke-width:1.5px,color:#f5eff7
    class X lab
    style MIKATA_BG fill:#0d0b10,stroke:#3d2b40,color:#0d0b10
```

## Diagrams

Every page has at least one, written as Mermaid in the page's text: it renders on GitHub as in
the book, diffs in review, and is checked by `node tools/check-diagrams.mjs` (in `just docs` and
CI), which fails on a diagram Mermaid cannot parse. Numbers the book reports are charted as
`xychart-beta`, one measure a chart (never two scales on one), its table beside it as the table
view.

They wear Mikata's design system, the same on GitHub as in the book: each block's front matter
carries the palette (`theme: base`), and the whole diagram sits on the night background. A block
is filled with its border's colour and its words are set to read on it: hot pink for Mikata's
core, plum for a step, lime for what checks and passes, amber for you, coral for what is kept,
muted plum for what is outside. Corners are rounded; a decision stays a diamond and a store a
cylinder. Lines are hot pink. In a chart each harness keeps its colour: Mikata hot `#ff168a`,
Maki amber `#cd7d00`, OpenCode blue `#3986e4`, checked apart for colour-blind readers. No emoji in
a diagram: words say what a block is.

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
