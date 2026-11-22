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

The same three harnesses, each as released, on two more screens, each task once
([measured](#a-second-codebase)):

| Harness | Ten fixes in mathjs (JavaScript), the same small model | Per pass | The ten sqlglot fixes, a stronger model (Claude Haiku 5.5) | Per pass |
|---|---|---|---|---|
| **Mikata 1.1.0** | **8 of 10** | $0.26 | **10 of 10** | $0.031 |
| Maki 0.7.3 | 7 of 10 | $0.08 | 10 of 10 | $0.015 |
| OpenCode 1.19.6 | 6 of 10 | $0.55 | 10 of 10 | $0.030 |

## In plain words

- **On passes the three harnesses are level: on two codebases, and with two models.** Same
  model, same tasks, same host: the harness is the only difference. By the bar written before
  each screen (ahead at three tasks of ten) no difference is shown, any of the four times.
- **On price there is no one answer, and Mikata is not always the cheapest.** On sqlglot with
  the small model a pass cost Mikata about half what it cost the others, both times. On mathjs,
  whose fixes are smaller, it cost half of OpenCode's and three times Maki's. With a stronger
  model every harness passes every task, and Mikata's pass costs what OpenCode's does and twice
  Maki's. Mikata writes down what done means and checks each line; that takes calls. Where the
  work is hard for the model it has paid for itself in passes. Where the work is small, or the
  model needs no help, it is a cost.
- **The ceiling on those ten tasks was the model's.** The two tasks all three harnesses failed
  with the small model pass in all three with the stronger one.
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
- **A list of what done means closes that gap where we measured it.** The lines say what must
  come out and what must stay. As the product ships it, a reader named and nothing added to the
  ask, the four prose asks passed **10 of 12 on Mikata 1.1.0 as released, against 6 of 12
  without a reader** (12 of 12 on the screen before it, which mixed two builds). Three of the
  four asks pass every time. The fourth turns on the letters the reader happens to write: with a
  list that spells the unit in capitals, as the code prints it, 6 of 6; in small letters, 4 of
  7. Not closed either: a body can hand in a check that only prints the expected words, which
  the pass now names beside its "verified" ([measured](#the-list-held)).
- **The list is the gain, and any harness gets it. Holding the list is not shown to add.** The
  same twelve lists, handed to each harness as a note after the ask and held by nothing: Mikata
  **11 of 12**, OpenCode **8**, Maki **8**, where they passed 6, 5 and 3 without. So the 10 of 12
  above against the others' 5 and 3 was a harness with a second model's help beside two with
  none. Held by Mikata the same lists pass 10, for 42% more dollars a run than as a note. An
  earlier, smaller screen had said the hold was what mattered (a note 2 of 6, the same lines
  held 6 of 6); that note stood before the ask, in other words, on an earlier build. What the
  hold still gives is a pass that names what verified it, and what must stay read from the code
  itself. Neither changed a pass here ([measured](#the-list-held)).
- **A stronger body does not need the list on these asks.** With Claude Haiku 5.5 as the body
  the same four prose asks passed 12 of 12 plain, for two cents a run. A reader added nothing,
  doubled the price, and one run in twelve fixed the code and failed its gate on a wrong line of
  the reader's ([measured](#the-list-held)).
- **A reader is for an ask without examples.** On the ten single fixes, whose asks carry their
  examples, 1.1.0 with a reader passed 7 of the 9 tasks run, where plain passes 7 of the same
  nine: one task won, one lost to a wrong line of the reader, at a third more a run. Against
  OpenCode and Maki that screen stays level ([measured](#the-list-held)).
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
| Four fixes as prose, a reader named, two builds | the build as it ships writes the list itself: one call to a stronger model, shown the ask alone | **12 of 12** (plain: 6 of 12); $0.31 a pass, against $0.59 |
| The same twelve, on 1.1.0 as released | the published archive's own binaries, every run counted | **10 of 12** (plain: 6 of 12); $0.45 a pass, against $0.59 |
| Ten single fixes, 1.1.0 with a reader named | each task once; one task not run under the cap | 7 of 9 (plain on the same nine: 7 of 9); $0.59 a pass, against $0.44 |

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
| A sentence at the hand-in naming the word a list writes in other letters than the code prints | 3 of 6, against 4 of 6 without it |
| A landing failed once for the list's letters, with a way out offered | 2 of 6: the way out taken 5 times in 5 |
| The same, no way out offered | 4 of 6 where we asked 5: the code's letters back 4 times in 4 |

A prompt sentence does not fix a misreading. The failures that remain are the model deciding a
case should change when the ask says it must stay.

**Nor does a sentence fix the reader's letters.** The last three rows are three tries, in one
evening, against the largest loss a reader has cost: a list that writes a unit in small letters
where the code prints capitals, and a body that builds the code to the list. Each try had its
bar first; two lists that had failed on the release, held again, three rounds each.

- **Told at the hand-in,** word by word (`day` where the code prints `DAY`, letter case is set
  aside, keep the code's): every run was told, none followed. One wrote down why it lowercased
  the unit: "DuckDB convention".
- **A landing failed for it,** once, in words that said "unless the ask itself asks for other
  letters" and that the landing would stand if left: the body began to put the capitals back,
  took the list for the ask, and left them, five times in five.
- **The same failure with no way out in it:** the capitals came back four times in four, and no
  tree ended in the list's letters, where 13 of 18 had. It passed 4 of 6, one short of the bar,
  and the two it lost were not the letters. By the bar it does not ship, and the tries end here.

What is kept is about words. This body does not follow a sentence. It does what a failed landing
asks. And it takes any way out the failure offers.

**Nor does a second head of the same kind.** The rows on a picker, on a model settling cases and
on a second reading are one study, on the kept runs of the four prose asks. By code alone, a failed run's own commands show its mistake in 14 of 16
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

Eight steps, each with its bar written first. The first three are on the two prose asks that
fail nearly always (one of them had passed once in the twenty-one kept runs of three harnesses);
the fourth is the product as it ships, on all four; the fifth is the same twelve again on the
release itself; the sixth is the release with a reader on the asks that carry their examples; the
seventh hands the release's own lists to every harness as a note; the eighth gives the release a
stronger body.

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

**The build as it ships, a reader named, all four prose asks.** Every run until here got a list we
had read first, and the counted runs were of a commit older than the release. So the release was
screened as you would get it: `models.reader` set, nothing added to the ask, a fresh list from
the reader in every run, three runs an ask.

| | 01 interval | 02 partition | 03 alter | 04 slice | All | A pass |
|---|---|---|---|---|---|---|
| plain light-coding | 0 of 3 | 2 of 3 | 3 of 3 | 1 of 3 | 6 of 12 | $0.59 |
| **a reader named** | **3 of 3** | **3 of 3** | **3 of 3** | **3 of 3** | **12 of 12** | $0.31 |

The bar asked 4 of 6 on the two asks that fail, 4 of 6 on the two that pass, since a list must
not cost what already works, and no fault of the harness. The first two are met. The third found
a fault, and reading the runs after the count found a limit.

- **The fault: right checks were refused, and the refusals stopped the run.** A reader left to
  itself names an output as the expression alone (`SLICE(x, 2, 3)`); a body's check prints the
  whole statement (`SELECT SLICE(x, 2, 3)`). The rule asked that the line hold the output given,
  and refused it, ten times a run. Each refusal counted as a failed act, five to a turn, before
  any write may land, and [the guts](02-loop.md#stress-in-four-instruments) stopped both runs of
  the slice ask at their 37th call with nothing done. The lists of the earlier steps stated whole
  statements, so nothing was refused there. Fixed before anything was tagged: an output that
  holds the one the line names is taken, word by word, and a check the harness corrects is an
  answer, not a failed act. The two stopped runs' own calls, replayed through the fix with no
  model call: 5 and 6 refused, each rightly, where 10 and 10 were.
- **So the table is of two builds.** By a rule written before the runs it decided, a run of the
  first build counts when it passed (the fault refuses and stops, it cannot help), and a run it
  stopped is run again on the fix, once. Six counted runs are of the first build (asks 01, 02
  and 03, two each) and six of the fix (the slice ask's three, and the third run of each other
  ask). On the fix every run answered. A clean twelve on the fix was not bought.
- **A list did not cost what already passed,** nor more money: 6 of 6 on the two asks that pass
  without one (plain: 5 of 6), and $0.31 a run with the reader's call in it, against $0.30. The
  reader wrote 12 to 28 lines a run for 1 to 3 cents. The checks cost calls before the work (100
  a run at the median, against 94) and save them after.
- **The limit: a check that only prints the words.** In three of the fourteen runs the body
  handed in commands that never run the code: `python3 -c "print('SLICE(x, 2, 3)')"`, and in one
  run an `echo` for each of its 28 lines. The harness took them, answered "It prints that now",
  and the gate said the list had verified the landing. All three pass the hidden tests, so the
  count stands; in those runs the harness held nothing of what must come out. One body did it
  after three honest checks of a line were refused, one after its first real check printed the
  old output, one as its first move when a write was made to wait. None of the fourteen runs of
  the step before did it.
- **What the pass says now.** The harness cannot tell a command that prints the words from a
  case that was right before the work: both give the output on the code as the ask found it. So
  it refuses nothing and names the lines: `verified by the reader's Done when list (28 lines; D1
  to D20 already gave their output on the code you gave it)`. By that rule the twelve passes
  name a line in seven runs, 47 of their 89 lines that must give: the 29 print-only checks, and
  18 that ran the code and printed the same before the work. Most of those print a statement
  back as it went in, which the parser did while it still took the statement for an opaque
  command: a true output, and no sign of the change the line was written for. Nothing a body is
  told has changed, so the twelve runs still describe the build that ships.

**The release itself, the same twelve.** The table above mixes two builds. After 1.1.0 was
published the twelve were run again on the published archive's own binaries, every run counted
and none done again.

| | 01 interval | 02 partition | 03 alter | 04 slice | All | A pass |
|---|---|---|---|---|---|---|
| plain light-coding | 0 of 3 | 2 of 3 | 3 of 3 | 1 of 3 | 6 of 12 | $0.59 |
| a reader named, two builds | 3 of 3 | 3 of 3 | 3 of 3 | 3 of 3 | 12 of 12 | $0.31 |
| **a reader named, 1.1.0 as released** | **1 of 3** | **3 of 3** | **3 of 3** | **3 of 3** | **10 of 12** | $0.45 |

The same bar, met, its first part with nothing to spare: the two asks that fail without a list
pass 4 of 6. No fault of the harness in the two runs that failed. **The count to quote for the
release is 10 of 12;** the first screen's 12 was the luckier draw on one ask.

- **Both failures are the interval ask, and both are the reader's letters.** The ask is prose
  with no example in it, so the reader invents its inputs, and twice it wrote the unit in small
  letters (`'1.5 day'`) where the code, and the upstream fix, print capitals. The harness takes
  either: a stated output holds with letter case set aside. The body made the code print small
  letters all the same. One run answered and fails a hidden test by those letters. The other
  spent its 200 steps on them and never answered. Over every run of this ask that had a list:
  **a list in capitals, 6 of 6; a list in small letters, 4 of 7.**
- **No pass was held by a check that only prints the words,** where the first screen had 3 of
  12. Two runs handed such checks in. One began with an `echo` for each of its 25 lines; each
  printed its line's whole sentence, not the output it had stated, so the gate failed the
  landing, and the body handed in 25 real checks and passed. Over both screens 5 runs of 26
  tried one.
- **The pass names what its rule says,** in all 11 runs that landed one: a line in 5 of the 10
  passes, 21 lines, each a command that runs the code and printed the same before the work.
- **A line that states no exact output can still get a real check.** The body's own wording of
  such a line was refused 37 times in four runs. In three of them it then wrote a command that
  prints the line's own words only when the tree has that shape.
- **Dollars.** $0.37 a run with the reader's call in it, against $0.30 plain; a pass $0.45,
  against $0.59. The run that took all its steps cost $0.86 alone.

**Asks that carry their examples: the ten single fixes.** The reader had only been measured on
prose. So 1.1.0 was run with a reader on the ten single fixes, each task once. Between two plain
screens that count moved from 9 to 7, so the bar was the one used between harnesses, three tasks
of ten: a cost at 4 or fewer, a gain only at 10.

| | Passes | Fails |
|---|---|---|
| Mikata plain, 13 November | 9 of 10 | 06 |
| Mikata plain, 20 November | 7 of 10 | 06, 07, 10 |
| **Mikata 1.1.0, a reader named** | **7 of 9** | 09, 10 (06 not run) |
| OpenCode 1.19.6, 20 November | 7 of 10 | 06, 09, 10 |
| Maki 0.7.3, 20 November | 7 of 10 | 01, 06, 10 |

**No difference shown.** Plain passes 7 of the same nine. Task 06 was not run: the cap was hard,
the runs cost more than counted on, and it is the task no harness has passed on either screen.

- **The task it won:** 07, which plain failed on 20 November because the body's own list
  reworded a check the ask spells out. The reader's first line holds it as the ask writes it.
- **The task it lost, to a wrong line obeyed:** 09, which plain passes both times. The ask is
  about `COLLATE` inside `ARRAY<...>` and `MAP<...>`. The reader added a struct of its own and
  stated its output without the colon Databricks writes (`STRUCT<a STRING ...>` for
  `STRUCT<a: STRING ...>`). It never saw the code. The body changed the generator to drop the
  colon, every struct lost it, and a hidden test of an untouched statement fails. The list
  passed the gate.
- **The list costs where nothing was broken.** On the six tasks that pass both ways: 47% more
  dollars, 16% more calls. On the task that runs out of steps either way, 43 `check` calls came
  before the first change to the code.
- No check that only prints the words was handed in, and the pass names what its rule says in
  all 8 runs that landed one.

Three of the four runs a reader has lost on the release are one thing: **the reader states how
the code spells its output, and it has not seen the code.** Twice a letter case, once a colon.

**The same lists for every harness.** The release with a reader passes 10 of 12 where OpenCode
passed 5 and Maki 3. That set a harness helped by a second model beside two that had none. So
each harness was handed the very list the reader wrote for Mikata's run of the same ask and
round, as a note after the ask: a heading, one sentence ("A second reader wrote these cases from
the request alone, without seeing the code. The work is done when each of them holds."), the
lines. Nothing holds it: in Mikata's twelve runs no `check` was called and every gate reads
"checked by nothing". A difference was set at four runs of twelve.

| | 01 | 02 | 03 | 04 | All | $ a run | $ a pass |
|---|---|---|---|---|---|---|---|
| Mikata 1.1.0, the lists held (the fifth step) | 1 of 3 | 3 of 3 | 3 of 3 | 3 of 3 | 10 of 12 | 0.37 | 0.45 |
| **Mikata 1.1.0, the lists as a note** | 2 of 3 | 3 of 3 | 3 of 3 | 3 of 3 | **11 of 12** | 0.26 | 0.29 |
| **OpenCode 1.19.6, the lists as a note** | 1 of 3 | 2 of 3 | 3 of 3 | 2 of 3 | **8 of 12** | 0.34 | 0.51 |
| **Maki 0.7.3, the lists as a note** | 0 of 3 | 2 of 3 | 3 of 3 | 3 of 3 | **8 of 12** | 0.38 | 0.56 |
| Plain: Mikata, OpenCode, Maki | | | | | 6, 5 and 3 of 12 | 0.30, 0.46, 0.29 | 0.59, 1.10, 1.17 |

- **The list is the gain, in every harness.** Mikata 6 to 11 and Maki 3 to 8 are differences.
  OpenCode's 5 to 8 is under the bar.
- **Holding is not shown to add:** 10 held, 11 as a note, at 42% more dollars a run and ten more
  calls.
- **Mikata holding the lists is not shown ahead of the others handed them:** 10 against 8 and 8.
  Nor is Mikata's note, at 11.
- **We had measured the opposite, and built on it.** In the second step above a note passed 2 of
  6 where the same lines held passed 6 of 6; here the note passes 5 of 6 on those two asks. The
  two notes differ in three ways and nothing here says which matters: the first stood before the
  ask and asked the body to put the lines at the top of its own list, on the build of 20
  November; this one stands after the ask, says who wrote the cases, on the release.
- **The reader's small letters cost a run in every harness.** Of the 8 runs handed a list that
  writes the unit in small letters, 5 were lost to it: both of Mikata's held, both of
  OpenCode's, one of Maki's. Mikata's two as a note passed (one tree prints capitals, one keeps
  the case it is given); that is two runs.

36 runs, each counted on its first try.

**A stronger body.** Everything above has one body. So the release was run on the four prose
asks with Claude Haiku 5.5 as the body, plain and with a reader named, three rounds:

| | Passes, by the hidden tests | Landed | $ a run | Calls a run |
|---|---|---|---|---|
| **Haiku 5.5, plain** | **12 of 12** | 12 | 0.022 | 23 |
| **Haiku 5.5, a reader named** | **12 of 12** | 11 | 0.048 | 25.5 |
| qwen 3.5 35B, plain | 6 of 12 | | 0.30 | 93.5 |
| qwen 3.5 35B, a reader named | 10 of 12 | | 0.37 | 100.5 |

- **No room.** This body passes the prose asks unaided, every time, so a reader has nothing to
  add on them. The feature was not tested here; it was not needed.
- **The reader costs what the run costs.** Its one call is 2.1 cents, the body's whole plain run
  2.2. With this body a reader more than doubles the price for no pass.
- **One reader run in twelve fixed the code and failed its gate.** The reader wrote that the
  very input the ask is about "must stay as it is today". The body's check was piped into
  `tail`, which ends well whatever came before it; the harness took the last line of the error
  for what the command prints and held the line hard to it. The reader's line was wrong, and
  reading a failed command as one that ran was ours: a check is now judged by the first stage
  of its pipeline ([the loop](02-loop.md#done-when)). Not screened.

24 runs, each counted on its first try, $0.85.

**What is not closed.** A check that only prints the words is named, not refused, and only for a
line that must give: the eight lines that must stay in the run of 28 `echo`s are named by
nothing. A line that states no exact output (`must give a list holding exactly one ... node`,
nine of the reader's lines in four runs) is held to the body's own wording of it. An input no
line names. A check whose command does not test what its line says: the harness cannot tell.
And a reader's spelling: it never saw the code, so it wrote a
unit in small letters where the code prints capitals. The harness holds a stated output with
letter case and blanks set aside, so it no longer forces the wrong letters, but the body builds
to them: every failure of that ask with a list is a list in small letters (4 of 7 pass, against
6 of 6 in capitals). The tool's own words now tell the body to keep the code's spelling, and
with them two small-letter lists in three were still built to the letter. Letter case is the
one spelling the harness sets aside: a colon it cannot, and a struct stated without the colon
the code writes cost a task that passes plain. The first live reading, on a
four-line function, also held one wrong line of thirteen: an input the ask itself changes, listed
as one that must stay. So a line whose command failed on the code as found, and no longer does,
is said once and refuses nothing. That does not make a wrong line harmless. On the third live
run the reader wrote that any other function called with a blank must stay as it is; `mean`
calls `total`, and so began to work. Told once, with nothing refused, the body still added a
check to `mean` to keep it failing. The list decides: read the one a reader writes.

Six runs an arm on two asks for the first three steps, $10.77; fourteen runs on four asks for the
fourth, $3.90; twelve for the fifth, $4.47; nine for the sixth, $4.15; thirty-six for the
seventh, $11.72; twenty-four for the eighth, $0.85. One codebase, asks that are not fresh, one
model as the reader, and but for the eighth step one as the body.

### A second codebase

Everything above is one Python codebase, sqlglot. So the bench got a second one in another
language: [mathjs](https://github.com/josdejong/mathjs), about 250,000 lines of JavaScript. Ten
upstream fixes of 2025 and 2026, each asked with what the code prints today and what is
expected, each judged by the fix's own unit tests. The same small model, each harness as
released, each task once. The bar was the one used on sqlglot: a difference is three tasks of
ten.

| | Passes | Fails | $ a run | $ a pass | Calls a run |
|---|---|---|---|---|---|
| **Mikata 1.1.0** | **8 of 10** | 04, 10 | 0.21 | 0.26 | 31.5 |
| Maki 0.7.3 | 7 of 10 | 04, 08, 10 | 0.05 | 0.08 | 27.5 |
| OpenCode 1.19.6 | 6 of 10 | 01, 04, 07, 10 | 0.33 | 0.55 | 21 |

- **Level on passes, again.** No difference shown. Two tasks fail in all three.
- **Not the cheapest here.** A pass cost Mikata about half of OpenCode's and more than three
  times Maki's. On sqlglot Maki's pass cost $0.91 and Mikata's $0.50. These fixes are smaller:
  five of Mikata's ten runs cost under five cents. Mikata made 590 calls with 55,000 prompt
  tokens each, Maki 300 with 26,000. On a small fix, writing down what done means and checking
  each line costs more than it saves.
- **One task may be owed to Maki.** Its world had no test runner left when it was judged: the
  run had removed or replaced `node_modules`, which nothing can fetch again with no network, so
  its fix was never judged. Mikata's own gate refuses an install; the other two run with every
  command approved. That is a fault of our suite, counted as it ran and fixed for the next
  screen: the suite's `node_modules` are put back before a world is judged. With it the passes
  read 8, 7 or 8, 6: level either way.
- **Mikata's two failures are the model's.** In both the ask states the output the tree does
  not give. Reading them found a fault of ours beside them, the next paragraph.

**What the second codebase found in Mikata itself.** Three faults, none of which a Python
project could show, each fixed with a test that fails without it:

- **The harness's own copies of the code had no `node_modules`.** "What changed" runs the
  body's checks again in two copies, and a `must stay` line handed in late is read from a copy.
  A copy left `node_modules` out for its size. Over Mikata's ten runs, 8 of 70 checks ran in
  the copies, 62 were counted as "fails on the finished work", and nothing was shown in ten
  runs of ten. A copy is now lent the world's `node_modules`, to read and never to write. The
  same ten runs asked again with no model call: 61 of 70 run, 30 print differently, and both
  failed runs show a wrong value.
- **A failing check was told "it said nothing".** A check written with `2>&1` says its error in
  its output, and Node ends every uncaught error on its own version. One run fixed the code,
  never learned which file its checks could not import, and ended on its step limit. A failing
  check is now told the line where its error names itself.
- **A check piped into `tail` was read as one that ran,** whatever its command did. Seen with
  the stronger body above; the same rule the stall detector already had.

**Prose asks, with a reader.** Four of the ten asks cut to a paragraph of prose, three rounds,
on the build that holds the first of those fixes. A difference was set at four runs of twelve.

| | 01 | 02 | 03 | 05 | Passes, by the hidden tests | Landed | $ a run | Calls a run |
|---|---|---|---|---|---|---|---|---|
| Plain | 0 of 3 | 2 of 3 | 3 of 3 | 3 of 3 | 8 of 12 | 11 | 0.15 | 39.5 |
| **A reader named, its list held** | 2 of 3 | 2 of 3 | 3 of 3 | 3 of 3 | **10 of 12** | 9 | 0.32 | 102.5 |

- **No difference shown,** and there was room for one. The whole gap is the first ask, 2 of 3
  against 0 of 3.
- **Held, a run costs 2.1 times the dollars** and 2.6 times the calls.
- **Two of the reader arm's ten passes did not land.** One never learned why its checks could
  not run (the second fault above). One was held to two wrong lines of the reader's: an
  expression spelled as the reader guessed it, and a value the fix itself changes listed as one
  that must stay. The body handed in the code's real output four times and was refused.
- **The reader's lists were nearly all right:** 3 wrong lines of 236. All three state something
  about code the reader has not seen, as on sqlglot.
- **A list can hold nothing.** 41 of the 236 checks never touch the code. One run passed and
  landed with every one of its 21 checks an `echo`.

30 runs for the ten fixes, $5.96; 24 for the prose asks, $5.73. One model as the body. The asks
are ours, written that day, and nobody else has read them.

### A second body

"Level on passes, about half the price a pass" also rested on one body. So the three harnesses
were run on the same ten sqlglot fixes with Claude Haiku 5.5 as the body, each as released, each
task once. Before a harness's counted runs, one run of it was read by code: tokens by kind, and
our dollars within a tenth of the harness's own.

| | Passes | $ a run | Calls a run | Seconds a run | Prompt tokens a call |
|---|---|---|---|---|---|
| **Mikata 1.1.0** | **10 of 10** | 0.031 | 26 | 229 | 39,000 |
| OpenCode 1.19.6 | 10 of 10 | 0.030 | 21.5 | 162 | 52,000 |
| Maki 0.7.3 | 10 of 10 | 0.015 | 13 | 110 | 30,000 |
| The small model, 20 November: Mikata, OpenCode, Maki | 7 of 10 each | 0.41, 0.74, 0.59 | 114, 176.5, 106.5 | | |

- **Every task passes in every harness.** Tasks 06 and 10, which all three failed with the small
  model, pass in all three. That ceiling was the model's. On passes this screen cannot tell the
  harnesses apart, and we said before it ran that it would then say nothing about them.
- **Mikata is not cheaper with this body.** Its pass costs 103% of OpenCode's and 204% of
  Maki's. The bar for "about half" was 60%, for "cheaper" 90%. All three read 92% to 96% of
  their prompt from the cache: the price turns on calls, and Mikata takes the most.
- **Maki finishes in half Mikata's calls and half its time.**

30 runs, each counted on its first try, $0.79. One stronger body, a small model of a large
maker; nothing here says what a larger one does.

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

- **One model, mostly.** Most screens ran on one small open model. A stronger model passes more
  of these tasks from scratch, in any harness; the gaps here are what a harness adds to a model
  that needs help. Where we gave Mikata a stronger body (Claude Haiku 5.5), the prose asks the
  small model passes half the time passed 12 of 12 with no help, for two cents a run, and the
  reader had nothing left to add ([measured](#the-list-held)); and on the ten single fixes all
  three harnesses passed everything ([measured](#a-second-body)).
- **One kind of work.** Fixes in two libraries with tests, one in Python and one in JavaScript.
  We have not measured front-end work, infrastructure, or code with no tests.
- **Small counts.** Ten tasks, run once or twice. We say "ahead" only where the gap is larger than
  what the same harness varies between two runs. A price per pass held between two screens of
  the same tasks and model; it did not hold across codebases or models, and we no longer quote
  one as the harness's own.
