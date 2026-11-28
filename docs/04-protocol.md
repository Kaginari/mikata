# The client protocol

How programs that are not Rust talk to Mikata: the TypeScript SDK, an editor plugin, the web
board. They all speak one JSON-RPC 2.0 API to `mikatad`.

## In one picture

```mermaid
---
config:
  theme: base
  themeVariables:
    darkMode: true
    background: "#07050a"
    fontFamily: "Inter, Segoe UI, system-ui, -apple-system, Helvetica, Arial, sans-serif"
    fontSize: "15px"
    primaryColor: "#3d2b40"
    primaryTextColor: "#f5eff7"
    primaryBorderColor: "#3d2b40"
    secondaryColor: "#59405d"
    secondaryTextColor: "#f5eff7"
    secondaryBorderColor: "#59405d"
    tertiaryColor: "#110c14"
    tertiaryTextColor: "#f5eff7"
    tertiaryBorderColor: "#3d2b40"
    lineColor: "#ff168a"
    textColor: "#f5eff7"
    titleColor: "#f5eff7"
    edgeLabelBackground: "#07050a"
    clusterBkg: "#110c14"
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
    sequenceNumberColor: "#07050a"
    transitionColor: "#ff168a"
    transitionLabelColor: "#f5eff7"
    stateLabelColor: "#f5eff7"
    stateBkg: "#3d2b40"
    labelBackgroundColor: "#07050a"
    compositeBackground: "#110c14"
    compositeTitleBackground: "#3d2b40"
    altBackground: "#110c14"
    specialStateColor: "#b6f542"
    innerEndBackground: "#b6f542"
  flowchart:
    wrappingWidth: 360
---
sequenceDiagram
    box rgb(7, 5, 10)
      participant C as SDK client
      participant D as mikatad
    end
    rect rgb(7, 5, 10)
      rect rgba(2,132,199,0.10)
      C->>D: initialize {clientInfo}
      D-->>C: {protocol, version, world, model, provider, session, run, holes, budget, key, sandbox, mode, modeWhy, body, held, guardsOff}
      C->>D: session/prompt {text}
      end
      rect rgba(63,169,245,0.14)
      D--)C: event/turnStarted
      D--)C: event/textDelta …
      D--)C: event/toolCall {id, name, input}
      D--)C: event/usage {tokens, cacheRead, usd, spentWt}
      end
      rect rgba(212,175,55,0.22)
      D->>C: gate/ask {body, race, class, act, why, outside, plan}
      C-->>D: {allow: true}
      end
      rect rgba(22,163,74,0.13)
      D--)C: event/gateVerdict {passed, verdict, changed, failures, unverified}
      D--)C: event/turnEnded {ending: answered}
      end
    end
```

## In plain words

`mikatad` speaks one JSON-RPC 2.0 API. The TypeScript SDK reaches it over stdio, the board over a
WebSocket on your machine. The types are defined once in Rust and generated into TypeScript
(`@mikata/protocol`), so a client can never drift from the core. The terminal app, `mikata`, needs
no wire at all: it runs the session in its own process and draws the same events, made by the same
function (`mikata_server::events`). `mikata run --json` prints those events as JSON lines.

A client opens a session, sends an ask, and reads the turn as a stream of events: text, tool
calls, spend, the end gate's verdict. When Mikata wants to do something that needs you, the core
asks the client (`gate/ask`) and waits. Nothing that needs your word runs until a client says yes,
and an ask left unanswered is a no.

## Technical detail

| Kind | Served | Planned |
|---|---|---|
| requests (client → core) | `initialize`, `session/prompt`, `session/send`, `session/interrupt`, `session/events {since}`, `units/list`, `units/change`, `shutdown` | `session/resume`, `queue/list`, `queue/move`, `world/status`, `ledger/query`, `config/explain` |
| requests (core → client) | `gate/ask` — the human gate, the only call the core blocks on | |
| events (core → client) | `turnStarted`, `queued`, `textDelta`, `toolStart`, `assistant`, `toolCall`, `toolResult`, `usage`, `note`, `warning`, `gateVerdict`, `changes`, `turnEnded`, `held`, `guards` | `dispatch`, `report`, `ascension`, `evolution` |

