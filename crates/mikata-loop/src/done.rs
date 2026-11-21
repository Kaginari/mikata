//! The ask's own `## Done when` list, held by the harness.
//!
//! An ask may say what done means in lines of its own: `D1: ... must give ...`, `D5: ... must
//! stay as it is today`. The human writes them, or a reader that saw the ask alone. A body handed
//! such a list builds to the lines that say what must come out: on our bench every run got every
//! one of them right. What it does not hold is the other half. Of four failed runs, two broke
//! lines the list said must stay: one read "must stay as it is today" as "must come out as it went
//! in" and changed the code until that was true, and neither ran the line on the code before
//! changing it. A prompt line asking for that was tried before, and failed.
//!
//! So the harness holds the list, by code. For each line the body hands in a check (the `check`
//! tool): a command that prints what the line is about. A line that says what it must give is held
//! to that output, which has to be one the line states. A line that must stay states no output:
//! the harness runs its command on the code as the ask found it, tells the body what it prints
//! today, and holds the work to that. The body never says what "today" is.
//!
//! When an answer lands, every check is run on the work as it stands. A line with no check, or a
//! check that prints something else, fails the landing, and the body is told which line and what
//! it printed. Nothing here reads prose: the list's two phrases decide a line's kind, and equal
//! text decides a pass.
//!
//! **Who writes the list.** The human, in the ask. Or a reader: a model the human names
//! (`models.reader`), asked once an ask, shown the ask alone and never the code, and asked only
//! for these lines ([`READER`]). Its list is put after the ask and held the same way. A reader
//! must be a model that reads better than the body: on our bench a strong one's lines were right
//! in every list, and the body's own model, asked apart, wrote one wrong by a letter that three
//! runs in three then built to.
//!
//! Equal, for a line that must stay, is exact: its truth was read from the code. For a line that
//! states its output, letter case and blanks are set aside. Whoever wrote the list seldom knows
//! how the code spells what it prints: a reader shown one of our asks wrote `'1.5 day'` where the
//! code prints `'1.5 DAY'`, and held letter for letter the harness would have made the body break
//! the code to match. A line that holds only that way is counted, and the human is told.

use std::fmt::Write as _;
use std::time::Duration;

use mikata_races::ToolClass;
use mikata_tools::{Context, Outcome, Tool, call};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::changes::Said;
use crate::lines::Line;

/// The tool's name.
pub const CHECK: &str = "check";
/// How every failure of the list begins: what the gate's note and its retries know it by.
pub const SAID: &str = "done when:";
/// The landings a body may answer again after, when only the list fails: each note names the
/// lines and what they printed, so a second and a third try are not blind.
pub const ROUNDS: u32 = 2;
/// The writes refused while a line of the ask's list has no check. Past them the write goes
/// through and the gate says what is missing: a refusal is never the only way on.
const REFUSALS: u32 = 2;
/// How long one check may run.
const EACH: Duration = Duration::from_secs(60);
/// The characters of an output said back to the body or the human.
const SHOWN: usize = 400;
/// The words that make a line one that must stay as it is, and one that states its output.
const STAYS: &str = "must stay";
const GIVES: &str = "must give";

/// What a reader's call is journaled and priced as.
pub const READ: &str = "reader";
/// The most a reader may answer, in tokens, and how long it is waited for.
pub const READER_TOKENS: u32 = 6000;
/// See [`READER_TOKENS`].
pub const READER_WAIT: Duration = Duration::from_secs(90);
/// The most lines of a reader's list that are kept.
const READER_LINES: usize = 30;
/// What a reader is told. It names no language and no tool: the lines are for any codebase.
pub const READER: &str = "\
You are handed a request for a change to a codebase, before anyone works on it. You see the \
request alone, never the code. Write what done means for it, as cases a tester could run.

One case a line, named D1, D2, and so on, and nothing else. A case is one of two kinds:

D1: <the input, exactly, and where it applies> must give `<the exact output>`
D2: <the input, exactly, and where it applies> must stay as it is today

