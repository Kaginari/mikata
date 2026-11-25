# Overview

## In one picture

<a href="./canvas/overview-light.svg">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./canvas/overview-dark.svg">
  <img src="./canvas/overview-light.svg" alt="The overview canvas: you give Mikata an ask; it does the work inside your repository; when the ask carries a Done when list, yours or a reader's, the harness runs every line and a failing line sends it back to the work; then the end gate runs your verify line and checks that no test was deleted or skipped; it answers with what changed. An ask that names several projects runs one body a project, at once">
</picture>
</a>

<details>
<summary>The sketch in Mermaid, the text this canvas is drawn from</summary>

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
    wrappingWidth: 360
---
flowchart TD
    V(["You"])
    M["Mikata<br/>one body over your repository"]
    P["several projects named in the ask?<br/>one body a project, at once,<br/>each in its own folder"]
    L[("your Done when list, or a reader's<br/>each line held by the harness")]
    W["the work<br/>reads only the lines it needs<br/>writes inside your repository"]
    C{"the list's lines<br/>each one run"}
    G{"the end gate<br/>your verify line runs green<br/>no test deleted or skipped"}
    A(["the answer, with what changed"])

    V -->|your ask| M
    M -.-> P
    M --> W
    V -.->|a list in the ask| L
    L -.-> C
    W -->|with a list| C
    C -->|a line does not hold| W
    C -->|every line holds| G
    W -->|no list| G
    G -->|refused| W
    G -->|passes| A
    A --> V
    classDef you fill:#332711,stroke:#ffb020,stroke-width:2px,color:#ffe1a8
    class V you
    classDef mikata fill:#3a0f26,stroke:#ff168a,stroke-width:3px,color:#ffffff
    class M,P mikata
    classDef store fill:#2a1820,stroke:#ff655c,stroke-width:1.5px,color:#ffe3e0
    class L store
    classDef gate fill:#202c13,stroke:#9be22d,stroke-width:2px,color:#d9f95c
    class C,G gate
```

</details>

## In plain words

**Mikata** (味方, "ally": the one on your side) is a coding agent. You give it an ask; one body
works through it over your whole repository, the way a careful engineer would:

1. **It does the work,** reading only the lines it needs, inside your repository, and runs what
   it needs to run: the tests, a build, a script.
2. **The end gate judges every file it changed:** your verify line runs green, and no test was
   deleted or skipped.
3. **What you say must hold is held.** Lines under `## Done when` in your ask, or the lines a
   reader you name writes, are run by the harness on the finished work, every one, before the
   answer lands.