Each message is one line of JSON. `initialize` opens the session and answers `{protocol, version,
world, model, provider, session, run, holes, budget, key, sandbox, mode, modeWhy, body, held, guardsOff}`:
`guardsOff` names the guards (`secrets`, `approval`, `gate`, `hold`) the session runs without, so
every client says so on it; a `guards {off}` event says them again whenever they change after a
turn. `mode` is `light-coding` (or one of the lab's two), and `modeWhy` says why the session opened in
it, in words for the human. A client that speaks another `protocol` stops rather than misread. Each event is the notification `event/<type>`, its fields beside the
envelope. `session/prompt {text}` answers `{queued}` at once: a line sent while a turn runs
waits its turn, and the session takes lines strictly in the order they came. `gate/ask` sends
`{race, class, act, why, outside, plan?}` (`race` names the asking body's kind: `summoner` for
Mikata's own), and the client answers `{allow, reason?}`. `plan` is
the short form of the plan a command's approval binds to
([the sandbox](03-architecture.md#sandbox)); an ask that spawns nothing has none. A `usage` event
carries the turn's tokens, cache reads and writes, its price, and the run's spend and context
against its stress line, for the status line. A `held` event, sent once the session's queued
lines and their wake are done, carries how many orders the shared work queue holds; the hello
carries the same count as the session opens, and the status line reads "N held" while it is not
zero. A `toolResult` carries at most 16 000 bytes and
says how many it cut; the journal keeps the whole.

**The composition, live (experimental).** `units/list` answers the units the session runs —
`{units: [{id, version, state}], digest, pending, refused}`, where a unit is a native unit's id
(`memory`) or `plugin:<id>` and `digest` fingerprints the composition. `units/change {op, id}`
asks for one change: `op` is `on` or `off`, or `swap` with `with` naming the unit that takes
`id`'s place. It answers `{queued, pending}` at once; the loop applies the change at the unit's
next safe point and the journal records it ([plugins](experimental/plugins.md)); the change is
kept for the world as the terminal's `/unit` keeps it, unless `keep: false` asks it for this session
alone (the board's Configure asks so for a guard switched off). Over the board, any tab may list and
only the tab that controls may change. Bad params
answer `-32602`; before `initialize`, both say no session is open.

`mikatad` serves it: `mikatad [--world <dir>] [--mode <mode>]` speaks on stdin and stdout,
opens the session from the world's configuration on `initialize`, in the mode asked for or the
world's own, and answers `--help` and `--version`
before it looks for any world. Inside, one thread owns the output, so no two lines interleave or
reorder; one thread reads the input and answers the gate's waits itself, so a gate holding a
turn never blocks the read that would release it; and one task owns the session.

The clients reach it through two packages. `@mikata/protocol` is the protocol's types,
generated from the Rust (`ts-rs`) into `src/generated` and checked in: the check regenerates them
and fails when they drift, and a test holds the TypeScript `PROTOCOL` to the core's. `@mikata/sdk`
is the client: `stdio()` starts `mikatad` and speaks on its pipes; a `Client` sends typed
requests, refuses a core that speaks another protocol, delivers every event to its listeners in
order and once each (`resync()` replays from the last `seq` it saw, dropping what it already
had), answers `gate/ask` with the handler the interface gives it — denying when there is none —
and fails whatever waits when the core's end closes, with the reason the core gave. `ws()` (from `@mikata/sdk/browser`, which leaves Node
out) is the board's transport: it reconnects, and the client then takes control back if it had it
and replays from the last `seq`, holding live events until the replay is done.

**The terminal client.** `mikata` opens a session in this world, hosted in its own process,
and draws it in the terminal ([the terminal UI](03-architecture.md#the-terminal-ui)); `mikata
run "<ask>"` asks once without the app, prints the answer on stdout and the calls and the ending
on stderr, and exits 0 only when the body answered and the end gate passed
([`mikata run`](03-architecture.md#mikata-run)). `run` reads its ask from stdin only when none
is on the command line (`run` or `run -`) and stdin is no terminal, and it asks no one: an act that
needs the human is denied, and the denial says so, unless `--gate-tty` puts it to them on the
terminal. `--help` and `--version` answer before any world is looked for; a session that will not
open says why in plain text before the terminal is taken. The app shows the transcript — the text
as it streams, the calls that run with their cards and diffs, the lines waiting their turn, an
approval when one is asked — beside a sidebar of vitals, memory, files and bodies, over the input
and the status line: model, spend (`unpriced` when the price tables do not know the model),
context against the stress line, the cache's share, and what the human's caps leave
([consumption](05-consumption.md)). A tool call shows as soon as it starts, before its input has
streamed. **`mikata status`** prints the build's version, the world, the model and its
provider, where the provider's key comes from (its variable and the file that names it, never the
value), the sandbox actually in force (or `none — <reason>`), what the caps leave, the
configuration's holes (and, in a world of the lab's bodies, each one's meters), and the last
run's acts counted per
[guts class](02-loop.md#stress-in-four-instruments), printed even when no session opens. **Enter** sends, and while a turn runs the line waits its turn; **Alt+Enter** starts a new
line; **`/send <text>`** steers the turn that runs, the text reaching the body after its next tool
step as the human's; **Esc** stops the turn; **Ctrl+C** stops the turn, or quits when nothing
runs; **↑ ↓** walk the history. An approval is chosen with **1** (**y**), **2** (**n**) or **3**
(no, with a reason) and confirmed with **Enter**, so a key typed as the card appears approves
nothing; one whose turn ended unanswered is denied and cleared. The cursor moves by whole
characters, an emoji as one.

Rules:

- Every event carries `{session, run, body, seq, at}`; `seq` rises by one, in the order events reach
  the wire, so a client that reconnects asks `session/events {since: seq}` and loses nothing. The
  core keeps the last 10 000: the answer says the oldest it kept (`oldest`), and `complete: false`
  when the client asked from further back, so a gap is never silent. The board's bridge answers the same
  request in pages, `more: true` while another follows; the core answers all it keeps
  (`more: false`).
- `at` is when the event reached the wire, in milliseconds since 1970 (UTC). A replay keeps it, so
  a client that reconnects still knows how long a step or a turn took; an event read back from a
  run's journal has none.
- In the lab, an event from a body the session hands work to carries that body and its run, at
  any depth; the hello names the session's own body, so a client tells them apart.
- Leaving ends everything at once: on `shutdown`, or when the client's end closes, every act
  waiting at the gate is denied, the turn in flight stops, lines still queued never run, and the
  core exits once its output is written. Esc (`session/interrupt`) denies an act waiting at the
  gate as part of the turn it stops.
- One message per line; a batch is refused by name, and a notification is never answered.
- An unanswered gate never allows. After 10 minutes the act is denied and the body is told the
  human did not answer. Planned, with no milestone: the run checkpoints as `SUSPENDED` instead (a
  neutral word: no strain, no failure) and resumes at that act when the human answers — an
  overnight run waits instead of dying.
- The board carries this protocol over `WS /ws/session/<id>`, a child `mikatad` per chat
  ([the chat bridge](03-architecture.md#the-chat-bridge)): its Host and Origin are checked, the
  cookie traded for the printed key and the CSRF token are required, and the bridge adds
  `board/role`, `board/askGone`, `board/refused`, `board/ended` and `board/oversize`, `leftMs` on a
  `gate/ask` sent again, and `board/takeOver` from a tab.
- Protocol changes are versioned; the generated TypeScript is checked in and CI fails on a diff.
