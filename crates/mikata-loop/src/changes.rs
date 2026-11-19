//! What changed, by the body's own checks.
//!
//! A body asks its code questions as it works: a line of Python that prints what a call returns.
//! It reads the answer and takes it for right, and on an ask stated in prose that is where a
//! misreading hides: the check passes because it holds the same misreading. A model shown those
//! answers sorts them no better than the body did; on our bench a second reading caught half the
//! failed runs. The human who wrote the ask can, at a glance.
//!
//! So when an answer lands, the body's last checks are run again by the harness: on a copy of the
//! finished work, and on a second copy put back to the tree as the ask found it. Where a check
//! prints differently, the human is shown the lines. Code decides what is shown. No model reads
//! it, and nothing of it goes back to the body.
//!
//! Two copies, never one put back in between: a check leaves caches where it ran (compiled
//! Python beside its source), and a file put back at the same size within the same second is
//! read from the cache of the other tree. For the same reason no cache is copied.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use mikata_races::Race;
use mikata_tools::{Attic, Context, Snapshot, Tool, call, stdout_of};
use serde_json::json;

/// How many of the body's checks are run again, the newest first.
pub const MOST: usize = 12;
/// How long one check may run.
const EACH: Duration = Duration::from_secs(20);
/// How long the whole step may take: checks not reached by then are said, not run.
const BUDGET: Duration = Duration::from_secs(60);
/// The largest world copied for it, in bytes and in files.
const COPY_BYTES: u64 = 256 * 1024 * 1024;
const COPY_FILES: u32 = 40_000;
/// The most lines of one check shown to the human; the whole of it is in the file.
const SHOWN: usize = 16;
/// The lines of an output set side by side; a longer one is compared whole and shown cut.
const DIFFED: usize = 400;
/// Where the whole of it is written, beneath the world.
pub const FILE: &str = ".mikata/tmp/what-changed.md";

/// The programs that take their code on the command line, each with the options that hand it
/// over. A command that starts one this way asks the code a question and prints the answer.
const INLINE: &[(&str, &[&str])] = &[
    ("python", &["-c"]),
    ("python3", &["-c"]),
    ("pypy", &["-c"]),
    ("pypy3", &["-c"]),
    ("node", &["-e", "--eval", "-p", "--print"]),
    ("nodejs", &["-e", "--eval", "-p", "--print"]),
    ("bun", &["-e", "--eval", "-p", "--print"]),
    ("deno", &["eval"]),
    ("ruby", &["-e"]),
    ("perl", &["-e", "-E"]),
    ("php", &["-r"]),
    ("lua", &["-e"]),
    ("Rscript", &["-e"]),
    ("julia", &["-e"]),
];

/// The words that may stand before such a program: what starts it in the project's environment.
const BEFORE: &[&str] = &[
    "uv", "poetry", "pdm", "hatch", "pipenv", "run", "exec", "npx", "pnpm", "yarn", "env", "time",
    "--",
];

/// Whether `command` is a check: its first program is an interpreter handed its code on the
/// command line or through a here-document. A test runner is not one (the end gate judges
/// those), nor is a look at a file, nor a command that does something else first.
#[must_use]
pub fn is_check(command: &str) -> bool {
    let mut rest = command.trim_start();
    // `cd <dir> &&` first, as bodies write it.
    while let Some(after) = rest.strip_prefix("cd ") {
        match after.split_once("&&") {
            Some((dir, then)) if !dir.contains(['|', ';', '\n']) => rest = then.trim_start(),
            _ => return false,
        }
    }
    let first = rest
        .split(['|', ';', '\n'])
        .next()
        .unwrap_or(rest)
        .split("&&")
        .next()
        .unwrap_or(rest);
    let mut words = first.split_whitespace().peekable();
    // Assignments and runners, then the program.
    while let Some(w) = words.peek() {
        let assignment = w.split_once('=').is_some_and(|(k, _)| {
            !k.is_empty() && k.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
        });
        if assignment || BEFORE.contains(w) {
            words.next();
        } else {
            break;
        }
    }
    let Some(program) = words.next() else {
        return false;
    };
    let name = program.rsplit('/').next().unwrap_or(program);
    // `python3.12` is `python3`.
    let name = match name.split_once('.') {
        Some((base, minor)) if minor.chars().all(|c| c.is_ascii_digit()) => base,
        _ => name,
    };
    let Some((_, hands)) = INLINE.iter().find(|(p, _)| *p == name) else {
        return false;
    };
    // Its code on the command line: one of its options among the first words, before any file.
    let inline = words.clone().take(6).any(|w| hands.contains(&w));
    // Or through a here-document, with no file named before it.
    let here = first.contains("<<")
        && !words.any(|w| !w.starts_with('-') && !w.starts_with("<<") && w.contains('.'));
    inline || here
}

