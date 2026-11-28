# The lab in the code

The main chapters describe Mikata as we test it: one body over your repository (light-coding). The
code also carries what Mikata was as Mnemorph: races, a chain of bodies, a collective, the roster
that says who is who, ascension and evolution, the Dark Elf's review, the Kijin's outward posts and
the Oni's envelopes. They still run, behind `--mode`, and are frozen ([the lab](README.md)). This
page keeps their exact technical detail, moved here from the chapters it used to interrupt; each
part names the chapter it came from.

## The board's lab pages: the roster, the bodies, the colony net, the race cards

> The board no longer draws these pages: the web platform keeps the sessions, the Harness Builder, the runs, the journal and the memory ([the board](../03-architecture.md#the-board)). The server's routes and the data behind them stay, for the lab.

*From [03-architecture.md](../03-architecture.md).*

### The roster's trust

A creature's card (`.mikata/<race>/<name>/README.md`) lives in the world, and nothing read from
inside a world grants trust: a body's commit, a merge or a pull writes a card past every tool
guard, and a cloned repository brings every card from a stranger. So the roster's source of truth
is the creatures' witness under the state directory; the cards are display-only. `Roster::load`
holds each card to its row, and every privilege reads that trusted view — the dispatch law and
the chain, write soil, the outward tool and the post, an Oni's envelopes, the MCP doors by race,
the Commission, Verify and Verdict tools, an Orc's `@V settle` and labels, queue moves, recall
kin, steps and the budget slice by race, Dark Elf routing, meters and rise proposals, relief and
the gate's findings. `Roster::cards` is the cards as they stand, for what only shows them.

- **A card that says what its row holds** runs as it says.
- **A card that differs from its row** runs at the lower of the two: the lower race (two ranks
  of one lineage read as its base, two lineages as a Slime), the ground both name, the row's
  chain of command, retired if either says so — and it loses its rank besides, so no ascended
  form, an Oni's envelopes above all, rides a card the witness does not hold. Each field that
  differs is a `roster-mismatch` finding, high.
- **A card with no row is a stranger**: a Slime of its own seat and scratch, reporting to no one,
  with no ground, no outward act, no MCP door, no settle and no dispatch — and no workspace root
  file, whatever name it bears.
- **A witness that does not walk whole**, or no home to keep one, trusts no row: every creature
  but the throne is a stranger until the human looks (`mikatad housekeep --verify`).

The throne, `summoner`, is implicit: the session's own seat, never a row. No tool writes a card —
the world's records are protected — so a card arrives by the human's hand, by the harness, or by
a route the witness exists to distrust. The harness writes the rows of the births it makes:
relief, for the parent it retires, the young at the parent's base race and the children it hands
on; and an evolution swap the human said yes to — a yes to bytes, so a card a row holds is held to
it as the trusted view holds it (the lower race, the row's chain, the ground and Minds both name:
a swap never raises, re-parents or widens), a new name stays a stranger until adopted, and a
card it removed is retired. Before that yes, `mikatad evolution swap` shows each card the plan
writes — race, post, ground, chain and Minds — against its row. The row holds the creature's
Minds too: a card that names a Mind its row does not hold wears none of it.
Rises, falls and adoptions are the human's alone: `mikatad roster adopt`, `set-race` and
`retire`, each asked at the terminal (`/dev/tty`; piped input is refused) or under `--yes` for a
script the human runs, and `roster rise` and `fall`, which apply the move the compiled law
proposes at the terminal alone ([applying a rise or a fall](races.md#applying-a-rise-or-a-fall)),
each refused inside a body's command, under any `mikata`, `mikatad` or
`mikata-guard` ancestor, and with a state directory inside the world. The ancestry is read
from the process tree, so a command that leaves it — `setsid`, or a daemon re-parented to init —
is caught only where the sandbox refuses that, as its seccomp filter refuses `setsid` under the
kernel guard; with no guard, the body's command is the human's to trust. A fresh world — its
throne alone — asks nothing. A card the human writes by hand, a cloned world, or one from before
the witness opens in the terminal app on one list of its strangers — each card's race, post,
ground and chain — and adopts them all only on the typed word `adopt`; anything else adopts
nothing, and they stay strangers until `mikatad roster adopt`. A witness that does not walk
whole is said there and on `mikata status`'s `roster` line, which also counts the trusted, the
mismatched and the strangers. A card that differs from its row
is never adopted there: it is shown with the way on, `mikatad roster adopt <creature>`.

### The bodies, queue and journal pages

All four read a world in-process with the CLI's own functions, and say where each part was read.
The roster's cards come from the creature docs, copied first as the overview copies (regular files
only, capped) with the Minds under `.mikata/minds/`: a body can write its doc, so a card is
shown and grants nothing. What each creature runs as is the creatures' witness's under the state
directory, read for the world's own path ([the roster's trust](#the-rosters-trust)): each body
says whether it is the throne, trusted, mismatched — the fields its card says otherwise, and the
race it runs as — or a stranger, with the way on; and the rise or fall the compiled law proposes
for it, with its meters, the law's lines and the command that applies it, or its probation
(`ascension`, read from the trusted journals and the witness, says why when none can be read). A Mind's standing — adopted, unadopted, changed, retired, unverified, or named
but absent — is the witness's under the state directory, held to its anchor (the anchor's fallback
in `log.md` read from the file's guarded end): trusted. The race law is compiled in: trusted.

**Bodies.** Each creature carries its race and lane (its base race), the creature its doc says it
reports to, its post (a Kijin's, from its name) or speciality (a Dark Elf's), the paths its doc
claims, its verify lines counted, its Minds — those its doc names, then the shared ones — and its
latest six acts. Its live state reads its latest commission: *working* when the journal has no end,
was written in the last 15 minutes and a session is live in the world (a board chat, or a run lock
held); *cut* when it has no end and nothing runs it, or does not read; *idle* when it ended;
*not commissioned yet*; *retired* by its doc. The throne is live while a session is. The roster's
problems and the creatures whose chain loops are listed.

**The colony net** is drawn from a model kept apart from its drawing (`src/data/colony.ts`):
one lane per base race in the law's order — the throne, Elf, Orc, Slime, Kijin, Dark Elf, an
ascended form in its base race's lane — and one edge per `Reports to`; a body whose doc names no
one and whose race reports to the throne hangs from it, and the throne is drawn for the session when
no doc describes it. An edge that would close a loop, or names no body, is left out and said under
the net, so the drawing is always a forest. Within a lane, bodies sit in their commanders' order,
then by name. Every node carries its words — name, race, post or speciality, live state — as its
label and on its face, never colour alone; it is a button, and opens the body card: the race's
card art, the body's facts, the law of its race, its Minds and its acts.

**Race cards** show every race of the law, born in the world or not: whom it reports to and
commands, its shelf, soil and gate, its steps, budget share and model tier, its headline meter's
bars, an ascended form's rise, probation and fall bars (`mikata_races::law_bars`), and a Kijin's
posts or a Dark Elf's specialities. The art is the cards of `assets/cards/` cut to WebP
(`tools/board-art.sh`, 1200 × 1680, about 110 KB a card), embedded by exact name; the page loads only an `art`
of the plain form `/art/cards/<race>.webp`.

## The board's lab pages: the queue and the timeline

> The board no longer draws these pages: the web platform keeps the sessions, the Harness Builder, the runs, the journal and the memory ([the board](../03-architecture.md#the-board)). The server's routes and the data behind them stay, for the lab.

*From [03-architecture.md](../03-architecture.md).*

**The queue** is `.mikata/shared/queue.jsonl`, copied as a regular file under the cap and folded
by the queue's own fold; an order the witness under the state directory holds exactly so is
*witnessed*, one it does not — a line no harness or human wrote — is marked *not witnessed*.

**The timeline** lists commissions — each body's run journal, `<body>-<hex nanos>-<n>.jsonl`,
its start read from the name, its first ask, its last gate word and its ending (the newest 500
journals, 4 MiB each, 64 MiB in all, each opened only as a regular file) — and the end gate's
verdicts from its chained journal, numbered as a settle cites them. A commission is in-world, a
verdict trusted. Each commission also shows its acts in order, the newest 200, with a chip per
[guts class](../02-loop.md#stress-in-four-instruments), read from the harness's chained guts record
under the state directory while its chain holds — trusted, whatever the journal is; the page says
why when they cannot be shown. It filters by kind and body, and pages.

<img src="../../assets/board/desktop-dark-bodies.png" width="640" alt="The colony net, dark: race lanes, the chain of command, each body labelled with its race and live state; a body card with its race art, law, Minds and acts"> <img src="../../assets/board/phone-dark-queue.png" width="180" alt="The queue at phone width, dark: orders by state, an unwitnessed one marked">

<img src="../../assets/board/desktop-light-races.png" width="640" alt="The race cards, light: each race's art and its law">

<img src="../../assets/board/desktop-light-timeline.png" width="640" alt="The timeline, light: commissions and verdicts, newest first, with their sources">

## The lab's commands

*From [03-architecture.md](../03-architecture.md).*

| You want to | Run |
|---|---|
| have Summoner work alone, in a world with creatures | `mikata --mode light-coding` |
| let Summoner hand work to its Elves | `mikata --mode structured-coding` (the default where an Elf reports to it) |
| work as a collective of equals around the work board | `mikata --mode collective-coding` |
| check the ontology, the queue, incidents | `mikatad onto check`, `mikatad q list`, `mikatad incident list` |
| read every creature's meters, see who is due to evolve, draft its evolution, review the proposals, swap one in, judge it, keep or roll it back, run a tick of evolution on its own | `mikatad meters`, `mikatad evolution due`, `mikatad evolution draft <creature>`, `mikatad evolution list \| show <id> \| apply <id> \| reject <id>`, `mikatad evolution swap <id>`, `mikatad evolution judge <id>`, `mikatad evolution keep \| rollback <id>`, `mikatad evolution auto` |

## The board's lab routes, and the overview's lab rows

> The board no longer draws these pages: the web platform keeps the sessions, the Harness Builder, the runs, the journal and the memory ([the board](../03-architecture.md#the-board)). The server's routes and the data behind them stay, for the lab.

*From [03-architecture.md](../03-architecture.md).*

| Route | Answers |
|---|---|
| `GET /api/w/<id>/bodies` | `BoardBodies`: every creature its docs describe — race, lane, `Reports to`, post or speciality, ground, live state, the Minds it wears with their standing, its latest acts — the roster's problems and loops, the throne's state, and every race of the compiled law with its card's art; each part with its source — see *The bodies, queue and journal pages* below |
| `GET /api/w/<id>/queue` | `BoardQueue`: the shared queue folded as `mikatad q list --all` folds it — the open orders (held, claimed) by rank then arrival, then the closed (done, dropped) — each marked witnessed or not, with the counts |
| `GET /api/w/<id>/timeline` | `BoardTimeline`: commissions (the bodies' run journals) and the gate's verdicts, newest first — `?kind=commission\|verdict`, `?body=`, `?offset=`, `?limit=` (50, at most 500) |
| `GET /art/cards/<race>.webp` | a race's card, embedded by its exact name like the page's other assets (no session needed), whether or not the page was built |

| Overview row | What | Read from | Trusted |
|---|---|---|---|
| `challenges` | the open challenges `pending`, `blocked`, `verifying`; those with an `unstaffed` speciality; their `findings`; `unreadable` record lines — as `mikatad challenge list` reads them | the challenge records under the state directory | yes |
| `meters` | each living creature's meters with their `status` (`counted`, `noReports`, `unavailable` with `why`), `successes`, `n`, the point estimate and the 95% Wilson interval `lo`–`hi`, or `null`s for no report — as `mikatad meters --json` (which also gives each creature its pending `proposal` and `probation`, and the `unknown` input its law reads that no trusted journal carries) | the roster in the world over the challenge records' folds and the report fold of the trusted journals under the state directory ([the report fold](races.md#the-report-fold)) | no |

## The loop in the lab: self-heal

*From [02-loop.md](../02-loop.md).*

### Self-heal

The world repairs itself in small, mechanical steps, each one written to the ledger:

- **Relief** splits a strained body's zone in two young of the same race — by its prefixes,
  or by the directory tree under a one-prefix zone — or, when the zone cannot split, retires the
  seat: it keeps its memory, claims no files, and stays visible. A `relief` row records the act.
  Relief is skipped for a body that just evolved. It relieves Slimes and Orcs: the two young are
  `<name>-a` and `<name>-b`, of the old one's base race, seated beside it, reporting to the same
  parent, with the same verify lines and Minds and a `Heir of` line naming the old one, each
  taking half the claims sorted — a claim
  another covers set aside — or half the entries under a one-directory ground, descending while
  that directory holds one entry (hidden entries go with the first, a link counts as a file). An
  Orc's living Slimes report to the young whose ground covers theirs. The old seat gains a
  `Retired: yes` line, and on a split a `Heirs` line naming its young, and nothing else changes;
  its open queue orders are named, to re-route.
  Every check runs before any write, and a failed write is undone. The
  human can act it with `mikatad relief <name> [--apply]` — without `--apply` it says what it
  would do; never from a body's command. `--around <part>,<part>` splits it around two parts of
  its ground — one to each young, the rest shared out evenly — as evolution's trigger names them
  where its trouble clusters.
- **A clock death leaves a memo**: a court killed by its deadline gets its one salvage turn, then
  its lane goes into the instruments as a `death` row (`.mikata/instruments/deaths.jsonl`, with
  the body named next) and the next-fit lane is suggested on the wire, as a hole — the living
  sibling of the same base race under the same parent, not stuck on its files, with the best
  worth (a tie to the one that spent least), or, with none, a smaller order or a longer clock —
  advice to the parent, never a change on its own. A salvage turn that wrote nothing is still a
  clock death.
- **The roster heals from the docs**: the world fingerprints its body docs, and when the
  fingerprint moves — another session's relief, a hand edit — the roster is rebuilt before the
  next dispatch, so no court is created for a retired seat. A heal never moves trust: a body's
  verify lines change only through its own gate.
- **`mikatad housekeep --heal`** fixes what is mechanical — a body without a doc, a broken
  *Reports to* link — and reports everything else. A seat with no doc gets a retired stub, so its
  memory stays visible and it claims nothing; a *Reports to* that names no living body is
  relinked to the nearest living ancestor of the race that may command it (an Orc for a Slime, an
  Elf for an Orc) — never past the race law; with none above, it is left. Each repair is one journal
  entry; only the field's own line changes, and nothing is written through a link. A doc that
  cannot be read, two seats with one name, a loop in the chain are said and left to judgment.
  Without `--heal` it says what it would fix. It exits 0 when the roster is whole, 1 while anything
  is left or a repair waits for `--heal`, 2 on an error or inside a body's command. With
  `--verify` it walks the chained journals instead and anchors their heads in `log.md`
  ([the chain](../05-consumption.md#the-usage-row)).
  It is also the board's routine: with `--heal` it forgets from the board's list the worlds missing
  longer than `board.prune_after_days` (14; 0 never), and without it names them — the list
  alone, never a directory or a journal ([missing worlds](../03-architecture.md#the-board-server)).
- **One live court per seat**: a second commission to a busy seat waits in its lane.

## The end gate in the lab

*From [02-loop.md](../02-loop.md): the pins' other writers, the gate's checks as a chain of bodies runs them, the Dark Elf's challenge and the stuck rule, as they were written.*

Three writers, none of them a body's command: the human's `mikatad verify pin <creature>`, which
shows the card's lines and asks at the terminal (`/dev/tty`; piped input is refused), or pins
without asking under `--yes` for a script the human runs — a benchmark harness that writes the
card itself; the harness, for the line an Orc codes on its own Slime with the `verify` tool (and
for the withdrawal of the line an Orc coded before for the same `Done when` name, which the new
one replaces; a line the human pinned is never withdrawn); and
relief, which pins each young to the lines its parent's pin held, never to the card's. Every write
is refused inside a body's command (`MIKATA_BODY` is set there), and the sandbox grants no
command the state directory. A body could unset the variable where no sandbox holds it
(`sandbox.allow_unguarded`), so the human's pin — the command and the first-run prompt — is also
refused when any process above it is `mikata`, `mikatad` or `mikata-guard` (read from
`/proc` on Linux, `ps` on macOS; an ancestry that cannot be read refuses). A state directory that
lies inside the world — `XDG_STATE_HOME` pointed there — holds no pin: nothing is written or read
there, and the gate fails closed.

**Each line keeps who pinned it.** A line the human pinned, or relief carried down from one, is
the human's check. A line an Orc coded is the Orc's own word on its Slime: the gate runs it, but
it earns no test bench for the guts classes (no `positive`, no `red-pin`; its failure is a
`fail`), and a pass on an Orc's lines alone is journaled "verified by `<orc>`'s pin", apart from
"the human's pin", so whatever weighs the verdicts can tell them apart. `mikatad verify list`
and the first-run prompt show each pinned line's author.

### The end gate, for every body

Runs on the body's own writes (minus what its Courts already landed on their own account) — a
commissioned body's, and the session's own: Mikata's verify lines are the `Verify:` line of its
card (`.mikata/summoner/summoner/README.md`), read when the session opens and run only as
[pinned](../02-loop.md#the-verify-pins), and in light-coding the whole repository is its soil, so right author holds
by construction. A card with no verify line runs no
check, and then no claim of a light-coding session is ever grounded: `mikatad memory status` and
`onto check` (`no-session-verify`, low) say so and suggest the line — the command the project's
manifest names (Cargo, Python, Node, Go, a Makefile's `test`), picked by the harness and never a
model's words, since the human may paste it — for the human to add. A Cargo project is suggested
whole, `cargo test --workspace`, with no name read from its manifest. For a compiled
project (Cargo, Go, TypeScript) the suggestion also says that a cold build may outrun the limit
on each verify command, and names the key that raises it. The gate runs:

1. **Right author** — every path lies in the writer's own soil ([soil](races.md#soil--who-may-write-what));
   for a Slime's doc the gate diffs by line range: the verify block is the Orc's, the rest is the
   Slime's. The records under `.mikata/` are outside the tree's snapshot: the tripwire refuses
   or reverts a tool's write there, beyond the shared queue, scratch and the advisory mirror.
   Writes are found by snapshotting the tree, whatever tool made them — a shell
   redirect counts the same as the write tool. A path's owner is read with `own` on the roster
   as the commission found it, and the failure names the owner and the route (`` `lib/x.rs` is
   slime-b's (zone lib/), not slime-a's — ask slime-b ``). A critical or high ontology finding
   that touches the landing fails this check, read on the tree as the landing leaves it, so a
   landing that ends a finding is not blocked by it ([ontology](ontology.md#findings)). Writes are
   attributed per body: under a kernel guard a commissioned body's commands write only what it is
   granted — its soil, its build output, its scratch, its plan — so a change outside that, a
   second session's on the same world, is not its to answer for, a failed landing's put-back
   never touches it, and the commission's report names it as another writer's. The grant is the
   one the body ran with, taken before it starts. With no guard, writers cannot be told apart, and every change in the tree is
   judged the body's.
2. **Traits hold** — the verify lines that stood *before* the turn, as pinned, run green, sandboxed, each
   within `gate.verify_secs` (120 s unless the configuration says; 10 to 600, the most the
   `bash` tool grants a command), and within the ask's own clock. A cold build of a large compiled
   project can take longer than 120 s; killed at the limit, it gives no result, so the landing is
   NOT_VERIFIED until the key is raised. The tracked file may set it: the project knows its build,
   and a longer limit runs the same classified, sandboxed command, only for longer. A line changed this turn is a hole and runs from the next turn (a Named Slime's own
   additions excepted). A check never writes: a verify run that changes the tree beyond what
   git ignores — a test run's build output — fails the landing whole, naming what it changed, with
   no second try (a retry would write nothing and pass what the first wrote as the body's work).
   What a check leaves behind goes to `.mikata/report/`, prepared when a session opens: the
   folder, and the common test leftovers (`__pycache__/`, `.pytest_cache/`, `.mypy_cache/`,
   `.ruff_cache/`, and at the root `/.coverage`, `/htmlcov/`, `/coverage/`, `/.nyc_output/`), are
   ignored through git's local exclude file, never committed, so the project's `.gitignore` stays
   the human's. The gate's own view of the tree does not read that file: what the harness ignores
   for checks never hides a body's change.
   **A `Done when` line that was red before the body began refuses no landing.** Such a line is a
   fact about the whole ask, and the work is handed out by zone, so it often needs another
   Slime's part too. Before a commissioned body's first step the gate runs each line an Orc coded
   that carries a `Done when` name, and keeps those that fail. At the landing, one of those that
   fails still is said **open**: the body is told once, so it makes true what its own zone can
   and names what needs another's; its part stands; the journal heads the verdict `PASS, 1 open`;
   and the report's mark for the line is `FAIL`, with the word that it was red before. A line that
   held before and fails after is the landing's own break, and refuses it as ever. So does a line
   with no name (the Orc's check of the Slime's own part) and every line the human pinned. An
   open line is no pass: no proof cites it, and a landing whose every line stands open counts as
   checked by nothing. Held as each Slime's gate, such a line deadlocked the chain: no part could
   land until the whole line passed, every refused part was put back, and so the line never
   passed (in one kept run, one Slime was refused ten times on lines its zone could not make
   true alone).
3. **Duties done** — a commissioned body answered with a non-empty `@S`.
4. **Doc truthful** — the owner's doc moved in the same change.
5. **Tests intact** — no test deleted or skipped. In light-coding, where the human is at hand and may
   have asked for exactly that, a landing that fails on this alone is put to the human on the
   approval gate — allowed, it lands; with no human attached, it fails as ever — and a failure
   on this alone never reconsolidates memory: it says nothing of what the body knew.
6. **No seal** — the territory is not sealed, or this landing is the fix: it carries the frozen
   witness into the zone's tests, unchanged by hash.

A landing on a Dark Elf surface passes as `PASS (challenge pending)`: its challenge opens at the
gate (or, if it cannot be recorded, it reads `PASS (challenge unrecorded)` and waits all the
same), the pass credits nothing, and the throne's harness commissions the routed specialities at the
end of its ask, from the challenge reserve, at most `gate.challenge_reviewers` at once. The
throne's answer says each challenge it moved — pending, verifying, blocked, unstaffed, queued or
cleared — with each finding's verdict, and the landing counts as landed only once its challenge
clears or the human waives it. A reviewer's finding of medium or higher is sent in the same sweep
to a verifier of another speciality, given the record alone
([the challenge](races.md#-dark-elf--the-adversary)).

In the lab the same verdict journal feeds the stuck rule:

A
verdict journal that does not verify trusts no body: dispatch waits for the human. Three fails running for the same
body on the same files harden into the lawful next acts: rewrite the commission, split, escalate.
The journal is the record — the body's last three verdicts all `FAIL` (a failure handed back to
be fixed does not count), and the files every one of them changed — and an order that names one of
those files again — as a word of the order, `./` and a `:line` dropped — is refused with the three
routes, and a held order on the queue naming one waits instead of being claimed; an order rewritten without them goes through,
and one `PASS` clears the streak.

## The council

*From [02-loop.md](../02-loop.md).*

The council's keys are `stress.escalate_at` (5), `stress.relieve_at` (7) and `stress.max_consults`
(3): after three warrants the report is forced to `@S ESCALATE` and reaches the human. Strain is
read from the body's own first `@S`, never from how the loop ended — a court that reports
`@S FAIL` still finishes its loop cleanly. A clean landing clears it; the human's denials and
interrupted or dry runs never add to it. Every relief resets its own trigger, or the next run dies
on its first act. The tally is `.mikata/instruments/council.jsonl`, sat after every commission that answered: a
run whose report did not say `DONE` adds its failed acts and one more — a gate denial is the
human's, never a failed act — and a `DONE` clears it. Relief acts only after a warrant, and a
relief that cannot act escalates instead, the tally kept; re-asks are not counted yet. The warrant reaches the parent as a hole in the child's report —
`@? council: <body> strain <n> — answer as its manager: @INSTRUCT …, @SPLIT …, or @ESCALATE` — the
last warrant forces the report's `@S` to `ESCALATE`, the body's own status kept beside it, and relief's act is a note in the report.

## The lab's storage

*From [03-architecture.md](../03-architecture.md).*

| What | Where | Tracked in git |
|---|---|---|
| shared work queue | `.mikata/shared/queue.jsonl` — append-only snapshots; its witness under `$XDG_STATE_HOME/mikata/queue/` | no — machine-local |
| Kijin post journals | `.mikata/kijin/<post>/post.jsonl` — each outward act's receipt, one row per state, stamped `build: { version, commit }`; an untyped command only by its program and digest; every row witnessed under `$XDG_STATE_HOME/mikata/kijin/<world>-<fnv64a>/<post>.jsonl`, which the tripwire rebuilds it from and the count reads — the witness hash-chained, its head anchored with the other chained journals as `kijin-<post>`, and the journal holding its bytes; beside the witness, `<post>.intents.jsonl` (each act's key, typed request and whole pin), `<post>.incidents.jsonl` (each rolled-back or corrected act's incident and its receipts) and `<post>.lock` (one act of a post at a time) — the folder `0700`, its files `0600` | yes (the witness, intents, incidents and lock: no — machine-local) |
| Dark Elf proving ground | `.mikata/dark-elf/` | yes (frozen witnesses are evidence) |
| Dark Elf challenges | `$XDG_STATE_HOME/mikata/challenges/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — each landing as it was routed with its tree hash and author, how each routed speciality is staffed (commissioned with its spend, queued, unstaffed), each speciality's coverage ledger, its findings of medium or higher with each verifier's staffing, verdict (the verifier, its speciality, the evidence hashes) and seal proposal (its witness frozen by hash), and the human's waiver, append-only and hash-chained ([the chain](../05-consumption.md#the-usage-row)) — each row that settles a finding (found, ruled, decided) stamped `build: { version, commit }`, the build `status` prints, a row from before reading `unknown`; a line that does not parse holds its challenge pending, never skipped — outside the world and out of every command's reach | no — machine-local |
| evolution ledger | `$XDG_STATE_HOME/mikata/evolution/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — each swap with its exact file set hashed before and after, the soil diff the harness saw and the tasks its plan was proven on (the record that keeps a held-out task to one use and out of two splits), each run of a swapped body or of its do-nothing control stamped with the swap's id, the build, the model that rode it and its provider, and the named files' state before and after the run, and each swap's settling — kept or rolled back, by the harness's rule or the human; written by the harness alone, append-only and hash-chained, its head anchored with the other chained journals; outside the world and out of every command's reach | no — machine-local |
| evolution proposals | `$XDG_STATE_HOME/mikata/proposals/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — each plan drawn for the human: its id, body, arm and file set, every file's digest as the world held it when drawn and as staged, and its closing (`applied` or `rejected`); written by the harness alone, hash-chained, its head anchored with the other chained journals; outside the world | no — machine-local |
| strain journal | `$XDG_STATE_HOME/mikata/strain/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — each body's strain, warrants, clears and doom checkpoints, for evolution's trigger; written by the harness alone, hash-chained, its head anchored with the other chained journals; outside the world | no — machine-local |
| report ledger | `$XDG_STATE_HOME/mikata/reports/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — every race's reports but the Dark Elf's, folded from the usage witness, the gate verdicts, the challenge records and the Kijin posts' witnesses alone, each walked and held to its anchor — a journal gone while its anchor stays stops the fold ([the report fold](races.md#the-report-fold)): one row per report, with `ts`, the trusted time it began — `run` (gate word, first try, holes, verify lines green, dispatches landed and re-asked, passes and those overturned), `ask` (a throne landing done or reopened) or `act` (an outward act clean or rolled back) — each with the `build` of the journal row that settled it (`unstamped` where that row carries none), chained from its genesis; a pass counts as verified only on a line the human pinned. Derived: `mikatad meters` rebuilds it whole and puts it in place by a rename when its bytes change, never edited in place, so a replay of the same journals reproduces it byte for byte; a fold whose journal does not hold keeps no new ledger | no — machine-local |
| bodies' witness | `$XDG_STATE_HOME/mikata/roster/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — each body's whole standing as of the row: its name, race (rank included), `reports_to`, post or speciality, ground with its `ground_sha256`, whether retired, the act (`relief`, `evolution`, `adopt`, `set-race`, `retire`, `rise`, `fall`), `by` and `build`, and on a `rise` or `fall` the `proposal` it applied, its `inputs_sha256` and the `as_of` it was read at, which `mikatad roster replay` recomputes from the journals; append-only and hash-chained, anchored as `roster` from the state directory alone. It is the roster's source of truth: every privilege reads a card only as held to its row, and the cards in the world are display-only ([the roster's trust](#the-rosters-trust)) | no — machine-local |
| envelope uses and revocations | `$XDG_STATE_HOME/mikata/oni/uses.jsonl` and `revoked.jsonl` (else `~/.local/state/…`), one pair for every world — each act an Oni envelope admitted, counted under `uses.lock`, and each revocation; append-only and hash-chained, anchored as `uses` and `revocations` with each world's chained journals | no — machine-local |
| dimension manifest | `.mikata/dimension.toml` — the copy of the relations this world states, for readers; what it states beyond them is a proposal, never a relation ([ontology](ontology.md#relations-between-worlds)) | yes |
| ontology facts | `.mikata/ontology/facts.jsonl` — the copy of the trusted facts, for readers; never read for trust ([ontology](ontology.md#facts-who-said-what)) | yes |
| trusted facts | `$XDG_STATE_HOME/mikata/facts/<world>-<fnv64a>.jsonl` (else `~/.local/state/…`) — territory facts, counters, labels, settles and the one import, as events; written by the harness and the human's settle alone, append-only and hash-chained, anchored as `facts`; outside the world | no — machine-local |
| relations | `$XDG_STATE_HOME/mikata/dimension/<world>-<fnv64a>/relations.jsonl` (else `~/.local/state/…`) — each relation the human stated or revoked, and each inverse declared, with `build`; written only by `mikatad onto relate`, `unrelate` and `inverse`, append-only and hash-chained, anchored as `relations`; outside the world | no — machine-local |
| relation sightings | `$XDG_STATE_HOME/mikata/dimension/<world>-<fnv64a>/sightings.jsonl` (else `~/.local/state/…`) — cross-world references the harness saw a body read in a committed manifest, the other world named by id; the ledger relations are learned from, the harness's alone | no — machine-local |

| Name | The journal | Writers |
|---|---|---|
| `value` | the Dark Elf's challenge records: findings, verdicts, seal proposals, waivers | the harness, under the records' lock |
| `evolution` | the evolution ledger | the harness |
| `proposals` | the evolution plans drawn for the human | the harness |
| `strain` | what the council and the doom checkpoint saw of each body | the harness |
| `facts` | the trusted territory facts | the harness (a body's `@U territory`, a parent's labels, an Orc's settle, the one import); the human's `mikatad onto settle` and the board's settle |
| `relations` | the relations this world states, and its inverses | the human's `mikatad onto relate`, `unrelate`, `inverse` |
| `roster` | the bodies' witness: each body's race and rank, chain, post or speciality, and ground | the harness, for relief and an evolution swap the human said yes to; the human's `mikatad roster adopt`, `set-race`, `rise`, `fall`, `retire` and the terminal app's first-run prompt |
| `uses`, `revocations` | the Oni envelopes' uses and revocations, shared by every world | the act runner, under `uses.lock`; `mikatad envelope revoke` |
| `kijin-<post>` | the post's witness; walked once kept, and for good once anchored | the act runner, under the witness's lock |

## The lab's board routes and lifecycles

> The board no longer draws these pages: the web platform keeps the sessions, the Harness Builder, the runs, the journal and the memory ([the board](../03-architecture.md#the-board)). The server's routes and the data behind them stay, for the lab.

*From [03-architecture.md](../03-architecture.md).*

| Route | What |
|---|---|
| `#/w/<world>/bodies/<colony\|races\|queue\|timeline>` | a tab of the bodies page |
| Bodies `#/w/<world>/bodies` | the colony net, race cards, the queue, the timeline | *No body here yet*; *No order here*; *Nothing on the timeline* |
| Colony net `#/net` | the colony as a hex city around the throne, on a canvas in 3D, the same shape in every world: the human, the six races and the five ascended forms on a terraced lattice, the chosen world's living bodies seated around their race, and its Minds as gems, gold once adopted: beside the body that names them, or round the rim when any body may wear them or none does yet (`src/data/net.ts`; a skill is a Mind). Waves run through the ground, signals along who commissions whom; a body at work pulses. A race picked lights the city it reaches and opens its card. Drag or the arrow keys orbit, scroll or `+` and `-` zoom; the races are also a legend and a list in words. Under reduced motion it stands still. Every colour is a theme token, and the dark theme adds light where the light theme lays it on | never empty: the shape is the law's, not a world's |

| Lifecycle | Driven through | Bound | Invariants in every state |
|---|---|---|---|
| Ascension, probation, fall (`mikata-races`) | `rises`, `falls`, `recovers`, per ascended form | fixpoint | the form changes only on the human's word; a reading at the rise bars never reads as falling; recovery needs five reports; no rise proposed in a declined candidate's cooldown; an Oni never reaches probation |
| An Oni's act, its envelope, its incident (`mikata-config`) | `Runner::act`, `undo`, `settle`, `open_incident`, `lesson`, `EnvelopeGate::revoke`, stub forge | depth 5 | each act moves only along the receipt's edges; a correction names an undo seen landed; a closed incident holds its four receipts, read again from the witness; no use past the cap, after a revocation or while suspended; no act on an open incident's target; nothing runs that neither the human nor an envelope let run |
| A Dark Elf challenge and its finding (`mikata-world`) | `Challenges` and its verification | fixpoint | cleared only with every routed ledger, no torn line, every finding ruled and none blocking; a critical is blocked until the human waives it; a waiver is final; a verdict only from the assigned verifier, once; a seal only on a confirmation at high or more; the value ledger's fold agrees |

## The lab's configuration

*From [03-architecture.md](../03-architecture.md): the keys only the lab reads, and the envelopes.*

  `gate.challenge_reviewers`, how many Dark Elf reviewers one landing's challenge commissions at
  once (3; 1 to 7), is the human's alone: fewer reviewers delay a review.
  `collective.on`, which opens every session in collective-coding where a body reports to
  Mikata, is the human's alone: collective-coding lifts `slime-truth` and lets Mikata summon
  any member directly. For one session, `mikata --mode collective-coding` does the same from the command
  line, which is the human's too; no world's file can ask for a mode.
  `evolution.mode`, `auto` (the default: evolution drafts, proves and swaps in a due body on
  trial, then keeps or rolls it back by the rule) or `propose` (every plan and every judgement
  waits for the human), is the human's alone: `auto` lets the harness swap without asking.

  `evolution.min_net_wins`, how many more paired tasks an evolution must win than lose against
  doing nothing to be kept ([judged after the freeze](races.md#evolution--the-other-way-to-change);
  2, at least 1), is the human's alone too: a lower bar keeps more.

An envelope is judged at its worst: a word that is a bare `*` stands for
  any arguments, so it is tried as each destructive one (`--force`, `--delete`, `+ref`,
  `delete`, `publish`, …); a `*` inside a word (`feat/*`) cannot become a flag.
- **Envelopes are bounded.** Each names the day it was `signed` (a bare TOML date) and lasts
  `oni.envelope_days` — 30 unless the human's own file says, from 1 to 365 — unless its own
  `expires` is sooner; never on or before the signing, never past that limit. Raising the limit
  loosens the gate, so the tracked file cannot set it. A revoked envelope covers nothing, so a
  limit lowered after it was signed does not judge it; a live one past the new limit is an error
  that says to sign it again, revoke it or delete its record. Days are UTC calendar days. The gate
  reads envelopes from the global file alone: a local file's are ignored, and
  `mikatad envelope list` says so. Each names the `world` (absolute) or the `repo`
  (`owner/name`) it is signed for, or both — one naming neither admits nothing — and may name a
  `post` and a `body`. `uses` caps the acts an envelope covers in
  all, at least one; no two envelopes share a name. `Envelope::admits(today,
  used)` is the check the gate makes at the moment of use ([races](races.md#ascension--earned-granted-revocable)),
  with the count of acts the envelope already admitted, kept in the uses ledger
  (`mikata_ledger::Uses`, `$XDG_STATE_HOME/mikata/oni/uses.jsonl`, outside every world and
  counted under its lock); it names the lapse that stops one: revoked, not yet in force, expired,
  or spent. The gate reads the envelopes afresh at each use, and only from a global file that is
  the human's own: one git tracks in the world, a link, another user's, or one that it or its
  folder lets others write is not honoured.
  Revoking (`mikatad envelope revoke <name>`) writes `revoked = <day>` into the
  human's file that holds the envelope, through `toml_edit` like any write, and only once: the
  envelope stays on record and covers nothing again; the revocation is journaled beside the
  uses (`revoked.jsonl`). Both journals are hash-chained and anchored with each world's chained
  journals. An envelope signed ahead may be revoked
  before its signing day. The dates are the ledger's `Day`, the one calendar in the
  workspace, so config reads the ledger crate.

- **Models in the lab**: `models.creature.<name>`, else `models.office.<race>.<office>`, else `models.race.<race>` (the race's model at its default tier), else `models.tier.<the race's tier>`.

The lab's keys in a global file:

```toml
[models.race]
dark-elf = "anthropic/claude-opus-5.5"    # …or one per race

[models.office.orc]
judge = "vllm:qwen3-coder"         # …or per race and office, through another backend

[oni]
envelope_days = 30                 # the longest an envelope lasts (1–365); 30 when unset

[[oni.envelopes]]
name = "feature branches"
world = "/home/me/src/name"
repo = "owner/name"
allow = ["git push origin feat/*", "gh pr comment * --repo owner/name --body *"]
signed = 2026-10-24                # expires 2026-11-23, unless `expires` says sooner
uses = 50                          # optional: at most 50 acts in all
```

## The sandbox for a chain of bodies

*From [03-architecture.md](../03-architecture.md).*

 A commissioned body's commands are granted
    exactly what its soil admits, and the rest of the world is read-only to them: landlock can
    allow a folder but never except a spot inside it, so a claim that holds another's spot (a
    seat, the queue, another's scratch) is granted part by part around that spot, existing
    entries each on its own. A claim reached through a link is no claim. Build output is granted
    besides the soil: what git ignores where `own` names the body or nobody — never another's
    ignored files, the records (`.mikata/`) or git's own files, and never through a link. The
    guard resolves every rule to the file it names, so every grant is resolved first and kept
    only if it still resolves inside the world to the body's own. Past a few hundred rules
    they reach the guard in an owner-only file it reads and removes before restricting itself.
    The temporary directory is the body's own scratch nest, or, when that nest is a link, a
    fresh owner-only directory outside the world for that body alone. Below landlock ABI 3,
    which cannot refuse `truncate` on a path, seccomp refuses the call.

    What this costs a commissioned body, by design: in a folder that holds another's spot it
    cannot create, delete or rename its own files, so `sed -i`, editors' atomic saves and
    formatters that replace a file fail there — the file tools do that work; a build
    whose output directory or root lockfile does not exist yet cannot create it; `.git` is not
    its to write (landing is the harness's); a claimed folder that does not exist yet gets no
    grant until it is made. Landlock never governs a file's mode or times, so the end gate's
    snapshot records each file's mode and inode change time, which no process can set back:
    a `chmod`, a `touch` or a rewrite dressed in its old mtime counts as a change and is judged.

## Carving the budget down a chain

*From [05-consumption.md](../05-consumption.md).*

- **Only cost is carved.** Steps and the wall clock pass down whole, per race
  ([the loop](../02-loop.md#budgets-are-readings)); splitting them would leave the Slime — the only
  body that writes code — a handful of steps.
- **Carving follows the plan.** A parent keeps a 10% reserve of its own envelope; splitting the
  rest across its plan's open items by their declared weight comes once plans carry weights. Each race has a
  compiled **maximum** share of what the parent has left when it carves (Elf 60%, Orc 60%,
  Slime 70%, Kijin 30%, Dark Elf 40%) — a ceiling, never a default — and the slices together
  never exceed what the parent holds.
- **Who gives.** Every slice comes out of the giver's own envelope: what a parent gives a child it
  can no longer spend itself, and what the child spends counts as the parent's. Work no body owns
  — an order the wake runs, one the human added with `mikatad q` — draws from the session's
  shared pool, the envelope itself, under the same clamps.
- **The grant is the giver's judgment, inside the harness's bounds.** The parent proposes its
  child's slice (`budget_wt`, default half of what it has left, within the ceiling); the harness
  grants at most
  `left × 90% × share(race) × worth` — the parent's reserve kept, the race's maximum share, the
  child's worth — and the child's report tells the parent when it granted less than asked. Whole,
  the slice is `max(min(asked, ceiling), min(20 000, left × 90%))`: never below 20 000 wt, about
  one call, so a floor outranks worth but never the parent's reserve. An ask under it is raised,
  and the report says so; an ask of 0 stays 0, and is refused. With no cap set there is no envelope to
  carve: a commission's `budget_wt` is not read, and the child's slice is unbounded.
- **The top-up.** A parent may also set aside a top-up out of its own envelope (`topup_wt`): its
  further trust, so slice and top-up together stay within what it may give at all (90% of what it
  has left) and within twice the child's ceiling — a parent can stretch its trust, never ignore
  worth. It is given once, when the child's slice runs out, and the child is told; it finishes
  instead of stopping at its budget checkpoint, and the human's caps are read all the same. With
  no cap set there is no slice to run out, and no top-up. What the child spends of it counts as the
  parent's, like the slice.
- **Worth, from the child's card.** Before it decides, a dispatching body reads one card per body
  below it: that body's record and the record of every body below it — weighted tokens spent
  (the usage ledger, rounded to the thousand), gate passes and fails (only the entries the end
  gate wrote in the journal; a `NOT_VERIFIED` counts as a fail, so not verifying never buys
  worth; one the machine would not run, `NOT_VERIFIED, unrun`, counts neither way). Worth is `0.5 + 0.5 × (passes + 1) / (passes + fails + handed
  back / 2 + 2)`: a failure handed back to be fixed counts half; a body with no record starts at
  0.75, a clean record nears 1, a failing one nears 0.5. **A slice that runs out is never a failure
  on the card:** a budget checkpoint ends the run before its body answers, and only an answer reaches
  the end gate, which alone writes the verdicts a card is read from. Counting it would let a thinly
  funded body fail for want of budget, lose worth, get a thinner slice and fail again; in
  [the research's simulation](https://github.com/Kaginari/mikata-research/blob/main/experience-value.md)
  that loop cost 2 to 10 landing points. A test holds the rule. A child's record counts in its parent's card, so a
  parent that funds failing children reads worse to the body above it. The learned value model
  ([M11](../roadmap.md)) replaces this score; the card and the clamp stay.
- **The wake's draw.** One wake draws at most 25% of what the pool has left, and runs at most 3
  orders (a refused one does not count); with no cap set there is no pool to share, and the order count alone holds.
- **The challenge reserve.** 10% of the session envelope is reserved for the automatic Dark Elf
  challenges; it cannot be carved for anything else. Each challenge draws a quarter of the
  reserve's full size (or what remains), and what it does not spend goes back to the reserve.
  Once it is spent, a routed reviewer waits queued for a later sweep; what each review spent is
  booked against its challenge.
- **Pre-flight**, for a carved slice (dispatch, M7). Before each model call the core estimates
  the call in wt (`estimate`: cached
  prefix at the cache-read weight, the rest uncached, `max_output` at `r`). If it does not fit
  what is left, the loop checkpoints with `@? budget: need ~N wt, have M wt`. The parent can top
  up the child's slice from its own envelope (`@INSTRUCT` with a new budget), split the work, or
  escalate to the human.

- Unspent slices return to the parent when the child reports; what the child spent counts as the
  parent's spend. A slice settles once, and only with the envelope that carved it — a replayed or
  stray report is refused and leaves the books alone. Envelopes and open slices serialise whole, so
  a checkpointed run resumes with the same books.
- Once dispatch carves (M7), a spent cap also stops new commissions while the turn in flight
  finishes its report.

## MCP for the lab's bodies

*From [06-mcp.md](../06-mcp.md).*

### Scoped by race and class, for every body

A body sees a tool only when both hold:

- its server names the body's race in `races` (an ascended form through its base: `kijin`
  serves an Oni);
- the tool's class is on the race's shelf. An `outward` or `destructive` tool reaches only Kijin
  and Oni, whatever `races` says; a `read` or `write` one reaches any race named. Selection
  never widens a shelf.

Before a catalog is known, a server is shown to a race only when some class its tools could be
given — declared, inferable or the default — is on the race's shelf. A tool off the shelf is
never rendered: not at any level, not in a search, not in the ranking, and a door asked for it
by name answers that no such tool is in scope.

Each race's cached prefix has its own ceiling: 4,000 tokens for the throne and a Slime, 5,000 for an Orc or a Dark Elf, 6,000 for an Elf, 8,000 for a Kijin, an ascended form as its base.

## Memory's accepted route

*From [07-memory.md](../07-memory.md): a claim grounded by a commissioner's word.*

**Accepted** is the commissioner's word, and it names the claim. A commission's report offers
each claim its body left proposed, by id, as a hole — never cut, however long the report — and the
commissioner checks what it can (reading the source, asking a second body) and accepts a claim with
an `@V accept <id>` line in its own answer. When that answer ends the ask, each id grounds its claim
if this ask's orders offered it and its body still holds it proposed, citing the acceptance. Only
this ask's offers count: a body never accepts its own words, an order that was refused offers
nothing, and the orders a wake or an escalation runs offer none. An id naming nothing it may accept
is said and grounds nothing; a claim it does not accept stays personal and proposed.

## Lessons the lab carries: territory, gate, dispatch

*From [lessons.md](../lessons.md).*

## World: territory, gate, dispatch

| Lesson | Becomes | Milestone |
|---|---|---|
| A body could rewrite the verify line it was judged by, because the gate reloaded the doc after the turn; checks a model runs can be skipped or weakened by the turn they judge. | The orc's checks are code the harness runs at the end of the authoring turn, from a verify set captured at turn open; a line added in the same turn is a hole, not a reading. Test: a rewritten verify fails the gate. | M7 |
| The turn-watch charged every write in the open window to the session whose turn was open, so sibling bodies failed each other's gates. | Write attribution is per body (per sandbox, or file events tagged by process). Test: two concurrent bodies writing distinct zones each pass their own gate. | M7 |
| "Tests intact" measured against git HEAD would count the human's own uncommitted edits. | The baseline for any before/after check is the harness's own snapshot at turn open. Test: lost tests and new skips fail; new tests pass. | M7 |
| A machine-wide agent file named like the throne was imported as a foreign body, so every session ran as an elf; separately, the throne carried an office whose model route could re-route the session. | The throne's name is reserved and discovery never imports under it; the throne holds no office. Tests run with a populated fixture HOME, not an empty one. | M7 |
| A body reporting `@S FAIL` ran its lifecycle to `DONE` and judges read the lifecycle; a body that wrote nothing never met the gate, so no verdict reached the log. | The verdict is a distinct type parsed from the report's wire, never the lifecycle status; every dispatch lands exactly one log row, written or not. | M7 |
| 42% of one session's tokens were higher ranks relaying a lower rank's text word for word. | Reports above a size floor are filed on disk; the parent receives an envelope (verdict, path, checksum, wrote set, short digest). Test: envelope rides, the full text is filed and intact. | M7 |
| A wire report ran 70% over its byte cap. | `@CAP` is enforced by the encoder, cutting the least important lines first and never `@S`, `@?` or `@U`. Test: oversize report keeps those lines. | M1 |
| A body whose Territory listed the world directory itself (`.isekai`) claimed every seat, the shared queue and scratch, and isekai's two owner functions broke ties differently (authoring rank first in `world.Owner`, longest prefix first in `onto owner`), so the gate and the owner query could disagree. | Ownership is one function with a fixed order — harness soil, scratch, seats, compiled soils, the longest Slime zone, the longest Orc domain — that every caller uses ([the ontology](ontology.md#ownership-one-function)). Test: a body claiming `.mikata` owns none of the seats, the queue or scratch. | M7 |
| Bodies re-derived "who owns this file" from scratch at every boundary hit and burned a day on it. | An out-of-territory refusal names the owner and carries the three arms (widen, birth, route); one dispatch-card read gives owner, rank, strain and domain. | M7 |
| Territory as a file list left new files ownerless; one wide flat directory made a single keeper a bottleneck that could only retire, not split; a retired seat kept claiming soil its successors overlapped. | Territories are directory prefixes; founding chunks wide flat directories into bounded zones; a one-prefix zone splits by its own subtree; a retired body derives no territory. Test: ontology check clean after split, retire and ascension. | M7 |
| A disciplinary body reverted and deleted files it had not authored. | No body deletes or reverts what it did not write; delete moves to an attic with a ledger line. Test: a foreign untracked file survives a cleanup turn. | M7 |
| Background reports from mid-chain bodies were stranded because the wake channel fed only the session. | A report lands in its own parent's turn; depth is tested from the throne down to a slime. | M7 |
| A nested dispatch's end never reached its card, leaving zombie "thinking" cards with a clock that kept running. | Every dispatch start emits exactly one terminal event, at any nesting depth; a done card's elapsed is frozen. Test: three-deep nesting closes every card. | M7 |
| A rank with no reportees could still be handed the dispatch tool by its tools list. | Dispatch is granted by the race's compiled powers, not by config. | M1 |
| Ascension counts must survive restarts and must never count a body's own `@S FAIL` as a pass. | The ascension tally is an append-only ledger replayed at start. Test: FAIL reports never raise the pass rate. | M11 |
| Mid-turn steering and the human's queued lines shared one inbox, so a queued line rode the running turn and broke its task. | Two lanes in the core: `send` reaches a running body at its next tool step, `queue` only at the turn's end. Test: a queued line never appears in the running turn's context. | M5 |
| `@$` orders lived in one process's memory: a restart, a second session or the terminal (which never drained them) lost them, and nobody could see what was waiting. | The shared work queue on disk holds every order (relay-hold) until the wake claims it; the status line shows what is held. Test: a restarted session dispatches the orders the last one left. | M7 |
| Two sessions could claim the same order. | Every move is a revisioned snapshot line; the fold seats only the line that follows the item's last revision, the other reads back as a typed refusal. Test: two concurrent claimants, one claim. | M7 |
| A move on shared bookkeeping during a body's turn was billed to that body's writes. | The shared shelf is harness soil: no body writes it by hand, the tree watch never stamps it. Test: a queue move mid-turn leaves the body's written set empty. | M7 |
| Another session's relief left the roster stale, and it kept creating courts for retired seats. | The roster is rebuilt when the body docs' fingerprint moves, before each dispatch; a heal never changes a body's verify lines. Test: a doc edited behind the world's back is seen at the next dispatch. | M7 |
| Two commissions to one seat ran two courts on the same zone at once. | One live court per seat; a second commission waits in the seat's lane. | M7 |
