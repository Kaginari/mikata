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
//! What is shown first is what a human can judge: a place that prints another value. A place that
//! only flips the body's own mark (`FAIL` to `PASS`) is the body's opinion of its work and comes
//! last. A check that fails on the code as the ask found it has nothing to stand beside there; what
//! it prints now is shown as new, after the checks that ran on both. On 42 kept runs of our bench
//! that order shows a wrong value in 18 of 26 failed runs, where the newest place first showed it
//! in 14, and no other order by code did better.
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
/// The most lines of the checks' outputs shown to the human; the whole of it is in the file.
const SHOWN: usize = 20;
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

/// The test runners a body drives from a line of code. What such a run prints is the runner's:
/// names, marks, and how long it took, which no two runs print alike.
const RUNNERS: [&str; 2] = ["unittest", "pytest"];

/// Whether `command` is a check: its first program is an interpreter handed its code on the
/// command line or through a here-document. A test runner is not one (the end gate judges
/// those), whether started by name or driven from the code handed over; nor is a look at a file,
/// nor a command that does something else first.
#[must_use]
pub fn is_check(command: &str) -> bool {
    if command
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|word| RUNNERS.contains(&word))
    {
        return false;
    }
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
    /// What it printed on the code as the ask found it: as far as it got, when it failed there.
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
    /// Those that run only on the finished work, the body's newest first: on the code as the ask
    /// found it they fail (a name the work added, an assertion the work makes true). `before` is
    /// what each printed there before it failed.
    pub only_now: Vec<Differ>,
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
    let root = scratch_root();
    let _ = fs::remove_dir_all(&root);
    report.why_not =
        run_in_copies(world, &root, from, attic, start, checks, tools, &mut report).err();
    let _ = fs::remove_dir_all(&root);
    report.ms = u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX);
    report
}

/// What a check did on one tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Said {
    /// Whether it ran to its end and left with 0.
    pub ok: bool,
    /// What it printed on its standard output, as far as it got.
    pub out: String,
    /// The last line of what it said besides: its error, when it failed.
    pub err: String,
}

/// A copy of a tree, and what runs a check in it.
struct Copied {
    at: PathBuf,
    cx: Context,
    /// The paths a command may name its world by: each is this copy's here.
    names: Vec<String>,
}

impl Copied {
    /// A copy of `tree` under `root/<name>`. Its commands run in `from`'s sandbox; `written_in` is the
    /// world the body's commands were written in, whose path they name in full more often than
    /// not.
    fn of(
        tree: &Path,
        written_in: &Path,
        root: &Path,
        name: &str,
        from: &Context,
    ) -> Result<Self, String> {
        let copy = root
            .join(name)
            .join(written_in.file_name().unwrap_or_else(|| "world".as_ref()));
        let (mut bytes, mut files) = (COPY_BYTES, COPY_FILES);
        match copy_capped(tree, &copy, true, &mut bytes, &mut files) {
            Ok(true) => {}
            Ok(false) => {
                return Err(format!(
                    "the world is too large to copy (over {} MiB or {COPY_FILES} files)",
                    COPY_BYTES / (1024 * 1024)
                ));
            }
            Err(e) => return Err(format!("the world could not be copied: {e}")),
        }
        let at = copy
            .canonicalize()
            .map_err(|e| format!("the copy could not be read: {e}"))?;
        let cx = Context::new(Race::Slime, &at, &["."])
            .with_sandbox(from.sandbox().clone())
            .with_unguarded(from.allows_unguarded())
            .with_home(from.home());
        let mut names = vec![written_in.display().to_string()];
        if let Ok(real) = written_in.canonicalize() {
            let real = real.display().to_string();
            if !names.contains(&real) {
                names.push(real);
            }
        }
        Ok(Self { at, cx, names })
    }