- `must give` only where the request fixes the output exactly. Write inputs and outputs in the \
request's own notation, in backticks.
- `must stay` for what the request says must keep working or must not change, and for the inputs \
nearest the change that it does not ask to change. Never write what such a case gives: the tester \
reads that from the code as it is today.
- Cover the rule the request states. Vary each number or part it names and include each boundary \
it draws. Choose inputs that tell the rule from its nearest misreading: an input on which two \
readings agree tells nothing.
- Where the request names several targets (dialects, platforms, commands), write each case for \
each of them.
- At most 30 lines. If the request asks for no change to code, write NONE.";

/// The `check` tool as the model sees it. The loop runs the call; this body only describes it.
pub struct CheckTool;

impl Tool for CheckTool {
    fn name(&self) -> &'static str {
        CHECK
    }

    fn description(&self) -> &'static str {
        "Hand the harness a check for one line of the `Done when` list: a command that prints what the line is about, and nothing else. With `gives`, the harness holds your work to that exact output. Without it the line is one that must stay as it is: the harness runs the command on the code as you found it, tells you what it prints today, and holds your work to that, so hand such a line in before you change a file. When you answer, the harness runs every check on your work, and one that prints something else fails your landing. One check a line: handing a line in again replaces its check."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "line": {"type": "string", "description": "The line, by name: `D5`."},
                "command": {"type": "string", "description": "One shell command that prints the line's output and nothing else: `python3 -c \"import calc; print(calc.total([1, 2]))\"`."},
                "gives": {"type": "string", "description": "The exact output the line states. Leave it out for a line that must stay as it is: the harness reads that from the code as you found it."}
            },
            "required": ["line", "command"],
            "additionalProperties": false
        })
    }

    fn class(&self) -> ToolClass {
        ToolClass::Read
    }

    fn example(&self) -> Value {
        json!({"line": "D2", "command": "python3 -c \"import calc; print(calc.total([1, 2]))\"", "gives": "3"})
    }

    fn run(&self, _: Value, _: &mut Context) -> Outcome {
        Outcome::error("no ask is open to hold a check for")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    line: String,
    command: String,
    #[serde(default)]
    gives: Option<String>,
}

/// One line's check, as the harness holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// The line it checks, by name.
    pub line: String,
    /// The command, as the body wrote it.
    pub command: String,
    /// What it must print.
    pub must: String,
    /// Whether that is what it printed on the code as the ask found it, and not an output the
    /// line states.
    pub stays: bool,
    /// Whether it was handed in, or handed in again, after the body had changed the code.
    pub late: bool,
}

/// What a line of the list is, by its own words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// It says `must stay`: held to what its command prints on the code as the ask found it.
    Stays,
    /// It says `must give`: held to an output it states.
    Gives,
    /// Neither: the body says which.
    Free,
}

fn kind(text: &str) -> Kind {
    let t = text.to_lowercase();
    if t.contains(STAYS) {
        Kind::Stays
    } else if t.contains(GIVES) {
        Kind::Gives
    } else {
        Kind::Free
    }
}

/// An ask's list and the checks handed in for it.
#[derive(Debug, Default)]
pub struct List {
    /// The lines the ask itself carries, in order. Empty for an ask with no list: the body may
    /// still hand in checks for lines of its own.
    lines: Vec<Line>,
    checks: Vec<Check>,
    refused: u32,
    /// Whether a reader wrote the lines, and not the ask itself: they are then said after the
    /// ask, which did not hold them.
    read: bool,
}

impl List {
    /// The list `ask` carries: its named lines under `## Done when`.
    #[must_use]
    pub fn of(ask: &str) -> Self {
        Self {
            lines: crate::lines::list(ask),
            ..Self::default()
        }
    }

    /// The list a reader wrote for an ask that carried none: its lines ([`read_lines`]), named
    /// in order.
    #[must_use]
    pub fn read(lines: &[String]) -> Self {
        Self {
            lines: lines
                .iter()
                .enumerate()
                .map(|(i, text)| (format!("D{}", i + 1), text.clone()))
                .collect(),
            read: !lines.is_empty(),
            ..Self::default()
        }
    }

    /// Whether the ask carries no list, its own or a reader's.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// What goes after the ask when a list is held: a reader's list itself, which the ask did
    /// not carry, then how the harness holds it ([`List::note`]). Nothing for an ask with no
    /// list.
    #[must_use]
    pub fn after_the_ask(&self) -> Option<String> {
        let note = self.note()?;
        if !self.read {
            return Some(note);
        }
        let mut said = "## Done when".to_owned();
        for (name, text) in &self.lines {
            let _ = write!(said, "\n{name}: {text}");
        }
        Some(format!("{said}\n\n{note}"))
    }

