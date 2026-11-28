#!/usr/bin/env python3
"""The record of screens: every change to the harness that was measured, kept under the problem
it was for.

    python3 bench/screens.py show [--problem ID] [--model TEXT] [--json]
    python3 bench/screens.py check [--quotes] [--since REV]
    python3 bench/screens.py problem ID "what goes wrong, said as a cause"
    python3 bench/screens.py bar ID --problem ID --what … --why … --bar … --model … --suite … --build …
                             --predicted TASK,TASK | --unpredicted WHY
    python3 bench/screens.py verdict ID --observed … --met yes|no|partly --decision …
    python3 bench/screens.py merge FROM INTO "why they are one problem"
    python3 bench/screens.py withdraw ID "why the row is wrong"

A screen is one change tried against a bar written first ([the bench](../docs/bench.md)). The book
tells each screen once, where it was run; this file keeps them all in one place, by the problem
each was for, so whoever proposes the next change — a person or an agent — reads what was already
tried for that problem, in which setting, and what came of it. The harness alone does not say why
a unit is there, what it was meant to fix, or whether it did.

The record is `bench/screens.jsonl`, one event a line, appended and never rewritten:

- `problem`: a recurring fault of the harness or of a body in it, named as a cause, with an id.
- `screen`: a change tried for a problem — what, why, the setting it ran in (model, suite, build,
  runs), **the bar** and **the prediction**, both written before any paid run: the tasks it is
  expected to fix (`predicted`), or why it can name none (`unpredicted`: a change for cost alone,
  say). Its verdict then says what came of each predicted task (`fixed` or `missed`).
- `verdict`: what a screen came to — what was observed, whether the bar was met, and what became
  of the change (`shipped`, `switch`: in the product and off by default, `not-shipped`, `removed`,
  `kept`: nothing changed, `open`). It is a line of its own, after its screen: a bar is never
  edited to fit what was seen, and a change that missed is recorded as one that was accepted.
- `merge`: two problems found to be one; the first id lives on as a name for the second.
- `withdraw`: a row found wrong is said to be, by a new line; the row stays.

A verdict holds for its setting. A change earned with one model on one codebase is a guess with
another: the plan passed 8 of 10 against 5 with the small model in November and earned no pass on
any later screen, with either body, on either codebase. So `show` prints every screen with its
model, suite and build, and a change is screened again before it is leaned on elsewhere.

`check` holds the record well-formed: every line an event, every reference resolved by an earlier
line, a bar and a prediction on every screen that was not read back from the book (`backfilled`),
one verdict a screen, dated no earlier than it, and every predicted task in its `fixed` or its
`missed`. `--since REV` also holds it append-only: the file at `REV` is the
beginning of the file now. `--quotes` holds a backfilled row to the book: each of its `quotes` is
found, word for word, in the file its `source` names.

The shape is ported from Self-Improvement Memory (Yu et al., 2026, arXiv 2611.18856): a memory
for whoever improves a harness, grouped by problem, each attempt with its rationale, its
prediction made before the evaluation, its outcome and where it was run. That work found a
harness carried to another model or benchmark continues better with this history than with the
harness alone, or with the problems' names alone. The rules are ported; no code is, and nothing
here runs a model. A prediction checked after the runs is that work's own rule: a screen that
names the tasks it should fix, and is then told which it fixed, says whether the change worked for
the reason given, and not only whether the count moved.
"""

import argparse
import datetime
import json
import re
import subprocess
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parent
ROOT = BENCH.parent
RECORD = BENCH / "screens.jsonl"

ID = re.compile(r"[a-z0-9]+(-[a-z0-9]+)*\Z")
ID_MAX = 48
DATE = re.compile(r"\d{4}-\d{2}(-\d{2})?\Z")
MET = ("yes", "no", "partly")
DECISIONS = ("shipped", "switch", "not-shipped", "removed", "kept", "open")

