"""The record of screens' rules: a bar before its verdict, every reference resolved by a line
above it, a row never rewritten, and the repository's own record held to them."""

import io
import json
import tempfile
import unittest
from pathlib import Path

import screens


def problem(id="p", **more):
    return dict(event="problem", id=id, date="2026-11-20", name="It goes wrong.", **more)


def screen(id="s", problem="p", bar="4 of 6.", **more):
    row = dict(event="screen", id=id, problem=problem, date="2026-11-21", what="A change.",
               why="A reason.", bar=bar, setting=dict(model="qwen", suite="sqlglot", build="1.3.0"),
               predicted=["t1"])
    row.update(more)
    return row


def verdict(screen="s", met="yes", **more):
    row = dict(event="verdict", screen=screen, date="2026-11-22", observed="6 of 6.", met=met,
               decision="shipped", fixed=["t1"])
    row.update(more)
    return row


def lines(*rows):
    return [(n, r if isinstance(r, str) else json.dumps(r)) for n, r in enumerate(rows, 1)]


def faults(*rows):
    return [what for _, what in screens.fold(lines(*rows))[1]]


def run(*argv):
    out, err = io.StringIO(), io.StringIO()
    code = screens.main(list(argv), out=out, err=err)
    return code, out.getvalue(), err.getvalue()


