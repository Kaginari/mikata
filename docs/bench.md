# The bench

What we measured, how, and what it changed. Every number on this page comes from runs on our
bench, with the bar written down before the money was spent, the failures included.

## In one picture

Ten real fixes in [sqlglot](https://github.com/tobymao/sqlglot), a large Python SQL library. The
same small open model in every harness, `qwen3.5-35b-a3b`, through the same provider host. Each
task once per harness, November 2026:

| Harness | Passed | Spent | Per pass |
|---|---|---|---|
| **Mnemorph, light-coding** (main 19fa71d, 13 November) | **9 of 10** | $3.87 | **$0.43** |
| Maki 0.7.1 | 8 of 10 | $7.73 | $0.97 |
| OpenCode 1.19.5 | 7 of 10 | $4.72 | $0.67 |
| Mnemorph before the "done" list (1ec6f80, 12 November) | 5 of 10 | $3.97 | $0.79 |

## In plain words

- **Light-coding passed the most and paid the least.** Same model, same tasks, same host: the
  harness is the only difference.
- **Read the count as "8 or 9".** Task 10 passes about one run in five with this model; it passed
  in this run. One run a task means a difference of one or two tasks between harnesses is inside
  the noise. The price per pass is not: it held in every screen.
- **What moved us from 5 to 9 is one habit:** write down what "done" means before any code, and
  check every line of it before answering. The earlier build answered "done" one check short;
  where OpenCode passed and it did not, OpenCode had re-run the tests after its last edit. Now
  light-coding does that every time, from its list.
- **Asks with examples pass twice as often.** The same four tasks: 4 of 4 with the ask's examples
  and expected output, 4 of 8 as one paragraph of prose, at the same price. Without examples the
  model checks against its own reading, and a wrong reading passes its own checks.
- **Separate projects in one ask: one body a project.** Five whole repositories asked as one: one
  light-coding body fixed 2.5 of 5 on average, 3.5 given 400 steps. Light-coding now runs one body
  a project, at once, each in its own folder: 5 and 4 of 5, about 19 minutes and $1.72 a run.
- **The other two modes have not paid yet.** The collective cost several times more for fewer
  passes. The chain, counted only after the fix each screen called for, did less than
  light-coding on work that fits one context, and on five whole repositories at once a little
  more than light-coding as shipped, less than one body given more steps, at three times the
  price; light-coding's split by project beat it. They are
  [experimental](experimental-modes.md), and the page says why.

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
| Ten single fixes (above) | the shipped build against Maki and OpenCode | 9 of 10, $0.43 a pass |
| Five of those fixes, twice each | the end gate runs the project's tests; then the "done" list too | 5 of 10, then **8 of 10**, cheaper |
| Thirty small projects in one ask | one repository, thirty folders, one problem each | **30 of 30**, twice, about 3.5 minutes and $0.10 |
| Four fixes, asked as prose | no examples, no expected output | 4 of 8 (4 of 4 with examples) |
| Five whole repositories in one ask | one sqlglot copy per folder, more than 200 steps of work | 3 and 2 of 5 as shipped, about 12 minutes, $0.75; 4 and 3 of 5 with 400 steps |

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

A prompt sentence does not fix a misreading. The failures that remain are the model deciding a
case should change when the ask says it must stay.

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
[experimental modes](experimental-modes.md).

### What the numbers do not say

- **One model.** Most screens ran on one small open model. A stronger model passes more of these
  tasks from scratch, in any harness; the gaps here are what a harness adds to a model that needs
  help.
- **One kind of work.** Fixes and features in Python libraries with tests. We have not measured
  front-end work, infrastructure, or code with no tests.
- **Small counts.** Ten tasks, run once or twice. We say "ahead" only where the gap is larger than
  what the same harness varies between two runs, and price per pass is the steadier number.