    /// Whether any check was handed in.
    #[must_use]
    pub fn any(&self) -> bool {
        !self.checks.is_empty()
    }

    /// The checks held, in the order they were first handed in.
    #[must_use]
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }

    fn span(&self) -> String {
        match self.lines.as_slice() {
            [] => String::new(),
            [(only, _)] => only.clone(),
            [(first, _), .., (last, _)] => format!("{first} to {last}"),
        }
    }

    /// What the body is told with an ask that carries a list; nothing for one that carries none.
    #[must_use]
    pub fn note(&self) -> Option<String> {
        (!self.is_empty()).then(|| {
            format!(
                "The ask holds a `## Done when` list ({}), and the harness holds your work to it. \
Before you change a file, hand in one check for each line with the `{CHECK}` tool: the line's name, \
and a command that prints what the line is about and nothing else. For a line that says what it \
{GIVES}, pass that output as `gives`. For a line that says it {STAYS}, pass no `gives`: the harness \
runs your command on the code as you found it, tells you what it prints today, and holds your work \
to that. When you answer, the harness runs every check on your work. A line with no check, or a \
check that prints something else, fails your landing.",
                self.span()
            )
        })
    }

    /// The lines of the ask's list with no check yet.
    fn missing(&self) -> Vec<&str> {
        self.lines
            .iter()
            .map(|(name, _)| name.as_str())
            .filter(|name| !self.checks.iter().any(|c| c.line == *name))
            .collect()
    }

    /// Why a write is refused now, when it is: the ask's list is not all in the harness's hands,
    /// and a line that must stay can be read cheaply only while the code is as the ask found it.
    /// Said [`REFUSALS`] times, then never again in this ask.
    pub fn refuses(&mut self) -> Option<String> {
        let missing = self.missing();
        if missing.is_empty() || self.refused >= REFUSALS {
            return None;
        }
        let why = format!(
            "refused: the ask's `Done when` list is not all in the harness's hands: {} {} no check. \
Hand one in for each with the `{CHECK}` tool, then make this change. A line that must stay is read \
from the code as you found it: hand it in before the first change.",
            and(&missing),
            if missing.len() == 1 { "has" } else { "have" }
        );
        self.refused += 1;
        Some(why)
    }

    /// The text of the line `line` names on the ask's list; `None` for a line of the body's own,
    /// when the ask carries no list.
    ///
    /// # Errors
    ///
    /// The refusal, when the name is off the ask's list or is no name at all.
    fn text_of(&self, line: &str) -> Result<Option<String>, String> {
        let text = self
            .lines
            .iter()
            .find(|(name, _)| name == line)
            .map(|(_, text)| text.clone());
        if text.is_none() && !self.is_empty() {
            return Err(format!(
                "refused: {line} is not on the ask's `Done when` list ({}). Name a line of it.",
                self.span()
            ));
        }
        if text.is_none() && !crate::lines::is_name(line) {
            return Err(format!(
                "refused: `{line}` names no line: a line is named `D1`, `D2`, … as in your \
`## Done when` list"
            ));
        }
        Ok(text)
    }

    /// One `check` call. `pristine` says whether the code is still as the ask found it; when it
    /// is not, `before` reads what a command prints on that code some other way, or says why it
    /// cannot. The check's own runs go through `bash`, classified and guarded like any command
    /// of the body.
    pub fn hand_in(
        &mut self,
        input: &str,
        pristine: bool,
        before: impl FnOnce(&str) -> Result<String, String>,
        tools: &[Box<dyn Tool>],
        cx: &mut Context,
    ) -> Outcome {
        let a: Args = match serde_json::from_str(input) {
            Ok(a) => a,
            Err(e) => return Outcome::error(format!("{CHECK}: {e}")),
        };
        let line = a.line.trim().trim_end_matches(':').to_owned();
        let command = a.command.trim().to_owned();
        if command.is_empty() {
            return Outcome::error(format!("refused: {line} needs a command that prints it"));
        }
        // A model on some hosts sends an optional string as an empty one: that is none.
        let gives = a.gives.as_deref().map(stated).filter(|g| !g.is_empty());
        let text = match self.text_of(&line) {
            Ok(text) => text,
            Err(refusal) => return Outcome::error(refusal),
        };
        let today = |command: &str, cx: &mut Context| {
            if pristine {
                steady(command, tools, cx)
            } else {
                before(command)
            }
        };
        let must = match held_to(&line, text.as_deref(), gives, || today(&command, cx)) {
            Ok(must) => must,
            Err(refusal) => return Outcome::error(refusal),
        };
        let check = Check {
            line: line.clone(),
            command,
            stays: must.1,
            must: must.0,
            late: !pristine,
        };
        let mut said = said_of(&check, tools, cx);
        self.checks.retain(|c| c.line != line);
        self.checks.push(check);
        if !self.is_empty() {
            let left = self.missing();
            if left.is_empty() {
                said.push_str("\nEvery line of the ask's list has its check.");
            } else {
                let _ = write!(said, "\nStill with no check: {}.", and(&left));
            }
        }
        Outcome::ok(said, ToolClass::Read)
    }

    /// The list judged over the work as it stands: what fails, one line each and each beginning
    /// [`SAID`], and how many lines hold. Every check is run once, through `bash`.
    pub fn judge(&self, tools: &[Box<dyn Tool>], cx: &mut Context) -> Judged {
        let mut failures = Vec::new();
        let (mut hold, mut loosely) = (0, 0);
        let missing = self.missing();
        if !missing.is_empty() {
            failures.push(format!(
                "{SAID} {} {} no check: hand one in for each with the `{CHECK}` tool",
                and(&missing),
                if missing.len() == 1 { "has" } else { "have" }
            ));
        }
        for c in &self.checks {
            let now = run(&c.command, tools, cx);
            let command = clipped(&c.command);
            if !now.ok {
                failures.push(format!(
                    "{SAID} {}: `{command}` fails on your work: {}",
                    c.line,
                    if now.err.is_empty() {
                        "it gave no result"
                    } else {
                        now.err.as_str()
                    }
                ));
            } else if now.out == c.must {
                hold += 1;
            } else if !c.stays && loose(&now.out) == loose(&c.must) {
                hold += 1;
                loosely += 1;
            } else if c.stays {
                failures.push(format!(
                    "{SAID} {} must stay: `{command}` printed `{}` on the code as you found it and \
prints `{}` on your work",
                    c.line,
                    clipped(&c.must),
                    clipped(&now.out)
                ));
            } else {
                failures.push(format!(
                    "{SAID} {} must give `{}`: `{command}` prints `{}` on your work",
                    c.line,
                    clipped(&c.must),
                    clipped(&now.out)
                ));
            }
        }
        Judged {
            failures,
            hold,
            loosely,
        }
    }

    /// What checked a landing the list passed, for the gate's word: the list, by how many lines,
    /// how many of them hold only with letter case or blanks set aside, and how many of its
    /// checks came in after the code had changed.
    #[must_use]
    pub fn by(&self, judged: &Judged) -> String {
        let late = self.checks.iter().filter(|c| c.late).count();
        let whose = if self.is_empty() {
            "Mikata's own"
        } else if self.read {
            "the reader's"
        } else {
            "the ask's"
        };
        let mut by = format!(
            "{whose} Done when list ({} {})",
            judged.hold,
            if judged.hold == 1 { "line" } else { "lines" }
        );
        if judged.loosely > 0 {
            let _ = write!(
                by,
                ", {} of them but for letter case or blanks",
                judged.loosely
            );
        }
        if late > 0 {
            let _ = write!(
                by,
                ", {late} of its checks handed in after the code had changed"
            );
        }
        by
    }
}