/// The body's checks of one ask, each once, in the order it last ran them.
#[derive(Debug, Default, Clone)]
pub struct Checks(Vec<String>);

impl Checks {
    /// A shell command the body ran, if the call was one: kept when it is a check, as its newest.
    pub fn saw(&mut self, command: Option<&str>) {
        let Some(command) = command.filter(|c| is_check(c)) else {
            return;
        };
        self.0.retain(|c| c != command);
        self.0.push(command.to_owned());
    }

    /// The newest `n`, newest first.
    #[must_use]
    pub fn newest(&self, n: usize) -> Vec<&str> {
        self.0.iter().rev().take(n).map(String::as_str).collect()
    }

    /// Whether the body ran none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One check that prints differently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Differ {
    /// The command, as the body wrote it.
    pub command: String,
    /// What it printed on the code as the ask found it.
    pub before: String,
    /// What it prints on the finished work.
    pub now: String,
}

/// What running the checks again found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// The checks taken: the body's newest, at most [`MOST`].
    pub taken: usize,
    /// Those that print the same on both trees.
    pub same: usize,
    /// Those that print differently, the body's newest first.
    pub differ: Vec<Differ>,
    /// Those that fail on the finished work itself: nothing to set side by side.
    pub failed_now: usize,
    /// Those that did not print the same twice on the finished work.
    pub unsteady: usize,
    /// Those that run only on the finished work: on the code as the ask found it they fail (a
    /// name the work added, an assertion the work makes true).
    pub only_now: usize,
    /// Those the step's time ran out before.
    pub not_reached: usize,
    /// Why nothing could be compared at all, when so.
    pub why_not: Option<String>,
    /// How long it took, in milliseconds.
    pub ms: u64,
}

/// Run `checks` (newest first) again: on a copy of `world` as it stands, and on a second copy
/// put back to `start`. `from` is the body's own context: the copies' commands run in its
/// sandbox, with no gate to ask and no attic. The world itself is never run in and never written.
#[must_use]
pub fn compare(
    world: &Path,
    from: &Context,
    attic: &Attic,
    start: &Snapshot,
    checks: &[&str],
    tools: &[Box<dyn Tool>],
) -> Report {
    let began = Instant::now();
    let mut report = Report {
        taken: checks.len(),
        ..Report::default()
    };
    if checks.is_empty() {
        return report;
    }
    let root = std::env::temp_dir().join(format!(
        "mikata-changes-{}-{}",
        std::process::id(),
        began.elapsed().as_nanos() ^ u128::from(std::process::id())
    ));
    let _ = fs::remove_dir_all(&root);
    report.why_not =
        run_in_copies(world, &root, from, attic, start, checks, tools, &mut report).err();
    let _ = fs::remove_dir_all(&root);
    report.ms = u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX);
    report
}

