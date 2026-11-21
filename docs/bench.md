# The bench

What we measured, how, and what it changed. Every number on this page comes from runs on our
bench, with the bar written down before the money was spent, the failures included.

## In one picture

Ten real fixes in [sqlglot](https://github.com/tobymao/sqlglot), a large Python SQL library. The
same small open model in every harness, `qwen3.5-35b-a3b`, through the same provider host. Each
task once per harness, twice a week apart, each harness at its version of the day, November 2026:

| Harness | 13 November | 20 November | Both | Per pass |
|---|---|---|---|---|
| **Mikata, light-coding** | 9 of 10 (main 19fa71d) | 7 of 10 (main b43cbdd) | **16 of 20** | **$0.50** |
| Maki | 8 of 10 (0.7.1) | 7 of 10 (0.7.3) | 15 of 20 | $0.91 |
| OpenCode | 7 of 10 (1.19.5) | 7 of 10 (1.19.6) | 14 of 20 | $0.87 |

Before the "done" list, on 12 November, Mikata passed 5 of 10 at $0.79 a pass.

## In plain words

- **On passes the three harnesses are level. Mikata pays about half.** Same model, same tasks,
  same host: the harness is the only difference. By the bar written before each screen (ahead at
  three tasks of ten) no difference is shown, either time. The price a pass is the steadier
  number, and it held in every screen.
- **A count moves by one or two between runs.** Mikata passed 9, then 7. Two tasks fail for all
  three harnesses on the second screen, and the step limit is not why: the others took more
  calls on them and failed too.
- **What moved us from 5 to 9 is one habit:** write down what "done" means before any code, and
  check every line of it before answering. The earlier build answered "done" one check short;
  where OpenCode passed and it did not, OpenCode had re-run the tests after its last edit. Now
  light-coding does that every time, from its list.
- **Asks with examples pass twice as often.** The same four tasks: 4 of 4 with the ask's examples
  and expected output, 4 of 8 as one paragraph of prose, at the same price. Without examples the
  model checks against its own reading, and a wrong reading passes its own checks. That is the
  model's, in any harness: on those prose asks, three runs each, Mikata passed 6 of 12, OpenCode
  5, Maki 3, and on one ask all eight failures of the three break the same line.
- **A list of what done means, held by the harness, closes that gap where we measured it.** On
  the two prose asks that fail nearly always: 1 of 6 plain; 2 of 6 with a stronger model's lines
  handed over as a note; **6 of 6 with the same lines held by the harness.** The lines say what
  must come out and what must stay; the harness reads what must stay from the code itself, so the
  body is never the one who says what "today" is ([measured](#the-list-held)).
- **The list of what changed is worth reading.** On those prose asks a wrong value is in its
  twenty lines in 18 of 26 failed runs, and one run in four that passes its tests shows a break
  the tests missed. Point at one wrong line and ask again: the work passes 6 times in 18. Ask
  again without pointing: never ([measured](#what-changed-measured)).
- **Separate projects in one ask: one body a project.** Five whole repositories asked as one: one
  light-coding body fixed 2.5 of 5 on average, 3.5 given 400 steps. Light-coding now runs one body
  a project, at once, each in its own folder: 5 and 4 of 5, about 19 minutes and $1.72 a run.
- **The other two modes have not paid yet.** The collective cost several times more for fewer
  passes. The chain, counted only after the fix each screen called for, did less than
  light-coding on work that fits one context, and on five whole repositories at once a little
  more than light-coding as shipped, less than one body given more steps, at three times the
  price; light-coding's split by project beat it. They are
  [experimental](experimental/modes.md), and the page says why.

## Technical detail

### How a screen is run

- **The bar comes first.** Before a paid run, a note says what result would count as "the change
  pays", what would not, and when the run stops. The verdict is read against it, never adjusted
  after.
- **One host.** Every call goes to the same provider host, so a slower or different deployment
  never passes for a harness difference.
- **Every row is kept:** the tree it left, its journal, its calls, and dollars as the provider
  billed them. A run that stops or crashes is counted, as a failure.
- **Hidden tests decide.** A task passes when the project's own tests for that fix pass and
  nothing that passed before fails. The harness never sees them.
- **Pre-flight.** Each arm's world opens a session with its config before any model call, so no
  run dies at its first call to a setting we got wrong.

### Light-coding, screen by screen

| Screen | What changed | Result |
|---|---|---|
| Ten single fixes, 13 November | the build of the day against Maki and OpenCode | 9 of 10, $0.43 a pass |
| Five of those fixes, twice each | the end gate runs the project's tests; then the "done" list too | 5 of 10, then **8 of 10**, cheaper |
| Thirty small projects in one ask | one repository, thirty folders, one problem each | **30 of 30**, twice, about 3.5 minutes and $0.10 |
| Four fixes, asked as prose | no examples, no expected output | 4 of 8 (4 of 4 with examples) |
| Five whole repositories in one ask | one sqlglot copy per folder, more than 200 steps of work | 3 and 2 of 5 as shipped, about 12 minutes, $0.75; 4 and 3 of 5 with 400 steps |
| Ten single fixes again, 20 November | each harness at its version of the day | 7 of 10 each; $0.59 a pass, against $0.84 and $1.06 |
| Four fixes as prose, three harnesses | three runs each, nobody told how to run the tests | 6 of 12, against 5 and 3; $0.59 a pass, against $1.10 and $1.17 |
| The two prose asks that fail nearly always | a `Done when` list in the ask, held by the harness | **6 of 6**, with two lists (plain: 1 of 6) |

On the largest work the single body never filled its context: no run was compacted. It ran out of
steps, not memory, and with more steps it finished on its own.

### What did not help

Measured, and left out of the product:

| Tried | Result |
|---|---|
| Four process lines in the prompt (find the tests, write your own list, see the bug fail first, change only what is asked) | 2 of 8 on prose asks, against 4 of 8 |
| One line: "run what must not change before any change" | 3 of 9 on prose asks |
| Statement rows: a script of the ask's claims, run after every write, saying which ones the change broke | 1 of 6, same as without |
| Raising the step and context limits on small work | 27 and 28 of 30, against 30 and 30: more room, less checking |
| Three runs of one prose ask and a picker (replayed from 25 kept runs, no model call) | at best 0.60 with a perfect picker, 0.50 picking by agreement, against 0.43 for one run, at three times the price |
| A model settling the cases where those runs disagree | right 67% and 63% of the time (two models), where we asked for 80% |
| A second reading: each answer a run's own commands print differently after its work, judged against the ask by a model | 8 and 9 of 16 failed runs caught, where we asked for 10; 1 to 3% false alarms |
| The body's own "done" list, its wrong lines corrected by a stand-in for you before the work | 1 of 6 (approved as it was: 0 of 3) |
| A stronger model's cases, from the ask alone, handed to the body as a note | 2 of 6, against 1 of 6 |
| The body's own model's cases, written in a call of its own, as a note | 0 of 3 |

A prompt sentence does not fix a misreading. The failures that remain are the model deciding a
case should change when the ask says it must stay.

**Nor does a second head of the same kind.** The last three rows are one study, on the kept runs
of the four prose asks. By code alone, a failed run's own commands show its mistake in 14 of 16
runs, once their output on the starting code is set beside their output on the finished work. But
a model asked to judge that list reads the ask as the coder did: it catches a stated limit the
work crossed (57% to 67% of such cases) and takes a wrong new value for the right one (29 and 23
times in 32). Another model reads a little better than the same one, at a fifteenth of the price,
and not enough. What the ask leaves out, no reader puts back; with its examples the same four
asks passed 15 of 16. So that list is now shown to the one reader who can judge it, you
([the loop](02-loop.md#what-changed-by-its-own-checks)), and it was measured next.

### What changed, measured

The list of what changed moves no pass rate: it is for you. So it was measured two ways.

**Is the mistake in what you are shown?** The code replayed with no model call over 42 kept runs
of the four prose asks, every check of each run again on the finished work, the starting code,
the upstream fix and three runs that passed. A wrong value is a new line the fix does not print
in its place, that is not the same line with the body's own mark flipped, and that holds
something to judge. 26 of the runs failed their hidden tests, 16 passed.

| A wrong value is | failed runs (26) | passed runs (16) |
|---|---|---|
| somewhere in the newest 12 checks that differ | 22 | 7 |
| in the twenty lines, as first built | 14 | 4 |
| in the twenty lines, a value before a mark | **18** | 4 |
| and the checks that run only on the finished work, shown after | **18** | 5 |

Where a body grades itself (`FAIL` turned `PASS`, "All tests passed!"), those lines took the list
and the value that gives the mistake away sat just past the twentieth line: so a value comes
first. Nothing else by code helped. Each kind of change once, the rarest first, the checks taking
turns, one line above instead of two, 24 checks, every check, 30 lines: tuned on the 25 older
runs, and the rule taken there gained nothing on the 17 newer ones. On 44 more runs that no rule
was chosen on, the order taken shows a wrong value in 11 of 36 failed runs against 9. The first
build also broke its own limit, 59 lines in one run; a place is now cut.

**The passed runs it flags are mostly right to be flagged.** In four of the five, every reference
refuses the line: a MySQL `ALTER TABLE` with several actions that parsed before and is an opaque
command now, `excluding` as a column no longer written in a named window, a slice with a step
rewritten, a node of another type than the ask named. The hidden tests cover none of them. One
run in four that passes its tests holds a departure only this list shows.

**Does Mikata repair what you point at?** No bench here has a human, so it was given a stand-in
that knows what was wanted and speaks only through what the list shows: the first wrong line,
word for word, and what it should be. In the trees of the 18 failed runs whose list shows a
wrong value, one fresh run each:

| | passes | the line pointed at now right |
|---|---|---|
| the stand-in points at one line | **6 of 18** | 12 of 18 |
| told only that an earlier attempt left its work | **0 of 18** | 0 of 18 |

The bar written first asked 9 of 18, and 7 more than the control: **missed**. The pointer works
on the line, two times in three, and never happens without it: fourteen of the control's
eighteen runs fail exactly where the first run did. But half the repairs do not finish the work.
One wrong line is seldom the whole mistake, and on the interval ask three runs made the pointed
case right by giving up the case the ask was about. Pointing at every wrong line and saying the
rest stays did no better (2 of 8, where one line gave 3).

So the list does what it is for: you see a wrong answer before you use it, in 18 of 26 failed
runs here, and in one passed run in four you see what the tests missed. Whether Mikata then puts
it right in one more ask is the model's to do, one time in three. $6.63, and nothing of it
changed what the body is told.

### The list held

Three steps, each with its bar written first, on the two prose asks that fail nearly always (one
of them had passed once in the twenty-one kept runs of three harnesses).

**The body's own list, corrected.** Every failed run's first "done" list held wrong lines. A
stand-in for you corrected each wrong line before the work, by what the upstream fix does: 1 of
6. Read by code, five of the six runs took every correction, and failed on an input their list
did not hold: no list had the case that tells the rule from its nearest misreading. A body builds
to the cases it holds, faithfully. The gap is in which cases those are.

**Someone else's cases, as a note.** A model asked only for the cases, shown the ask alone:

| Who wrote the cases | Lines right by the upstream fix | Given to the body as a note |
|---|---|---|
| a stronger model (Claude Sonnet 5.5), one call, 1 to 3 cents | every line, two prompts, four asks | 2 of 6 |
| the body's own model, in a call of its own | one list wrong by a letter | 0 of 3: all three built to the wrong letter |

In all six runs every line that says what must come out is right on the finished tree: 39 of 39.
Two of the four failures broke lines the note said must stay. One read "must stay as it is today"
as "must come out as it went in", wrote that down as the expected output, saw it fail on the
untouched code, and changed the code until it passed. Neither ran such a line on the code before
changing it, though the note asked for that. The other two failed on inputs no line held.

**The same lines, held** ([the loop](02-loop.md#a-done-when-list-in-the-ask)). The harness reads
each line that must stay from the code as the ask found it, tells the body what it prints today,
and runs every line again when the answer lands:

| | Passes | A run |
|---|---|---|
| plain light-coding | 1 of 6 | $0.26, 90 calls |
| the lines as a note | 2 of 6 | $0.29, 94 calls |
| **the same lines, held** | **6 of 6** | $0.30, 100 calls |
| **the lines of the reader as the product asks it, held** | **6 of 6** | $0.39, 113 calls |

The bar asked 4 of 6 for each, and the gate seen failing a landing for a line that must stay.
All met. What the runs show beyond the count:

- **It works mostly by coming first.** In all twelve runs every line's check was in the
  harness's hands before the code changed, in ten of them after one write had been refused. Ten
  of the twelve first landings passed. No run broke a line after the harness had read it and
  said what it prints today.
- **The misreading was tried again, and stopped.** One body saw its work had changed what a line
  that must stay prints, and handed the line in again with the new output as what it "gives".
  Refused, twice: what a line prints today is not the body's to say. The gate failed its landing;
  it found the real cause, a check command that tested another case than its line, and passed.
- **One run of the twelve never answered.** Same trap, and the body did not find the command in
  its 200 steps. Its tree passes the hidden tests; you would have had no answer, and the line
  and its two outputs on your screen.

**What is not closed.** An input no line names. A check whose command does not test what its
line says: the harness cannot tell. And a reader's spelling: it never saw the code, so it wrote a
unit in small letters where the code prints capitals. The harness holds a stated output with
letter case and blanks set aside, so it no longer forces the wrong letters, but on the build as
it ships one run of two made the code print them anyway, and failed. The tool's words to the body
said "that exact output" in every run counted here; they now say what the harness does, and that
is not measured. The first live reading, on a
four-line function, also held one wrong line of thirteen: an input the ask itself changes, listed
as one that must stay. So a line whose command failed on the code as found, and no longer does,
is said once and refuses nothing. That does not make a wrong line harmless. On the third live
run the reader wrote that any other function called with a blank must stay as it is; `mean`
calls `total`, and so began to work. Told once, with nothing refused, the body still added a
check to `mean` to keep it failing. The list decides: read the one a reader writes.

Six runs an arm, two asks, one codebase. $10.77 for the three steps. Nothing here says what a
list does to an ask that already passes.

### The modes against each other

Each mode against light-coding, on the same tasks, same model and host. Structured-coding counts
only runs made after the fix its screen called for; a run that hit a since-fixed failure measured
the failure, not the mode:

| Work | Light-coding | Structured-coding | Collective-coding |
|---|---|---|---|
| Multi-part fixes (composite) | 3 of 8, $0.62 | not run since the work board and the gate fix | 1 of 4, $2.02 |
| Two-part fix, after the chain's fixes | pass, 6 min, $0.32 | half done at the 45-minute clock, $1.99 | — |
| Thirty small projects, after the tool fixes | 30 of 30, $0.10 | 28 of 30, 7.5 min, $0.33 | — |
| Five whole repositories, handed out by domain | 2.5 of 5 on average, 12 min, $0.75 (3.5 with 400 steps, about $1) | 3.3 of 5 on average (3, 4, 3), 17 to 40 min, $2.36 a run | — |
| Five whole repositories, light-coding one body a project | 4.5 of 5 on average (5, 4), 19 min, $1.72 a run | — | — |

The bench's own reports name the modes `solo`, `hierarchy` and `collective`, the names they had
when they ran. Why they lose, and what we are building next, is on
[experimental modes](experimental/modes.md).

### What the numbers do not say

- **One model.** Most screens ran on one small open model. A stronger model passes more of these
  tasks from scratch, in any harness; the gaps here are what a harness adds to a model that needs
  help.
- **One kind of work.** Fixes and features in Python libraries with tests. We have not measured
  front-end work, infrastructure, or code with no tests.
- **Small counts.** Ten tasks, run once or twice. We say "ahead" only where the gap is larger than
  what the same harness varies between two runs, and price per pass is the steadier number.