/// A list judged over the work ([`List::judge`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Judged {
    /// What fails, one line each, each beginning [`SAID`]. Empty when the list holds.
    pub failures: Vec<String>,
    /// The lines that hold.
    pub hold: usize,
    /// Those of them that state an output and hold only with letter case or blanks set aside.
    pub loosely: usize,
}

/// A reader's lines out of its reply: each line named `D1`, `D2`, … that says what it must give
/// or that it must stay, without its name, in order, at most [`READER_LINES`]. Anything else in
/// the reply is dropped: a heading, a remark, a line of neither kind.
#[must_use]
pub fn read_lines(reply: &str) -> Vec<String> {
    reply
        .lines()
        .filter_map(|line| {
            let t = line.trim().trim_start_matches(['-', '*']).trim_start();
            let (name, text) = t.split_once([':', '.'])?;
            let text = text.trim();
            (crate::lines::is_name(name.trim()) && kind(text) != Kind::Free)
                .then(|| text.to_owned())
        })
        .take(READER_LINES)
        .collect()
}

/// What a check for `line` is held to, and whether that is what it prints on the code as the ask
/// found it (`today` reads it) and not an output the line states. `text` is the line on the ask's
/// list, when it is on one: its two phrases say which it must be.
///
/// # Errors
///
/// The refusal: an output given for a line that must stay, none for a line that states one, an
/// output the line does not state, or a command that says nothing of the code as found.
fn held_to(
    line: &str,
    text: Option<&str>,
    gives: Option<String>,
    today: impl FnOnce() -> Result<String, String>,
) -> Result<(String, bool), String> {
    match (text.map_or(Kind::Free, kind), gives) {
        (Kind::Stays, Some(_)) => Err(format!(
            "refused: {line} says it {STAYS}: it states no output, and what it prints today is \
not yours to say. Hand it in with no `gives`: the harness runs your command on the code as you \
found it and tells you."
        )),
        (Kind::Gives, None) => Err(format!(
            "refused: {line} says what it {GIVES}: hand it in with that output as `gives`."
        )),
        // An output the line does not state is the body's own: the list is the human's.
        (_, Some(gives)) => match text {
            Some(text) if !loose(text).contains(&loose(&gives)) => Err(format!(
                "refused: {line} reads `{}`. It does not state `{}`: give the output the line \
states, as the line writes it.",
                clipped(text),
                clipped(&gives)
            )),
            _ => Ok((gives, false)),
        },
        (_, None) => today().map(|today| (today, true)).map_err(|why| {
            format!(
                "refused: {line} must stay, so its check has to run on the code as you found it, \
and {why}. Hand in a command that prints what the line is about there."
            )
        }),
    }
}