# Every event's fields: the ones it must carry, and the ones it may.
FIELDS = {
    "problem": (("id", "date", "name"), ("backfilled", "source", "quotes")),
    "screen": (
        ("id", "problem", "date", "what", "why", "bar", "setting"),
        ("predicted", "unpredicted", "backfilled", "source", "quotes"),
    ),
    "verdict": (
        ("screen", "date", "observed", "met", "decision"),
        ("passed", "control", "fixed", "missed", "usd", "backfilled", "source", "quotes"),
    ),
    "merge": (("from", "into", "date", "why"), ()),
    "withdraw": (("id", "date", "why"), ()),
}
SETTING = (("model", "suite", "build"), ("runs",))


def read(path):
    """The record's lines, each with its number from 1; blank lines are not rows."""
    if not Path(path).exists():
        return []
    with open(path, encoding="utf-8") as f:
        return [(n, line.rstrip("\n")) for n, line in enumerate(f, 1) if line.strip()]


def text(v):
    return isinstance(v, str) and v.strip() != ""


def tally(v):
    return (isinstance(v, list) and len(v) == 2 and all(type(x) is int for x in v)
            and 0 <= v[0] <= v[1] and v[1] > 0)


def names(v):
    return isinstance(v, list) and all(text(x) for x in v)


def full(date):
    return len(date) == 10


def shape(row):
    """What is wrong with one event read alone; nothing when it is well-formed."""
    if not isinstance(row, dict):
        return ["not an object"]
    event = row.get("event")
    if event not in FIELDS:
        return [f"unknown event {event!r}: one of {', '.join(FIELDS)}"]
    must, may = FIELDS[event]
    out = [f"{event}: no `{k}`" for k in must if k not in row]
    out += [f"{event}: unknown field `{k}`" for k in row if k not in must + may + ("event",)]
    if out:
        return out
    for k in ("id", "problem", "screen", "from", "into"):
        if k in row and not (isinstance(row[k], str) and ID.match(row[k])
                             and len(row[k]) <= ID_MAX):
            out.append(f"{event}: `{k}` is {row[k]!r}: lower-case words joined by `-`, "
                       f"at most {ID_MAX} characters")
    if not (isinstance(row["date"], str) and DATE.match(row["date"]) and valid(row["date"])):
        out.append(f"{event}: `date` is {row['date']!r}: YYYY-MM-DD, or YYYY-MM when the book "
                   "gives no day")
    for k in ("name", "what", "why", "observed", "unpredicted"):
        if k in row and not text(row[k]):
            out.append(f"{event}: `{k}` is empty")
    for k in ("passed", "control"):
        if k in row and not tally(row[k]):
            out.append(f"{event}: `{k}` is {row[k]!r}: [passed, of], whole numbers")
    for k in ("predicted", "fixed", "missed", "quotes"):
        if k in row and not names(row[k]):
            out.append(f"{event}: `{k}` is a list of words")
    if "usd" in row and not (type(row["usd"]) in (int, float) and row["usd"] >= 0):
        out.append(f"{event}: `usd` is dollars, a number")
    if "backfilled" in row and row["backfilled"] is not True:
        out.append(f"{event}: `backfilled` is true or left out")
    if "source" in row and not text(row["source"]):
        out.append(f"{event}: `source` is empty")
    if row.get("backfilled") and "source" not in row:
        out.append(f"{event}: read back from the book (`backfilled`), and no `source` says where")
    if "quotes" in row and "source" not in row:
        out.append(f"{event}: `quotes` with no `source` to find them in")
    if event == "screen":
        out += screen_shape(row)
    if event == "verdict":
        if row["met"] is not None and row["met"] not in MET:
            out.append(f"verdict: `met` is {row['met']!r}: one of {', '.join(MET)}, or null "
                       "when its screen had no bar")
        if row["decision"] not in DECISIONS:
            out.append(f"verdict: `decision` is {row['decision']!r}: one of "
                       f"{', '.join(DECISIONS)}")
    return out


def valid(date):
    try:
        datetime.date.fromisoformat(date if full(date) else date + "-01")
    except ValueError:
        return False
    return True