class Fold(unittest.TestCase):
    def test_a_problem_its_screen_and_its_verdict_fold_into_one(self):
        folded, errors = screens.fold(lines(problem(), screen(), verdict(passed=[6, 6])))
        self.assertEqual(errors, [])
        (p,) = folded["problems"]
        (s,) = p["screens"]
        self.assertEqual((p["id"], s["id"], s["bar"]), ("p", "s", "4 of 6."))
        self.assertEqual((s["verdict"]["met"], s["verdict"]["passed"]), ("yes", [6, 6]))
        self.assertEqual((p["line"], s["line"], s["verdict"]["line"]), (1, 2, 3))

    def test_a_verdict_comes_after_its_bar(self):
        (early,) = faults(problem(), verdict(), screen())
        self.assertIn("no screen `s` above it: the bar comes first", early)
        (dated,) = faults(problem(), screen(), verdict(date="2026-11-20"))
        self.assertIn("before its bar (2026-11-21)", dated)
        # A day the book does not give is no day to compare.
        self.assertEqual(faults(problem(), screen(date="2026-11"), verdict(date="2026-11")), [])

    def test_a_screen_names_a_problem_above_it(self):
        (f,) = faults(screen(), problem())
        self.assertIn("no problem `p` above it: name the problem first", f)

    def test_a_screen_has_its_bar_unless_it_was_read_back_from_the_book(self):
        (f,) = faults(problem(), screen(bar=None))
        self.assertIn("the bar is written before the runs", f)
        self.assertIn("`bar` is empty", faults(problem(), screen(bar="  "))[0])
        read_back = screen(bar=None, backfilled=True, source="docs/bench.md#what-did-not-help")
        self.assertEqual(faults(problem(), read_back, verdict(met=None)), [])
        # Met or missed is said exactly of a bar.
        for s, v in ((read_back, verdict(met="yes")), (screen(), verdict(met=None))):
            (f,) = faults(problem(), s, v)
            self.assertIn("`met` is null exactly when the screen had no bar", f)

    def test_a_screen_names_what_it_should_fix_before_its_runs(self):
        (f,) = faults(problem(), screen(predicted=[]))
        self.assertIn("`predicted` names no task", f)
        bare = screen()
        del bare["predicted"]
        (f,) = faults(problem(), bare)
        self.assertIn("screen: no prediction", f)
        # Why none can be named stands in its place; both at once say nothing.
        cost = dict(bare, unpredicted="A change for cost alone: no task should pass or fail.")
        self.assertEqual(faults(problem(), cost, verdict(fixed=[])), [])
        (f,) = faults(problem(), dict(cost, predicted=["t1"]))
        self.assertIn("`predicted` and `unpredicted` both", f)
        self.assertIn("`unpredicted` is empty", faults(problem(), dict(bare, unpredicted=" "))[0])
        # A screen read back from the book had none to give.
        read_back = dict(bare, bar=None, backfilled=True, source="docs/bench.md")
        self.assertEqual(faults(problem(), read_back, verdict(met=None, fixed=[])), [])

    def test_a_verdict_says_what_came_of_each_predicted_task(self):
        two = screen(predicted=["01-interval", "04-slice"])
        self.assertEqual(faults(problem(), two, verdict(fixed=["01-interval"],
                                                        missed=["04-slice"])), [])
        (f,) = faults(problem(), two, verdict(fixed=["01-interval"]))
        self.assertIn("predicted 04-slice, and its verdict does not say what came of it", f)
        (f,) = faults(problem(), two, verdict(fixed=["01-interval", "04-slice"],
                                              missed=["04-slice"]))
        self.assertIn("04-slice both fixed and missed", f)
        # A task fixed that nobody named is said, not refused.
        self.assertEqual(faults(problem(), two, verdict(
            fixed=["01-interval", "04-slice", "09-cast"])), [])

    def test_a_screen_has_one_verdict(self):
        (f,) = faults(problem(), screen(), verdict(), verdict(met="no"))
        self.assertIn("has its verdict at line 3: a second screen is a new row", f)

    def test_an_id_is_taken_once(self):
        self.assertIn("the id `p` is taken", faults(problem(), problem())[0])
        self.assertIn("the id `p` is taken", faults(problem(), screen(id="p"))[0])
        self.assertIn("the id `s` is taken", faults(problem(), screen(), screen())[0])

    def test_a_row_is_read_whole_or_not_at_all(self):
        cases = [
            ("{not json", "not JSON"),
            (dict(event="guess"), "unknown event 'guess'"),
            (dict(problem(), note="x"), "unknown field `note`"),
            (problem(id="Prose_Misreading"), "lower-case words joined by `-`"),
            (problem(id="a" * 49), "at most 48 characters"),
            (dict(problem(), date="2026-13-01"), "`date` is '2026-13-01'"),
            (dict(problem(), date="November"), "`date` is 'November'"),
            (dict(problem(), name=" "), "`name` is empty"),
            (dict(problem(), backfilled=True), "no `source` says where"),
            (dict(problem(), quotes=["x"]), "`quotes` with no `source`"),
        ]
        for row, said in cases:
            with self.subTest(said=said):
                self.assertIn(said, " | ".join(faults(row)))
        for row, said in [
            (dict(screen(), setting=dict(model="qwen", suite="sqlglot")), "no `build`"),
            (dict(screen(), setting=dict(model="qwen", suite="s", build="b", runs=0)),
             "`setting.runs` is a count of runs"),
            (dict(screen(), setting=dict(model="qwen", suite="s", build="b", host="x")),
             "unknown field `host`"),
            (dict(screen(), what=""), "`what` is empty"),
        ]:
            with self.subTest(said=said):
                self.assertIn(said, " | ".join(faults(problem(), row)))
        for row, said in [
            (verdict(passed=[7, 6]), "`passed` is [7, 6]"),
            (verdict(met="almost"), "`met` is 'almost'"),
            (verdict(decision="merged"), "`decision` is 'merged'"),
            (verdict(usd="cheap"), "`usd` is dollars"),
        ]:
            with self.subTest(said=said):
                self.assertIn(said, " | ".join(faults(problem(), screen(), row)))

    def test_two_problems_found_to_be_one_are_read_as_one(self):
        merge = dict(event="merge", date="2026-11-23", why="One cause.")
        folded, errors = screens.fold(lines(
            problem("a"), problem("b"), screen("s1", "a"), dict(merge, **{"from": "a", "into": "b"}),
            screen("s2", "a"), screen("s3", "b")))
        self.assertEqual(errors, [])
        (p,) = folded["problems"]
        self.assertEqual((p["id"], p["also"]), ("b", ["a"]))
        self.assertEqual([s["id"] for s in p["screens"]], ["s1", "s2", "s3"])
        self.assertEqual(folded["aliases"], {"a": "b"})
        for row, said in [
            (dict(merge, **{"from": "a", "into": "c"}), "are both problems named above it"),
            (dict(merge, **{"from": "a", "into": "b"}), "`a` is `b` already"),
            (dict(merge, **{"from": "b", "into": "a"}), "a problem is not merged into itself"),
        ]:
            with self.subTest(said=said):
                rows = (problem("a"), problem("b"), dict(merge, **{"from": "a", "into": "b"}), row)
                self.assertIn(said, " | ".join(faults(*rows)))

    def test_a_withdrawn_row_stays_and_is_shown_no_more(self):
        gone = dict(event="withdraw", id="s", date="2026-11-23", why="Counted twice.")
        rows = lines(problem(), screen(), verdict(), screen("t"), gone)
        folded, errors = screens.fold(rows)
        self.assertEqual(errors, [])
        self.assertEqual(len(folded["problems"][0]["screens"]), 2, "the row stays")
        shown = screens.render(folded)
        self.assertIn(" - t [no verdict yet]", shown)
        self.assertNotIn(" - s [", shown)
        self.assertIn("was withdrawn at line 5", faults(*[r for _, r in rows], gone)[0])
        self.assertIn("no row `u` above it", faults(problem(), dict(gone, id="u"))[0])
        # A problem goes only once nothing live stands under it.
        (f,) = faults(problem(), screen(), dict(gone, id="p"))
        self.assertIn("problem `p` still has 1 screen(s)", f)


