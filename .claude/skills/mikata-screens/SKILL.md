---
name: mikata-screens
description: Mikata's record of screens — what was already tried for each problem of the harness, with which model on which codebase, and what came of it. Use before proposing, building or measuring any change to how the harness behaves (a unit, the prompt, the gate, memory, a default, a profile), and whenever a bar or a verdict is written.
---

# The record of screens

Mikata changes one screen at a time: a bar written first, then the runs, then the verdict,
whatever it says ([the bench](../../../docs/bench.md#how-a-screen-is-run)). The record,
`bench/screens.jsonl`, keeps every screen under the problem it was for. The harness alone does not
say why a unit is there, what it was meant to fix, or whether it did.

## Before you propose a change

1. **Read the record.** `python3 bench/screens.py show` prints it whole; `--problem <id>` one
   problem, `--model <text>` the screens of one body.
2. **Name the problem your change is for,** by its id, in the pull request. When none explains
   the failure you saw, name a new one, as a cause and not a remedy:
   `python3 bench/screens.py problem <id> "<what goes wrong>"`.
3. **Read what was tried under it, with its setting.** A verdict holds for its model, suite and
   build. Give a screen on the same model and codebase more weight than one on another, and
   little to one where both differ.
4. **Start from the reason, not from the remedy.** A reason that still holds may need another
   remedy with another model; a remedy that shipped may need revising or taking out.
5. **Do not try again what missed** unless something new says why it would work now. Say what is
   new.

## When you measure it

6. **Write the bar before any paid run, and commit it before the runs:**

       python3 bench/screens.py bar <id> --problem <id> --what "<the change>" \
         --why "<why it should help>" --bar "<what result counts, and what does not>" \
         --model "<body, and reader if any>" --suite "<codebase and asks>" \
         --build "<version (commit), as `mikata status` prints it>" [--runs N] \
         [--predicted <task>,<task>]

   `--predicted` names the tasks you expect it to fix; the verdict says which it did.
7. **Write the verdict after, whatever it says:**

       python3 bench/screens.py verdict <id> --observed "<what was seen>" \
         --met yes|no|partly --decision shipped|switch|not-shipped|removed|kept|open \
         [--passed 10/12] [--control 6/12] [--fixed …] [--missed …] [--source docs/bench.md#…]

   A miss is recorded as a pass is. `switch` is in the product and off by default.
8. **Never edit a line.** A row found wrong is withdrawn
   (`python3 bench/screens.py withdraw <id> "<why>"`) and written again; two problems found to be
   one are merged (`merge <from> <into> "<why>"`).

`python3 bench/screens.py check` holds the record well-formed and is part of `just bench`;
`check --since origin/main` holds that your branch only added lines. The book still tells the
screen in words ([the bench](../../../docs/bench.md)); the record is where the next proposer
finds it.