def screen_shape(row):
    out = []
    bar = row["bar"]
    if bar is None:
        if not row.get("backfilled"):
            out.append("screen: no bar: the bar is written before the runs — only a screen "
                       "read back from the book (`backfilled`) may have none")
    elif not text(bar):
        out.append("screen: `bar` is empty")
    if "predicted" in row and "unpredicted" in row:
        out.append("screen: `predicted` and `unpredicted` both: name the tasks, or say why none")
    elif "predicted" in row and not row["predicted"]:
        out.append("screen: `predicted` names no task: name them, or say why none "
                   "(`unpredicted`)")
    elif not row.get("backfilled") and "predicted" not in row and "unpredicted" not in row:
        out.append("screen: no prediction: the tasks it is expected to fix (`predicted`) are "
                   "named with the bar, before the runs, or why it can name none "
                   "(`unpredicted`) — only a screen read back from the book (`backfilled`) may "
                   "have neither")
    s = row["setting"]
    if not isinstance(s, dict):
        return out + ["screen: `setting` is an object: model, suite, build"]
    must, may = SETTING
    out += [f"screen: `setting` has no `{k}`" for k in must if not text(s.get(k))]
    out += [f"screen: `setting` has an unknown field `{k}`" for k in s if k not in must + may]
    if "runs" in s and not (type(s["runs"]) is int and s["runs"] > 0):
        out.append("screen: `setting.runs` is a count of runs")
    return out


def fold(lines):
    """The record folded: its problems in the order they were opened, each with its screens and
    their verdicts, and every fault found on the way as `(line, what)`."""
    problems, screens, alias, errors = {}, {}, {}, []
    order = []

    def resolve(pid):
        seen = set()
        while pid in alias and pid not in seen:
            seen.add(pid)
            pid = alias[pid]
        return pid

    for n, raw in lines:
        try:
            row = json.loads(raw)
        except ValueError as e:
            errors.append((n, f"not JSON: {e}"))
            continue
        faults = shape(row)
        if faults:
            errors += [(n, f) for f in faults]
            continue
        event = row["event"]
        if event == "problem":
            if row["id"] in problems or row["id"] in screens:
                errors.append((n, f"the id `{row['id']}` is taken"))
                continue
            problems[row["id"]] = dict(row, line=n, screens=[], withdrawn=None)
            order.append(row["id"])
        elif event == "screen":
            if row["id"] in problems or row["id"] in screens:
                errors.append((n, f"the id `{row['id']}` is taken"))
                continue
            if row["problem"] not in problems:
                errors.append((n, f"screen `{row['id']}`: no problem `{row['problem']}` above "
                               "it: name the problem first"))
                continue
            screens[row["id"]] = dict(row, line=n, verdict=None, withdrawn=None)
        elif event == "verdict":
            s = screens.get(row["screen"])
            if s is None:
                errors.append((n, f"verdict: no screen `{row['screen']}` above it: the bar "
                               "comes first"))
                continue
            if s["verdict"] is not None:
                errors.append((n, f"screen `{row['screen']}` has its verdict at line "
                               f"{s['verdict']['line']}: a second screen is a new row"))
                continue
            if (s["bar"] is None) != (row["met"] is None):
                errors.append((n, f"screen `{row['screen']}`: `met` is null exactly when the "
                               "screen had no bar"))
                continue
            if full(s["date"]) and full(row["date"]) and row["date"] < s["date"]:
                errors.append((n, f"screen `{row['screen']}`: its verdict is dated "
                               f"{row['date']}, before its bar ({s['date']})"))
                continue
            fixed, missed = set(row.get("fixed", [])), set(row.get("missed", []))
            both = sorted(fixed & missed)
            if both:
                errors.append((n, f"screen `{row['screen']}`: {', '.join(both)} both fixed and "
                               "missed"))
                continue
            unsaid = [t for t in s.get("predicted", []) if t not in fixed | missed]
            if unsaid:
                errors.append((n, f"screen `{row['screen']}` predicted {', '.join(unsaid)}, and "
                               "its verdict does not say what came of it: each predicted task "
                               "is in `fixed` or in `missed`"))
                continue
            s["verdict"] = dict(row, line=n)
        elif event == "merge":
            a, b = row["from"], row["into"]
            if a not in problems or b not in problems:
                errors.append((n, f"merge: `{a}` and `{b}` are both problems named above it"))
            elif a in alias:
                errors.append((n, f"merge: `{a}` is `{alias[a]}` already"))
            elif resolve(b) == a:
                errors.append((n, f"merge: `{b}` is `{a}` already: a problem is not merged "
                               "into itself"))
            else:
                alias[a] = b
        elif event == "withdraw":
            target = screens.get(row["id"]) or problems.get(row["id"])
            if target is None:
                errors.append((n, f"withdraw: no row `{row['id']}` above it"))
            elif target["withdrawn"] is not None:
                errors.append((n, f"withdraw: `{row['id']}` was withdrawn at line "
                               f"{target['withdrawn']['line']}"))
            else:
                target["withdrawn"] = dict(row, line=n)
    for s in screens.values():
        problems[resolve(s["problem"])]["screens"].append(s)
    for pid in order:
        p = problems[pid]
        live = [s for s in p["screens"] if s["withdrawn"] is None]
        if p["withdrawn"] is not None and pid not in alias and live:
            errors.append((p["withdrawn"]["line"], f"withdraw: problem `{pid}` still has "
                           f"{len(live)} screen(s): withdraw or move them first"))
    kept = [problems[pid] for pid in order if pid not in alias]
    for p in kept:
        p["also"] = sorted(a for a in alias if resolve(a) == p["id"])
    return {"problems": kept, "aliases": {a: resolve(a) for a in alias}}, errors