class Show(unittest.TestCase):
    def test_a_screen_is_shown_with_its_setting_and_what_came_of_it(self):
        folded, _ = screens.fold(lines(
            problem(), screen(predicted=["03-alter", "04-slice"]),
            verdict(met="no", decision="not-shipped", passed=[2, 6], control=[4, 6],
                    fixed=["03-alter", "07-flatten"], missed=["04-slice"],
                    source="docs/bench.md#what-did-not-help")))
        shown = screens.render(folded)
        for said in (
            "The record of screens: 1 problem(s), 1 screen(s).",
            "Predictions checked: 1 of 2 named task(s) fixed, over 1 screen(s).",
            "== p — It goes wrong.",
            "opened 2026-11-20 · 1 screen(s): 1 not-shipped",
            " - s [not-shipped] 2026-11-21 · qwen · sqlglot · 1.3.0",
            "     bar: 4 of 6.",
            "     predicted to fix: 03-alter, 04-slice",
            "     observed: missed; 2 of 6 (without: 4 of 6) — 6 of 6.",
            "     missed: 04-slice",
            "     prediction: 1 of 2 right",
            "     fixed, not predicted: 07-flatten",
            "     told in: docs/bench.md#what-did-not-help",
        ):
            self.assertIn(said, shown)

    def test_one_problem_or_one_model_is_shown_alone(self):
        haiku = dict(model="Claude Haiku 5.5", suite="sqlglot", build="1.3.0")
        folded, _ = screens.fold(lines(
            problem("a"), problem("b"), screen("s1", "a"), screen("s2", "b", setting=haiku)))
        self.assertNotIn("== b", screens.render(folded, problem="a"))
        by_model = screens.render(folded, model="haiku")
        self.assertIn(" - s2 ", by_model)
        self.assertNotIn("== a", by_model)
        self.assertIn("1 problem(s), 1 screen(s)", by_model)


class Kept(unittest.TestCase):
    def test_the_record_is_appended_to_never_rewritten(self):
        self.assertIsNone(screens.appended(["a", "b"], ["a", "b", "c"]))
        self.assertIsNone(screens.appended([], ["a"]))
        self.assertIn("line 2 was rewritten", screens.appended(["a", "b"], ["a", "B", "c"]))
        self.assertIn("line 2 is gone", screens.appended(["a", "b"], ["a"]))

    def test_a_quote_stands_in_its_source_word_for_word(self):
        with tempfile.TemporaryDirectory() as root:
            (Path(root) / "docs").mkdir()
            (Path(root) / "docs" / "bench.md").write_text(
                "| Four process lines | 2 of 8 on prose asks,\n  against 4 of 8 |\n")
            row = dict(problem(), backfilled=True, source="docs/bench.md#what-did-not-help",
                       quotes=["2 of 8 on prose asks, against 4 of 8"])
            self.assertEqual(screens.quoted(lines(row), root), [])
            wrong = dict(row, quotes=["3 of 8 on prose asks"])
            ((n, said),) = screens.quoted(lines(wrong), root)
            self.assertEqual(n, 1)
            self.assertIn("`docs/bench.md` no longer says: 3 of 8 on prose asks", said)
            for source in ("docs/gone.md", "../outside.md"):
                ((_, said),) = screens.quoted(lines(dict(row, source=source)), root)
                self.assertIn("is no file of the repository", said)