4. **You are shown what changed, by its own checks:** when the answer has landed, its last
   checks are run again on the code as you gave it and on the finished work, and you see where
   they print differently ([the loop](02-loop.md#what-changed-by-its-own-checks)). No model reads
   that list. You do.

**Its own plan is one switch away.** Asked for (`--profile planned`, or `plan = true` under
`[units]`), the body first writes down what "done" means, weighs two or three ways, and runs every
line and the tests it touched before it answers. That habit took Mikata from 5 of 10 to level with
OpenCode and Maki on our bench. Measured again on 25 November it earned no pass: on twenty mathjs
fixes Mikata passed 17 with it and 17 without, and without it a pass cost 18% less, 1.18 times
what one costs with Maki, which passed 15 ([the bench](bench.md#the-plan-measured-again)). So it is
off by default, and Mikata needs nothing set up.

**You can write the list yourself, or have a reader write it.** Lines under `## Done when` in your
ask (`D1: ... must give ...`, `D2: ... must stay as it is today`) are held by the harness, not
by Mikata's word: it hands in one command a line, the harness reads what must stay from your
code before anything changes, and runs every line when the answer lands
([the loop](02-loop.md#a-done-when-list-in-the-ask)). Name a model as `models.reader` and it
writes those lines for each ask, from the ask alone. On the two asks our bench fails most, that
is the difference between 1 pass in 6 and 6 in 6; on four asks stated as prose, with nothing
added but the reader, between 6 passes in 12 and 10. The list is what does it: the same lists
handed over as a plain note, held by nothing, passed 11 in 12, and 8 in 12 in each of the two
other harnesses we run beside Mikata, where those passed 5 and 3 without
([the bench](bench.md#the-list-held)). What the hold adds is a pass that names what verified it
and what must stay read from your code; it costs about four dollars in ten more a run and is not
shown to add passes. Read the list a reader writes: the harness holds whatever it says, and
names in its pass any line whose check showed nothing of the work.

**An ask that names several projects is split by itself.** When the ask names two or more folders
at the root of your repository that each hold a project file of their own (`pyproject.toml`,
`Cargo.toml`, `package.json`, `go.mod`, …), Mikata runs one body a project, at once, each with its
folder as its root, then checks each project once before it answers. Five whole repositories in
one ask: one body fixed 2.5 of 5 on average; one body a project, 4.5 of 5.

**It acts outside only with your word.** Mikata reads and writes inside your repository, and every
command runs in a sandbox. When the ask needs an act that leaves your machine (a push, a pull
request, a publish, a deploy), Mikata does it itself, and each one shows you an approval card first:
denied, it stays undone. `mikata run` denies every such act unless you pass `--gate-tty`.

**What it learns stays with the repository.** Everything that matters is written to the world
folder, `.mikata/`: the journal `log.md`, the ledgers, memory. Sessions forget; the world
remembers.

### Say what the result must be

Mikata checks its work against what the ask states. On four tasks, an ask with its examples and
their expected output passed 4 times in 4; the same ask as one paragraph of prose passed 4 times
in 8, at the same price. Without examples the body writes its own checks from its own reading of
the ask, and a wrong reading passes them: one task failed 4 times in 4 on a case the ask said, in
words, must stay as it was. One example of what must change and one of what must not is worth more
than a paragraph.

When you did write prose, read the last lines Mikata shows you: what its own checks print now and
printed before. In 21 of the 25 runs that failed on our prose asks, the mistake is in those lines,
and it is quicker to see there than in a diff.

### The modes

Every session opens in **light-coding**: the one body above. `mikata status` says the mode and
why. Two other modes, where the work is handed down a chain of bodies or shared by a collective,
are kept as [experimental](experimental/): usable, measured, and frozen, since one body a project
did better on every screen.

### Compose it (experimental)

Every part of the loop is a **unit** you can switch: memory, the end gate, the held `Done when`
list, the fold, the stress signals, what-changed, the reader, the challenge, and the prompt's plan
and its lighter form, both off by default (`[units]` in your global file, `--units` on `mikata
run`, `/unit on|off` in the terminal). A **profile** names a mix for a kind of work (`--profile
planned` puts the plan back, `--profile prose` its lighter form).
A **plugin** is a program of yours that speaks MCP plus `mikata/hook`, run in the sandbox under a
deadline, and loaded, unloaded or swapped while a session runs ([plugins](experimental/plugins.md)).
The board's **Harness Builder** draws the units round the core and starts a run with exactly
that composition. What each unit is worth is measured one at a time
([plugins: where it stands](experimental/plugins.md#where-it-stands)).

## Technical detail

### The processes

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
    wrappingWidth: 360
---
flowchart TD
    subgraph user["your machine"]
      CLI["mikata<br/>Rust · Ratatui<br/>the core in-process"]
      BOARD["board<br/>TypeScript · React<br/>in the browser"]
      CORE["mikatad<br/>Rust core, served"]
      SB["sandboxed tools<br/>landlock + seccomp / Seatbelt"]
      W[(".mikata/<br/>log.md · ledgers · memory")]
    end
    LLM[("model providers<br/>Anthropic · OpenAI-compatible")]

    BOARD <-->|WebSocket, loopback + token| CORE
    CLI --> SB
    CLI <--> W
    CLI <-->|HTTPS streaming| LLM
    CORE --> SB
    CORE <--> W
    CORE <-->|HTTPS streaming| LLM
    classDef ts fill:#211824,stroke:#ff168a,stroke-width:1.5px,color:#f5eff7
    class BOARD ts
    classDef rs fill:#2a1820,stroke:#ff655c,stroke-width:1.5px,color:#ffe3e0
    class CLI,CORE rs
    classDef gate fill:#202c13,stroke:#9be22d,stroke-width:2px,color:#d9f95c
    class SB gate
    classDef store fill:#2a1820,stroke:#ff655c,stroke-width:1.5px,color:#ffe3e0
    class W store
    classDef outside fill:#16111b,stroke:#95849c,stroke-width:1.5px,color:#d8cfdc
    class LLM outside
    style user fill:#16111b,stroke:#59405d,color:#f5eff7
```

- **The core** (Rust crates) owns everything that must be correct: the loop, the gate, the
  sandbox, the ledgers, the provider clients. It has no UI, and every surface links the same
  crates.
- **`mikata`** (Rust, Ratatui) is the command you run. With no subcommand it is the terminal
  app; `mikata run "<ask>"` asks once without it, for scripts. It hosts the session in its own
  process, with no protocol between the screen and the core, and draws the same events the server
  sends ([the terminal UI](03-architecture.md#the-terminal-ui)). It renders; it decides nothing.
- **`mikatad`** (Rust) is the core as a server: JSON-RPC 2.0 over stdio for the TypeScript SDK
  ([the client protocol](04-protocol.md)), the web board on loopback, and the world's admin
  commands (`mikatad runs`, `mikatad memory status`, …).
- **The board** (TypeScript, React) is the web cockpit, served by `mikatad` on loopback. It
  speaks the same protocol over a WebSocket; its Harness Builder composes a run's units and
  follows the run live.
- One protocol, defined once in Rust; the TypeScript types are generated from it. A client can
  never drift from the core.

### What is fixed and what is yours

| Compiled in (code) | Configured (data) |
|---|---|
| the loop, its budgets and stress instruments | the provider and the model |
| the end gate's checks | your verify line (the project's test command) |
| the classes of an act: `read`, `write`, `outward`, `destructive` | guard rules that ask you instead of refusing |
| the sandbox's rules | budgets: amounts per session, day, month |
| the meters and the ledgers | provider credentials, by reference, never inline |
| the split by project | MCP servers, and Minds: know-how a body can load |

### Where it comes from

Mikata grew out of [isekai](https://github.com/Kaginari/isekai) and was called **Mnemorph** until
November 2026. Mnemorph's idea was a kingdom of races, each a different kind of body with its own
tools: Elves that plan, Orcs that judge, Slimes that write. The bench kept the discipline and
dropped the kingdom: one body that writes "done" down and checks it beat the chain of races on
every screen. The races live on as the [experimental lab](experimental/), and their rules live
on inside the one body.