    /// What `command` does here, this copy's own path written `<world>`.
    fn says(&mut self, command: &str, tools: &[Box<dyn Tool>]) -> Said {
        let here = self.at.display().to_string();
        let mut command = command.to_owned();
        for name in &self.names {
            command = command.replace(name.as_str(), &here);
        }
        let args = json!({
            "command": command,
            "timeout_ms": u64::try_from(EACH.as_millis()).unwrap_or(20_000),
        });
        let o = call(tools, "bash", &args.to_string(), &mut self.cx);
        said_of(&o, &[here])
    }
}

/// What a `bash` call did, as [`Said`]: `names` are the paths of the tree it ran in, each
/// written `<world>`, so that two trees say the same thing in the same words.
pub(crate) fn said_of(o: &mikata_tools::Outcome, names: &[String]) -> Said {
    let placed = |text: &str| {
        names.iter().fold(text.to_owned(), |t, name| {
            t.replace(name.as_str(), "<world>")
        })
    };
    let out = stdout_of(&o.content);
    // What it said besides: the last line of it, which is where an error names itself.
    let err = o.content[out.len()..]
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty() && *l != "[stderr]" && !l.starts_with("[exit "))
        .unwrap_or_default();
    Said {
        ok: !o.is_error,
        out: placed(out).trim_end().to_owned(),
        err: placed(err),
    }
}

/// What a command prints, from two runs of it on one tree: its output when it ran both times and
/// printed the same, something, twice. Otherwise why it says nothing about that tree, in words
/// that go after "and".
pub(crate) fn one_of(first: &Said, second: &Said) -> Result<String, String> {
    if !first.ok {
        return Err(if first.err.is_empty() {
            "it fails there".to_owned()
        } else {
            format!("it fails there ({})", first.err)
        });
    }
    if !second.ok || first.out != second.out {
        return Err("it does not print the same twice there".to_owned());
    }
    if first.out.is_empty() {
        return Err("it prints nothing there".to_owned());
    }
    Ok(first.out.clone())
}

/// What `command` prints on the code as the ask found it, once the world has moved on: run twice
/// in a copy of the world put back to `start`.
///
/// # Errors
///
/// Why it could not be read, in words that go after "and": the copy, a run that fails, two runs
/// that differ, or nothing printed.
pub(crate) fn printed_before(
    world: &Path,
    from: &Context,
    attic: &Attic,
    start: &Snapshot,
    command: &str,
    tools: &[Box<dyn Tool>],
) -> Result<String, String> {
    let root = scratch_root();
    let _ = fs::remove_dir_all(&root);
    let read = (|| {
        let mut before = Copied::of(world, world, &root, "before", from)?;
        attic
            .export_into(&start.tree, &before.at)
            .map_err(|e| format!("the code as you found it could not be laid out: {e}"))?;
        before.cx.set_deadline(Some(Instant::now() + BUDGET));
        let (first, second) = (before.says(command, tools), before.says(command, tools));
        one_of(&first, &second)
    })();
    let _ = fs::remove_dir_all(&root);
    read
}

/// A folder for a step's copies, its own each time.
fn scratch_root() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "mikata-changes-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}

