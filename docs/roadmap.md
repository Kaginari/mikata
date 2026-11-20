# Roadmap

What comes next for Mikata, in order. Each step is measured on [the bench](bench.md) before the
book says anything of it: a bar written first, then the runs, then the verdict, whatever it says.
How everything so far was built, milestone by milestone, is [M0 to M14](experimental/roadmap-m0-m14.md).

## Now: 1.0.0, as Mikata

Mnemorph is renamed Mikata: the binaries, the crates, the npm packages, the world folder and the
repository. Old worlds and homes are moved by themselves the first time Mikata runs. Next, the
1.0.0 release ([releasing](releasing.md)) and the site.

## Next: asks without examples

The largest gap we have measured. An ask with examples of its expected output passed 4 times in 4;
the same ask as one paragraph of prose, 4 times in 8. The failure is a misreading: the body turns
its own reading of the ask into its checks, they pass, and it answers done. Four lines added to the
prompt did not help (2 of 8), so the next try changes what the body does, not what it is told.

A second head does not close it either ([the bench](bench.md#what-did-not-help)): three runs and a
picker, a model settling their disagreements, and a model auditing what one run changed all
missed the bars we wrote for them. Two things held. By code alone, the list of what a body's own
commands print before and after its work shows the mistake in 14 of 16 failed runs. And where two
runs contradict each other on a case, the ask is unclear there. Both are things to show you, not
things a model can settle.

**Built: the list.** A landed answer is followed by what changed, by the body's own checks
([the loop](02-loop.md#what-changed-by-its-own-checks)). **Measured**
([the bench](bench.md#what-changed-measured)): a wrong value is in its twenty lines in 18 of 26
failed runs, and in one passed run in four it shows a break the tests missed. A stand-in human
pointing at one wrong line got the work passing in 6 of 18 runs, where a second try alone got 0:
under the bar we wrote, so the list stays what it is, a thing to read, and no loop is built on
it. Open: it is silent when a run stops with no answer, and on an ask that adds a syntax the
failed runs show no wrong value in twenty lines by any order we tried.
**Not built: the one case.** Two bodies that contradict each other on a case, put to you as a
question, costs a second run; it waits until the list has been lived with.

## Then: a fold for long runs

When a run's context fills, Mikata today summarises the whole history, at most twice, and then
stops at a checkpoint. On one model that wall was the only cause of failure (3 of 13 runs). The plan
borrows the mechanism of *Model-Driven Incremental Hierarchical Compression* (Ran, 2026): fold only
the new stretch of the run, append it after the ask, keep every folded stretch restorable to a file.
First the harness folds; only then, if a screen says it pays, the body gets its own `fold` tool.

## Later: the split, wider

Today an ask is split only across folders at the root of the repository that each hold a project of
their own. Whether the same split pays inside one repository, across the packages of a workspace,
is not measured yet.

## Kept, not developed: the lab

Structured-coding and collective-coding stay usable and documented in [the lab](experimental/),
and frozen: no work and no bench runs on them until there is work where structure should clearly
win.