def squeeze(s):
    return " ".join(s.split())


def quoted(lines, root):
    """Every quote of the record that its source no longer holds, as `(line, what)`."""
    out, pages = [], {}
    for n, raw in lines:
        try:
            row = json.loads(raw)
        except ValueError:
            continue
        if not isinstance(row, dict) or "quotes" not in row or "source" not in row:
            continue
        rel = str(row["source"]).split("#", 1)[0]
        if rel not in pages:
            at = (Path(root) / rel).resolve()
            inside = Path(root).resolve() in at.parents
            pages[rel] = squeeze(at.read_text(encoding="utf-8")) if inside and at.is_file() \
                else None
        if pages[rel] is None:
            out.append((n, f"its source `{rel}` is no file of the repository"))
            continue
        for q in row["quotes"]:
            if squeeze(str(q)) not in pages[rel]:
                out.append((n, f"`{rel}` no longer says: {squeeze(str(q))[:90]}"))
    return out


def appended(before, now):
    """Why `now` is not `before` with lines added at its end; `None` when it is."""
    for i, old in enumerate(before):
        if i >= len(now):
            return f"line {i + 1} is gone: {old[:80]}"
        if now[i] != old:
            return f"line {i + 1} was rewritten: it read {old[:80]}"
    return None


def at_rev(rev, path):
    """The record's lines as `rev` holds them; none when it holds no record yet."""
    try:
        rel = Path(path).resolve().relative_to(ROOT)
    except ValueError:
        raise SystemExit(f"--since reads the record's history from git, and {path} is outside "
                         "the repository") from None
    out = subprocess.run(["git", "-C", str(ROOT), "show", f"{rev}:{rel.as_posix()}"],
                         capture_output=True, text=True)
    if out.returncode != 0:
        if "exists on disk, but not in" in out.stderr or "does not exist in" in out.stderr:
            return []
        raise SystemExit(f"git show {rev}: {out.stderr.strip()}")
    return [l for l in out.stdout.split("\n") if l.strip()]


def setting(s):
    st = s["setting"]
    runs = f" · {st['runs']} runs" if "runs" in st else ""
    return f"{st['model']} · {st['suite']} · {st['build']}{runs}"


def tallied(v):
    out = f"{v['passed'][0]} of {v['passed'][1]}" if "passed" in v else ""
    if "control" in v:
        out += f" (without: {v['control'][0]} of {v['control'][1]})"
    return out