/// A copy of `world` under `root/<tree>`, with the context its commands run in.
fn copy_of(
    world: &Path,
    root: &Path,
    tree: &str,
    from: &Context,
) -> Result<(PathBuf, Context), String> {
    let copy = root
        .join(tree)
        .join(world.file_name().unwrap_or_else(|| "world".as_ref()));
    let (mut bytes, mut files) = (COPY_BYTES, COPY_FILES);
    match copy_capped(world, &copy, true, &mut bytes, &mut files) {
        Ok(true) => {}
        Ok(false) => {
            return Err(format!(
                "the world is too large to copy (over {} MiB or {COPY_FILES} files)",
                COPY_BYTES / (1024 * 1024)
            ));
        }
        Err(e) => return Err(format!("the world could not be copied: {e}")),
    }
    let copy = copy
        .canonicalize()
        .map_err(|e| format!("the copy could not be read: {e}"))?;
    let cx = Context::new(Race::Slime, &copy, &["."])
        .with_sandbox(from.sandbox().clone())
        .with_unguarded(from.allows_unguarded())
        .with_home(from.home());
    Ok((copy, cx))
}

#[allow(clippy::too_many_arguments)]
fn run_in_copies(
    world: &Path,
    root: &Path,
    from: &Context,
    attic: &Attic,
    start: &Snapshot,
    checks: &[&str],
    tools: &[Box<dyn Tool>],
    report: &mut Report,
) -> Result<(), String> {
    let (now_at, mut now_cx) = copy_of(world, root, "now", from)?;
    let (before_at, mut before_cx) = copy_of(world, root, "before", from)?;
    // The second copy, put back to the tree as the ask found it: what the world ignores (its
    // dependencies) stays as it is, so a check finds them there as it did.
    attic
        .export_into(&start.tree, &before_at)
        .map_err(|e| format!("the code as the ask found it could not be laid out: {e}"))?;
    let until = Instant::now() + BUDGET;
    now_cx.set_deadline(Some(until));
    before_cx.set_deadline(Some(until));
    // A command names its world in full more often than not: each such path is the copy's there.
    let mut names = vec![world.display().to_string()];
    if let Ok(real) = world.canonicalize() {
        let real = real.display().to_string();
        if !names.contains(&real) {
            names.push(real);
        }
    }
    let run = |cx: &mut Context, at: &Path, command: &str| -> Option<String> {
        let here = at.display().to_string();
        let mut command = command.to_owned();
        for name in &names {
            command = command.replace(name.as_str(), &here);
        }
        let args = json!({
            "command": command,
            "timeout_ms": u64::try_from(EACH.as_millis()).unwrap_or(20_000),
        });
        let o = call(tools, "bash", &args.to_string(), cx);
        (!o.is_error).then(|| {
            stdout_of(&o.content)
                .replace(&here, "<world>")
                .trim_end()
                .to_owned()
        })
    };
    for (i, check) in checks.iter().enumerate() {
        if Instant::now() >= until {
            report.not_reached += checks.len() - i;
            break;
        }
        // The finished work, twice: a check that will not print the same twice says nothing by
        // printing differently on another tree.
        let after = match (
            run(&mut now_cx, &now_at, check),
            run(&mut now_cx, &now_at, check),
        ) {
            (Some(a), Some(b)) if a == b => a,
            (Some(_), Some(_)) => {
                report.unsteady += 1;
                continue;
            }
            _ => {
                report.failed_now += 1;
                continue;
            }
        };
        match run(&mut before_cx, &before_at, check) {
            Some(before) if before == after => report.same += 1,
            Some(before) => report.differ.push(Differ {
                command: (*check).to_owned(),
                before,
                now: after,
            }),
            None => report.only_now += 1,
        }
    }
    Ok(())
}

/// The directories a copy leaves out at the world's root: the repository and the harness's own
/// records (a check needs neither), and build output, which rebuilds.
const UNCOPIED: [&str; 4] = [".git", ".mikata", "target", "node_modules"];
/// The caches a copy leaves out wherever they lie: each was written for another tree.
const CACHES: [&str; 5] = [
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".hypothesis",
];