class Verbs(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        self.file = str(Path(self.dir.name) / "screens.jsonl")

    def test_the_verbs_write_one_line_each_and_show_reads_them(self):
        f = ("--file", self.file)
        self.assertEqual(run(*f, "problem", "prose-misreading", "A prose ask is misread.",
                             "--date", "2026-11-17")[0], 0)
        self.assertEqual(run(*f, "bar", "held-list", "--problem", "prose-misreading",
                             "--what", "A list held by the harness.", "--why", "A body builds "
                             "to the cases it holds.", "--bar", "4 of 6 for each list.",
                             "--model", "qwen 3.5 35B", "--suite", "sqlglot, two prose asks",
                             "--build", "1.1.0 (203e8a30a6)", "--runs", "12",
                             "--predicted", "01-interval, 04-slice", "--date", "2026-11-20")[0], 0)
        self.assertEqual(run(*f, "verdict", "held-list", "--observed", "6 of 6, twice.", "--met",
                             "yes", "--decision", "shipped", "--passed", "6/6", "--control",
                             "1/6", "--fixed", "01-interval,04-slice", "--usd", "10.77",
                             "--date", "2026-11-21")[0], 0)
        rows = [json.loads(l) for l in Path(self.file).read_text().splitlines()]
        self.assertEqual([r["event"] for r in rows], ["problem", "screen", "verdict"])
        self.assertEqual(rows[1]["setting"], dict(model="qwen 3.5 35B", suite="sqlglot, two "
                                                  "prose asks", build="1.1.0 (203e8a30a6)",
                                                  runs=12))
        self.assertEqual(rows[1]["predicted"], ["01-interval", "04-slice"])
        self.assertEqual((rows[2]["passed"], rows[2]["control"], rows[2]["usd"]),
                         ([6, 6], [1, 6], 10.77))
        code, out, _ = run(*f, "show")
        self.assertEqual(code, 0)
        self.assertIn("observed: met; 6 of 6 (without: 1 of 6) — 6 of 6, twice.", out)
        code, out, _ = run(*f, "show", "--json")
        self.assertEqual(json.loads(out)["problems"][0]["screens"][0]["id"], "held-list")
        self.assertEqual(run(*f, "check")[:2], (0, "the record reads: 1 problem(s), 1 screen(s)\n"))

    def test_a_row_that_would_break_the_record_is_not_written(self):
        f = ("--file", self.file)
        code, _, err = run(*f, "verdict", "held-list", "--observed", "6 of 6.", "--met", "yes",
                           "--decision", "shipped")
        self.assertEqual(code, 1)
        self.assertIn("not written: verdict: no screen `held-list` above it", err)
        self.assertFalse(Path(self.file).exists())
        run(*f, "problem", "p", "It goes wrong.")
        before = Path(self.file).read_text()
        code, _, err = run(*f, "bar", "s", "--problem", "p", "--what", "w", "--why", "y", "--bar",
                           " ", "--model", "m", "--suite", "s", "--build", "b")
        self.assertEqual((code, Path(self.file).read_text()), (1, before))
        self.assertIn("not written: screen: `bar` is empty", err)
        bar = ("bar", "s", "--problem", "p", "--what", "w", "--why", "y", "--bar", "4 of 6.",
               "--model", "m", "--suite", "s", "--build", "b")
        code, _, err = run(*f, *bar)
        self.assertEqual((code, Path(self.file).read_text()), (1, before))
        self.assertIn("not written: screen: no prediction", err)
        self.assertEqual(run(*f, *bar, "--unpredicted", "For cost alone.")[0], 0)
        self.assertEqual(json.loads(Path(self.file).read_text().splitlines()[-1])["unpredicted"],
                         "For cost alone.")

    def test_check_names_each_fault_by_its_line(self):
        Path(self.file).write_text("\n".join(json.dumps(r) for r in (
            problem(), verdict(), screen(bar=None))) + "\n")
        code, out, err = run("--file", self.file, "check")
        self.assertEqual((code, out), (1, ""))
        self.assertIn("screens.jsonl:2: verdict: no screen `s` above it", err)
        self.assertIn("screens.jsonl:3: screen: no bar", err)
        code, _, err = run("--file", self.file, "show")
        self.assertEqual(code, 1)
        self.assertIn("the record does not read: 2 fault(s)", err)


class Repository(unittest.TestCase):
    def test_the_repositorys_own_record_reads(self):
        folded, errors = screens.fold(screens.read(screens.RECORD))
        self.assertEqual(errors, [])
        self.assertGreater(len(folded["problems"]), 0)
        for p in folded["problems"]:
            for s in p["screens"]:
                self.assertIsNotNone(s["verdict"] or s["bar"],
                                     f"{s['id']}: a screen with neither bar nor verdict")


if __name__ == "__main__":
    unittest.main()