/// What the body is told of a check just taken: what it is held to, and for a stated output,
/// what its command prints on the world as it stands.
fn said_of(check: &Check, tools: &[Box<dyn Tool>], cx: &mut Context) -> String {
    let line = &check.line;
    if check.stays {
        return format!(
            "{line} is held. On the code as you found it, your command prints:\n{}\nYour work \
must print the same.",
            clipped(&check.must)
        );
    }
    let now = run(&check.command, tools, cx);
    let state = if !now.ok {
        format!(
            "Now the command fails ({}): it must print that when you are done.",
            if now.err.is_empty() {
                "no output"
            } else {
                now.err.as_str()
            }
        )
    } else if now.out == check.must {
        "It prints that now.".to_owned()
    } else if loose(&now.out) == loose(&check.must) {
        format!(
            "It prints that now, but for letter case or blanks, which are set aside:\n{}",
            clipped(&now.out)
        )
    } else {
        format!("Now it prints:\n{}", clipped(&now.out))
    };
    format!(
        "{line} is held: your command must print:\n{}\n{state}",
        clipped(&check.must)
    )
}

/// What the gate's note adds when the list fails: the trade a body makes when told one line.
pub const TOGETHER: &str = " Every line of the list has to hold at once: do not make one true by \
giving up another.";

/// `names` as words: `D1`, `D1 and D2`, `D1, D2 and D3`.
fn and(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [most @ .., last] => format!("{} and {last}", most.join(", ")),
    }
}

/// An output as a body states it: one pair of backticks around it dropped, line ends made one
/// kind, and the blanks at its end gone.
fn stated(text: &str) -> String {
    let t = text.replace("\r\n", "\n");
    let t = t.trim();
    let t = t
        .strip_prefix('`')
        .and_then(|rest| rest.strip_suffix('`'))
        .filter(|inner| !inner.contains('`'))
        .unwrap_or(t);
    t.trim_end().to_owned()
}

