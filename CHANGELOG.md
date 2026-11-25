# Changelog

Every change that reaches users is written here, newest first. Changes merged since the last
release sit under `## [Unreleased]`; cutting a release renames that header to the version and its
date (`## [1.0.0] - 2026-11-02`) and opens a new, empty `## [Unreleased]` above it. The release
workflow drafts the GitHub release's notes from the tagged version's section, and refuses a tag
whose section is missing or undated ([releasing](docs/releasing.md)).

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Security

- **The URL library under the HTTP client is updated past an advisory published on 22 November
  2026** (`url` 2.5.8 to 2.5.9, RUSTSEC-2026-0592): a `blob:` URL nested deep enough overflowed
  the stack in `Url::origin()`. Mikata's own code does not call that function; the update is
  the one the advisory names.

### Added

- **A `/tmp` of the command's own.** On Linux, where the kernel allows it, each sandboxed
  command gets its own `/tmp`: a folder of the project's scratch (`.mikata/tmp/slash-tmp/`)
  mounted there for that command alone. A scratch file kept in `/tmp` is now a plain write that
  asks nobody, and the machine's shared `/tmp` stays out of reach. `mikata status` says
  `private /tmp` where it holds. With Claude Haiku 5.5 it removed the approval gate's commonest
  denial (100 of 115 in 190 runs) and took 12% fewer calls in the bare loop.

- **Experimental: every part of the loop is a unit, and plugins load while a session runs.**
  Memory, the end gate, the held `Done when` list, the fold, the stress signals, what-changed,
  the reader and the challenge switch on and off (`[units]` in your global file, `--units` on
  `mikata run`, `/units`, `/unit on|off <id>` in the terminal UI, `units/list` and `units/change`
  over the server); a world's file may only switch off what loosens nothing. A plugin is a
  program you declare in `[plugins.<id>]`: it speaks MCP plus `mikata/hook`, runs in the body's
  sandbox with a deadline on every hook and three strikes before it is taken out, and is loaded,
  unloaded or swapped mid-session (`/plugin load|unload|swap`). Examples and a one-file Python
  SDK are in `examples/plugins/`, among them a memory that stands in for the native one. The
  prompt never changes while a session runs; the journal records every change of units, and a
  resume rebuilds them. See `docs/experimental/plugins.md`.
- **Profiles: `mikata run --profile small-fix`.** A named mix of units for a kind of work;
  `[profiles.<name>] units = "…"` in your global file names more. `small-fix` leaves out the
  prompt's plan (write a `Done when` list and run each line before answering): with Claude Haiku
  5.5 it took three quarters of the calls and two thirds of the dollars on small fixes and prose
  asks, 41 passes of 42 against 42; with qwen on small fixes it changed nothing measured. The
  default keeps the plan.
- **Your own command for your case.** `mikatad memory wrong "<words>" --check "<command>"` (and
  `/wrong <words> --check <command>`) keeps the command that shows your case with the correction.
  When the ask comes again the harness hands it in as that line's check and refuses a check of the
  model's for it: a model's check could run your case the other way round and print what your
  line states. Without one, a model's check for your line that prints what you want before any
  work is questioned at once.
- **Say an answer was wrong from inside the session.** `/wrong <what is wrong>` in the terminal
  UI shows, before anything is kept, whether your words will be held as a line of that ask's
  `Done when` list or kept as advice, and under which ask. `/keep` keeps it, `/drop` keeps
  nothing, `/withdraw` takes back the last one kept. It is the same act as
  `mikatad memory wrong`, now one function for both.
- **Say when an answer was wrong, and the next session is told first.** `mikatad memory wrong
  "<what still fails, in your words>"` writes your words to the journal and keeps them as a
  correction in shared memory, under the ask of the journal's last entry (or the one `--ask`
  names by its first line). A session that meets that ask again is shown the correction ahead of
  what the earlier session believed, with that belief marked `corrected`. Nothing is deleted.
  It is the human's alone: a body's command is refused.
- **What you say must come out is held.** A correction that says so in the words a `Done when`
  list is read by, ``mikatad memory wrong "`total 1 2` must give `6`; it gives `3`"``, is now a
  line of that ask's list whenever the ask comes again: after the ask's own lines or a
  reader's, in the order you said them, the newest twelve. The body is told the lines are yours,
  that they are not its to rewrite, and that they are cases of the ask and not the whole of it;
  it hands in a check for each, and an answer that leaves one wrong does not land. The line is
  your words up to the output they state. Words that state no output stay advice, as before,
  and `memory wrong` now says which it was. On our bench a correction that was only told was
  taken up by every run and then overruled by five runs of six where the fix was hard; held,
  the case came out right in eight runs of eight, and with memory keeping three such words
  seven of eight asks the small model fails came to pass.
- **Take a correction back.** `mikatad memory withdraw <note>` puts a correction of yours to
  sleep: it is told and held no more, the journal keeps the act, and nothing is deleted.
- **A session that stops leaves an account.** When a session spends its steps, its clock, its
  context or its budget before it answers, the journal now gets an entry (what was asked, why it
  stopped, which files it had changed, how to resume) and memory a note of the stop under that
  ask; the session's last words are kept as a handoff. Before, such a session left the journal
  and memory as it found them: on our bench one task stopped at its step limit two rounds
  running, 200 calls each, and each time the next session started it from nothing. When the ask
  lands at last, the stop's notes go out of recall.

### Changed

- **The terminal UI is laid out as a dashboard.** A header bar names the repo, the branch, the
  mode, the model and whether commands are sandboxed; the conversation sits in an *Agent loop*
  panel; beside it, *Done when* ticks the body's list once the end gate passes the answer,
  *Budgets* shows steps, wall clock, context against its stress line and cost, *Checks* shows
  each check the end gate holds (each verify command, tests intact, duties done) as ok, failed
  or no result, from the gate's verdict or the body's own last run, *Guardrails* says what the
  sandbox and the gates hold the session to, *Units* lists every unit with an ON or OFF pill,
  and *Session* keeps memory, files and bodies (with Mikata's logo where the terminal draws
  pictures and the panel has room). From 140 columns, Checks, Guardrails and Units stand in a
  column of their own on the left. The approval card shows the act boxed as a command line
  under its reason, and its answers as buttons. **F2** (or `/view center|focus|zen`) switches,
  while the session runs, between this dashboard, *Focus* (the conversation with its bars, no
  panels) and *Zen* (the conversation and the input alone). The default `neon` scheme takes a
  new palette: hot pink `#ff168a` through coral to amber `#ffb020` on a near-black plum
  (`#0d0b10`, `#18121d`, borders `#382638`), success `#a3ff56`, error `#ff4757`, text
  `#f4eef5`; the board's `neon` scheme follows it. A footer bar holds the verify line, the last answer's
  word, the keys that matter now, and the steps, time and spend. Borders run in one pink-to-gold
  gradient; cards are rounded boxes, no longer shaded slabs. Small terminals keep one-row bars
  and drop the frame round the conversation; a short column gives up panels with nothing in
  them.

- **Fewer calls lost to the harness.** `multiedit` is no longer offered (its nested arguments
  came back as broken JSON in one call in eight); several edits are several `edit` calls in one
  message, and a session that used it is still answered. A command the approval gate refuses is
  told why and where scratch files belong. The prompt says one message may carry several tool
  calls, and says only what the session's units hold.