def render(folded, problem=None, model=None):
    """The record as text, whole or one problem or one model: what a proposer reads."""
    shown = []
    for p in folded["problems"]:
        if p["withdrawn"] is not None:
            continue
        if problem and problem not in (p["id"], *p["also"]):
            continue
        screens = [s for s in p["screens"] if s["withdrawn"] is None
                   and (not model or model.lower() in s["setting"]["model"].lower())]
        if model and not screens:
            continue
        shown.append((p, screens))
    n = sum(len(s) for _, s in shown)
    back = sum(1 for _, ss in shown for s in ss if s.get("backfilled"))
    out = [f"The record of screens: {len(shown)} problem(s), {n} screen(s)"
           + (f", {back} read back from the book" if back else "") + ".",
           "A verdict holds for its setting (model · suite · build): screen a change again "
           "before leaning on it elsewhere."]
    judged = [(s, s["verdict"]) for _, ss in shown for s in ss
              if s.get("predicted") and s["verdict"]]
    if judged:
        named = sum(len(s["predicted"]) for s, _ in judged)
        right = sum(len(set(s["predicted"]) & set(v.get("fixed", []))) for s, v in judged)
        out.append(f"Predictions checked: {right} of {named} named task(s) fixed, over "
                   f"{len(judged)} screen(s).")
    out.append("")
    for p, screens in shown:
        also = f" (also: {', '.join(p['also'])})" if p["also"] else ""
        out.append(f"== {p['id']}{also} — {p['name']}")
        counts = {}
        for s in screens:
            d = s["verdict"]["decision"] if s["verdict"] else "no verdict yet"
            counts[d] = counts.get(d, 0) + 1
        said = ", ".join(f"{c} {d}" for d, c in counts.items()) or "no screen yet"
        out.append(f"   opened {p['date']} · {len(screens)} screen(s): {said}")
        for s in screens:
            v = s["verdict"]
            mark = v["decision"] if v else "no verdict yet"
            out.append(f" - {s['id']} [{mark}] {s['date']} · {setting(s)}")
            out.append(f"     what: {s['what']}")
            out.append(f"     why: {s['why']}")
            bar = s["bar"] if s["bar"] is not None else "none recorded"
            out.append(f"     bar: {bar}")
            if s.get("predicted"):
                out.append(f"     predicted to fix: {', '.join(s['predicted'])}")
            if s.get("unpredicted"):
                out.append(f"     no task named: {s['unpredicted']}")
            if v:
                met = {"yes": "met", "no": "missed", "partly": "met in part", None: ""}[v["met"]]
                head = "; ".join(x for x in (met, tallied(v)) if x)
                out.append(f"     observed: {(head + ' — ') if head else ''}{v['observed']}")
                for k in ("fixed", "missed"):
                    if v.get(k):
                        out.append(f"     {k}: {', '.join(v[k])}")
                if s.get("predicted"):
                    hit = [t for t in s["predicted"] if t in v.get("fixed", [])]
                    out.append(f"     prediction: {len(hit)} of {len(s['predicted'])} right")
                    surprise = [t for t in v.get("fixed", []) if t not in s["predicted"]]
                    if surprise:
                        out.append(f"     fixed, not predicted: {', '.join(surprise)}")
                if v.get("source") or s.get("source"):
                    out.append(f"     told in: {v.get('source') or s.get('source')}")
        out.append("")
    return "\n".join(out).rstrip() + "\n"


def today():
    return datetime.date.today().isoformat()


def add(path, row):
    """Appends `row` when the record stays well-formed with it; else says why and writes nothing."""
    lines = read(path)
    n = (lines[-1][0] if lines else 0) + 1
    raw = json.dumps(row, ensure_ascii=False)
    _, before = fold(lines)
    _, after = fold(lines + [(n, raw)])
    new = [what for line, what in after if (line, what) not in before]
    if new:
        return new
    whole = Path(path).read_text(encoding="utf-8") if Path(path).exists() else ""
    with open(path, "a", encoding="utf-8") as f:
        f.write(("" if whole == "" or whole.endswith("\n") else "\n") + raw + "\n")
    return []


def split(v):
    return [x.strip() for x in v.split(",") if x.strip()]


def of(v):
    try:
        a, b = v.split("/")
        return [int(a), int(b)]
    except ValueError:
        raise argparse.ArgumentTypeError(f"{v!r}: passed/of, as 10/12") from None