/// What each of `checks` does in a copy of `tree`, once. `named` is the world the checks were
/// written in. For a study that sets a third tree beside the two [`compare`] runs: the code as it
/// should have become.
///
/// # Errors
///
/// The tree cannot be copied.
pub fn printed(
    tree: &Path,
    named: &Path,
    from: &Context,
    checks: &[&str],
    tools: &[Box<dyn Tool>],
) -> Result<Vec<Said>, String> {
    let root = scratch_root();
    let _ = fs::remove_dir_all(&root);
    let out = Copied::of(tree, named, &root, "tree", from)
        .map(|mut copy| checks.iter().map(|check| copy.says(check, tools)).collect());
    let _ = fs::remove_dir_all(&root);
    out
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
    let mut now = Copied::of(world, world, root, "now", from)?;
    let mut before = Copied::of(world, world, root, "before", from)?;
    // The second copy, put back to the tree as the ask found it: what the world ignores (its
    // dependencies) stays as it is, so a check finds them there as it did.
    attic
        .export_into(&start.tree, &before.at)
        .map_err(|e| format!("the code as the ask found it could not be laid out: {e}"))?;
    let until = Instant::now() + BUDGET;
    now.cx.set_deadline(Some(until));
    before.cx.set_deadline(Some(until));
    for (i, check) in checks.iter().enumerate() {
        if Instant::now() >= until {
            report.not_reached += checks.len() - i;
            break;
        }
        // The finished work, twice: a check that will not print the same twice says nothing by
        // printing differently on another tree.
        let (first, second) = (now.says(check, tools), now.says(check, tools));
        if !(first.ok && second.ok) {
            report.failed_now += 1;
            continue;
        }
        if first.out != second.out {
            report.unsteady += 1;
            continue;
        }
        let was = before.says(check, tools);
        if was.ok && was.out == first.out {
            report.same += 1;
            continue;
        }
        let ran_before = was.ok;
        let differ = Differ {
            command: (*check).to_owned(),
            before: was.out,
            now: first.out,
        };
        if ran_before {
            report.differ.push(differ);
        } else {
            report.only_now.push(differ);
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

/// One place a check prints differently: its changed lines, with the two lines above the first
/// of them, which most often say what was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Place {
    /// The lines, each with its sign: two spaces, `- ` or `+ `.
    lines: Vec<String>,
    /// How many of them changed.
    changed: usize,
}

impl Place {
    /// What makes two places the same one: their changed lines, whatever stood above them.
    fn key(&self) -> String {
        self.lines
            .iter()
            .filter(|l| !l.starts_with("  "))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The body's own marks on its checks, and the words a verdict line is made of ("All tests
/// passed", "Pass: True"). `true` and `false` can be values: taking them for marks only ever
/// counts a value as a mark, which puts a place later, never hides it.
const MARKS: [&str; 23] = [
    "pass",
    "passed",
    "passes",
    "passing",
    "fail",
    "failed",
    "fails",
    "failing",
    "ok",
    "error",
    "errors",
    "true",
    "false",
    "yes",
    "no",
    "all",
    "some",
    "success",
    "successful",
    "successfully",
    "correct",
    "incorrect",
    "wrong",
];
/// The same said with a sign.
const SIGNS: [char; 6] = ['✓', '✗', '✔', '✘', '✅', '❌'];

/// A line with each mark put as `#`: two lines equal so differ only by a mark.
fn masked(line: &str) -> String {
    fn close(word: &mut String, out: &mut String) {
        if MARKS.contains(&word.to_lowercase().as_str()) {
            out.push('#');
        } else {
            out.push_str(word);
        }
        word.clear();
    }
    let mut out = String::with_capacity(line.len());
    let mut word = String::new();
    for c in line.chars() {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        close(&mut word, &mut out);
        out.push(if SIGNS.contains(&c) { '#' } else { c });
    }
    close(&mut word, &mut out);
    out
}

/// Whether a line, its marks aside, holds anything a reader could judge: a number, a quoted or
/// bracketed thing, or a word after a colon or an equals sign. A verdict in plain words ("All
/// comprehensive tests passed!") holds none, however it is worded.
fn says_something(line: &str) -> bool {
    let line: Vec<char> = masked(line).chars().collect();
    line.iter().enumerate().any(|(i, c)| {
        c.is_ascii_digit()
            || "'\"`()[]{}<>".contains(*c)
            || (matches!(c, ':' | '=')
                && line[i + 1..]
                    .iter()
                    .find(|next| !next.is_whitespace())
                    .is_some_and(|next| !matches!(next, '#' | '=')))
    })
}

impl Place {
    /// Whether none of its new lines holds a value: each is a mark flipped (the line, marks
    /// aside, is a line of it that is gone) or a verdict in plain words. A place with no new
    /// line (lines gone and nothing in their stead) is not one.
    fn marks_only(&self) -> bool {
        let gone: std::collections::BTreeSet<String> = self
            .lines
            .iter()
            .filter_map(|l| l.strip_prefix("- "))
            .map(masked)
            .collect();
        let mut new = self
            .lines
            .iter()
            .filter_map(|l| l.strip_prefix("+ "))
            .peekable();
        new.peek().is_some()
            && new.all(|line| {
                let aside = masked(line);
                (aside.contains('#') && gone.contains(&aside)) || !says_something(line)
            })
    }
}

/// The places `before` and `now` differ, in the order they were printed.
fn places(before: &str, now: &str) -> Vec<Place> {
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
    let mut out: Vec<Place> = Vec::new();
    let mut last: Option<usize> = None;
    for (i, line) in all.iter().enumerate() {
        if !keep[i] {
            continue;
        }
        // Lines next to each other are one place.
        if last.is_none_or(|l| l + 1 != i) {
            out.push(Place {
                lines: Vec::new(),
                changed: 0,
            });
        }
        last = Some(i);
        let place = out.last_mut().expect("a place was just opened");
        place.changed += usize::from(changed[i]);
        place.lines.push(match line {
            Line::Both(l) => format!("  {l}"),
            Line::Gone(l) => format!("- {l}"),
            Line::New(l) => format!("+ {l}"),
        });
    }
    // A blank line above a change says nothing of it.
    for place in &mut out {
        while place.lines.first().is_some_and(|l| l.trim().is_empty()) {
            place.lines.remove(0);
        }
    }
    out
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What is left out of the comparison, said by count and by why: `None` when nothing is.
fn left_out(r: &Report) -> Option<String> {
    let mut said = Vec::new();
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

/// The line that counts: how many of the checks print differently, the same, or only now.
fn counted(r: &Report, who: &str) -> String {
    let of = format!("of {who}'s last {}", plural(r.taken, "check", "checks"));
    let differently = format!(
        "{} differently than on the code you gave it",
        plural(r.differ.len(), "prints", "print")
    );
    let the_same = format!(
        "{} the same as on the code you gave it",
        plural(r.same, "prints", "print")
    );
    let only_now = format!(
        "{} only on the finished work",
        plural(r.only_now.len(), "runs", "run")
    );
    let mut s = match (r.differ.is_empty(), r.only_now.is_empty(), r.same == 0) {
        (true, true, true) => {
            format!("what changed: none {of} could be set beside the code you gave it")
        }
        (true, true, false) => format!("what changed: {of}, {the_same}"),
        (true, false, true) => format!("what changed: {of}, {only_now}"),
        (true, false, false) => format!("what changed: {of}, {the_same} and {only_now}"),
        (false, true, _) => format!("what changed: {of}, {differently}"),
        (false, false, _) => format!("what changed: {of}, {differently} and {only_now}"),
    };
    if let Some(left) = left_out(r) {
        let _ = write!(s, " ({left})");
    }
    s
}

/// The places to show, in order: those of the checks that print differently, the body's newest
/// check first, then those of the checks that run only now; and of all of them, the places that
/// print a value before the ones that only flip a mark.
fn ordered(r: &Report) -> Vec<Place> {
    let (values, marks): (Vec<Place>, Vec<Place>) = r
        .differ
        .iter()
        .chain(&r.only_now)
        .flat_map(|d| places(&d.before, &d.now))
        .partition(|place| !place.marks_only());
    values.into_iter().chain(marks).collect()
}

/// What the human is shown: one line that counts, then the places the checks print differently
/// ([`ordered`]), each once, until [`SHOWN`] lines are used, then where the whole of it is. A
/// place too long for the list is cut, and says by how much. `None` when there is nothing to say:
/// no check was taken.
#[must_use]
pub fn said(r: &Report, who: &str) -> Option<String> {
    if r.taken == 0 {
        return None;
    }
    if let Some(why) = &r.why_not {
        return Some(format!("what changed: not looked at, {why}"));
    }
    let mut s = counted(r, who);
    if r.differ.is_empty() && r.only_now.is_empty() {
        return Some(s);
    }
    s.push(':');
    let mut seen = std::collections::BTreeSet::new();
    let (mut used, mut more) = (0, 0);
    for place in ordered(r) {
        // The same change printed by two checks is said once.
        if !seen.insert(place.key()) {
            continue;
        }
        // A place that with its blank line would pass the list's lines is shown two short of
        // them, then one line that says how much more of it there is.
        let cut = place.lines.len() + 1 > SHOWN;
        let (shown, cost) = if cut {
            (SHOWN - 2, SHOWN - 1)
        } else {
            (place.lines.len(), place.lines.len())
        };
        if used > 0 && used + cost + 1 > SHOWN {
            more += place.changed;
            continue;
        }
        s.push('\n');
        for line in &place.lines[..shown] {
            s.push('\n');
            s.push_str(line);
        }
        if cut {
            let rest = &place.lines[shown..];
            more += rest.iter().filter(|l| !l.starts_with("  ")).count();
            let _ = write!(
                s,
                "\n  … {} of this place",
                plural(rest.len(), "more line", "more lines")
            );
        }
        used += cost + 1;
    }
    let whole = format!(
        "the {} whole: {FILE}",
        plural(r.differ.len() + r.only_now.len(), "check", "checks")
    );
    if more > 0 {
        let _ = write!(
            s,
            "\n\n{} not shown; {whole}",
            plural(more, "more changed line", "more changed lines")
        );
    } else {
        let _ = write!(s, "\n\n{whole}");
    }
    Some(s)
}

/// The whole of it, for the file: every check that prints differently or runs only now, with its
/// command.
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
        "{} differently, {} the same, {} only on the finished work.{}\n",
        plural(r.differ.len(), "prints", "print"),
        plural(r.same, "prints", "print"),
        plural(r.only_now.len(), "runs", "run"),
        left_out(r)
            .map(|l| format!(" Left out: {l}."))
            .unwrap_or_default()
    );
    let only_now = r.only_now.iter().map(|d| (d, true));
    for (n, (d, new)) in r
        .differ
        .iter()
        .map(|d| (d, false))
        .chain(only_now)
        .enumerate()
    {
        let _ = writeln!(
            s,
            "## {}{}\n\n```sh\n{}\n```\n",
            n + 1,
            if new {
                ": runs only on the finished work"
            } else {
                ""
            },
            d.command.trim_end()
        );
        let found = places(&d.before, &d.now);
        if found.is_empty() {
            // It failed on the code as the ask found it after printing all it prints now.
            s.push_str("It prints the same lines on both.\n\n");
            continue;
        }
        s.push_str("```diff\n");
        for (i, place) in found.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            for line in &place.lines {
                let _ = writeln!(s, "{line}");
            }
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
            // A runner driven from the code handed over is a test run all the same.
            "python3 -c \"import unittest; from tests.test_a import A; unittest.TextTestRunner().run(unittest.makeSuite(A))\"",
            "cd /w && python3 -c 'import pytest; pytest.main([\"-x\", \"tests\"])'",
            "",
        ] {
            assert!(!is_check(no), "{no}");
        }
        // A word that only holds a runner's name names none.
        assert!(is_check(
            "python3 -c 'import my_unittest_helpers as h; print(h.f())'"
        ));
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
            "Tests\n\nOne:\n  Input:  a\n  Output: 9\n\nTwo:\n  Input:  b\n  Output: 3\n\ndone";
        let found = places(before, now);
        assert_eq!(
            found.len(),
            2,
            "two places, not next to each other: {found:?}"
        );
        assert_eq!(
            found[1].lines,
            ["  Two:", "    Input:  b", "-   Output: 2", "+   Output: 3"]
        );
        assert_eq!(found[1].changed, 2);
        assert_eq!(found[1].key(), "-   Output: 2\n+   Output: 3");
        assert_eq!(places("same\n", "same\n"), Vec::new());
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
            // The newest check first; the second prints the first's change again, and one more.
            differ: vec![
                differ("in: a\nout: 1", "in: a\nout: 2"),
                differ(
                    "in: a\nout: 1\n\n\nin: b\nout: 5",
                    "in: a\nout: 2\n\n\nin: b\nout: 6",
                ),
            ],
            // One more fails on the code as it was: what it prints now is new, and comes after.
            only_now: vec![differ("", "made: 7")],
            unsteady: 1,
            ..Report::default()
        };
        assert_eq!(
            said(&r, "Mikata").unwrap(),
            "what changed: of Mikata's last 5 checks, 2 print differently than on the code you gave it \
             and 1 runs only on the finished work (1 does not print the same twice):\n\n  \
             in: a\n- out: 1\n+ out: 2\n\n  in: b\n- out: 5\n+ out: 6\n\n+ made: 7\n\n\
             the 3 checks whole: .mikata/tmp/what-changed.md"
        );
        // Past the lines shown, what is left is counted.
        let mut many = String::new();
        for i in 0..30 {
            let _ = write!(many, "k{i}: old\n\n\n\n");
        }
        let long = Report {
            taken: 1,
            differ: vec![differ(&many, &many.replace("old", "new"))],
            ..Report::default()
        };
        let text = said(&long, "Mikata").unwrap();
        assert!(text.lines().count() <= SHOWN + 6, "{text}");
        assert!(
            text.ends_with(
                "more changed lines not shown; the 1 check whole: .mikata/tmp/what-changed.md"
            ),
            "{text}"
        );
        // An ask that adds what the code could not do: every check fails on the code as it was.
        let new = Report {
            taken: 3,
            same: 1,
            only_now: vec![differ("", "made: 7"), differ("", "made: 7\nkept: 2")],
            ..Report::default()
        };
        assert_eq!(
            said(&new, "Mikata").unwrap(),
            "what changed: of Mikata's last 3 checks, 1 prints the same as on the code you gave it \
             and 2 run only on the finished work:\n\n+ made: 7\n\n+ made: 7\n+ kept: 2\n\n\
             the 2 checks whole: .mikata/tmp/what-changed.md"
        );
        let none = Report {
            taken: 2,
            failed_now: 2,
            ..Report::default()
        };
        assert_eq!(
            said(&none, "Mikata").unwrap(),
            "what changed: none of Mikata's last 2 checks could be set beside the code you gave it \
             (2 fail on the finished work)"
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
    fn a_mark_is_told_from_a_value() {
        for (line, aside, says) in [
            (
                "  Output: SELECT INTERVAL '1.5 HOURS'",
                "  Output: SELECT INTERVAL '1.5 HOURS'",
                true,
            ),
            ("D1: PASS", "D1: #", true),
            (
                "✓ duckdb -> spark: SLICE(arr, 1, 6)",
                "# duckdb -> spark: SLICE(arr, 1, 6)",
                true,
            ),
            ("tables type: Identifier", "tables type: Identifier", true),
            ("num tables: 3", "num tables: 3", true),
            (
                "All comprehensive tests passed!",
                "# comprehensive tests #!",
                false,
            ),
            ("=== ALL TESTS PASSED ===", "=== # TESTS # ===", false),
            ("Some tests failed!", "# tests #!", false),
            ("OVERALL: ALL TESTS PASSED", "OVERALL: # TESTS #", false),
            (
                "Fractional preserved: True",
                "Fractional preserved: #",
                false,
            ),
            // A word that only holds a mark is no mark.
            ("bypass_all: skipped", "bypass_all: skipped", true),
        ] {
            assert_eq!(masked(line), aside, "{line}");
            assert_eq!(says_something(line), says, "{line}");
        }
    }

    #[test]
    fn a_place_that_prints_a_value_comes_before_one_that_only_flips_a_mark() {
        // The newest check grades itself; the older one prints what the code returns.
        let graded = differ(
            "D1: FAIL\n  Expected: 6\n  Got: 5\n\n\n\nSome tests failed!",
            "D1: PASS\n\n\n\nAll comprehensive tests passed!",
        );
        let printed = differ("in: a\nout: 1", "in: a\nout: 2");
        let found = places(&graded.before, &graded.now);
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found.iter().all(Place::marks_only), "{found:?}");
        assert!(!places(&printed.before, &printed.now)[0].marks_only());
        // Lines gone with nothing in their stead are no mark flipped.
        assert!(!places("a: 1\nb: 2", "a: 1")[0].marks_only());
        let r = Report {
            taken: 2,
            differ: vec![graded, printed],
            ..Report::default()
        };
        assert_eq!(
            said(&r, "Mikata").unwrap(),
            "what changed: of Mikata's last 2 checks, 2 print differently than on the code you gave it:\n\n  \
             in: a\n- out: 1\n+ out: 2\n\n\
             - D1: FAIL\n-   Expected: 6\n-   Got: 5\n+ D1: PASS\n\n\
             - Some tests failed!\n+ All comprehensive tests passed!\n\n\
             the 2 checks whole: .mikata/tmp/what-changed.md"
        );
    }

    #[test]
    fn a_place_too_long_for_the_list_is_cut_and_says_by_how_much() {
        let old = |n: usize| (0..n).fold(String::new(), |s, i| s + &format!("k{i}: old\n"));
        let before = old(30);
        let r = Report {
            taken: 2,
            // The long place is the newest check's; a short one of an older check still fits no
            // more, and is counted.
            differ: vec![
                differ(&before, &before.replace("old", "new")),
                differ("in: a\nout: 1", "in: a\nout: 2"),
            ],
            ..Report::default()
        };
        let text = said(&r, "Mikata").unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // The line that counts, a blank, eighteen lines of the place, the line that says how many
        // more, a blank, and where the whole is.
        assert_eq!(lines.len(), 1 + 1 + (SHOWN - 2) + 1 + 1 + 1, "{text}");
        assert_eq!(lines[2], "- k0: old");
        assert_eq!(lines[SHOWN], "  … 42 more lines of this place");
        assert_eq!(
            lines[SHOWN + 2],
            "44 more changed lines not shown; the 2 checks whole: .mikata/tmp/what-changed.md"
        );
        // One line short of the list, a place is shown whole.
        let before = old(9);
        let whole = Report {
            taken: 1,
            differ: vec![differ(
                &format!("top\n{before}"),
                &format!("top\n{}", before.replace("old", "new")),
            )],
            ..Report::default()
        };
        let text = said(&whole, "Mikata").unwrap();
        assert_eq!(text.lines().count(), 1 + 1 + (SHOWN - 1) + 1 + 1, "{text}");
        assert!(!text.contains("of this place"), "{text}");
    }

    #[test]
    fn the_file_holds_every_check_that_differs_with_its_command() {
        let r = Report {
            taken: 4,
            differ: vec![differ("a\nb", "a\nc")],
            same: 1,
            only_now: vec![differ("", "made: 7"), differ("made: 7", "made: 7")],
            ..Report::default()
        };
        let w = written(&r, "Mikata");
        assert!(w.contains("## 1\n\n```sh\npython3 -c 'x'\n```"), "{w}");
        assert!(w.contains("```diff\n  a\n- b\n+ c\n```"), "{w}");
        assert!(
            w.contains("1 prints differently, 1 prints the same, 2 run only on the finished work."),
            "{w}"
        );
        assert!(
            w.contains("## 2: runs only on the finished work\n\n```sh\npython3 -c 'x'\n```\n\n```diff\n+ made: 7\n```"),
            "{w}"
        );
        // It printed all of it before it failed there.
        assert!(
            w.contains("## 3: runs only on the finished work\n\n```sh\npython3 -c 'x'\n```\n\nIt prints the same lines on both."),
            "{w}"
        );
    }
}