/// `from` copied into `to`, links kept as links and never followed, until the budget of bytes or
/// files is spent: `false` when it was.
fn copy_capped(
    from: &Path,
    to: &Path,
    top: bool,
    bytes: &mut u64,
    files: &mut u32,
) -> io::Result<bool> {
    fs::create_dir_all(to)?;
    for e in fs::read_dir(from)? {
        let e = e?;
        let name = e.file_name();
        if top && UNCOPIED.iter().any(|u| name == *u) || CACHES.iter().any(|c| name == *c) {
            continue;
        }
        let kind = e.file_type()?;
        let dest = to.join(&name);
        if kind.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(fs::read_link(e.path())?, &dest)?;
        } else if kind.is_dir() {
            if !copy_capped(&e.path(), &dest, false, bytes, files)? {
                return Ok(false);
            }
        } else if kind.is_file() {
            let size = e.metadata()?.len();
            if size > *bytes || *files == 0 {
                return Ok(false);
            }
            *bytes -= size;
            *files -= 1;
            fs::copy(e.path(), &dest)?;
        }
    }
    Ok(true)
}

/// One line of two outputs set side by side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line<'a> {
    /// In both.
    Both(&'a str),
    /// Only in what the check printed before.
    Gone(&'a str),
    /// Only in what it prints now.
    New(&'a str),
}

/// `before` and `now`, line by line: the longest run of lines they share is kept, the rest is
/// gone or new. Outputs longer than [`DIFFED`] lines are set side by side up to there.
fn lines<'a>(before: &'a str, now: &'a str) -> Vec<Line<'a>> {
    let a: Vec<&str> = before.lines().take(DIFFED).collect();
    let b: Vec<&str> = now.lines().take(DIFFED).collect();
    // The length of the longest common run of lines from each pair of places on.
    let mut longest = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            longest[i][j] = if a[i] == b[j] {
                longest[i + 1][j + 1] + 1
            } else {
                longest[i + 1][j].max(longest[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            out.push(Line::Both(a[i]));
            i += 1;
            j += 1;
        } else if longest[i + 1][j] >= longest[i][j + 1] {
            out.push(Line::Gone(a[i]));
            i += 1;
        } else {
            out.push(Line::New(b[j]));
            j += 1;
        }
    }
    out.extend(a[i..].iter().map(|l| Line::Gone(l)));
    out.extend(b[j..].iter().map(|l| Line::New(l)));
    out
}

/// The changed lines of one check with the two lines above each change, which most often say
/// what was asked: at most `most` lines, and how many changed lines were left out.
fn shown(before: &str, now: &str, most: usize) -> (Vec<String>, usize) {
    let all = lines(before, now);
    let changed: Vec<bool> = all.iter().map(|l| !matches!(l, Line::Both(_))).collect();
    let mut keep = vec![false; all.len()];
    for (i, c) in changed.iter().enumerate() {
        if *c {
            for k in keep.iter_mut().take(i + 1).skip(i.saturating_sub(2)) {
                *k = true;
            }
        }
    }
    let mut out = Vec::new();
    let mut left = 0;
    let mut last: Option<usize> = None;
    for (i, line) in all.iter().enumerate() {
        if !keep[i] {
            continue;
        }
        if out.len() >= most {
            left += usize::from(changed[i]);
            continue;
        }
        // A blank line between two places that are not next to each other.
        if last.is_some_and(|l| l + 1 != i) && !out.last().is_some_and(String::is_empty) {
            out.push(String::new());
        }
        last = Some(i);
        out.push(match line {
            Line::Both(l) => format!("  {l}"),
            Line::Gone(l) => format!("- {l}"),
            Line::New(l) => format!("+ {l}"),
        });
    }
    (out, left)
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What is left out of the comparison, said by count and by why: `None` when nothing is.
fn left_out(r: &Report) -> Option<String> {
    let mut said = Vec::new();
    if r.only_now > 0 {
        said.push(format!(
            "{} only on the finished work",
            plural(r.only_now, "runs", "run")
        ));
    }
    if r.failed_now > 0 {
        said.push(format!(
            "{} on the finished work",
            plural(r.failed_now, "fails", "fail")
        ));
    }
    if r.unsteady > 0 {
        said.push(format!(
            "{} not print the same twice",
            plural(r.unsteady, "does", "do")
        ));
    }
    if r.not_reached > 0 {
        said.push(format!(
            "{} not reached in time",
            plural(r.not_reached, "was", "were")
        ));
    }
    (!said.is_empty()).then(|| said.join(", "))
}

/// What the human is shown: one line that counts, then the lines of the newest check that
/// prints differently, then where the rest is. `None` when there is nothing to say: no check
/// was taken.
#[must_use]
pub fn said(r: &Report, who: &str) -> Option<String> {
    if r.taken == 0 {
        return None;
    }
    let of = format!("of {who}'s last {}", plural(r.taken, "check", "checks"));
    if let Some(why) = &r.why_not {
        return Some(format!("what changed: not looked at, {why}"));
    }
    let compared = r.same + r.differ.len();
    let mut s = if compared == 0 {
        format!("what changed: none {of} could be set beside the code you gave it")
    } else if r.differ.is_empty() {
        format!("what changed: {of}, {compared} print the same as on the code you gave it")
    } else {
        format!(
            "what changed: {of}, {} differently than on the code you gave it",
            plural(r.differ.len(), "prints", "print")
        )
    };
    if let Some(left) = left_out(r) {
        let _ = write!(s, " ({left})");
    }
    let Some(first) = r.differ.first() else {
        return Some(s);
    };
    let (shown, more) = shown(&first.before, &first.now, SHOWN);
    s.push_str(if r.differ.len() == 1 {
        ":\n"
    } else {
        ". The last of them:\n"
    });
    for line in &shown {
        s.push('\n');
        s.push_str(line);
    }
    let mut rest = Vec::new();
    if more > 0 {
        rest.push(format!(
            "{} in this check",
            plural(more, "more changed line", "more changed lines")
        ));
    }
    if r.differ.len() > 1 {
        rest.push(plural(r.differ.len() - 1, "more check", "more checks"));
    }
    let _ = write!(
        s,
        "\n\n{}{FILE}",
        if rest.is_empty() {
            "whole: ".to_owned()
        } else {
            format!("{}: ", rest.join(", "))
        }
    );
    Some(s)
}

/// The whole of it, for the file: every check that prints differently, with its command.
#[must_use]
pub fn written(r: &Report, who: &str) -> String {
    let mut s = String::from("# What changed, by the checks\n\n");
    let _ = writeln!(
        s,
        "{who}'s last {} run again by the harness, on the code as the ask found it and on the finished work. A line with `-` is what a check printed before; one with `+`, what it prints now.\n",
        plural(r.taken, "check", "checks")
    );
    let _ = writeln!(
        s,
        "{} differently, {} the same.{}\n",
        plural(r.differ.len(), "prints", "print"),
        plural(r.same, "prints", "print"),
        left_out(r)
            .map(|l| format!(" Left out: {l}."))
            .unwrap_or_default()
    );
    for (n, d) in r.differ.iter().enumerate() {
        let _ = writeln!(
            s,
            "## {}\n\n```sh\n{}\n```\n\n```diff",
            n + 1,
            d.command.trim_end()
        );
        let (all, _) = shown(&d.before, &d.now, usize::MAX);
        for line in all {
            let _ = writeln!(s, "{line}");
        }
        s.push_str("```\n\n");
    }
    s
}

/// Where the file goes, beneath `world`.
#[must_use]
pub fn file(world: &Path) -> PathBuf {
    world.join(FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_check_is_an_interpreter_handed_its_code() {
        for yes in [
            r#"python3 -c "import a; print(a.f(1))""#,
            r#"cd /w && python3 -c "print(1)" 2>&1 | head -20"#,
            "cd /w && PYTHONPATH=. python -c 'print(1)'",
            "python3.12 -W ignore -c 'print(1)'",
            "uv run python -c 'print(1)'",
            "cd pkg && node -e 'console.log(1)'",
            "python3 << 'EOF'\nprint(1)\nEOF",
            "cd /w && python3 - <<EOF\nprint(1)\nEOF",
            "/usr/bin/perl -e 'print 1'",
        ] {
            assert!(is_check(yes), "{yes}");
        }
        for no in [
            "python3 -m pytest tests -x",
            "python3 script.py",
            "python3 script.py <<EOF\n1\nEOF",
            "grep -n interval sqlglot/parser.py",
            "git stash && python3 -c 'print(1)'",
            "cd /w && git diff",
            "cargo test",
            "ls; python3 -c 'print(1)'",
            "",
        ] {
            assert!(!is_check(no), "{no}");
        }
    }

    #[test]
    fn checks_are_kept_once_each_newest_last() {
        let mut c = Checks::default();
        for cmd in ["python3 -c 'a'", "ls", "python3 -c 'b'", "python3 -c 'a'"] {
            c.saw(Some(cmd));
        }
        assert_eq!(c.newest(12), ["python3 -c 'a'", "python3 -c 'b'"]);
        assert_eq!(c.newest(1), ["python3 -c 'a'"]);
        assert!(Checks::default().is_empty());
    }

    #[test]
    fn two_outputs_are_set_side_by_side_with_what_was_asked_above() {
        let before =
            "Tests\n\nOne:\n  Input:  a\n  Output: 1\n\nTwo:\n  Input:  b\n  Output: 2\n\ndone";
        let now =
            "Tests\n\nOne:\n  Input:  a\n  Output: 1\n\nTwo:\n  Input:  b\n  Output: 3\n\ndone";
        let (shown, more) = shown(before, now, 16);
        assert_eq!(
            shown,
            ["  Two:", "    Input:  b", "-   Output: 2", "+   Output: 3"]
        );
        assert_eq!(more, 0);
        // Cut at the most lines, the changed lines left out counted.
        let (shown, more) = super::shown("a\nb\nc", "x\ny\nz", 2);
        assert_eq!(shown, ["- a", "- b"]);
        assert_eq!(more, 4);
    }

    fn differ(before: &str, now: &str) -> Differ {
        Differ {
            command: "python3 -c 'x'".into(),
            before: before.into(),
            now: now.into(),
        }
    }

    #[test]
    fn the_human_is_told_by_count_then_shown_the_newest() {
        assert_eq!(
            said(&Report::default(), "Mikata"),
            None,
            "no check, nothing to say"
        );
        let same = Report {
            taken: 3,
            same: 3,
            ..Report::default()
        };
        assert_eq!(
            said(&same, "Mikata").unwrap(),
            "what changed: of Mikata's last 3 checks, 3 print the same as on the code you gave it"
        );
        let r = Report {
            taken: 5,
            same: 1,
            differ: vec![differ("in: a\nout: 1", "in: a\nout: 2"), differ("p", "q")],
            only_now: 1,
            unsteady: 1,
            ..Report::default()
        };
        assert_eq!(
            said(&r, "Mikata").unwrap(),
            "what changed: of Mikata's last 5 checks, 2 print differently than on the code you gave it \
             (1 runs only on the finished work, 1 does not print the same twice). The last of them:\n\n  \
             in: a\n- out: 1\n+ out: 2\n\n1 more check: .mikata/tmp/what-changed.md"
        );
        let none = Report {
            taken: 2,
            only_now: 2,
            ..Report::default()
        };
        assert_eq!(
            said(&none, "Mikata").unwrap(),
            "what changed: none of Mikata's last 2 checks could be set beside the code you gave it \
             (2 run only on the finished work)"
        );
        let cannot = Report {
            taken: 2,
            why_not: Some("the world is too large to copy (over 256 MiB or 40000 files)".into()),
            ..Report::default()
        };
        assert!(
            said(&cannot, "Mikata")
                .unwrap()
                .starts_with("what changed: not looked at, the world is too large")
        );
    }

    #[test]
    fn the_file_holds_every_check_that_differs_with_its_command() {
        let r = Report {
            taken: 2,
            differ: vec![differ("a\nb", "a\nc")],
            same: 1,
            ..Report::default()
        };
        let w = written(&r, "Mikata");
        assert!(w.contains("```sh\npython3 -c 'x'\n```"), "{w}");
        assert!(w.contains("```diff\n  a\n- b\n+ c\n```"), "{w}");
        assert!(
            w.contains("1 prints differently, 1 prints the same."),
            "{w}"
        );
    }
}