- **Every entry the harness writes for a session says what it was asked** (`- **Asked:**`, the
  ask's first line), so the journal reads as what happened to which ask.
- **Where quoted memory ends and your ask begins is marked in words** whenever notes, facts or
  journal entries stand above an ask. An ask with nothing above it is sent as you wrote it.
- **A correction is said to be one.** When a recalled note is a correction, the line over the
  notes says what that means and that it is not to be done that way again.

### Fixed

- **A check that cannot run is no longer called a failure of your work.** A check whose command
  could not load what it names (a program that is not installed, a module or file that is not
  there) was told "fails on your work" at the gate, in the same words as a wrong value. One run
  with right code was sent back four times and ended as failed. The body is now told at hand-in
  that such a command stops before it reaches the work, and the gate says it `cannot run here`
  and shows nothing of the work when it fails as it did then. The line still does not land. A
  command that fails any other way, unchanged by the work, is said to be unmoved by it.
- **A stopped session's account names files as the world does.** A body that wrote a file by
  the world's whole path had that path listed as changed, its own plan under `.mikata/` among
  them. Paths are now said relative to the world, and the plan is left out whichever way it was
  named.
- **A first failure is remembered.** When the end gate failed twice, what failed was written to
  memory only as a correction of the notes that ask had recalled. With none recalled, nothing
  was written: the first failure on any task left no trace, and the same ask met the same wall.
  It is now kept under the ask, and said first when the ask comes again.
- **A task that comes again is shown what was learned on it.** Recall matched a note by its
  own words, and a lesson is written in the words of what was found, not of what was asked. In
  one codebase a long ask shares its words with half the memory, so other tasks' notes took the
  seats: carried through ten tasks three times over, 18 of the 20 runs that met a task again
  were shown a note of their own, and of about six notes shown 1.3 were their own task's. A
  lesson is now kept with the first line of the ask it was learned under and matched with those
  words too: 20 of 20, and 2.8 of the six. Measured with the product's own recall on that
  campaign's 88 notes, which a test now holds. It is not shown to make a run cheaper: on that
  campaign a body that passes every task cold passed every task with memory, in more calls.
- **What a session learned is kept when it is written as a list.** At the end of an answer
  Mikata writes down what it learned that is written nowhere, one line each, and the harness
  keeps those lines as notes for later sessions. It read such a line only where it opened a line
  of the answer. A model answering in markdown writes them as a list, inside a code span or in
  bold, and each of those was dropped without a word. Counted over 284 kept runs of our bench:
  one model lost 80 of its 440 lessons this way and the other 97 of 108. Of that model's 36 runs
  that wrote a lesson, 32 left an empty memory behind them. A lesson is now read behind a list
  mark, a number, a heading or a quote mark, and out of the code span or the bold that opens on
  it. Nothing else is read that way: a status or a verdict still counts only where it opens a
  line, and a marked line is a lesson only when it names its kind, so a heading that reads
  `@U lines` stays a heading. A colon after the kind (`@U colony: ...`) is no part of the fact.
- **In a JavaScript project, the harness's own copies of your code now find what is installed.**
  "What changed" runs Mikata's checks again on two copies of the world, and a line of your list
  that must stay, handed in after the code changed, is read from a copy as well. The copies left
  `node_modules` out, so a check that imports a package failed there whatever the code did. Two
  things followed, both wrong: every such check was counted as "fails on the finished work" and
  nothing was shown; and a late `must stay` line was held to that failure, so a landing whose
  lines held was failed. A copy is now lent the world's `node_modules`: a folder of links its
  commands may read and never write. Found on the first run of our bench on a second codebase
  (mathjs): of the 70 checks of ten runs, 8 ran in the copies and none could be compared. With
  the fix 61 run, and both runs that failed their tests show a wrong value in the list.
- **A check that fails is told how, whichever pipe the error came down and whatever the runtime
  prints after it.** The harness took a failure's last line of standard error for the error. A
  check written with `2>&1` has none there, so Mikata was told "it said nothing" of a command
  that had printed why. And Node ends every uncaught error on its own version, so every failure
  of a Node check read the same. Both also made a `must stay` line held to a failure hold for
  any failure at all. The error is now the last line that begins with an error's name or tells
  of a panic, found in the output too when the command failed. Seen in a run that fixed the
  code, could not make its own checks run, and spent its steps on them.
- **A check piped into `tail` or `grep` is judged by its command, not by the pipe.** A pipeline
  ends as its last stage does, so a check whose command raised still "ran", and a line that
  must stay was held hard to the last line of an error message. A reader had written such a
  line for the very input an ask was about; the fix was right, the line "broke", and the
  landing failed twice. The harness now reads the first stage's status from the shell, as its
  stall detector already did: that check failed on the code as you gave it, so the line is held
  as a failure is, and work that makes the command run is said once and refuses nothing.
- **A session stopped between `git stash` and its pop no longer ends with its work out of the
  tree.** Mikata often puts its edits aside for a moment to see what the code as you gave it
  prints: `git stash && <a probe> && git stash pop`. A checkpoint that fell in between (steps,
  clock, budget or context spent, or a stall) took the tools away, and nothing popped the stash
  or said it was there: the files read as you gave them, the work sat in `git stash`, and the
  last words said what had been changed. A checkpoint now puts that stash back, before the last
  words are asked, with `git stash pop --index` run under the same guard as Mikata's own
  commands, and only while nothing has moved since the stash: the tree and the index clean, no
  commit made. Otherwise the stash is left, and named with the way to bring it back: on your
  screen, in the journal, in the stop's account, and to the body, in its last turn's note or
  in the conversation for the ask that goes on. A stash that was there when the ask began is
  never touched. Counted over 490 kept runs of our bench: 146 put their edits aside this way,
  and one was stopped in between. It ended with 103 lines in the stash and was scored a
  failure on a tree it had not changed.

## [1.1.0] - 2026-11-21

Say what done means, and the harness holds it: a `## Done when` list in your ask, yours or one a
reader you name writes from the ask alone. And when an answer lands, you are shown what changed,
by Mikata's own checks.

**What to expect, as measured on our own bench** (fixes in a large Python library, one small open
model). Asked as one paragraph of prose with no examples, four fixes passed 6 times in 12; with a
reader named, 12 in 12, at the same price a run. Six of those twelve runs are of the build before
a fix the screen itself called for, which this release holds
([the bench](docs/bench.md#the-list-held)). Read the list a reader writes: a wrong line is
obeyed, and a body can still answer a line with a command that only prints the expected words,
which the pass names and does not refuse. Against the two other harnesses run beside it, each at
its version of the day, Mikata is level on passes at about half their price a pass: the 1.0.0
notes said ahead, and by the bar we wrote first no difference is shown.

**Measured again on the release itself, the same day.** The published archive, the same twelve,
every run counted: 10 of 12. Three of the four asks pass every time; the fourth passed 1 time in
3, and fails when the reader writes a unit in small letters where the code prints capitals. No
pass of that screen was held by a check that only prints the words
([the bench](docs/bench.md#the-list-held)).

**Measured that evening: the same lists, handed to every harness.** The twelve lists the reader
wrote for that screen, given as a plain note after the ask and held by nothing: Mikata 11 of 12,
and 8 of 12 in each of the two other harnesses run beside it, where those passed 5 and 3
without. So the list is the gain, and holding it is not shown to add a pass: held, the same
lists passed 10, for about four dollars in ten more a run. What the hold gives is a pass that
names what verified it ([the bench](docs/bench.md#the-list-held)).

**Measured that night: a second codebase and a second body.** On ten fixes in a JavaScript
library the three harnesses are level again (Mikata 8, Maki 7, OpenCode 6), and Mikata's pass
cost half of OpenCode's and three times Maki's. With a stronger model (Claude Haiku 5.5) all
three pass all ten sqlglot fixes, and Mikata's pass costs what OpenCode's does and twice Maki's.
So "about half their price a pass", above, is a count on one codebase with one small model, not
a property of the harness. With that stronger model the four prose asks pass 12 times in 12
with no reader ([the bench](docs/bench.md#a-second-codebase)).

### Added

- **A `Done when` list in your ask is held by the harness.** Say what done means in lines of your
  own under `## Done when` (`D1: ... must give ...`, `D2: ... must stay as it is today`). Mikata
  holds a new `check` tool and hands the harness one command a line. A line that must give is
  held to an output the line states, letter case and blanks set aside; the check may print the
  whole statement around it. A line that must stay states none: the harness runs its command on
  the code as your ask found it, tells Mikata what it does today, and holds the work to that. A
  write to the code waits, twice at most, until every line has its check. A check or a write the
  harness sends back with how to make it is a correction, and never counts against the body as
  a failed act. When the answer lands every check is run on the work: a line with no check, or
  one that prints something else, fails the landing, and the note names the line and both
  outputs. A line whose command failed before and no longer does is said once and refuses
  nothing. A landing the list passed says so (`verified by the ask's Done when list (11 lines)`)
  and names every line whose check already gave its output before the work (`D1 to D4 already
  gave their output on the code you gave it`): a command that only prints the words holds
  whatever the work is, and you are told which lines showed nothing. With no list in the ask,
  nothing changes.
- **A reader can write that list: `models.reader`.** Name a model in your own configuration and
  every light-coding ask that carries no list is read apart first: one call, shown the ask
  alone, never the code. Its lines go after your ask and are held like your own; you see one
  line with what it cost. Yours alone to set: a repository's tracked file cannot name a second
  model you pay for on every ask. Name one that reads better than the body. On our bench the two
  prose asks that fail nearly always passed 1 time in 6 plain, 2 in 6 with a strong reader's
  lines handed over as a note, and 6 in 6 with the same lines held. As this release ships it, a
  reader named and nothing added to the ask, four prose asks passed 12 times in 12, against 6 in
  12 without a reader, for one to three cents a reading
  ([the bench](docs/bench.md#the-list-held)).
- **What changed, by Mikata's own checks.** When a light-coding answer lands, the harness runs the
  body's last checks again (the `python3 -c` and its kin that it asked its code questions with),
  on a copy of the finished work and on a copy of the code as the ask found it, and shows you
  where they print differently: one line that counts, then the places they differ, `- ` before
  and `+ ` now, at most 20 lines. A place that prints another value comes before one that only
  flips the body's own mark (`FAIL` to `PASS`, "All tests passed!"); a check that runs only on
  the finished work is shown as new, after the ones that ran on both; a place too long for the
  list is cut and says by how much. Every check that differs or runs only now is written whole
  to `.mikata/tmp/what-changed.md`. No model reads it and nothing of it goes back to the body. In
  the terminal app, `mikata run`, the board and the protocol (a `changes` event).
  `gate.changes = false` in your own configuration turns it off; a repository's file cannot. On
  42 kept runs of our bench a wrong value is in those twenty lines in 18 of 26 failed runs, where
  a model reading the same answers caught half, and one passed run in four shows a break its
  tests missed ([the bench](docs/bench.md#what-changed-measured)).
- **A command that undoes the body's own edits says so, and the edits are kept in reach.** When a
  command puts a file back as the session found it (a `git checkout -- file`, a copy laid over
  it), its result names the file and where the version it replaced is kept:
  `.mikata/tmp/kept/<snapshot>/<path>`, which a plain `cp` brings back. Read from the files, not
  from the command's words; nothing is said when the command's own stash holds the work. On our
  bench a body lost twenty edits to a checkout inside a script and wrote them again from memory
  until its steps ran out.

### Fixed

- **The board's chat no longer pulls you back to the end while you scroll up.** The list knew it
  was at its end from scroll events only, and a redraw that came before the event of a scroll
  just made (a message arriving, a row measured) put it back at the end. Each draw now looks at
  the scroller first: one the reader has moved up and off the end is left where it is. It showed
  as a test that failed on main twice in two days and passed everywhere else.
- **A release's notes link into the repository.** The notes are the changelog's section, whose
  links to the book are relative; on a release page they led nowhere. The release workflow now
  writes each one whole, at the tag (1.0.0's were corrected by hand).
- **Seven links in the book pointed at a heading that had moved,** six of them at the overview's
  section on the modes.
- **The terminal app and the board take a queued line off the queue when its turn starts, whatever
  the harness says after it.** Both looked for the line at the end of the ask as the body was
  given it; the harness may now say more after the ask.
- **`mikata status` no longer says "summoner"** when the verify witness is broken: the message
  named the body, though the break holds every verify line.

## [1.0.0] - 2026-11-18

The first release of **Mikata** (味方, "ally": the one on your side), called Mnemorph while it was
built: the Rust core, the `mikata` terminal app and `mikata run`, the `mikatad` server, the board
and the admin commands, the TypeScript SDK, and the book. What changed since the first cut comes
first; then everything, grouped by the [milestones](docs/experimental/roadmap-m0-m14.md) it was
built by.

**What to expect, as measured on our own bench** (fixes in a large Python library, one small open
model): light-coding, the default, passed 8 or 9 of 10 issue-style tasks, ahead of the two other
harnesses run beside it; 30 of 30 small projects asked as one; and five whole repositories in one
ask, one body a project, 4.5 of 5. An ask with its examples and expected output does best: the
same tasks stated in one paragraph of prose passed half as often. Structured-coding and
collective-coding are kept as the lab, usable and frozen.

### Renamed

- **Mnemorph is now Mikata** (味方, "ally": the one on your side). The binaries are `mikata`,
  `mikatad` and `mikata-guard`; the crates `mikata-*`; the npm packages `@mikata/*`; the
  environment variables `MIKATA_*` (the `MNEMORPH_*` names are no longer read); the repository
  `Kaginari/mikata`. A world's `.mnemorph/` folder and the `mnemorph` folders under the config and
  state homes are moved to their Mikata names the first time Mikata runs, and the move is said; when
  both exist, nothing is touched.
- **Mikata's own look.** The logo is two cells mid-division, gold for you and cyan for Mikata;
  `assets/brand/` holds it for dark and light pages, the icon at 512 to 32 px, the favicon and the
  social card, rendered from `assets/source/mikata-*.svg`. In the terminal app, the sidebar's head
  is Mikata's logo, still, as the real picture where the terminal draws images, and nothing where
  it does not (no logo built from characters; `MIKATA_TUI_GLYPHS` is gone): the races'
  animated portraits and Player's are gone, with their 7.6 MB of embedded clips, and nothing ticks
  for the head any more; the picture shows from 30 rows, not 36. The welcome opens on one title line,
  **◆ Mikata** in the logo's gold into cyan, the version and "the coding agent on your side", with
  no block letters; its four lines say what light-coding does, and the race chips are gone.
- **The book is Mikata's.** The README and `docs/` are about Mikata: the overview, the loop, the
  architecture, the client protocol (now its own page), consumption, MCP and memory, renumbered.
  Mnemorph's races, its chain and collective modes, the ontology, the old M0 to M14 roadmap, its
  emblem, banner and deck are in `docs/experimental/`, the lab, frozen. A short roadmap says what
  comes next. Claims the docs made of things not built (a Claude Code hooks adapter, a
  `--container` sandbox mode) now say they are not built.

### Changed

- **Mikata acts outside the machine itself, with your word.** In light-coding, Mikata may push,
  open a pull request, publish or deploy when the ask needs it: every outward or destructive act,
  MCP tools' included, shows you an approval card first, and `mikata run` denies them unless you pass
  `--gate-tty`. The lab's modes are unchanged: there the throne hands such acts to a Kijin. The
  prompt says "You are Mikata", and an MCP server may name `mikata` in its `races`.
- **The session's body is shown as Mikata.** The terminal app and `mikata run` show the session's
  body as `mikata`, and a split's project bodies as `mikata/<project>`; inside, its name is still
  `summoner`, which its memory, its MCP scope and its events keep.
- **The board's rail opens on Mikata's icon and name**, the name in the logo's gold into cyan. The
  theme has fixed brand colours for it, `--mn-brand-you`, `--mn-brand-ally` and their inks for a
  light page, which no scheme changes.
- **Light-coding splits an ask across separate projects.** When an ask names two or more folders at
  the world's root that each hold a project file of their own (`pyproject.toml`, `Cargo.toml`,
  `package.json`, …), one light-coding body a project runs at once, its folder as its root, and the
  Summoner then reads their answers and checks each project once. Measured before: five sqlglot
  copies asked as one, one body fixed 2.5 of 5 on average (3.5 given 400 steps); each project alone
  fixed 5 of 5, twice. Measured after, on the same ask: 5 and 4 of 5, about 19 minutes and $1.72 a
  run.
- **Structured-coding is frozen.** It stays usable and documented, but is not developed or
  benchmarked further until there is work where structure should clearly win.
- **Structured-coding fans out by itself.** When an Elf orders work, the harness also starts every
  other Orc of its lane that an open item of its plan names, at once, with that item, the plan's
  `## Contract` and the Elf's ask as its brief; each item is started this way once per Elf body,
  and its report comes back on the Elf's own order. Measured before: told to order independent
  parts in one turn, the Elf handed five projects out one at a time and never reached three of
  them.
- **A part is told its own lines only.** A part the harness starts no longer carries its Elf's whole
  `Done when` list as context, and an Orc's `verify` refuses a line name its brief does not carry.
  Measured before: in a five-project run every Orc the harness started was told all 31 lines, coded
  checks for other projects' lines, ran them in its own project's folder, and reported one unfixed
  project done.
- **Structured-coding hands a lane out by domain.** When an Elf's ask names the domains of two or
  more of its Orcs, the harness starts each of them before the Elf's first turn, at once, on the
  ask's work in its own domain and the `Done when` lines that name it; the Elf reads their reports
  and sends failures back. Measured before: on five projects, one Orc a folder, four runs broke on
  four different plans by the Elf.

### Fixed

- **A split ask is held to your caps.** The bodies light-coding starts for each project were given
  the session's limits but not its spend caps, so a split run could spend past a month, day or
  session cap. They now read the same ledger and stop at the same caps.
- **A piped check is judged by its own exit status.** The shell now reads a pipeline's first stage's
  status (`PIPESTATUS`), so `pytest | tail` or `python3 -c '…' 2>&1 | grep -v noise` counts as the
  pass or failure it was, not as a run whose status was hidden. Hidden runs stop a body after eight;
  in one five-project run they stopped a body after 59 of its 200 steps while it was still fixing its
  project. A command with no pipe, a heredoc or a trailing `\` runs exactly as written.

### M14 — 1.0

- Three frictions that stopped working runs, read from step 20's journals:
  - An optional number or bool written as a null in words (`"None"`, `"null"`, an empty
    string) is left out, as a JSON null is. One run sent `"end": "None"` to `read` 23 times, and
    three bodies ended on those refusals alone.
  - `index` given a folder indexes its files, as many as the call has room for, and says so. Ten
    folders given by name were ten refusals for one body.
  - A test bench is told apart by the folder its command moves to first (`cd 07-flatten && …`).
    A run checking thirty small projects in turn read as twelve red visits of one bench, and was
    stopped as stuck.
- The board opens in Chromium from its launcher file again. Chromium calls the launcher's
  refresh, a `file://` page's, a cross-site navigation, and the board refused every cross-site
  request, so `board.open` and `mikatad board --open` showed "refused: a request from another
  site". A top-level `GET` navigation of a page outside `/api/` and `/ws` is now let through
  from another site: it needs no session, and the page cannot be framed. A cross-site fetch, a
  frame, an API route, the socket and every mutation are refused as before.
- The terminal app wears colour schemes: `neon` by default, then `ember`, `lagoon`, `harbor`,
  `dune` and `spice`, each from a four-colour palette, and `hud`, the board's cyan, as before.
  `--theme` and `MIKATA_THEME` take a scheme, a variant or both (`lagoon-light`), and
  `/theme <scheme> [dark|light]` changes it live. The board wears the same schemes, chosen on
  its Settings page and kept per browser; neither choice moves the other. Every scheme is held
  to WCAG AA in both variants, every text colour on every surface, as the board's tokens are.
- The board's metrics show the window's runs, folded from their journals as `mikatad runs
  --metrics` folds them: the last gate verdicts and the endings counted, how many runs tested
  after their last code write, and the newest thirty runs one row each — turns, calls, time in
  the model and in tools, the re-read share, results cut short, lines written, the verdict and
  the ending, linked to the run. Spend per model and per body is drawn, and the totals line
  splits input, output and the cache's reads and writes.
- The board starts with the terminal app only when it is launched with `--web`, which also opens
  it in the browser. `board.autostart` and `board.open` are no longer read by the app; they still
  load. `mikatad board` starts one on its own as before.
- The ask is a raised panel in the human's colour, as their sent lines are, with three rows to
  write in on a window 30 rows tall or more, and a footer that names the body, the mode and what
  enter does now.
- A session opens on tagged rows — MODEL, SANDBOX, MODE, BOARD, RESUME, WARN, CONFIG, IMAGES —
  each with what matters sharp and the rest quiet: the board's address underlined, a board that
  did not start, a warning or a configuration hole in gold. What is logged is unchanged.
- The model's reasoning is a block while it comes, its newest lines under a turning head; a
  click or ctrl+t hides it to the head, and the next thoughts open the same way. The header's
  chip and the sidebar's state turn with the same spinner.
- A drag over the terminal app's conversation selects its text alone, highlighted in the scheme,
  and copies it on release: OSC 52 for the terminal, and `wl-copy`, `xclip`, `xsel` or `pbcopy`
  where one is found. The panels' spines, bevels and shadows are left out and a line's own indent
  kept; the sidebar is never selected. Esc or the wheel lets it go, and a click without a drag
  still expands a card. Shift and a drag stays the terminal's own selection.
- The panes' labels are plain capitals, as the wordmark is: Unicode's small capitals drew in a
  fallback font.
- In VS Code's terminal the portrait draws again: xterm.js 6.1 answers Kitty's graphics query
  but draws none of its images, so it is given iTerm2's; and a cell it says a pixel taller than
  its window gives (18 over rows of 17.5) gives way to the window's, so a Sixel portrait no
  longer spills over the line under it.
- The book says what was measured on 13 November: structured-coding after its gate fixes (its
  parts land now; still about seven times slower than light-coding on work that fits one
  context), and light-coding on an ask with no examples (4 of 8, against 4 of 4 with them), with
  the advice that follows: say what the result must be, with one example of what must change and
  one of what must not.
- An Orc that codes a verify line again for a `Done when` name replaces the line coded for that
  name before: one coded line a name. A coded line was only ever added, so one that could not
  run, or that needed another zone's code to pass, failed every later landing of its Slime. In a
  kept structured-coding run, 14 of 16 Slime runs failed at their gate, every one on such a line,
  though the Orcs had coded better lines since. A line the human pinned is never replaced. The
  verify pins gain a `withdraw` row, which an older build does not read.
- The Orc's card says that a Slime's gate runs every line on its card, that a line needing more
  than the Slime's own zone is not its verify line, and that the Orc runs a `Done when` line
  spanning several Slimes itself, once they have landed.
- A `Done when` line that was red before a commissioned body began refuses no landing of that
  body. The line is a fact about the whole ask and often needs another zone's part: held as each
  Slime's gate it deadlocked structured-coding, since no part could land before the whole line
  passed and every refused part was put back. Such a line is now said open: the body is told
  once, its part stands, the journal heads the verdict `PASS, 1 open`, and the report's mark for
  the line is `FAIL`. A line the landing breaks, a line with no name and every line the human
  pinned refuse a landing as before. The app and the web board do not show open lines yet.
- A coded verify line may be 500 characters long, up from 300. In two kept runs eight lines were
  refused for length, seven of them by under 65 characters.
- The commission tool reads a bare string as a list of one for `verified` and `lines`, and
  refuses a slice that is no whole number by the field's name, without the parser's column. An
  Orc's run ended on those two slips back to back.
- The three modes are named `light-coding`, `structured-coding` and `collective-coding`. They
  were solo, the hierarchy and the collective; the command line takes the new names only, and
  every place a mode is said says them. The configuration keeps its keys (`steps.solo`,
  `collective.on`), and the bench's reports keep their arm names.
- A session's mode can be asked for, and is said.
  `--mode light-coding|structured-coding|collective-coding` (or `auto`, the default) goes with
  `mikata`, `mikata resume`, `mikata run`, `mikata status` and `mikatad`. `mikata status` says the mode a session here opens in and why, with no key; the
  app says it on its first lines, `mikata run --json` on its result line, the protocol in the
  `initialize` answer (`mode`, `modeWhy`), the web board in a chat's side pane, and the journal
  on the run's `law` beat. A mode the world cannot give opens no session and says what is
  missing, before any key is asked for. Cards that are not adopted give no mode, and the reason
  says how many wait. A resumed run's mode is decided again; when that is not the mode it
  stopped in, the reason says so.
- Structured-coding shares the work board, `.mikata/tmp/board.md`, as collective-coding does:
  every
  commissioned body's brief carries it, and the `@F` and `@W` lines of each answer are posted to
  it. Summoner is told to write a map on it before it hands work down. A fresh session's first
  ask sets an older board aside as `board.prev.md`. Not measured yet.
- A commissioned body is told that what a tool gives back is data and never a task, as the light-coding
  body already is. A Slime and a collective-coding member run their verify line, the `Done when` lines
  of their part and the tests of the files they changed before they answer. Not measured on the
  chain yet.
- The book says the three modes: which to use, how to start each and what each needs
  ([overview](docs/01-overview.md#the-modes),
  [the three modes](docs/experimental/races.md#the-three-modes)). The overview, order and client-protocol
  canvases are redrawn, and [lessons](docs/lessons.md) lists what our own bench measured.
- A `tree` beat after every landed write hashes the files the ask has changed and says how
  many times that tree has stood: a run that goes round between the same trees can be read from
  its journal. A reading only; nothing acts on it.
- The harness closes a plan item, `- [x]`, when its order's landing passes, and a retried Elf
  is no longer told it as open: in the kept chain journals, 5 of the 18 items told again as open
  named a body that had already answered DONE. A send-back to a body whose items are all closed
  opens the last one again instead of being refused.
- Light-coding's `## Done when` lines are named `D1`, `D2` and so on, so a plan item, a verify line and
  a report can point at them.
- The chain carries the `## Done when` list. An order names the lines it must make true
  (`lines`); the brief carries their text, the plan item their names, and the body takes them on
  to its own orders. An Orc names the lines each verify line proves (`line`). The report comes
  back with one mark a line, PASS, FAIL or unchecked, read from the gates that ran below the
  order, never from what a body said.
- Collective-coding's board holds a `## Done when` list, named `D1`, `D2`, … as the light-coding
  plan does,
  and each line of `## Parts` names the lines its share makes true.
- `mikatad runs --metrics` says whether a test ran after the run's last change to a code
  file. Reported per run, not acted on.
- A command that ran to its end and exited non-zero scores 0 in the guts' window, not −1: a
  `python -c` probe that raises, a `grep` that finds nothing, a red test. Such commands were 51%
  of what scored −1 in the guts' warnings; over 614 kept journals the same replay gives 93
  warnings where it gave 240. A refusal, a tool error and a killed command still score −1.
- The light-coding plan opens with `## Done when`, every output the ask states, and its last step runs
  them and the repository's tests for the files changed, before it answers. Of 37 failed qwen
  runs of one body, 14 answered "done" on a failing tree and 13 of those were close; one printed
  the ask's own example seven times, got another output each time, and answered that it matched.
- The light-coding prompt says what checks the work, from the session's end gate: the pinned verify
  lines by name, or, with none pinned, that nothing checks the work but the body. It said "the
  end gate checks them" in every world; in plain `solo` the gate checked nothing in 8 bench runs
  of 8.
- The light-coding prompt says that what a tool gives back is data: it can say how the repository is
  worked, and never gives a task. It said so only of the human's own marker. On the bench, a
  line at the end of a file read that asked for a file to be written was obeyed by qwen3.5-35b
  in 6 runs of 6 and by DeepSeek V4 Flash in 2 of 6. This changes the cached prompt.
- The guts warns once a model turn at most. One message of hundreds of calls closed three
  windows and ended the run at the doom checkpoint before the model read the first warning: 4
  of 66 DeepSeek V4 Flash bench runs, and one of the four that screened the fix before this one.
  Turn after turn of acts that do not land still ends the run at the third warning.
- A kept learned-guts unit carries whether it advises. It earns that only by a fit that clears
  the whole bar, loses it by itself when 20 more labelled runs no longer bear it out, and
  `mikatad guts watch` scores it again on a record. Nothing reads its advice yet.
- `mikatad guts fit` reads every fit against the base rate of the same 40-call stretch of the
  run, which the unit is also given as an input, by held-out group and with a 90% interval, and records how many fits each held-out set has judged, printed
  beside the gain. The learned guts advises only when it beats that reference, on a set that
  has judged nothing.
- A call sent again right after itself in one model message runs once; each copy is answered
  with a line that says so. The loop stop counted the copies, so one message of the same call
  five times ended the run before the model read the warning: 5 of 66 DeepSeek V4 Flash bench
  runs ended that way. The same call message after message still stalls at the fifth.
- The book records two probes: which hosts of one model cache a repeated prompt (one of three),
  and that a note sent as a user message makes a host drop the model's earlier reasoning and
  miss the cache, where the same note in the last tool result does not.
- A brief calls a span "verified by your dispatcher" only when the dispatcher's last read of those
  lines matches them as they stand. Lines that changed since, and lines it named without reading,
  are attached under their own headings. Every named span was called verified before; in 49 kept
  runs, 6% of them named a file written after the last read and 13% a file never read.
- Shell output cut for length keeps its head and its tail and says how many bytes went between
  them. It kept only the first 30,000 bytes, so a long test log lost its verdict.
- A command that ran and exited non-zero is a result, not a failed act, for the
  three-failed-turns stop: that rule ended a fifth of members' orders, most often on a probe
  that exits 1. A refusal, a tool error and a killed command still count.
- A body's shell `timeout_ms` under the 120-second default is read as the default: models made
  to fill the field invented 5 to 30 seconds on every call. A longer limit stands, and the
  harness's own short limits are kept.
- Reasoning replay is a per-model switch, `models.replay_reasoning."<model>"`, on by default
  as before. Off, the reasoning an OpenAI-compatible server returned is no longer sent back with
  later turns; the journal keeps it and the `rides` beat names the arm.
- The sampling can be said, per model: `models.temperature`, `models.top_p` and `models.effort`
  are sent with every turn when set, and journaled on the `rides` beat. Unset, requests are
  byte-identical and each host keeps its own defaults.
- Each model has its own context window, `models.window."<model>"` (200,000 when unnamed), and
  the stress line is held to 90% of it. A `stress.context_tokens` above that opens no session
  and says why; it was clipped in silence. A model id with dots may be a configuration key.
- `providers.<name>.upstreams` names the only hosts a router may send a provider's calls to
  (OpenRouter's `provider.only`): a host's prompt cache is its own, and a session that moves
  between hosts pays for its context again. Unset, requests are unchanged.
- The learned guts reads three inputs the harness takes itself: edit misses in the window, the
  share of statement rows passing, and the turns since the failing rows last shrank. Each act's
  readings are kept in the trusted guts journal, stamped with their input version, and old and new
  readings never meet in one fit.
- `mikatad guts fit` prints a shuffled-label control after each split. It must show no gain;
  one that does says the split or the interval cannot be trusted.
- `mikatad guts fit` holds out whole groups when runs carry one (a bench's task): a task's runs
  are not independent, so a split by run scored the learned guts on what it had memorised. Both
  splits are printed; the throne's runs under its old name fit with the Summoner's.
- A cache read weighs in wt what the model bills it against input, from the price table: a third
  on a model that bills $0.05 against $0.15, not a fixed tenth. An unpriced model keeps 0.1.
- `mikatad runs --session` names the note effect: a cache miss after a note or a new order
  whose prompt came back smaller is its own class, with the tokens the host dropped.
- `mikatad runs --session` files each unexplained cache miss under its cause (a host that
  never cached, a host switch, a note or a new order, nothing visible), prices each upstream host
  on its own, and joins a turn to its usage row by the provider's message id, now kept on both.
- `mikatad runs --session` reads a whole session across its bodies: what unexplained cache
  misses cost, the rent a member pays on the context it carried into its later orders, member
  re-read bytes by cause, and the members' orders by outcome with their dollars.
- Every usage row behind a router names the host that served the call (`upstream`, OpenRouter's
  `provider`): cache misses and latency drift can be traced to a host, and an arm served by a
  different mix of hosts shows it.
- A run's journal keeps its numbers: each result its time, bytes, lines, whether a cap cut it and,
  for a read, the file, the lines and whether it was skipped as unchanged; each model turn when it
  answered and how long it took. `mikatad runs --metrics <run>` reads them back: bytes and
  context share by tool, the re-read ratio, lines written, and time in the model against tools.
- An experiment against the chain, off by default: with `collective.on` in the global file, the
  bodies that report to the Summoner are a collective of equals (collective-coding). Only the Summoner summons, it may
  commission any member directly, and the members share a board (`.mikata/tmp/board.md`) that
  every brief carries and every member's findings are posted back to.
- The Summoner plans before it writes: two or three ways under `## Paths` in its scratch plan,
  each with its cost and risks, the chosen one under `## Chosen`, each named risk tested before it
  answers. On Terminal-Bench it took light-coding from 0 of 4 to 5 of 5 on `cancel-async-tasks`, and
  kept `feal-differential-cryptanalysis` at 3 of 3 in about 0.6× its time.
- A turn whose acts fail counts once toward escalation: three orders refused in one turn ended an
  Elf before it had read the first refusal. An order carries its own `plan_item`, which the
  harness opens in the dispatcher's plan, and the refusal shows the exact line. A refused order is
  never said to have run at once. A dispatcher keeps two of its own turns, at the pace it has
  shown, when it hands its clock down.
- Evolution's do-nothing control starts from the tree its window run started from, kept in the
  run's row, never from the world the window finished; and a run that changed nothing is judged
  by its verify lines over the world as it stands, not by the gate's pass on no change. On the
  finished world, with that pass, a control that did nothing landed every task.
- The learned guts fit every act at once (batch logistic regression, order-free) instead of one
  step of 0.5 per act, which left a model saying what the last runs said; and `mikatad guts`
  says it beats the base rate only when a 90% interval, resampled by run, lies above it.
- A commissioned body's clock ends before its dispatcher's: all but a twentieth of the time left,
  never the last 20 seconds. A child that kept its race's own hour ran past a 30-minute task's end.
- **Renamed:** the throne is the Summoner (was Rimuru) and the human is the Player (was Veldora),
  everywhere: the race, its seat `.mikata/summoner/summoner`, the cards, the board, the terminal
  app, the wire and the SDK. A world made before moves with `sh tools/migrate-summoner.sh <world>`,
  then the human re-adopts (`mikatad roster adopt --yes`) and re-pins the Summoner's verify line
  if it has one (`mikatad verify pin summoner --yes`): trust is never carried over by a script.
- A refusal on ground a body's own chain keeps routes down: "commission orc-o, whose chain keeps
  it", not up past the commander it already is. An Elf told to send it up escalated work its own
  Orc could do.
- An Elf splits work across domains instead of chaining it: a `## Contract` in its plan, then one
  commission per Orc in the same turn, run at once and settled in order, and the whole check once
  all have reported.
- Summoner routes by what it knows: a fix its own reading already names (the files, the lines, the
  change) it does itself; unclear, large or many-part work goes down. A commission names the spans
  its writer checked in `verified` (`path:start-end`); the harness attaches their lines to the
  brief, so the body builds on them instead of reading them again.
- A refusal names only what a body can write: a seat under `.mikata` is named by its `plan.md`,
  never as a folder the body may fill. An Orc's coded verify line is run once as it is coded, its
  output given back, so a line broken in its setup is seen before a Slime lands on it; and a commissioned body whose end gate fails on its Orc's
  line is told to answer with the error instead of searching for a way round it. An Elf with
  nothing handed down is told to stop reading at twelve steps, or half its steps when it has
  fewer: given eighty, one read the whole fix its brief already named.
- An Orc starts Slimes on its own domain for its commission with the `summon` tool: no human asked,
  each one recorded in the witness as the Orc's, at most three at once, all retired when the
  commission ends. The same ground wakes the same Slime, its memory kept, and `roster list` says
  how often each was started.
- The learned guts: one logistic unit per race that predicts, from a body's last acts, whether its
  run lands. It learns from failed runs as from landed ones, never from a run no check judged, and
  never acts. `mikatad guts runs|fit|show`.
- `sandbox.disposable`: on a machine made to be thrown away, in a container and from the global
  file alone, a fetch, an install or a write outside the world runs with no shelf and no gate;
  every act that leaves the machine is still refused.
- The release-candidate smoke test: the built binaries, in a fresh HOME with realistic dotfiles
  and a world with history, against a scripted provider on loopback (`just rc-smoke`).
- `install.sh`: the release for this OS and architecture, verified against `SHA256SUMS` before it
  is unpacked, into `~/.local/bin`, no sudo, no profile edits; tested offline (`just install-test`).
- The release workflow: on a `v*` tag, archives for Linux and macOS on x86_64 and arm64, their
  checksums (and build provenance, once turned on), smoke-tested, then a draft release for
  review.
- This changelog.
- Summoner hands each body a brief it writes, and an Elf weighs two or three ways under `## Paths`
  and says which it takes under `## Chosen`.
- `limits.reply_tokens` (default 16,384, up from a fixed 8,192) and `limits.session_minutes`
  (default 60): a model's thinking shares the reply cap, and a cut plan wasted a step.
- A session's default steps are 200: at 100, real fixes did the work and stopped before answering.
- A body's card names the doc of each body below it, and a stop after an outward act nobody can take
  still reaches the Elf.
- A number or a bool a model sends as a string reaches a harness tool as what it meant: the nullable
  type made Qwen models do this, and every ranged read was refused.
- A model call that breaks mid-answer is asked again, twice at most, instead of ending the run.
- A body stopped on its steps writes down what it learned, and a commissioned body's report carries
  that to its dispatcher.
- The bench's `auto` arm: the plain ask in the full world, the score the harness's own decision
  must hold. `solo` now means the throne alone.
- A handoff carries what Summoner found alone, an Elf that only reads is told to hand down, and a retry goes
  on from what its plan left open.
- Light-coding escalates when it stops short: a stall, doom or spent steps hand the ask to an Elf, as a
  full context did, and a finished handoff is an answer.
- A session in a world with Elves keeps light-coding's steps: the throne had 40, so a plain ask stopped at
  a checkpoint before its work was done.
- The presentation: `assets/deck/mikata.html` and its PDF, made with `tools/deck.mjs`.
- Tool calls keep their optional parameters on every OpenAI-compatible host: some hosts skip an
  optional parameter, so `read` lost its range and a run stalled. Each one is now sent required and
  nullable, and its null taken back out. `read` drops `limit`, which only repeated `end`.

### M13 — Integrations

- MCP client: race-scoped tools by class, three priced levels, lazy and health-checked servers,
  BM25 ranking with a fast-tier rerank, learned from a chained journal, a stable prefix and
  native deferral [#270] [#299] [#300] [#301].
- The index tool: a tree-sitter skeleton with line ranges; a large file read by range [#269].
- Relations between worlds: a registry, harness sightings, named-twice proposals, gluing [#271].
- Recorded provider cassettes for OpenRouter and Ollama, provider-reported cost, unknown usage
  estimated and unpriced [#285] [#303].
- A provider's key from a file (`key_file`, global file only, 0600, never inside a world), read
  when its variable is not set: a board started from any shell opens chats; the integrations
  show the file's state, never its contents.

### M12 — Board

- `mikatad board`: a secure loopback server, keyed once, traded for a session cookie [#286].
- The board: shell, router, themes and safe text [#287]; the session protocol over WebSocket
  [#289]; worlds and the world overview [#291]; chat [#292]; memory and ontology [#293]; web
  approvals under the terminal's protections [#294]; metrics and integrations [#295]; bodies,
  race cards, queue, timeline and journal [#296]; polish and the M12 exit [#297].
- One escaping range list for Rust and the board: every format, separator and default-ignorable
  character shown, never obeyed [#290].

### M11 — Value, evolution, ascension

- Oni envelopes: signed, expiring, capped, revocable, and honoured by the gate [#108] [#110]
  [#118] [#262]; each listed in `mikata status`, one line with its state, dates and uses.
- Meters with their count and 95% interval; value rows stamped with their build [#166] [#260]
  [#267] [#268].
- Hash-chained trust journals, the head anchored in `log.md` [#261] [#267].
- The end gate runs only pinned verify lines: `mikatad verify pin`, a chained witness under the
  state directory, the terminal app's first-run prompt; a card that differs fails closed.
- Explainable falls and honest bad news [#263]; lifecycles explored whole, with reachability
  witnesses [#265].
- Evolution judged after the freeze, against a do-nothing control, on an exact staged file set
  [#264] [#268] [#298]; a claim graph for memory [#266].
- Evolution's trigger, propose only: a creature stressed on its trusted gate verdicts (its last 3
  failed) with a full desk (more than 7 live personal notes) is due, by the arm the rules pick —
  split, rebirth or mutate — and why; cooling while its swap's window is open. Shown in
  `mikata status` and `mikatad evolution due`; nothing is drafted or swapped.
- CI as an instrument: `mikatad ci` reads the forge's newest runs through the read-only probe
  into `.mikata/instruments/ci.jsonl`, and `mikata status` shows each branch's state as last
  read.
- The advisory mirror: `mikatad scout` keeps each ecosystem the world's lockfiles name under
  `.mikata/intel/osv-scalibr/`, the layout `osv-scanner` reads offline; `mikata status` says
  each one's age. `mikatad scout audit` scans the lockfiles against it with the network off, for
  `mikata status` and `dark-elf-supply`.
- `@mikata/protocol` and `@mikata/sdk` can be published to npm: built to `dist`, MIT, and
  `just check` installs and imports the packed tarballs.
- The race cards at 1200 × 1680 for the full card view, on 1024 px portraits; the board serves them.
- An approved typed outward act can reach the forge: it reads the forge's logins as the probe
  does, so `git push` and `gh pr create` work under the kernel guard, and nothing stops to ask for
  a username. Found by the first live `kijin-forge` run.
- Recall holds as memory grows: BM25's saturation grows with the notes weighed
  (`recall.saturation_growth`), fitted against 101 notes real runs wrote. Over that many notes a
  fixed saturation let nearly every note clear the floor.
- The board's colony net (`#/net`) is a 3D hex city around the throne: the races on a terraced
  lattice, a world's living bodies seated around their race, waves and signals through it, and a
  race picked lights what it reaches.
- The standing reads run on their own: `mikatad refresh` reads CI and refreshes the advisory
  mirror and its audit when due, after each session and hourly from the board
  (`refresh.ci_minutes`, `refresh.scout_hours`, 0 for off).
- The colony net shows a world's Minds: gems beside the bodies that name them, and round the rim
  for those any body may wear or none does yet. `GET /api/w/<id>/bodies` gains `unworn`.
- Strain and doom count for evolution: a trusted journal under the state directory keeps what
  the council and the doom checkpoint saw, and a creature with a full desk is due once its strain
  reaches the council's line or a run meets the checkpoint.
- Evolution's drafter: `mikatad evolution draft <creature>` asks the configured model, with no
  tool and the doc and desk shown as data, for at most 7 traits each citing notes on the desk;
  the harness writes the doc's `## Traits` section, its header untouched, and stages the plan for
  the human's swap.
- Evolution's proof: `mikatad evolution swap` proves the staged plan before it asks — written
  over a scratch copy of the world, where `onto check` must find nothing the world does not and
  every doc the plan writes must have its verify lines pinned and passing in the sandbox. A plan
  that fails is refused with every reason; one that re-parents or widens a card is refused so.
- Evolution proposals: every plan the drafter draws is kept under the state directory, chained
  and anchored, with each file's digest as drawn; `mikatad evolution list | show | apply |
  reject` reviews them, `apply` refused when the world moved or the staging changed since, and
  `mikata status` shows each one waiting.
- Evolution on its own (`evolution.mode = "auto"`, the default, now the human's global file's
  alone): after each session a background tick (`mikatad evolution auto`) drafts, proves and
  swaps in one due creature on trial, and settles a swap whose 10-run window is full by the rule —
  kept on its net wins against doing nothing, else rolled back, its files put back as they were.
  `mikatad evolution keep | rollback <id>` lets the human settle first; each settle is a
  `settle` row in the evolution ledger.
- A kept evolution puts the desk notes its traits cite to sleep (dormant, their events kept), so
  a creature stressed again evolves on new notes; a rollback leaves them on the desk.
- A split's young from relief: the trigger names the two parts a creature's trouble clusters in,
  and `mikatad relief <name> --around <part>,<part>` gives one to each young, the rest shared
  out evenly. `evolution due`, `mikata status` and the tick print that command; a split stays
  the human's to apply.
- Guts classes on every act (`fail`, `red-pin`, `positive`, `repeat`, `echo`, `thrash`), read from
  structured outcomes only, kept chained outside the world, counted in `mikata status` and
  chipped on the board's timeline; signals only, the red pin weighing 0.
- Rises and falls from the compiled race law alone: proposed from the report ledger over the
  28-day window and through probation, shown in `mikata status`, `mikatad meters` and the
  board's bodies page with the command that applies them, applied only by the human at the
  terminal (`mikatad roster rise|fall`), and replayed exactly from the journals and the
  creatures' witness (`mikatad roster replay`). An input no trusted journal carries yet is said
  unknown, never zero, and proposes nothing.

- M11's lesson as a test: a landing the gate did not pass (failed, handed back, not verified,
  unrun, unpinned, unchecked), as a new run, an old pass judged again, or the throne's, never
  raises any creature's pass meter.

### M10 — Dark Elf

- The specialities, attack classes, surfaces and route, compiled; the finding fields on the wire
  [#245] [#246] [#247].
- The finding record and its validator, the coverage ledger, the compiled playbooks [#251] [#252]
  [#254].
- Challenge routing at the gate; independent verification before a finding counts [#255] [#256].
- The seeded fixture repository, the ledger's fold per speciality, the law audit's checklist and
  the reviewers' calibration [#257] [#258] [#259]; Mind provenance a body cannot forge [#272].

### M9 — Kijin

- Each post's outward acts compiled in [#239]; a plan with no declared undo is destructive [#234].
- Receipts observed by a read-only probe, never by exit 0; a cut act probed, never replayed;
  receipt-based closure [#240] [#242] [#243].
- A guarded attic restore [#235]; an approval bound to the plan's fingerprint [#237].
- A typed pull-request act: a branch and its pull request under one approval [#248].

### M8 — Memory and toolbox

- Notes as events, recall with a seat for dissent, the write rule [#192] [#193] [#194] [#195].
- Reconsolidation, consolidation and forgetting, the unproven write, the replay loop and lock-in
  [#197] [#198] [#199] [#200] [#202].
- Session start, the compaction drain, the toolbox [#201] [#203] [#205].
- Citation by use, the accepted, corroborated and confirmed routes, a correction standing beside
  its note, recall's weights in config, the triage fixture [#206] [#207] [#208] [#213] [#215]
  [#216] [#217] [#218]; the ontology mesh live [#214].
- The **checked** route grounds a claim only on the check it names: `@U law <path> <fact>
  check=<verify line or test>`, a verify line that ran in that gate (or a test it names), cited in
  the grounded note. A claim naming no check, or one that did not run, stays proposed and the
  report says why, so a compile check no longer grounds a behaviour claim [#317].

### M7 — The world

- Creatures as data and ownership as one function; the closed graph, each body's soil, the shapes
  as findings [#138] [#143] [#144].
- Dispatch, the right author, the commission runner, territory facts [#145] [#146] [#149] [#150]
  [#151] [#152] [#153].
- The shared work queue, the wake, the journal's writer, grants bounded by worth, the top-up
  [#154] [#155] [#156] [#157] [#158] [#159].
- Hardening after three fails, triage, `housekeep --heal`, relief, the council, the clock-death
  memo, offices; a slice that runs out is never a failure [#160] [#161] [#162] [#163] [#164]
  [#165] [#167] [#168] [#169].
- Light-coding escalates on signals read by code; the solo-against-hierarchy bench, judged by why
  each run ended, with the law arm and paired intervals [#170] [#171] [#181] [#182] [#307].
- The verify tool, the report folder, PASS / FAIL / NOT_VERIFIED at the end gate, unrun checks
  blameless [#173] [#174] [#175] [#176] [#178].
- Never routed above the parent; ontology findings with codes; heirs and shared Minds [#177]
  [#179] [#180].
- Territory facts and the relations between worlds trusted only from chained journals under the
  state directory: the in-world `facts.jsonl` and `dimension.toml` are copies, what they say
  beyond them flagged (`untrusted-fact`, `untrusted-relation`); in-world facts imported once,
  unsettled; `onto relate`, `unrelate` and the new `inverse` asked at the terminal.
- Append-only requests; the provenance lint; refusals that name the owner; writes attributed per
  body [#187] [#188] [#189] [#190] [#191] [#231].
- Decisions: System One behind config, bounded, Jev first with the LLM answering when none is
  there [#112] [#113] [#116] [#119] [#122] [#305] [#308]; the offices and Metis [#117] [#121].
- The session lands through the end gate and suggests its verify line; step budgets per race
  [#172] [#186] [#219] [#220] [#302].

### M6 — Protocol, SDK, CLI

- The client protocol over stdio, generated for TypeScript, the SDK and a first terminal client
  [#123] [#133] [#134].
- `mikata`, the terminal app in Rust: sidebar, mouse, themes, the race portraits in real pixels
  and their animations, the welcome [#222] [#224] [#225] [#226] [#227] [#229] [#232] [#241].
- One `mikata`: the Rust binary is the command, `mikata run` for scripts, the TypeScript CLI
  retired; why there are two binaries [#250] [#253].
- `mikata resume <run>`: a stopped run picked up through the session's one opening [#233].
- Budget caps enforced, `/send`, an approval that starts unchosen, the milestone review's fixes
  [#135] [#136] [#137] [#139] [#142].

### M5b — Long runs

- Esc and the clock reach a running command; replay compaction and the absolute stress line
  [#82] [#83].
- The loop breaker, the bench tally, the prefix ceiling [#84] [#85].
- Structured compaction: a full context summarised and the run goes on [#209] [#211] [#212].

### M5 — One body loops

- The loop core, light-coding and the end gate, the session's queue and Esc, a live task whose
  spend matches the ledger [#74] [#75] [#76] [#77] [#79].
- The attic: what commands take away comes back, and it prunes itself [#80] [#81] [#185].
- A call with broken arguments is answered, not fatal; a light-coding answer keeps its claims [#210]
  [#223] [#228].

### M4a — Config

- Configuration: global and world-local, the holes said, the gate and sandbox only loosened by
  the global file [#70]; the end gate's verify limit [#221].

### M4 — Tools, sandbox, human gate

- The shell classifier, the file tools, the human gate and bash [#62] [#64] [#65].
- The sandbox and its escape suite; a kernel guard, not a jail — no guard, no command [#66] [#68]
  [#73]; credentials closed inside toolchains [#183] [#230]; the harness non-dumpable [#288].
- Files opened beneath the world, no check-then-open race [#71]; multiedit, move, delete, patch,
  a forgiving edit [#147] [#148] [#236]; the closing review's fixes [#72].

### M3 — Providers

- The Anthropic client and the OpenAI-compatible client, one driver for every provider [#50]
  [#51]; live OpenRouter tests and real cassettes [#54]; the conversation cached [#109].

### M2 — Ledger and meters

- The ledger: count, carve and stop before the first model call [#37]; a torn line counted high,
  unseen spend drawn from a hidden reserve [#140] [#141].

### M1 — Wire and races

- The wire's envelope, parsed and emitted [#33]; prose that reads as a tag rides escaped [#238].
- The race law, compiled and covered whole [#35].

### M0 — Foundations

- The workspaces, layered crates, `just check`, CI on Linux and macOS, the world seeded [#31].
- The book as a website; every diagram parsed in CI; no conflict marker anywhere tracked [#32]
  [#36] [#114].

### The book and the brand

- The book's pages: the lanes, MCP, memory, the ontology, permanent light-coding, isekai's self-heal,
  the audit against the code [#46] [#47] [#49] [#53] [#60] [#69] [#87] [#120] [#249].
- The name Mikata, the emblem, the banner, the race icons and cards, `@mikata/theme` [#55]
  [#56] [#57] [#58] [#59] [#61].
- The canvases: each page's sketches drawn by tldraw's Mermaid converter as `.tldr` documents and
  light and dark SVGs in the pages, every canvas checked for script and outside links [#311].

[Unreleased]: https://github.com/Kaginari/mikata/compare/v1.1.0...main
[1.1.0]: https://github.com/Kaginari/mikata/releases/tag/v1.1.0
[1.0.0]: https://github.com/Kaginari/mikata/releases/tag/v1.0.0
[#31]: https://github.com/Kaginari/mikata/pull/31
[#32]: https://github.com/Kaginari/mikata/pull/32
[#33]: https://github.com/Kaginari/mikata/pull/33
[#35]: https://github.com/Kaginari/mikata/pull/35
[#36]: https://github.com/Kaginari/mikata/pull/36
[#37]: https://github.com/Kaginari/mikata/pull/37
[#46]: https://github.com/Kaginari/mikata/pull/46
[#47]: https://github.com/Kaginari/mikata/pull/47
[#49]: https://github.com/Kaginari/mikata/pull/49
[#50]: https://github.com/Kaginari/mikata/pull/50
[#51]: https://github.com/Kaginari/mikata/pull/51
[#53]: https://github.com/Kaginari/mikata/pull/53
[#54]: https://github.com/Kaginari/mikata/pull/54
[#55]: https://github.com/Kaginari/mikata/pull/55
[#56]: https://github.com/Kaginari/mikata/pull/56
[#57]: https://github.com/Kaginari/mikata/pull/57
[#58]: https://github.com/Kaginari/mikata/pull/58
[#59]: https://github.com/Kaginari/mikata/pull/59
[#60]: https://github.com/Kaginari/mikata/pull/60
[#61]: https://github.com/Kaginari/mikata/pull/61
[#62]: https://github.com/Kaginari/mikata/pull/62
[#64]: https://github.com/Kaginari/mikata/pull/64
[#65]: https://github.com/Kaginari/mikata/pull/65
[#66]: https://github.com/Kaginari/mikata/pull/66
[#68]: https://github.com/Kaginari/mikata/pull/68
[#69]: https://github.com/Kaginari/mikata/pull/69
[#70]: https://github.com/Kaginari/mikata/pull/70
[#71]: https://github.com/Kaginari/mikata/pull/71
[#72]: https://github.com/Kaginari/mikata/pull/72
[#73]: https://github.com/Kaginari/mikata/pull/73
[#74]: https://github.com/Kaginari/mikata/pull/74
[#75]: https://github.com/Kaginari/mikata/pull/75
[#76]: https://github.com/Kaginari/mikata/pull/76
[#77]: https://github.com/Kaginari/mikata/pull/77
[#79]: https://github.com/Kaginari/mikata/pull/79
[#80]: https://github.com/Kaginari/mikata/pull/80
[#81]: https://github.com/Kaginari/mikata/pull/81
[#82]: https://github.com/Kaginari/mikata/pull/82
[#83]: https://github.com/Kaginari/mikata/pull/83
[#84]: https://github.com/Kaginari/mikata/pull/84
[#85]: https://github.com/Kaginari/mikata/pull/85
[#87]: https://github.com/Kaginari/mikata/pull/87
[#108]: https://github.com/Kaginari/mikata/pull/108
[#109]: https://github.com/Kaginari/mikata/pull/109
[#110]: https://github.com/Kaginari/mikata/pull/110
[#112]: https://github.com/Kaginari/mikata/pull/112
[#113]: https://github.com/Kaginari/mikata/pull/113
[#114]: https://github.com/Kaginari/mikata/pull/114
[#116]: https://github.com/Kaginari/mikata/pull/116
[#117]: https://github.com/Kaginari/mikata/pull/117
[#118]: https://github.com/Kaginari/mikata/pull/118
[#119]: https://github.com/Kaginari/mikata/pull/119
[#120]: https://github.com/Kaginari/mikata/pull/120
[#121]: https://github.com/Kaginari/mikata/pull/121
[#122]: https://github.com/Kaginari/mikata/pull/122
[#123]: https://github.com/Kaginari/mikata/pull/123
[#133]: https://github.com/Kaginari/mikata/pull/133
[#134]: https://github.com/Kaginari/mikata/pull/134
[#135]: https://github.com/Kaginari/mikata/pull/135
[#136]: https://github.com/Kaginari/mikata/pull/136
[#137]: https://github.com/Kaginari/mikata/pull/137
[#138]: https://github.com/Kaginari/mikata/pull/138
[#139]: https://github.com/Kaginari/mikata/pull/139
[#140]: https://github.com/Kaginari/mikata/pull/140
[#141]: https://github.com/Kaginari/mikata/pull/141
[#142]: https://github.com/Kaginari/mikata/pull/142
[#143]: https://github.com/Kaginari/mikata/pull/143
[#144]: https://github.com/Kaginari/mikata/pull/144
[#145]: https://github.com/Kaginari/mikata/pull/145
[#146]: https://github.com/Kaginari/mikata/pull/146
[#147]: https://github.com/Kaginari/mikata/pull/147
[#148]: https://github.com/Kaginari/mikata/pull/148
[#149]: https://github.com/Kaginari/mikata/pull/149
[#150]: https://github.com/Kaginari/mikata/pull/150
[#151]: https://github.com/Kaginari/mikata/pull/151
[#152]: https://github.com/Kaginari/mikata/pull/152
[#153]: https://github.com/Kaginari/mikata/pull/153
[#154]: https://github.com/Kaginari/mikata/pull/154
[#155]: https://github.com/Kaginari/mikata/pull/155
[#156]: https://github.com/Kaginari/mikata/pull/156
[#157]: https://github.com/Kaginari/mikata/pull/157
[#158]: https://github.com/Kaginari/mikata/pull/158
[#159]: https://github.com/Kaginari/mikata/pull/159
[#160]: https://github.com/Kaginari/mikata/pull/160
[#161]: https://github.com/Kaginari/mikata/pull/161
[#162]: https://github.com/Kaginari/mikata/pull/162
[#163]: https://github.com/Kaginari/mikata/pull/163
[#164]: https://github.com/Kaginari/mikata/pull/164
[#165]: https://github.com/Kaginari/mikata/pull/165
[#166]: https://github.com/Kaginari/mikata/pull/166
[#167]: https://github.com/Kaginari/mikata/pull/167
[#168]: https://github.com/Kaginari/mikata/pull/168
[#169]: https://github.com/Kaginari/mikata/pull/169
[#170]: https://github.com/Kaginari/mikata/pull/170
[#171]: https://github.com/Kaginari/mikata/pull/171
[#172]: https://github.com/Kaginari/mikata/pull/172
[#173]: https://github.com/Kaginari/mikata/pull/173
[#174]: https://github.com/Kaginari/mikata/pull/174
[#175]: https://github.com/Kaginari/mikata/pull/175
[#176]: https://github.com/Kaginari/mikata/pull/176
[#177]: https://github.com/Kaginari/mikata/pull/177
[#178]: https://github.com/Kaginari/mikata/pull/178
[#179]: https://github.com/Kaginari/mikata/pull/179
[#180]: https://github.com/Kaginari/mikata/pull/180
[#181]: https://github.com/Kaginari/mikata/pull/181
[#182]: https://github.com/Kaginari/mikata/pull/182
[#183]: https://github.com/Kaginari/mikata/pull/183
[#185]: https://github.com/Kaginari/mikata/pull/185
[#186]: https://github.com/Kaginari/mikata/pull/186
[#187]: https://github.com/Kaginari/mikata/pull/187
[#188]: https://github.com/Kaginari/mikata/pull/188
[#189]: https://github.com/Kaginari/mikata/pull/189
[#190]: https://github.com/Kaginari/mikata/pull/190
[#191]: https://github.com/Kaginari/mikata/pull/191
[#192]: https://github.com/Kaginari/mikata/pull/192
[#193]: https://github.com/Kaginari/mikata/pull/193
[#194]: https://github.com/Kaginari/mikata/pull/194
[#195]: https://github.com/Kaginari/mikata/pull/195
[#197]: https://github.com/Kaginari/mikata/pull/197
[#198]: https://github.com/Kaginari/mikata/pull/198
[#199]: https://github.com/Kaginari/mikata/pull/199
[#200]: https://github.com/Kaginari/mikata/pull/200
[#201]: https://github.com/Kaginari/mikata/pull/201
[#202]: https://github.com/Kaginari/mikata/pull/202
[#203]: https://github.com/Kaginari/mikata/pull/203
[#205]: https://github.com/Kaginari/mikata/pull/205
[#206]: https://github.com/Kaginari/mikata/pull/206
[#207]: https://github.com/Kaginari/mikata/pull/207
[#208]: https://github.com/Kaginari/mikata/pull/208
[#209]: https://github.com/Kaginari/mikata/pull/209
[#210]: https://github.com/Kaginari/mikata/pull/210
[#211]: https://github.com/Kaginari/mikata/pull/211
[#212]: https://github.com/Kaginari/mikata/pull/212
[#213]: https://github.com/Kaginari/mikata/pull/213
[#214]: https://github.com/Kaginari/mikata/pull/214
[#215]: https://github.com/Kaginari/mikata/pull/215
[#216]: https://github.com/Kaginari/mikata/pull/216
[#217]: https://github.com/Kaginari/mikata/pull/217
[#218]: https://github.com/Kaginari/mikata/pull/218
[#219]: https://github.com/Kaginari/mikata/pull/219
[#220]: https://github.com/Kaginari/mikata/pull/220
[#221]: https://github.com/Kaginari/mikata/pull/221
[#222]: https://github.com/Kaginari/mikata/pull/222
[#223]: https://github.com/Kaginari/mikata/pull/223
[#224]: https://github.com/Kaginari/mikata/pull/224
[#225]: https://github.com/Kaginari/mikata/pull/225
[#226]: https://github.com/Kaginari/mikata/pull/226
[#227]: https://github.com/Kaginari/mikata/pull/227
[#228]: https://github.com/Kaginari/mikata/pull/228
[#229]: https://github.com/Kaginari/mikata/pull/229
[#230]: https://github.com/Kaginari/mikata/pull/230
[#231]: https://github.com/Kaginari/mikata/pull/231
[#232]: https://github.com/Kaginari/mikata/pull/232
[#233]: https://github.com/Kaginari/mikata/pull/233
[#234]: https://github.com/Kaginari/mikata/pull/234
[#235]: https://github.com/Kaginari/mikata/pull/235
[#236]: https://github.com/Kaginari/mikata/pull/236
[#237]: https://github.com/Kaginari/mikata/pull/237
[#238]: https://github.com/Kaginari/mikata/pull/238
[#239]: https://github.com/Kaginari/mikata/pull/239
[#240]: https://github.com/Kaginari/mikata/pull/240
[#241]: https://github.com/Kaginari/mikata/pull/241
[#242]: https://github.com/Kaginari/mikata/pull/242
[#243]: https://github.com/Kaginari/mikata/pull/243
[#245]: https://github.com/Kaginari/mikata/pull/245
[#246]: https://github.com/Kaginari/mikata/pull/246
[#247]: https://github.com/Kaginari/mikata/pull/247
[#248]: https://github.com/Kaginari/mikata/pull/248
[#249]: https://github.com/Kaginari/mikata/pull/249
[#250]: https://github.com/Kaginari/mikata/pull/250
[#251]: https://github.com/Kaginari/mikata/pull/251
[#252]: https://github.com/Kaginari/mikata/pull/252
[#253]: https://github.com/Kaginari/mikata/pull/253
[#254]: https://github.com/Kaginari/mikata/pull/254
[#255]: https://github.com/Kaginari/mikata/pull/255
[#256]: https://github.com/Kaginari/mikata/pull/256
[#257]: https://github.com/Kaginari/mikata/pull/257
[#258]: https://github.com/Kaginari/mikata/pull/258
[#259]: https://github.com/Kaginari/mikata/pull/259
[#260]: https://github.com/Kaginari/mikata/pull/260
[#261]: https://github.com/Kaginari/mikata/pull/261
[#262]: https://github.com/Kaginari/mikata/pull/262
[#263]: https://github.com/Kaginari/mikata/pull/263
[#264]: https://github.com/Kaginari/mikata/pull/264
[#265]: https://github.com/Kaginari/mikata/pull/265
[#266]: https://github.com/Kaginari/mikata/pull/266
[#267]: https://github.com/Kaginari/mikata/pull/267
[#268]: https://github.com/Kaginari/mikata/pull/268
[#269]: https://github.com/Kaginari/mikata/pull/269
[#270]: https://github.com/Kaginari/mikata/pull/270
[#271]: https://github.com/Kaginari/mikata/pull/271
[#272]: https://github.com/Kaginari/mikata/pull/272
[#285]: https://github.com/Kaginari/mikata/pull/285
[#286]: https://github.com/Kaginari/mikata/pull/286
[#287]: https://github.com/Kaginari/mikata/pull/287
[#288]: https://github.com/Kaginari/mikata/pull/288
[#289]: https://github.com/Kaginari/mikata/pull/289
[#290]: https://github.com/Kaginari/mikata/pull/290
[#291]: https://github.com/Kaginari/mikata/pull/291
[#292]: https://github.com/Kaginari/mikata/pull/292
[#293]: https://github.com/Kaginari/mikata/pull/293
[#294]: https://github.com/Kaginari/mikata/pull/294
[#295]: https://github.com/Kaginari/mikata/pull/295
[#296]: https://github.com/Kaginari/mikata/pull/296
[#297]: https://github.com/Kaginari/mikata/pull/297
[#298]: https://github.com/Kaginari/mikata/pull/298
[#299]: https://github.com/Kaginari/mikata/pull/299
[#300]: https://github.com/Kaginari/mikata/pull/300
[#301]: https://github.com/Kaginari/mikata/pull/301
[#302]: https://github.com/Kaginari/mikata/pull/302
[#303]: https://github.com/Kaginari/mikata/pull/303
[#305]: https://github.com/Kaginari/mikata/pull/305
[#307]: https://github.com/Kaginari/mikata/pull/307
[#308]: https://github.com/Kaginari/mikata/pull/308
[#311]: https://github.com/Kaginari/mikata/pull/311