def parser():
    p = argparse.ArgumentParser(description="The record of screens: what was tried for each "
                                "problem of the harness, in which setting, and what came of it.")
    p.add_argument("--file", default=str(RECORD), help="the record (bench/screens.jsonl)")
    sub = p.add_subparsers(dest="verb", required=True)
    s = sub.add_parser("show", help="the record as text: read it before proposing a change")
    s.add_argument("--problem")
    s.add_argument("--model", help="only screens whose model holds this text")
    s.add_argument("--json", action="store_true")
    c = sub.add_parser("check", help="the record is well-formed")
    c.add_argument("--quotes", action="store_true", help="each quote stands in its source")
    c.add_argument("--since", metavar="REV", help="and only grew since this commit")
    for verb, words in (("problem", ("id", "name")), ("merge", ("from_", "into", "why")),
                        ("withdraw", ("id", "why"))):
        a = sub.add_parser(verb)
        for w in words:
            a.add_argument(w)
        a.add_argument("--date", default=today())
    b = sub.add_parser("bar", help="a screen and its bar, before any paid run")
    b.add_argument("id")
    for k in ("problem", "what", "why", "bar", "model", "suite", "build"):
        b.add_argument(f"--{k}", required=True)
    b.add_argument("--runs", type=int)
    b.add_argument("--predicted", type=split, help="the tasks it is expected to fix, a,b,c")
    b.add_argument("--unpredicted", metavar="WHY",
                   help="why it can name no task to fix (a change for cost alone, say)")
    b.add_argument("--date", default=today())
    v = sub.add_parser("verdict", help="what a screen came to, after its runs")
    v.add_argument("screen")
    v.add_argument("--observed", required=True)
    v.add_argument("--met", required=True, choices=MET)
    v.add_argument("--decision", required=True, choices=DECISIONS)
    v.add_argument("--passed", type=of)
    v.add_argument("--control", type=of)
    v.add_argument("--fixed", type=split)
    v.add_argument("--missed", type=split)
    v.add_argument("--usd", type=float)
    v.add_argument("--source")
    v.add_argument("--date", default=today())
    return p


def row_of(a):
    """The event a writing verb asks for."""
    if a.verb == "problem":
        return {"event": "problem", "id": a.id, "date": a.date, "name": a.name}
    if a.verb == "merge":
        return {"event": "merge", "from": a.from_, "into": a.into, "date": a.date, "why": a.why}
    if a.verb == "withdraw":
        return {"event": "withdraw", "id": a.id, "date": a.date, "why": a.why}
    if a.verb == "bar":
        st = {"model": a.model, "suite": a.suite, "build": a.build}
        if a.runs:
            st["runs"] = a.runs
        row = {"event": "screen", "id": a.id, "problem": a.problem, "date": a.date,
               "what": a.what, "why": a.why, "bar": a.bar, "setting": st}
        if a.predicted is not None:
            row["predicted"] = a.predicted
        if a.unpredicted is not None:
            row["unpredicted"] = a.unpredicted
        return row
    row = {"event": "verdict", "screen": a.screen, "date": a.date, "observed": a.observed,
           "met": a.met, "decision": a.decision}
    for k in ("passed", "control", "fixed", "missed", "usd", "source"):
        if getattr(a, k) is not None:
            row[k] = getattr(a, k)
    return row


def main(argv=None, out=sys.stdout, err=sys.stderr):
    a = parser().parse_args(argv)
    lines = read(a.file)
    if a.verb == "show":
        folded, errors = fold(lines)
        if errors:
            print(f"the record does not read: {len(errors)} fault(s); "
                  "`python3 bench/screens.py check` names them", file=err)
            return 1
        print(json.dumps(folded, indent=2, ensure_ascii=False) if a.json
              else render(folded, a.problem, a.model), end="" if not a.json else "\n", file=out)
        return 0
    if a.verb == "check":
        _, errors = fold(lines)
        if a.quotes:
            errors += quoted(lines, ROOT)
        if a.since:
            why = appended(at_rev(a.since, a.file), [raw for _, raw in lines])
            if why:
                errors.append((0, f"the record is appended to, never rewritten: since "
                               f"{a.since}, {why}"))
        for n, what in sorted(errors):
            print(f"{Path(a.file).name}:{n}: {what}" if n else what, file=err)
        if not errors:
            folded, _ = fold(lines)
            n = sum(len(p["screens"]) for p in folded["problems"])
            print(f"the record reads: {len(folded['problems'])} problem(s), {n} screen(s)",
                  file=out)
        return 1 if errors else 0
    faults = add(a.file, row_of(a))
    for f in faults:
        print(f"not written: {f}", file=err)
    return 1 if faults else 0


if __name__ == "__main__":
    sys.exit(main())