/// `text` with its blanks dropped and its letters made small: what a stated output is compared
/// by, and how a line is searched for the output it states.
fn loose(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// `text` cut to what is said back, on a character.
fn clipped(text: &str) -> String {
    if text.chars().count() <= SHOWN {
        return text.to_owned();
    }
    let cut: String = text.chars().take(SHOWN).collect();
    format!("{cut} … ({} more characters)", text.chars().count() - SHOWN)
}

/// What `command` does on the world as it stands, the world's own path written `<world>`.
fn run(command: &str, tools: &[Box<dyn Tool>], cx: &mut Context) -> Said {
    let args = json!({
        "command": command,
        "timeout_ms": u64::try_from(EACH.as_millis()).unwrap_or(60_000),
    });
    let o = call(tools, "bash", &args.to_string(), cx);
    let mut names = vec![cx.world().display().to_string()];
    if let Ok(real) = cx.world().canonicalize() {
        let real = real.display().to_string();
        if !names.contains(&real) {
            names.push(real);
        }
    }
    crate::changes::said_of(&o, &names)
}

/// What `command` prints on the world as it stands, when it runs and prints the same twice.
fn steady(command: &str, tools: &[Box<dyn Tool>], cx: &mut Context) -> Result<String, String> {
    let (first, second) = (run(command, tools, cx), run(command, tools, cx));
    crate::changes::one_of(&first, &second)
}

#[cfg(test)]
mod tests {
    use super::{
        CHECK, CheckTool, Kind, List, READER, SAID, and, clipped, kind, loose, read_lines, stated,
    };
    use mikata_races::Race;
    use mikata_tools::{Context, Sandbox, Tool, registry};

    const ASK: &str = "Make `total` skip blanks.\n\n## Done when\nD1: `total(['1', '', '2'])` must give `3`\nD2: `total(['4'])` must stay as it is today\nD3: the help text is clear\n";

    fn world() -> (tempfile::TempDir, Context) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("calc.sh"),
            "#!/bin/sh\n# total: the sum of its arguments\nn=0\nfor a in \"$@\"; do n=$((n + a)); done\necho $n\n",
        )
        .unwrap();
        let cx = Context::new(Race::Slime, dir.path().canonicalize().unwrap(), &["."])
            .with_sandbox(Sandbox::None("these tests run unguarded".into()))
            .with_unguarded(true);
        (dir, cx)
    }

    fn call_in(
        list: &mut List,
        cx: &mut Context,
        input: &str,
        pristine: bool,
    ) -> mikata_tools::Outcome {
        list.hand_in(
            input,
            pristine,
            |_| Err("the code as you found it is not kept".to_owned()),
            &registry(),
            cx,
        )
    }

    /// The tool's example is a call its own schema takes.
    #[test]
    fn the_example_fits_the_schema() {
        let tool = CheckTool;
        let schema = tool.schema();
        let example = tool.example();
        for required in schema["required"].as_array().unwrap() {
            assert!(example.get(required.as_str().unwrap()).is_some());
        }
        for key in example.as_object().unwrap().keys() {
            assert!(schema["properties"].get(key).is_some(), "{key}");
        }
        assert_eq!(tool.name(), CHECK);
    }

    /// A line's own words say what it is; an ask with no list holds nothing.
    #[test]
    fn a_line_is_read_by_its_two_phrases() {
        assert_eq!(kind("`f(1)` must give `2`"), Kind::Gives);
        assert_eq!(kind("`f(1)` Must Stay as it is today"), Kind::Stays);
        assert_eq!(kind("the help text is clear"), Kind::Free);
        let list = List::of(ASK);
        assert!(!list.is_empty());
        assert_eq!(list.missing(), ["D1", "D2", "D3"]);
        let note = list.note().unwrap();
        assert!(
            note.contains("(D1 to D3)") && note.contains("`check` tool"),
            "{note}"
        );
        assert!(List::of("Make it faster.").note().is_none());
        assert_eq!(and(&["D1", "D2", "D3"]), "D1, D2 and D3");
        assert_eq!(and(&["D1", "D2"]), "D1 and D2");
    }

    /// An output is read as the body states it: backticks around it and blanks at its end are
    /// not part of it, and a line states it whatever its spacing.
    #[test]
    fn an_output_is_read_as_stated() {
        assert_eq!(stated("`SELECT 1`\n"), "SELECT 1");
        assert_eq!(stated("a `b` c"), "a `b` c");
        assert_eq!(stated("x\r\ny  "), "x\ny");
        assert!(loose("must give `SLICE(x, 1,  2)`").contains(&loose("slice(x, 1, 2)")));
        assert!(clipped(&"é".repeat(500)).ends_with("(100 more characters)"));
    }

    /// A line that must stay is read from the code as the ask found it, by the harness: the body
    /// may not say what it prints, and the landing is held to what it printed.
    #[test]
    fn a_line_that_must_stay_is_read_from_the_code_as_found() {
        let (dir, mut cx) = world();
        let mut list = List::of(ASK);
        // The body's own idea of today is refused.
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh calc.sh 4", "gives": "four"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("no `gives`"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh calc.sh 4"}"#,
            true,
        );
        assert!(!o.is_error, "{}", o.content);
        assert!(o.content.contains("prints:\n4\n"), "{}", o.content);
        assert!(
            o.content.contains("Still with no check: D1 and D3."),
            "{}",
            o.content
        );
        assert!(list.checks()[0].stays && !list.checks()[0].late);
        // The work changes what the line prints: the landing fails, with both outputs.
        std::fs::write(dir.path().join("calc.sh"), "echo 5\n").unwrap();
        let judged = list.judge(&registry(), &mut cx);
        let failures = &judged.failures;
        assert_eq!(judged.hold, 0);
        assert!(failures[0].starts_with(SAID) && failures[0].contains("D1 and D3 have no check"));
        assert!(
            failures[1].contains("D2 must stay")
                && failures[1].contains("printed `4` on the code as you found it")
                && failures[1].contains("prints `5` on your work"),
            "{failures:?}"
        );
    }

    /// A line that states its output is held to it, and only to an output the line states.
    #[test]
    fn a_line_that_gives_is_held_to_the_output_it_states() {
        let (dir, mut cx) = world();
        let mut list = List::of(ASK);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh calc.sh 1 2"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("as `gives`"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh calc.sh 1 2", "gives": "4"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("does not state `4`"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh calc.sh 1 '' 2", "gives": "`3`"}"#,
            true,
        );
        assert!(
            !o.is_error && o.content.contains("It prints that now."),
            "{}",
            o.content
        );
        // A line of neither kind takes either form; a name off the list is refused.
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D3", "command": "sed -n 2p calc.sh", "gives": ""}"#,
            true,
        );
        assert!(
            !o.is_error && o.content.contains("# total: the sum"),
            "{}",
            o.content
        );
        assert!(
            o.content.contains("Still with no check: D2."),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2:", "command": "sh calc.sh 4"}"#,
            true,
        );
        assert!(
            o.content
                .contains("Every line of the ask's list has its check."),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D9", "command": "true"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("not on the ask's"),
            "{}",
            o.content
        );
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!(
            (judged.failures.len(), judged.hold, judged.loosely),
            (0, 3, 0),
            "{judged:?}"
        );
        assert_eq!(list.by(&judged), "the ask's Done when list (3 lines)");
        // The work breaks the output: said with what it prints.
        std::fs::write(dir.path().join("calc.sh"), "echo 7\n").unwrap();
        let failures = list.judge(&registry(), &mut cx).failures;
        assert!(
            failures
                .iter()
                .any(|f| f.contains("D1 must give `3`") && f.contains("prints `7`")),
            "{failures:?}"
        );
    }

    /// Whoever wrote the list seldom knows how the code spells its output: a stated output holds
    /// with letter case and blanks set aside, and the pass says so. What must stay was read from
    /// the code, and holds exactly or not at all.
    #[test]
    fn a_stated_output_holds_but_for_letter_case_and_what_must_stay_holds_exactly() {
        let (dir, mut cx) = world();
        std::fs::write(dir.path().join("unit.sh"), "echo \"INTERVAL '1.5 DAY'\"\n").unwrap();
        let ask = "Keep the fraction.\n\n## Done when\nD1: `1.5 day` must give `interval '1.5 day'`\nD2: the unit must stay as it is today\n";
        let mut list = List::of(ask);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh unit.sh", "gives": "INTERVAL '1.5 DAY'"}"#,
            true,
        );
        assert!(
            !o.is_error && o.content.contains("It prints that now."),
            "the line states it, in other letters: {}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh unit.sh", "gives": "interval '1.5 day'"}"#,
            true,
        );
        assert!(
            !o.is_error && o.content.contains("but for letter case or blanks"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh unit.sh"}"#,
            true,
        );
        assert!(!o.is_error, "{}", o.content);
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!(
            (judged.failures.len(), judged.hold, judged.loosely),
            (0, 2, 1),
            "{judged:?}"
        );
        assert_eq!(
            list.by(&judged),
            "the ask's Done when list (2 lines), 1 of them but for letter case or blanks"
        );
        // The same change of letters breaks the line that must stay.
        std::fs::write(dir.path().join("unit.sh"), "echo \"interval '1.5 day'\"\n").unwrap();
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!((judged.hold, judged.loosely), (1, 0), "{judged:?}");
        assert!(
            judged.failures[0].contains("D2 must stay")
                && judged.failures[0].contains("prints `interval '1.5 day'` on your work"),
            "{judged:?}"
        );
    }

    /// A reader's reply is read for its lines alone, and its list is said after the ask it was
    /// written for, then held like the ask's own.
    #[test]
    fn a_readers_lines_are_put_after_the_ask_and_held() {
        let reply = "Here is what done means:\n\nD1: `f(1)` must give `2`\n- D2: `f(0)` must stay as it is today\nD3. `f(-1)` MUST STAY as it is today\nD4: the code is clean\nNote: none.\n";
        let lines = read_lines(reply);
        assert_eq!(
            lines,
            [
                "`f(1)` must give `2`",
                "`f(0)` must stay as it is today",
                "`f(-1)` MUST STAY as it is today"
            ]
        );
        assert_eq!(read_lines("NONE"), [] as [String; 0]);
        let many = (1..=40).fold(String::new(), |all, i| {
            all + &format!("D{i}: `g({i})` must stay\n")
        });
        assert_eq!(read_lines(&many).len(), 30);
        let list = List::read(&lines);
        let after = list.after_the_ask().unwrap();
        assert!(
            after.starts_with("## Done when\nD1: `f(1)` must give `2`\nD2: `f(0)` must stay as it is today\nD3: `f(-1)` MUST STAY as it is today\n\nThe ask holds a `## Done when` list (D1 to D3)"),
            "{after}"
        );
        assert_eq!(list.missing(), ["D1", "D2", "D3"]);
        // The ask's own list is only told of; no list, nothing.
        let own = List::of(ASK).after_the_ask().unwrap();
        assert!(own.starts_with("The ask holds"), "{own}");
        assert!(List::read(&[]).after_the_ask().is_none());
        assert!(List::read(&[]).is_empty());
        // What the reader is asked names both phrases the harness reads a line by.
        assert!(READER.contains("must give") && READER.contains("must stay"));
    }

    /// A check that cannot say what must stay is refused: one that fails on the code as found,
    /// one that prints nothing, and one handed in after the code changed with no way back to it.
    #[test]
    fn a_check_that_cannot_be_read_today_is_refused() {
        let (_dir, mut cx) = world();
        let mut list = List::of(ASK);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh nothing.sh"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("it fails there"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "true"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("prints nothing"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh calc.sh 4"}"#,
            false,
        );
        assert!(
            o.is_error && o.content.contains("is not kept"),
            "{}",
            o.content
        );
        assert!(!list.any());
    }

    /// A write is refused while a line has no check, twice, and then goes through: the gate
    /// says what is missing, and a body is never left with no way on.
    #[test]
    fn a_write_waits_for_the_list_twice_then_goes() {
        let mut list = List::of(ASK);
        let why = list.refuses().unwrap();
        assert!(why.contains("D1, D2 and D3 have no check"), "{why}");
        assert!(list.refuses().is_some());
        assert!(list.refuses().is_none());
        assert!(List::of("no list here").refuses().is_none());
    }

    /// With no list in the ask a body may hold lines of its own, named as its plan names them.
    #[test]
    fn a_body_may_hold_lines_of_its_own() {
        let (_dir, mut cx) = world();
        let mut list = List::of("Make `total` skip blanks.");
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "the sum", "command": "sh calc.sh 1"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("names no line"),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh calc.sh 1 2"}"#,
            true,
        );
        assert!(
            !o.is_error && !o.content.contains("list has its check"),
            "{}",
            o.content
        );
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!((judged.failures.len(), judged.hold), (0, 1));
        assert_eq!(list.by(&judged), "Mikata's own Done when list (1 line)");
    }
}
