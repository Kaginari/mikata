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
//! **A check runs its line's case.** Held to what it prints alone, a check may print its answer
//! (`echo -5`) or run another case: on our bench 169 of 3,483 did, and a line that must stay,
//! held to a command that prints the same whatever the code does, can never fail. So before a
//! check runs, a command that only prints words is refused, and so is one that never names the
//! values of its line's case ([`runs_its_case`]).
//!
//! When an answer lands, every check is run on the work as it stands. A line with no check, or a
//! check that prints something else, fails the landing, and the body is told which line and what
//! it printed. Nothing here reads prose: the list's two phrases decide a line's kind, and equal
//! text decides a pass.
//!
//! **What must stay is whatever the command does today.** It may print, print nothing, or fail:
//! "everything else must keep working as it does today" covers the input that raises today too,
//! and a reader writes such lines. A line whose command ran is held to what it printed: breaking
//! what worked is refused. A line whose command failed holds while it fails the same way; when
//! it no longer does, the harness cannot know whether the ask asked for that (asks are for making
//! something work) or the work reached too far. So that is said to the body once, and to the
//! human, and refuses nothing. The first live run of this, on a four-line function, had a strong
//! reader list an input the ask itself changes as one that must stay: held hard, a wrong line of
//! that kind leaves a body no honest way through.
//!
//! **What the human said was wrong is a line too.** When the human says an earlier answer to an
//! ask was wrong, and says what must come out (`<input>` must give `<output>`), memory keeps the
//! words under that ask, and whenever the ask comes again the case they state joins its list
//! ([`held_line`], [`List::holding`]). Told as advice, the same words were taken up and then
//! overruled: on our bench a body wrote the human's case first in its own plan, could not make
//! it true, rewrote the plan to say the opposite, ticked it, and answered that the ask was done.
//! A line the harness holds is not the body's to rewrite.
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
/// The most lines held from the human's corrections of one ask: the newest.
const CORRECTED_LINES: usize = 12;
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
        "Hand the harness a check for one line of the `Done when` list: a command that runs the line's own case, its values as the line writes them, and prints what comes out, and nothing else. A command that only prints words is refused. With `gives`, the harness holds your work to that output, letter case and blanks set aside: keep the code's own spelling of what it prints. Without it the line is one that must stay as it is: the harness runs the command on the code as you found it, tells you what it does today, and holds your work to that, so hand such a line in before you change a file. When you answer, the harness runs every check on your work, and one that prints something else fails your landing. One check a line: handing a line in again with another command replaces its check, and with the same command says how the line stands now."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "line": {"type": "string", "description": "The line, by name: `D5`."},
                "command": {"type": "string", "description": "One shell command that prints the line's output and nothing else: `python3 -c \"import calc; print(calc.total([1, 2]))\"`."},
                "gives": {"type": "string", "description": "The output the line states. Leave it out for a line that must stay as it is: the harness reads that from the code as you found it."}
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
    /// What the command must do when the answer lands.
    pub must: Must,
    /// Whether it was handed in, or handed in again, after the body had changed the code.
    pub late: bool,
    /// Whether its command already printed the output its line states on the code as the ask
    /// found it. Such a check holds when the answer lands and shows nothing of the work: the
    /// case was right before, or the command only prints the words. The harness cannot tell
    /// which, so it refuses nothing and the pass names the line ([`List::by`]). Never set for a
    /// check that came in late: what its command did before the work is not known then.
    pub already: bool,
    /// How its command failed when it was handed in, for a line that states an output: `None`
    /// when it ran. A command that fails the same way when the answer lands has not been moved
    /// by the work, and one that failed to load never reached it ([`CANNOT_RUN`]).
    pub failed: Option<String>,
}

/// What a check's command must do on the work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Must {
    /// Print an output the line states, letter case and blanks set aside.
    Give(String),
    /// Print what it printed on the code as the ask found it, exactly: it ran there.
    Print(String),
    /// Fail as it failed on the code as the ask found it: the last line of what it said there.
    /// A line that no longer does is said once and refuses nothing.
    Fail(String),
}

impl Must {
    /// Whether this is read from the code as the ask found it, and not an output a line states.
    #[must_use]
    pub fn stays(&self) -> bool {
        !matches!(self, Must::Give(_))
    }
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
    /// The lines the body was told no longer fail as they did ([`Judged::moved`]): said once.
    told: Vec<String>,
    /// The lines that are the human's corrections of earlier answers to this ask
    /// ([`List::holding`]), by name: said after the ask, which did not hold them.
    humans: Vec<String>,
    /// The human's lines checked by the human's own command, by name, with the command: handed
    /// in when the ask opens, never replaced by a command of the body's ([`List::pin`]).
    pinned: Vec<(String, String)>,
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

    /// The list with what the human's corrections of earlier answers to this ask say must come
    /// out (`cases`, oldest first, each a line: [`held_line`]) held too, named on from the
    /// list's last line. The newest [`CORRECTED_LINES`] are kept, and a case the list already
    /// holds word for word is one line.
    #[must_use]
    pub fn holding(mut self, cases: &[String]) -> Self {
        let mut last = self
            .lines
            .iter()
            .filter_map(|(name, _)| number(name))
            .max()
            .unwrap_or(0);
        let from = cases.len().saturating_sub(CORRECTED_LINES);
        for text in &cases[from..] {
            if self.lines.iter().any(|(_, held)| held == text) {
                continue;
            }
            last += 1;
            let name = format!("D{last}");
            self.humans.push(name.clone());
            self.lines.push((name, text.clone()));
        }
        self
    }

    /// The lines that are the human's corrections, by name.
    #[must_use]
    pub fn humans(&self) -> &[String] {
        &self.humans
    }

    /// The name of the human's line that holds `text`, if it is one ([`List::holding`]).
    #[must_use]
    pub fn human_line(&self, text: &str) -> Option<&str> {
        self.lines
            .iter()
            .find(|(name, held)| held == text && self.humans.contains(name))
            .map(|(name, _)| name.as_str())
    }

    /// Hold the human's line `line` to the human's own `command`, handed in already: from now on
    /// a `check` of the body's for that line is refused. A body's check that runs another case
    /// prints what the line states as well as one that runs the human's, and the harness cannot
    /// tell them apart; the human's command is the human's case.
    pub fn pin(&mut self, line: &str, command: &str) {
        self.pinned.retain(|(name, _)| name != line);
        self.pinned.push((line.to_owned(), command.to_owned()));
    }

    /// The human's lines checked by their own command: `(line, command)`.
    #[must_use]
    pub fn pinned(&self) -> &[(String, String)] {
        &self.pinned
    }

    /// What goes after the ask when a list is held: the lines the ask's own words did not carry
    /// (a reader's list, and what the human's corrections say must come out, with whose words
    /// those are), then how the harness holds the list ([`List::note`]). Nothing for an ask with
    /// no list.
    #[must_use]
    pub fn after_the_ask(&self) -> Option<String> {
        let note = self.note()?;
        let after: Vec<&Line> = self
            .lines
            .iter()
            .filter(|(name, _)| self.read || self.humans.contains(name))
            .collect();
        if after.is_empty() {
            return Some(note);
        }
        let mut said = "## Done when".to_owned();
        for (name, text) in after {
            let _ = write!(said, "\n{name}: {text}");
        }
        if !self.humans.is_empty() {
            let names: Vec<&str> = self.humans.iter().map(String::as_str).collect();
            // One line, or several: said as what they are, and as part of the ask, not all of it.
            // A body held to one line has made it true and left the rest of its ask undone.
            let (are, they_are, cases) = if names.len() == 1 {
                ("is the human's own word", "It is", "it is one case")
            } else {
                ("are the human's own words", "They are", "they are cases")
            };
            let _ = write!(
                said,
                "\n\n{} {are}: they said an earlier answer to this ask was wrong, and this is \
what must come out. {they_are} not yours to rewrite, and {cases} of the ask, not the whole of it: \
the ask stands as written.",
                runs(&names)
            );
        }
        for (name, command) in &self.pinned {
            let _ = write!(
                said,
                "\n{name} is checked by the human's own command, handed in already: `{}`.",
                clipped(command)
            );
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

    /// The lines whose check shows nothing of the work ([`Check::already`]), in the order of
    /// their names.
    fn already(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .checks
            .iter()
            .filter(|c| c.already)
            .map(|c| c.line.as_str())
            .collect();
        names.sort_by_key(|name| number(name));
        names
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
    /// is not, `before` reads what a command does on that code some other way, or says why it
    /// cannot. The check's own runs go through `bash`, classified and guarded like any command
    /// of the body.
    pub fn hand_in(
        &mut self,
        input: &str,
        pristine: bool,
        before: impl FnOnce(&str) -> Result<Said, String>,
        tools: &[Box<dyn Tool>],
        cx: &mut Context,
    ) -> Outcome {
        let a: Args = match serde_json::from_str(input) {
            Ok(a) => a,
            Err(e) => return corrected(format!("{CHECK}: {e}")),
        };
        let line = a.line.trim().trim_end_matches(':').to_owned();
        let command = a.command.trim().to_owned();
        if command.is_empty() {
            return corrected(format!("refused: {line} needs a command that prints it"));
        }
        // A model on some hosts sends an optional string as an empty one: that is none.
        let gives = a.gives.as_deref().map(stated).filter(|g| !g.is_empty());
        if let Some((_, theirs)) = self
            .pinned
            .iter()
            .find(|(name, theirs)| *name == line && *theirs != command)
        {
            return corrected(format!(
                "refused: {line} is checked by the human's own command, `{}`: it is handed in \
already and not yours to replace. It must print what {line} states when you are done. Hand in \
checks of your own for the other lines.",
                clipped(theirs)
            ));
        }
        let text = match self.text_of(&line) {
            Ok(text) => text,
            Err(refusal) => return corrected(refusal),
        };
        if let Err(refusal) = runs_its_case(&line, text.as_deref(), &command, cx.world()) {
            return corrected(refusal);
        }
        // The same line and command again asks how the line stands now: the check is kept as it
        // is, with what it was held to when it came in.
        if let Some(held) = self.checks.iter().find(|c| {
            c.line == line
                && c.command == command
                && match (&c.must, &gives) {
                    (Must::Give(must), Some(gives)) => must == gives,
                    (must, None) => must.stays(),
                    _ => false,
                }
        }) {
            let said = format!("{}{}", stands(held, tools, cx), self.left());
            return Outcome::ok(said, ToolClass::Read);
        }
        let today = |command: &str, cx: &mut Context| {
            if pristine {
                let (first, second) = (run(command, tools, cx), run(command, tools, cx));
                crate::changes::same_twice(first, &second)
            } else {
                before(command)
            }
        };
        let must = match held_to(&line, text.as_deref(), gives, || today(&command, cx)) {
            Ok(must) => must,
            Err(refusal) => return corrected(refusal),
        };
        let mut check = Check {
            line: line.clone(),
            command,
            must,
            late: !pristine,
            already: false,
            failed: None,
        };
        let (mut said, prints, failed) = said_of(&check, tools, cx);
        // Only the code as the ask found it says what a check will show of the work.
        check.already = pristine && prints;
        check.failed = failed;
        // The human said this case comes out wrong. A command that prints what they want before
        // any work most often runs another case: the input, or which way it is written, read
        // otherwise. Said once, here, while another command can still be written.
        if check.already
            && self.humans.contains(&line)
            && !self.pinned.iter().any(|(n, _)| *n == line)
        {
            let _ = write!(
                said,
                "\nThe human said this case comes out wrong, and your command prints what they want \
before any work. Unless the code was changed since they said so, your command runs another case. \
Read {line} again, its input and which way it is written, and hand in a command that runs exactly \
that."
            );
        }
        self.checks.retain(|c| c.line != line);
        self.told.retain(|t| *t != line);
        self.checks.push(check);
        Outcome::ok(format!("{said}{}", self.left()), ToolClass::Read)
    }

    /// What ends a `check` call's answer for an ask that carries a list: the lines with no check
    /// yet, or that every line has one.
    fn left(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let left = self.missing();
        if left.is_empty() {
            "\nEvery line of the ask's list has its check.".to_owned()
        } else {
            format!("\nStill with no check: {}.", and(&left))
        }
    }

    /// The list judged over the work as it stands ([`Judged`]). Every check is run once, through
    /// `bash`.
    pub fn judge(&self, tools: &[Box<dyn Tool>], cx: &mut Context) -> Judged {
        let mut judged = Judged::default();
        let missing = self.missing();
        if !missing.is_empty() {
            judged.failures.push(format!(
                "{SAID} {} {} no check: hand one in for each with the `{CHECK}` tool",
                and(&missing),
                if missing.len() == 1 { "has" } else { "have" }
            ));
        }
        for c in &self.checks {
            let now = run(&c.command, tools, cx);
            let (line, command) = (&c.line, clipped(&c.command));
            let did = if now.ok {
                format!("prints `{}`", clipped(&now.out))
            } else {
                format!("fails ({})", failure(&now.err))
            };
            match &c.must {
                Must::Give(must) if now.ok && now.out == *must => judged.hold += 1,
                Must::Give(must) if now.ok && loose(&now.out) == loose(must) => {
                    judged.hold += 1;
                    judged.loosely += 1;
                }
                // The command fails as it did when it was handed in: nothing the body did
                // since has changed what it does. One that failed to load never reached the
                // work, so it shows nothing of it, and saying only "fails on your work" sends a
                // body with right code back to change that code.
                Must::Give(must) if !now.ok && c.failed.as_deref() == Some(now.err.as_str()) => {
                    let must = clipped(must);
                    let err = failure(&now.err);
                    judged.failures.push(if unloaded(&now.err) {
                        format!(
                            "{SAID} {line} must give `{must}`: `{command}` {CANNOT_RUN}: it \
fails as it did when you handed it in ({err}), before it reaches your work. That shows nothing of \
your work, right or wrong: hand {line} in again with a command that runs"
                        )
                    } else {
                        format!(
                            "{SAID} {line} must give `{must}`: `{command}` fails ({err}) on \
your work, as it did when you handed it in: your work has not changed what this command does"
                        )
                    });
                }
                Must::Give(must) => judged.failures.push(format!(
                    "{SAID} {line} must give `{}`: `{command}` {did} on your work",
                    clipped(must)
                )),
                Must::Print(must) if now.ok && now.out == *must => judged.hold += 1,
                Must::Print(must) => judged.failures.push(format!(
                    "{SAID} {line} must stay: `{command}` printed `{}` on the code as you found it \
and {did} on your work",
                    clipped(must)
                )),
                Must::Fail(must) if !now.ok && now.err == *must => judged.hold += 1,
                Must::Fail(must) => judged.moved.push((
                    line.clone(),
                    format!(
                        "{line} must stay: `{command}` failed on the code as you found it ({}) \
and {did} on your work",
                        failure(must)
                    ),
                )),
            }
        }
        judged
    }

    /// What of `judged` the body has not been told yet: the lines that no longer fail as they
    /// did, each said once ([`moved_note`]). Telling them is the caller's; this marks them told.
    pub fn news(&mut self, judged: &Judged) -> Vec<String> {
        let news: Vec<(String, String)> = judged
            .moved
            .iter()
            .filter(|(line, _)| !self.told.contains(line))
            .cloned()
            .collect();
        self.told.extend(news.iter().map(|(line, _)| line.clone()));
        news.into_iter().map(|(_, said)| said).collect()
    }

    /// What checked a landing the list passed, for the gate's word: the list, by how many lines,
    /// how many of them hold only with letter case or blanks set aside, which no longer fail as
    /// they did, which already gave their output before the work ([`Check::already`]), and how
    /// many of its checks came in after the code had changed.
    #[must_use]
    pub fn by(&self, judged: &Judged) -> String {
        let late = self.checks.iter().filter(|c| c.late).count();
        let whose = if self.is_empty() {
            "Mikata's own"
        } else if self.humans.len() == self.lines.len() {
            "the human's"
        } else if self.read {
            "the reader's"
        } else {
            "the ask's"
        };
        // One unit in brackets: what the gate's word says after it must still read.
        let mut said = vec![format!(
            "{} {}",
            judged.hold,
            if judged.hold == 1 { "line" } else { "lines" }
        )];
        if judged.loosely > 0 {
            let _ = write!(
                said[0],
                ", {} but for letter case or blanks",
                judged.loosely
            );
        }
        if !judged.moved.is_empty() {
            let lines: Vec<&str> = judged.moved.iter().map(|(line, _)| line.as_str()).collect();
            said.push(format!(
                "{} no longer {} as on the code you gave it",
                and(&lines),
                if lines.len() == 1 { "fails" } else { "fail" }
            ));
        }
        if !self.humans.is_empty() && self.humans.len() < self.lines.len() {
            let names: Vec<&str> = self.humans.iter().map(String::as_str).collect();
            said.push(format!(
                "{} from what the human said was wrong before",
                runs(&names)
            ));
        }
        let already = self.already();
        if !already.is_empty() {
            said.push(format!(
                "{} already gave {} output on the code you gave it",
                runs(&already),
                if already.len() == 1 { "its" } else { "their" }
            ));
        }
        if late > 0 {
            said.push(format!(
                "{late} {} handed in after the code had changed",
                if late == 1 { "check" } else { "checks" }
            ));
        }
        format!("{whose} Done when list ({})", said.join("; "))
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
    /// The lines that must stay whose command failed on the code as the ask found it and no
    /// longer fails that way: each by name, with what it did then and does now. They refuse
    /// nothing: a failure that is gone is what an ask is for, or a reach too far, and the
    /// harness cannot tell which.
    pub moved: Vec<(String, String)>,
}

/// What the body is told, once, of lines that no longer fail as they did ([`List::news`]). Its
/// landing stands either way.
#[must_use]
pub fn moved_note(news: &[String]) -> String {
    format!(
        "The lines the harness holds all hold. One thing more, said once: on the code as you \
found it {} failed, and {} said to stay as it is.\n- {}\nIf the ask asks for that change, or it \
follows from what the ask asks, leave it and say so in your answer. If your work reached further \
than the ask, undo the reach. Never add code only to make something fail as it did. Then answer \
again: this refuses no landing.",
        if news.len() == 1 {
            "this command"
        } else {
            "these commands"
        },
        if news.len() == 1 {
            "its line is"
        } else {
            "their lines are"
        },
        news.join("\n- ")
    )
}

/// The case a correction states, as a line the harness can hold: the human's words up to the
/// end of the output they say something `must give`, so that what they say came out instead is
/// no part of the line. `None` for words that state no output that way: they stay advice. Words
/// that say something `must stay` are never held: what the code does today may be the very
/// answer the human called wrong.
#[must_use]
pub fn held_line(words: &str) -> Option<String> {
    // The phrase is plain ASCII: found in the words as they are, its ends are ends of letters.
    let at = words
        .as_bytes()
        .windows(GIVES.len())
        .position(|w| w.eq_ignore_ascii_case(GIVES.as_bytes()))?;
    let (input, after) = (words[..at].trim(), &words[at + GIVES.len()..]);
    // The output: the first code span after the phrase, whole; with none, the words up to a `;`
    // or the sentence's end.
    let end = match after.find('`') {
        Some(open) => after[open + 1..].find('`').map(|close| open + close + 2)?,
        None => after
            .char_indices()
            .zip(after.chars().skip(1).chain([' ']))
            .find(|((_, c), next)| *c == ';' || (matches!(c, '.' | '!' | '?') && *next == ' '))
            .map_or(after.len(), |((i, _), _)| i),
    };
    let output = after[..end].trim();
    // Words that also say something must stay would be read as a line of that kind, and held
    // to what the code does today: they are advice.
    (!input.is_empty() && !output.is_empty())
        .then(|| format!("{input} {GIVES} {output}"))
        .filter(|line| kind(line) == Kind::Gives)
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

/// What a check for `line` is held to. `text` is the line on the ask's list, when it is on one:
/// its two phrases say which kind it must be. `today` reads what the command does on the code as
/// the ask found it, for a line that must stay.
///
/// # Errors
///
/// The refusal: an output given for a line that must stay, none for a line that states one, an
/// output the line does not state, or a command that does not do the same thing twice on the
/// code as found.
fn held_to(
    line: &str,
    text: Option<&str>,
    gives: Option<String>,
    today: impl FnOnce() -> Result<Said, String>,
) -> Result<Must, String> {
    match (text.map_or(Kind::Free, kind), gives) {
        (Kind::Stays, Some(_)) => Err(format!(
            "refused: {line} says it {STAYS}: it states no output, and what it prints today is \
not yours to say. Hand it in with no `gives`: the harness runs your command on the code as you \
found it and tells you."
        )),
        (Kind::Gives, None) => Err(format!(
            "refused: {line} says what it {GIVES}: hand it in with that output as `gives`."
        )),
        // An output the line does not state is the body's own: the list is the human's. The line
        // states it when the line holds the output given, or the output given holds the one the
        // line names after `must give`: a check that prints the whole statement around it.
        (_, Some(gives)) => match text {
            Some(text) if !states(text, &gives) => Err(format!(
                "refused: {line} reads `{}`. It does not state `{}`: give the output the line \
states, as the line writes it.",
                clipped(text),
                clipped(&gives)
            )),
            _ => Ok(Must::Give(gives)),
        },
        (_, None) => match today() {
            Ok(today) if today.ok => Ok(Must::Print(today.out)),
            Ok(today) => Ok(Must::Fail(today.err)),
            Err(why) => Err(format!(
                "refused: {line} must stay, so its check is read from the code as you found it, \
and {why}. Hand in a command that does the same thing each time it runs."
            )),
        },
    }
}

/// The verbs that end a line's case when the line has no backticks: what it gives comes after.
const SAYS: [&str; 7] = [
    " gives ",
    " returns ",
    " prints ",
    " outputs ",
    " stays ",
    " should ",
    " still ",
];
/// The most files a check's command names that are read for its case, and the most of each.
const NAMED_FILES: usize = 8;
const NAMED_BYTES: u64 = 256 * 1024;

/// Why `command` cannot be `line`'s check, when it cannot (`text` is the line, when the list
/// carries it). Two kinds of command are refused, both read off the command alone:
///
/// - One that only prints words it holds (`echo -5`, `print('SLICE(x, 4, 1)')`): it runs nothing
///   of the code, so it holds whatever the work does. Never a check of anything.
/// - One that names none of the line's case as written. A line's case is its backticked input,
///   the part before `must give` or `must stay` (in a line with no backticks, what comes before
///   its verb: [`SAYS`]), and of it what no spelling of a call can change:
///   its numbers and its quoted words. Each of them must stand in the command, or in a file the
///   command names, blanks and letter case set aside. Names are not asked for: a command-line
///   check (`cargo run -- 90` for `format(90)`) calls the code by another name. A line whose
///   case has no such values is held to nothing here.
///
/// On our bench, over 3,483 checks handed in for a list's line, the second refused 74: commands
/// that print the answer, checks of another input than the line's (`ARRAY_SLICE(x, 1, 10)` for
/// `ARRAY_SLICE(x, 2, 4)`), a library other than the project's, and checks of half a line (the
/// derivative, but never its value at `x = 2`). None was a whole and honest check. The first
/// kind took 83 more that print a label holding the case (`echo "D1: ARRAY_SLICE(x,2,4) ->
/// SLICE(x,2,3)"`). A line that must stay held to such a command can never fail.
fn runs_its_case(
    line: &str,
    text: Option<&str>,
    command: &str,
    world: &std::path::Path,
) -> Result<(), String> {
    if prints_only_words(command) {
        return Err(format!(
            "refused: {line}'s command only prints words it already holds: it runs nothing of the \
code, so it would hold whatever the work does. Hand in a command that runs {line}'s case on the code \
and prints what comes out."
        ));
    }
    let Some(text) = text else { return Ok(()) };
    let small = text.to_lowercase();
    let before = [GIVES, STAYS]
        .iter()
        .filter_map(|phrase| small.find(phrase))
        .min()
        .and_then(|at| text.get(..at))
        .unwrap_or(text);
    let cases: Vec<Vec<String>> = if before.contains('`') {
        before
            .split('`')
            .skip(1)
            .step_by(2)
            .map(values)
            .filter(|v| !v.is_empty())
            .collect()
    } else {
        // A line written with no backticks (`format(90) gives '90m'`, as people write them): its
        // case is what comes before its first verb.
        let small = before.to_lowercase();
        let at = SAYS
            .iter()
            .filter_map(|verb| small.find(verb))
            .min()
            .unwrap_or(before.len());
        let case = values(before.get(..at).unwrap_or(before));
        if case.is_empty() { vec![] } else { vec![case] }
    };
    if cases.is_empty() {
        return Ok(());
    }
    let mut hay = flat(command);
    for body in named_files(command, world) {
        hay.push('\n');
        hay.push_str(&flat(&body));
    }
    if cases
        .iter()
        .any(|case| case.iter().all(|v| hay.contains(&flat(v))))
    {
        return Ok(());
    }
    let unnamed: Vec<String> = cases[0]
        .iter()
        .filter(|v| !hay.contains(&flat(v)))
        .map(|v| format!("`{v}`"))
        .collect();
    let unnamed: Vec<&str> = unnamed.iter().map(String::as_str).collect();
    Err(format!(
        "refused: {line} reads `{}`, and your command never names {} of its case. A check runs \
the line's own case: hand in a command that runs exactly that input, its values as the line writes \
them, in the command or in a file the command runs.",
        clipped(text),
        and(&unnamed)
    ))
}

/// The values of a case no spelling of a call changes: each number (with its sign) and each
/// quoted word, made small, in order. A quote left open is no word.
fn values(case: &str) -> Vec<String> {
    let chars: Vec<char> = case.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' || c == '"' {
            if let Some(end) = chars[i + 1..].iter().position(|&d| d == c) {
                let word: String = chars[i + 1..i + 1 + end].iter().collect();
                let word = word.trim().to_lowercase();
                if !word.is_empty() {
                    out.push(word);
                }
                i += end + 2;
                continue;
            }
        } else if c.is_ascii_digit()
            || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || (chars[i] == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
            {
                i += 1;
            }
            out.push(chars[start..i].iter().collect());
            continue;
        }
        i += 1;
    }
    out
}

/// A text made small with its blanks gone: how a case is looked for in a command.
fn flat(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// What the files `command` names hold, for those that are files of the world: a check may run
/// its case from a script of its own.
fn named_files(command: &str, world: &std::path::Path) -> Vec<String> {
    command
        .split(|c: char| c.is_whitespace() || "'\"`;|&()<>=".contains(c))
        .filter(|word| word.contains('.') && !word.starts_with('-'))
        .filter_map(|word| {
            let path = world.join(word);
            let meta = std::fs::metadata(&path).ok()?;
            (meta.is_file() && meta.len() <= NAMED_BYTES)
                .then(|| std::fs::read_to_string(&path).ok())
                .flatten()
        })
        .take(NAMED_FILES)
        .collect()
}

/// Whether `command` only prints words it holds: a lone `echo` or `printf` with nothing worked
/// out in it, or a one-line program (`python3 -c`, `node -e`, …) of imports and prints of
/// quoted words or numbers alone. Read narrowly: what it cannot read as such is a command that
/// may run code, and is not refused.
fn prints_only_words(command: &str) -> bool {
    let command = command.trim();
    let (first, rest) = command
        .split_once(char::is_whitespace)
        .map_or((command, ""), |(a, b)| (a, b.trim()));
    let first = first.rsplit('/').next().unwrap_or(first);
    match first {
        "echo" | "printf" => shell_words_only(rest),
        "python" | "python3" | "node" | "ruby" | "perl" | "php" => {
            let Some((flag, program)) = rest.split_once(char::is_whitespace) else {
                return false;
            };
            if !matches!(flag, "-c" | "-e" | "--eval" | "-r") {
                return false;
            }
            let program = program.trim();
            let Some(quote) = program.chars().next().filter(|q| *q == '"' || *q == '\'') else {
                return false;
            };
            let Some(body) = program
                .strip_prefix(quote)
                .and_then(|p| p.strip_suffix(quote))
            else {
                return false;
            };
            let body = body.replace(&format!("\\{quote}"), &quote.to_string());
            let statements: Vec<&str> = body
                .split([';', '\n'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            statements.iter().any(|s| prints_a_literal(s))
                && statements.iter().all(|s| prints_a_literal(s) || imports(s))
        }
        _ => false,
    }
}

/// An `echo`'s words with nothing worked out: no substitution, pipe, list or redirection
/// outside single quotes, and no `$` or backquote inside double ones.
fn shell_words_only(words: &str) -> bool {
    let mut quote = None;
    for c in words.chars() {
        match (quote, c) {
            (None, '\'' | '"') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None | Some('"'), '$' | '`' | '\\')
            | (None, '|' | ';' | '&' | '<' | '>' | '(' | ')') => return false,
            _ => {}
        }
    }
    quote.is_none()
}

/// A statement that prints quoted words or numbers alone: `print('x')`, `console.log("x", 3)`,
/// `puts 'x'`.
fn prints_a_literal(statement: &str) -> bool {
    let args = ["console.log", "print", "puts", "say", "echo", "printf"]
        .iter()
        .find_map(|name| statement.strip_prefix(name))
        .map(str::trim);
    let Some(args) = args else { return false };
    let args = args
        .strip_prefix('(')
        .and_then(|a| a.strip_suffix(')'))
        .unwrap_or(args)
        .trim();
    !args.is_empty() && split_args(args).iter().all(|a| literal(a))
}

/// Arguments split at their commas, a comma inside quotes kept: `'[4, 16]', 3` is two.
fn split_args(args: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut quote, mut start) = (None, 0);
    for (i, c) in args.char_indices() {
        match (quote, c) {
            (None, '\'' | '"') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, ',') => {
                out.push(args[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(args[start..].trim());
    out
}

/// A quoted word with nothing worked out in it, or a number.
fn literal(text: &str) -> bool {
    let quoted = ['\'', '"'].iter().any(|q| {
        text.len() >= 2
            && text.starts_with(*q)
            && text.ends_with(*q)
            && !text[1..text.len() - 1].contains(*q)
    });
    (quoted && !text.contains("${") && !text.contains('%'))
        || (!text.is_empty() && text.parse::<f64>().is_ok())
}

/// A statement that only brings code in: `import x`, `from x import y`, `const x = require('x')`.
fn imports(statement: &str) -> bool {
    statement.starts_with("import ")
        || (statement.starts_with("from ") && statement.contains(" import "))
        || (["const ", "let ", "var "]
            .iter()
            .any(|k| statement.starts_with(k))
            && statement.contains("require("))
        || statement.starts_with("require ")
        || statement.starts_with("use ")
}

/// Whether the line `text` states the output `gives`, letter case and blanks set aside: the line
/// holds it, or it holds the output the line names in backticks after `must give`. Held word by
/// word ([`words`]), so `ARRAY_SLICE(x, 1, 1)` never passes for `SLICE(x, 1, 1)`, nor `30` for
/// `3`.
fn states(text: &str, gives: &str) -> bool {
    let (line, output) = (words(text), words(gives));
    if within(&output, &line) {
        return true;
    }
    // Searched and cut in one copy, made small first: making letters small can change a text's
    // length, and the place found in one copy is no place in another.
    let small = text.to_lowercase();
    small
        .find(GIVES)
        .and_then(|at| small.get(at + GIVES.len()..))
        .and_then(|after| after.split('`').nth(1))
        .map(words)
        .is_some_and(|named| within(&named, &output))
}

/// A text as its words, made small: each run of letters, digits and `_` one word, every other
/// mark a word of its own, blanks none.
fn words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut open = false;
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() || c == '_' {
            match out.last_mut().filter(|_| open) {
                Some(word) => word.push(c),
                None => out.push(c.to_string()),
            }
            open = true;
        } else {
            open = false;
            if !c.is_whitespace() {
                out.push(c.to_string());
            }
        }
    }
    out
}

/// Whether the words `part` stand together, in order, somewhere in `whole`. Never when `part`
/// is empty: nothing states nothing.
fn within(part: &[String], whole: &[String]) -> bool {
    !part.is_empty() && whole.windows(part.len()).any(|at| at == part)
}

/// A `check` call the harness does not take as it was made, with how to make it: a correction,
/// which the guts reads as an answer and never as a failed act.
#[must_use]
pub fn corrected(said: String) -> Outcome {
    let mut o = Outcome::error(said);
    o.meta.corrected = true;
    o
}

/// How a command failed, as it is said: the last line of what it said, or that it said nothing.
fn failure(err: &str) -> &str {
    if err.is_empty() {
        "it said nothing"
    } else {
        err
    }
}

/// What the gate says of a check whose command failed to load when it was handed in and fails
/// the same way on the finished work. The gate's note and the journal find such a failure by
/// these words.
pub const CANNOT_RUN: &str = "cannot run here";

/// Whether an error says a command could not load what it names: a program that is not
/// installed, a module or a file that is not there. Such a command stops before it reaches what
/// its line is about. Kept narrow: an error of any other kind may be the very thing the ask is
/// there to change.
fn unloaded(err: &str) -> bool {
    const SIGNS: [&str; 7] = [
        "ERR_MODULE_NOT_FOUND",
        "Cannot find module",
        "ModuleNotFoundError",
        "No module named",
        "command not found",
        "No such file or directory",
        "can't open file",
    ];
    SIGNS.iter().any(|sign| err.contains(sign)) || err.trim_end().ends_with(": not found")
}

/// What the body is told of a check just taken: what it is held to, and for a stated output,
/// what its command does on the world as it stands. With it, whether the command prints that
/// stated output now, letter case and blanks set aside, and how it failed if it did.
fn said_of(
    check: &Check,
    tools: &[Box<dyn Tool>],
    cx: &mut Context,
) -> (String, bool, Option<String>) {
    let line = &check.line;
    let must = match &check.must {
        Must::Print(must) if must.is_empty() => {
            let said = format!(
                "{line} is held. On the code as you found it, your command prints nothing and \
succeeds. Your work must do the same."
            );
            return (said, false, None);
        }
        Must::Print(must) => {
            let said = format!(
                "{line} is held. On the code as you found it, your command prints:\n{}\nYour \
work must print the same.",
                clipped(must)
            );
            return (said, false, None);
        }
        Must::Fail(must) => {
            let said = format!(
                "{line} is held. On the code as you found it, your command fails:\n{}\nThat is \
what stays: while it fails that way the line holds. If your work makes it run, or fail another \
way, you are told once, and it refuses nothing: never add code only to keep it failing. If the \
failure is in the command itself and not what the line is about, hand the line in again.",
                failure(must)
            );
            return (said, false, None);
        }
        Must::Give(must) => must,
    };
    let now = run(&check.command, tools, cx);
    let prints = now.ok && loose(&now.out) == loose(must);
    let state = if !now.ok && unloaded(&now.err) {
        // Said at once, while the body can still write another command: at the gate it is a
        // failed landing, and a body told only that its command "fails" has gone on to change
        // code that was right.
        format!(
            "Now the command fails ({}). That is the command failing to load something, before \
it reaches what {line} is about. Unless your work is to make that exist, this command cannot run \
here and will show nothing of your work: hand {line} in again with a command that runs. It must \
print that when you are done.",
            failure(&now.err)
        )
    } else if !now.ok {
        format!(
            "Now the command fails ({}): it must print that when you are done.",
            failure(&now.err)
        )
    } else if now.out == *must {
        "It prints that now.".to_owned()
    } else if loose(&now.out) == loose(must) {
        format!(
            "It prints that now, but for letter case or blanks, which are set aside:\n{}",
            clipped(&now.out)
        )
    } else {
        format!("Now it prints:\n{}", clipped(&now.out))
    };
    let said = format!(
        "{line} is held: your command must print:\n{}\n{state}",
        clipped(must)
    );
    (said, prints, (!now.ok).then(|| now.err.clone()))
}

/// How a check already held stands on the world as it is: what a body asks by handing the same
/// line and command in again.
fn stands(check: &Check, tools: &[Box<dyn Tool>], cx: &mut Context) -> String {
    let line = &check.line;
    let (must, failed) = match &check.must {
        Must::Give(_) => return said_of(check, tools, cx).0,
        Must::Print(must) => (must, false),
        Must::Fail(must) => (must, true),
    };
    let now = run(&check.command, tools, cx);
    let did = if now.ok {
        format!("prints `{}`", clipped(&now.out))
    } else {
        format!("fails ({})", failure(&now.err))
    };
    if failed {
        let state = if !now.ok && now.err == *must {
            "It fails that way now: the line holds.".to_owned()
        } else {
            format!(
                "Now it {did}: that is said once when you answer, and refuses nothing. Never add \
code only to keep it failing."
            )
        };
        return format!(
            "{line} is held as before. On the code as you found it, your command fails:\n{}\n{state}",
            failure(must)
        );
    }
    let state = if now.ok && now.out == *must {
        "Your work does the same now: the line holds.".to_owned()
    } else {
        format!("Now it {did}: the line does not hold.")
    };
    format!(
        "{line} is held as before. On the code as you found it, your command prints:\n{}\n{state}",
        if must.is_empty() {
            "(nothing)".to_owned()
        } else {
            clipped(must)
        }
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

/// The number in a line's name: 12 for `D12`.
fn number(name: &str) -> Option<u64> {
    name.strip_prefix('D').and_then(|n| n.parse().ok())
}

/// `names` as words, three or more that follow one another said by their ends: `D1 to D20`,
/// `D1 to D4, D7 and D9`. A long list is said in a few words, and no name is left out.
fn runs(names: &[&str]) -> String {
    let mut said: Vec<String> = Vec::new();
    let mut at = 0;
    while let Some(first) = names.get(at) {
        let follow = names
            .iter()
            .skip(at)
            .zip(number(first).into_iter().flat_map(|n| n..=u64::MAX))
            .take_while(|(name, n)| number(name) == Some(*n))
            .count()
            .max(1);
        match names.get(at + follow - 1).filter(|_| follow > 2) {
            Some(last) => said.push(format!("{first} to {last}")),
            None => said.extend(names.iter().skip(at).take(follow).map(|n| (*n).to_owned())),
        }
        at += follow;
    }
    let said: Vec<&str> = said.iter().map(String::as_str).collect();
    and(&said)
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

#[cfg(test)]
mod tests {
    use super::{
        CANNOT_RUN, CHECK, CORRECTED_LINES, CheckTool, Judged, Kind, List, Must, READER, SAID, and,
        clipped, held_line, kind, loose, moved_note, prints_only_words, read_lines, runs,
        runs_its_case, stated, states, unloaded, values, words,
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
        assert!(list.checks()[0].must.stays() && !list.checks()[0].late);
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
        // D1's command printed `3` before any work: its check shows nothing of the work, and
        // the pass names it.
        assert!(list.checks().iter().all(|c| c.already == (c.line == "D1")));
        assert_eq!(
            list.by(&judged),
            "the ask's Done when list (3 lines; D1 already gave its output on the code you gave it)"
        );
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
            "the ask's Done when list (2 lines, 1 but for letter case or blanks; D1 already gave \
its output on the code you gave it)"
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

    /// What the human said must come out is a line the harness can hold, in their own words up
    /// to the output they state: what they say came out instead is no part of it. Words that
    /// state no output, or that say something must stay, are advice.
    #[test]
    fn a_correction_that_states_its_output_is_a_line() {
        assert_eq!(
            held_line("Still wrong. `rightLogShift(-1, 0)` must give `4294967295`; it gives `-1`.")
                .as_deref(),
            Some("Still wrong. `rightLogShift(-1, 0)` must give `4294967295`")
        );
        assert_eq!(
            held_line(
                "In DuckDB, `SELECT INTERVAL '1.5' DAY` MUST GIVE `SELECT INTERVAL '1.5' DAY`, \
                 not the one-string form"
            )
            .as_deref(),
            Some("In DuckDB, `SELECT INTERVAL '1.5' DAY` must give `SELECT INTERVAL '1.5' DAY`")
        );
        // No code span: the output ends at a `;` or with its sentence.
        assert_eq!(
            held_line("f(1) must give 2; it gives 3").as_deref(),
            Some("f(1) must give 2")
        );
        assert_eq!(
            held_line("f(1) must give 1.5 days. It gives 2.").as_deref(),
            Some("f(1) must give 1.5 days")
        );
        for advice in [
            "it still fails on ties",
            "`f(0)` must stay as it is today",
            "must give `2`",
            "`f(1)` must give",
            "`f(1)` must give `2",
            "`f(0)` must stay as it is and `f(1)` must give `2`",
        ] {
            assert_eq!(held_line(advice), None, "{advice}");
        }
        // The line states the output the human wants, never the one they called wrong.
        let line = held_line("`f(1)` must give `2`; it gives `3`").unwrap();
        assert!(states(&line, "2") && !states(&line, "3"), "{line}");
        assert_eq!(kind(&line), Kind::Gives);
    }

    /// A client reads an ask's list from the text the turn started with: the ask's own lines and
    /// the human's, once each, never the harness's sentences about them.
    #[test]
    fn the_list_an_ask_is_held_to_reads_from_its_text() {
        let mut list = List::of(ASK).holding(&["`f(1)` must give `2`".to_owned()]);
        list.pin("D4", "./f 1");
        let after = list.after_the_ask().unwrap();
        assert!(after.contains(crate::lines::HOLDS), "{after}");
        let text = format!(
            "remembered: D9 is old\n\n{}\n\n{ASK}\n\n{after}",
            crate::run::THE_ASK
        );
        let names: Vec<String> = crate::lines::of_ask(&text)
            .into_iter()
            .map(|(name, text)| format!("{name} {text}"))
            .collect();
        assert_eq!(
            names,
            [
                "D1 `total(['1', '', '2'])` must give `3`",
                "D2 `total(['4'])` must stay as it is today",
                "D3 the help text is clear",
                "D4 `f(1)` must give `2`",
            ]
        );
        assert_eq!(
            crate::lines::of_ask("just do it"),
            Vec::<crate::lines::Line>::new()
        );
    }

    /// What the human's corrections say must come out joins the list after its own lines, is
    /// said after the ask as the human's, and is held like any line. The same case twice is one
    /// line, and only the newest are kept.
    #[test]
    fn what_the_human_said_was_wrong_joins_the_list() {
        let cases = [
            "`f(1)` must give `2`".to_owned(),
            "`f(2)` must give `4`".to_owned(),
        ];
        // An ask with no list: the corrections are its list.
        let list = List::default().holding(&cases);
        assert_eq!(list.humans(), ["D1", "D2"]);
        assert_eq!(list.missing(), ["D1", "D2"]);
        let after = list.after_the_ask().unwrap();
        assert!(
            after.starts_with(
                "## Done when\nD1: `f(1)` must give `2`\nD2: `f(2)` must give `4`\n\nD1 and D2 are \
                 the human's own words: they said an earlier answer to this ask was wrong, and \
                 this is what must come out. They are not yours to rewrite, and they are cases \
                 of the ask, not the whole of it: the ask stands as written.\n\nThe ask holds a \
                 `## Done when` list (D1 to D2)"
            ),
            "{after}"
        );
        assert!(
            list.by(&Judged::default())
                .starts_with("the human's Done when list ("),
            "{}",
            list.by(&Judged::default())
        );
        // An ask with its own list: named on from its last line, and only they are said after.
        let list = List::of(ASK).holding(&cases[..1]);
        assert_eq!(list.humans(), ["D4"]);
        let after = list.after_the_ask().unwrap();
        assert!(
            after.starts_with(
                "## Done when\nD4: `f(1)` must give `2`\n\nD4 is the human's own word: they said \
                 an earlier answer to this ask was wrong, and this is what must come out. It is \
                 not yours to rewrite, and it is one case of the ask, not the whole of it: the ask \
                 stands as written.\n\nThe ask holds a `## Done when` list (D1 to D4)"
            ),
            "{after}"
        );
        assert_eq!(
            list.text_of("D4").unwrap().as_deref(),
            Some("`f(1)` must give `2`")
        );
        let by = list.by(&Judged::default());
        assert!(
            by.starts_with("the ask's Done when list (")
                && by.contains("D4 from what the human said was wrong before"),
            "{by}"
        );
        // A reader's list: its lines and the human's, all said after the ask.
        let list = List::read(&["`g(0)` must stay as it is today".to_owned()]).holding(&cases);
        assert_eq!(list.humans(), ["D2", "D3"]);
        assert!(
            list.after_the_ask().unwrap().starts_with(
                "## Done when\nD1: `g(0)` must stay as it is today\nD2: `f(1)` must give `2`\nD3: \
                 `f(2)` must give `4`\n\nD2 and D3 are the human's own words"
            ),
            "{:?}",
            list.after_the_ask()
        );
        // The same case said twice is one line, and a case the ask's own list holds is not said again.
        let twice = [cases[0].clone(), cases[0].clone()];
        assert_eq!(List::default().holding(&twice).humans(), ["D1"]);
        let own = "x\n\n## Done when\nD1: `f(1)` must give `2`\n";
        assert_eq!(
            List::of(own).holding(&cases[..1]).humans(),
            [] as [String; 0]
        );
        // Only the newest are held.
        let many: Vec<String> = (1..=20)
            .map(|i| format!("`h({i})` must give `{i}`"))
            .collect();
        let list = List::default().holding(&many);
        assert_eq!(list.humans().len(), CORRECTED_LINES);
        assert_eq!(
            list.text_of("D1").unwrap().as_deref(),
            Some("`h(9)` must give `9`")
        );
        // No correction: the list is as it was.
        assert!(List::default().holding(&[]).is_empty());
        assert!(List::default().holding(&[]).after_the_ask().is_none());
    }

    /// A line states an output when it holds the output given, or when the output given holds
    /// the one the line names after `must give`: a check that prints the whole statement around
    /// the line's expression is a right check. On our bench that form was refused ten times a
    /// run, and the refusals stopped two runs with nothing done.
    #[test]
    fn an_output_that_holds_the_lines_own_is_one_the_line_states() {
        let line =
            "`ARRAY_SLICE(x, 2, 4)` read as DuckDB, written for Spark, must give `SLICE(x, 2, 3)`";
        assert!(states(line, "SLICE(x, 2, 3)"));
        assert!(
            states(line, "SELECT SLICE(x, 2, 3)"),
            "the statement around it"
        );
        assert!(
            states(line, "select slice(x,2,3)"),
            "letter case and blanks aside"
        );
        assert!(!states(line, "SELECT SLICE(x, 2, 4)"), "another output");
        assert!(
            !states(line, "SELECT ARRAY_SLICE(x, 2, 4)"),
            "the input, dressed up"
        );
        assert!(
            !states(line, "D1: ARRAY_SLICE(x,2,4) from DuckDB to Spark"),
            "a label"
        );
        // Word by word: a longer name that ends like the output is not the output.
        let near = "`ARRAY_SLICE(x, 1, 1)` read as DuckDB must give `SLICE(x, 1, 1)`";
        assert!(!states(near, "D2: ARRAY_SLICE(x,1,1) from DuckDB to Spark"));
        assert!(states(near, "SELECT SLICE(x,1,1)"));
        assert!(!states("`f(1)` must give `30`", "3") && states("`f(1)` must give `30`", "30"));
        assert!(!states(line, ""), "nothing states nothing");
        assert_eq!(words("a_b(X,  1)"), ["a_b", "(", "x", ",", "1", ")"]);
        // A line that names no output in backticks after the phrase: only what it holds.
        let plain = "the total of 1 and 2 Must Give 3";
        assert!(states(plain, "3") && !states(plain, "SELECT 3"));
        // Letters that change length when made small move no place in the line.
        assert!(states("İ `a` must give `İb`", "x İb y"));

        let (_dir, mut cx) = world();
        let mut list = List::of(
            "Sum.\n\n## Done when\nD1: `1 2` in calc must give `3`\nD2: `4` must stay as it is today\n",
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "echo total: $(sh calc.sh 1 2)", "gives": "total: 3"}"#,
            true,
        );
        assert!(
            !o.is_error && o.content.contains("It prints that now."),
            "{}",
            o.content
        );
    }

    /// A check that only prints the words holds when the answer lands and shows nothing of the
    /// work. The harness cannot tell it from a case that was right before, so it refuses
    /// nothing, and the pass names every line whose check gave its output before any change: on
    /// our bench one run handed in `echo` for each of its twenty-eight lines and was told
    /// "verified".
    #[test]
    fn a_check_that_gave_its_output_before_the_work_is_named_in_the_pass() {
        let (dir, mut cx) = world();
        let ask = "Count from one.\n\n## Done when\nD1: `total(1, 2)` must give `4`\nD2: `total(2, 2)` must give `5`\nD3: `total(0)` must give `1`\nD4: `total()` must give `1`\nD5: `total(5)` must give `6`\n";
        let mut list = List::of(ask);
        // Two commands that run the code: it does not give the output yet.
        for (line, args, gives) in [("D1", "1 2", "4"), ("D2", "2 2", "5")] {
            let o = call_in(
                &mut list,
                &mut cx,
                &format!(
                    r#"{{"line": "{line}", "command": "sh calc.sh {args}", "gives": "{gives}"}}"#
                ),
                true,
            );
            assert!(
                !o.is_error && o.content.contains("Now it prints:"),
                "{}",
                o.content
            );
        }
        // Two that run the code and then print the words, and take the same answer as a right
        // check would. (One that only prints the words is refused: it runs nothing.)
        for (line, args, gives) in [("D3", "0", "1"), ("D4", "", "1")] {
            let o = call_in(
                &mut list,
                &mut cx,
                &format!(
                    r#"{{"line": "{line}", "command": "sh calc.sh {args} >/dev/null && echo {gives}", "gives": "{gives}"}}"#
                ),
                true,
            );
            assert!(
                !o.is_error && o.content.contains("It prints that now."),
                "{}",
                o.content
            );
        }
        // One that comes in after the code has changed: what it did before is not known.
        std::fs::write(
            dir.path().join("calc.sh"),
            "n=1\nfor a in \"$@\"; do n=$((n + a)); done\necho $n\n",
        )
        .unwrap();
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D5", "command": "sh calc.sh 5", "gives": "6"}"#,
            false,
        );
        assert!(!o.is_error, "{}", o.content);
        let already: Vec<(&str, bool, bool)> = list
            .checks()
            .iter()
            .map(|c| (c.line.as_str(), c.already, c.late))
            .collect();
        assert_eq!(
            already,
            [
                ("D1", false, false),
                ("D2", false, false),
                ("D3", true, false),
                ("D4", true, false),
                ("D5", false, true)
            ]
        );
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!((judged.failures.len(), judged.hold), (0, 5), "{judged:?}");
        assert_eq!(
            list.by(&judged),
            "the ask's Done when list (5 lines; D3 and D4 already gave their output on the code \
you gave it; 1 check handed in after the code had changed)"
        );
        // The same line and command again keeps the check as it came in, and what it showed.
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D3", "command": "sh calc.sh 0 >/dev/null && echo 1", "gives": "1"}"#,
            false,
        );
        assert!(!o.is_error, "{}", o.content);
        assert!(
            list.checks()
                .iter()
                .any(|c| c.line == "D3" && c.already && !c.late)
        );
        // Another command for the line is another check: a real one, handed in late.
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D3", "command": "sh calc.sh 0", "gives": "1"}"#,
            false,
        );
        assert!(!o.is_error, "{}", o.content);
        assert!(
            list.checks()
                .iter()
                .any(|c| c.line == "D3" && !c.already && c.late)
        );
        // Names that follow one another are said by their ends, and none is left out.
        assert_eq!(
            runs(&["D1", "D2", "D3", "D4", "D7", "D9", "D10"]),
            "D1 to D4, D7, D9 and D10"
        );
        assert_eq!(runs(&["D3"]), "D3");
        assert_eq!(runs(&["D3", "D4"]), "D3 and D4");
        assert_eq!(runs(&["D1", "D2", "D3"]), "D1 to D3");
        assert_eq!(runs(&[]), "");
    }

    /// A check the harness does not take as it was made is a correction of the call: the body is
    /// told how to make it, and the guts reads no failed act in it.
    #[test]
    fn a_check_not_taken_is_a_correction_and_no_failed_act() {
        let (_dir, mut cx) = world();
        let mut list = List::of(ASK);
        for input in [
            r#"{"line": "D2", "command": "sh calc.sh 4", "gives": "four"}"#,
            r#"{"line": "D1", "command": "sh calc.sh 1 2"}"#,
            r#"{"line": "D1", "command": "sh calc.sh 1 2", "gives": "4"}"#,
            r#"{"line": "D9", "command": "true"}"#,
            r#"{"line": "D1", "command": "  "}"#,
            r#"{"line": "D1"}"#,
        ] {
            let o = call_in(&mut list, &mut cx, input, true);
            assert!(o.is_error && o.meta.corrected, "{input}: {}", o.content);
            assert!(!crate::stress::failed_act(&o), "{input}");
        }
        assert!(crate::stress::failed_act(&mikata_tools::Outcome::error(
            "refused: outside your soil"
        )));
    }

    /// What must stay is whatever the command does on the code as found: it may print nothing,
    /// and it may fail. Only a command that does not do the same thing twice says nothing, and
    /// one handed in after the code changed with no way back to that code.
    #[test]
    fn what_must_stay_is_whatever_the_command_does_today() {
        let (_dir, mut cx) = world();
        let mut list = List::of(ASK);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "true 4"}"#,
            true,
        );
        assert!(
            !o.is_error && o.content.contains("prints nothing and succeeds"),
            "{}",
            o.content
        );
        assert_eq!(list.checks()[0].must, Must::Print(String::new()));
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "echo 4 $(date +%s%N)"}"#,
            true,
        );
        assert!(
            o.is_error && o.content.contains("does not do the same twice"),
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
        assert_eq!(list.checks().len(), 1, "a refused check replaces none");
    }

    /// The same line and command handed in again asks how the line stands now. The check is
    /// kept as it came in: nothing is read again from the code as found, and it is not late.
    #[test]
    fn the_same_check_again_says_how_the_line_stands_now() {
        let (dir, mut cx) = world();
        let mut list = List::of(ASK);
        let d2 = r#"{"line": "D2", "command": "sh calc.sh 4"}"#;
        let d1 = r#"{"line": "D1", "command": "sh calc.sh 1 2", "gives": "3"}"#;
        assert!(!call_in(&mut list, &mut cx, d2, true).is_error);
        assert!(!call_in(&mut list, &mut cx, d1, true).is_error);
        // The work breaks D2. Asked again after the change, with no way back to the code as
        // found: the check is not read again, so nothing is refused, and it is told as it is.
        std::fs::write(dir.path().join("calc.sh"), "echo 3\n").unwrap();
        let o = call_in(&mut list, &mut cx, d2, false);
        assert!(
            !o.is_error
                && o.content.contains("D2 is held as before")
                && o.content.contains("your command prints:\n4\n")
                && o.content
                    .contains("Now it prints `3`: the line does not hold."),
            "{}",
            o.content
        );
        let o = call_in(&mut list, &mut cx, d1, false);
        assert!(
            !o.is_error && o.content.contains("It prints that now."),
            "{}",
            o.content
        );
        assert!(list.checks().iter().all(|c| !c.late), "{:?}", list.checks());
        std::fs::write(dir.path().join("calc.sh"), "echo 4\n").unwrap();
        let o = call_in(&mut list, &mut cx, d2, false);
        assert!(o.content.contains("the line holds"), "{}", o.content);
        // Another command for the line is another check: it is read again, and here it cannot be.
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh calc.sh 4 0"}"#,
            false,
        );
        assert!(
            o.is_error && o.content.contains("is not kept"),
            "{}",
            o.content
        );
    }

    /// A line that must stay and fails on the code as found holds while it fails the same way.
    /// When the work makes it run, or fail another way, nothing is refused: asks are for making
    /// things work, and the harness cannot know this one was not asked for. It is said once.
    #[test]
    fn a_line_that_failed_holds_while_it_fails_and_is_said_once_when_it_does_not() {
        let (dir, mut cx) = world();
        let script = |text: &str| std::fs::write(dir.path().join("strict.sh"), text).unwrap();
        script("echo 'not a number: x' >&2\nexit 2\n");
        let mut list = List::of("Be kinder.\n\n## Done when\nD1: `x` must stay as it is today\n");
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh x"}"#,
            true,
        );
        assert!(
            !o.is_error
                && o.content.contains("your command fails:\nnot a number: x\n")
                && o.content.contains("it refuses nothing"),
            "{}",
            o.content
        );
        assert_eq!(list.checks()[0].must, Must::Fail("not a number: x".into()));
        // It fails the same way: the line holds.
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!(
            (judged.failures.len(), judged.hold, judged.moved.len()),
            (0, 1, 0),
            "{judged:?}"
        );
        assert_eq!(list.news(&judged), [] as [String; 0]);
        // The work makes it run: no failure, said once, and the pass names the line.
        script("echo 0\n");
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!((judged.failures.len(), judged.hold), (0, 0), "{judged:?}");
        let news = list.news(&judged);
        assert_eq!(news.len(), 1);
        assert!(
            news[0].contains("D1 must stay: `sh strict.sh x` failed on the code as you found it (not a number: x) and prints `0` on your work"),
            "{news:?}"
        );
        let note = moved_note(&news);
        assert!(
            note.contains("said once") && note.contains("this refuses no landing"),
            "{note}"
        );
        assert_eq!(list.news(&judged), [] as [String; 0], "said once");
        assert_eq!(
            list.by(&judged),
            "the ask's Done when list (0 lines; D1 no longer fails as on the code you gave it)"
        );
        // Failing another way is the same: not what it did, and not refused.
        script("echo 'no such unit' >&2\nexit 3\n");
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!(
            (judged.failures.len(), judged.moved.len()),
            (0, 1),
            "{judged:?}"
        );
        assert!(
            judged.moved[0]
                .1
                .contains("fails (no such unit) on your work"),
            "{judged:?}"
        );
        // A line that ran and now fails is a break, and refused.
        script("echo 5\n");
        let mut list = List::of("Be kinder.\n\n## Done when\nD1: `x` must stay as it is today\n");
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh x"}"#,
            true,
        );
        assert!(!o.is_error, "{}", o.content);
        script("exit 1\n");
        let judged = list.judge(&registry(), &mut cx);
        assert!(
            judged.failures[0].contains("D1 must stay: `sh strict.sh x` printed `5` on the code as you found it and fails (it said nothing) on your work"),
            "{judged:?}"
        );
    }

    /// A check that sends its error down the same pipe as its output (`2>&1`) is still told how
    /// it failed, and a line held to a failure is held to that failure, not to any failure: the
    /// error is the line that names itself, whatever a runtime prints after it.
    #[test]
    fn a_failing_check_is_told_its_error_whichever_pipe_it_came_down() {
        let (dir, mut cx) = world();
        let script = |text: &str| std::fs::write(dir.path().join("strict.sh"), text).unwrap();
        // As Node ends an uncaught error: the error, its stack, then its own version.
        let node = |what: &str| {
            format!(
                "echo 'Error [ERR_MODULE_NOT_FOUND]: Cannot find module {what}' >&2\necho '    at resolve (node:internal:1:1)' >&2\necho 'Node.js v22.0.0' >&2\nexit 1\n"
            )
        };
        script(&node("gen.js"));
        let ask =
            "Be kinder.\n\n## Done when\nD1: `x` must give `5`\nD2: `y` must stay as it is today\n";
        let mut list = List::of(ask);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh x 2>&1", "gives": "5"}"#,
            true,
        );
        assert!(
            o.content.contains(
                "Now the command fails (Error [ERR_MODULE_NOT_FOUND]: Cannot find module gen.js)"
            ),
            "{}",
            o.content
        );
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D2", "command": "sh strict.sh y 2>&1"}"#,
            true,
        );
        assert!(
            o.content.contains(
                "your command fails:\nError [ERR_MODULE_NOT_FOUND]: Cannot find module gen.js\n"
            ),
            "{}",
            o.content
        );
        // Failing another way is not the failure the line was held to.
        script(&node("other.js"));
        let judged = list.judge(&registry(), &mut cx);
        assert!(
            judged.moved.iter().any(|(line, said)| line == "D2"
                && said.contains(
                    "fails (Error [ERR_MODULE_NOT_FOUND]: Cannot find module other.js) on your work"
                )),
            "{judged:?}"
        );
    }

    /// The mathjs case, with its real error: three checks loaded the library from a file a
    /// source tree does not have, the body was told "fails on your work" at four landings, and
    /// its code was right all along. A command that failed to load when it was handed in and
    /// fails the same way on the finished work is said to be a command that cannot run: at
    /// hand-in while another can still be written, and at the gate apart from a wrong value.
    #[test]
    fn a_check_that_cannot_load_is_told_apart_from_one_the_work_breaks() {
        let (dir, mut cx) = world();
        let script = |text: &str| std::fs::write(dir.path().join("strict.sh"), text).unwrap();
        let missing = "Error [ERR_MODULE_NOT_FOUND]: Cannot find module \
'/w/src/entry/pureFunctionsAny.generated.js' imported from /w/src/entry/mainAny.js";
        let fails = |err: &str| format!("echo \"{err}\" >&2\necho 'Node.js v22.0.0' >&2\nexit 1\n");
        script(&fails(missing));
        let ask =
            "Shift right.\n\n## Done when\nD1: `rightLogShift(-1, 0)` must give `4294967295`\n";
        let mut list = List::of(ask);
        let hand = r#"{"line": "D1", "command": "sh strict.sh -1 0 2>&1", "gives": "4294967295"}"#;
        let o = call_in(&mut list, &mut cx, hand, true);
        assert!(
            o.content
                .contains(&format!("Now the command fails ({missing}).")),
            "{}",
            o.content
        );
        assert!(
            o.content
                .contains("failing to load something, before it reaches what D1 is about")
                && o.content
                    .contains("hand D1 in again with a command that runs"),
            "{}",
            o.content
        );
        // The work is done and the command is as it was: the gate does not call the work wrong.
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!((judged.failures.len(), judged.hold), (1, 0), "{judged:?}");
        let said = &judged.failures[0];
        assert!(
            said.contains(&format!(
                "D1 must give `4294967295`: `sh strict.sh -1 0 2>&1` {CANNOT_RUN}: it fails as it \
did when you handed it in ({missing}), before it reaches your work."
            )) && said.contains("hand D1 in again with a command that runs")
                && !said.contains("on your work"),
            "{said}"
        );
        // The same line and command handed in again asks how it stands, and is told the same.
        let o = call_in(&mut list, &mut cx, hand, false);
        assert!(
            o.content.contains("failing to load something"),
            "{}",
            o.content
        );
        // Work that makes the missing thing exist is what the check was for: it holds.
        script("echo 4294967295\n");
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!((judged.failures.len(), judged.hold), (0, 1), "{judged:?}");
        // Failing another way is a failure on the work, said as before.
        script(&fails("RangeError: shift out of range"));
        let judged = list.judge(&registry(), &mut cx);
        assert!(
            judged.failures[0].contains("fails (RangeError: shift out of range) on your work")
                && !judged.failures[0].contains(CANNOT_RUN),
            "{judged:?}"
        );
    }

    /// A command that fails when handed in for a reason the ask may be there to change (the
    /// function is not written yet) is not called a command that cannot run. Failing the same
    /// way on the finished work, it is said to be unmoved by the work, which is true whether the
    /// work or the command is at fault.
    #[test]
    fn a_check_the_work_did_not_move_is_said_to_be_unmoved() {
        let (dir, mut cx) = world();
        let script = |text: &str| std::fs::write(dir.path().join("strict.sh"), text).unwrap();
        let unwritten = "TypeError: math.rightLogShift is not a function";
        script(&format!("echo '{unwritten}' >&2\nexit 1\n"));
        let mut list = List::of("Shift right.\n\n## Done when\nD1: `f(1)` must give `2`\n");
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh", "gives": "2"}"#,
            true,
        );
        assert!(
            o.content.contains(&format!(
                "Now the command fails ({unwritten}): it must print that"
            )) && !o.content.contains("failing to load"),
            "{}",
            o.content
        );
        let judged = list.judge(&registry(), &mut cx);
        assert!(
            judged.failures[0].contains(&format!(
                "`sh strict.sh` fails ({unwritten}) on your work, as it did when you handed it \
in: your work has not changed what this command does"
            )) && !judged.failures[0].contains(CANNOT_RUN),
            "{judged:?}"
        );
        // Each sign of a failure to load, and what is not one.
        for err in [
            "sh: 1: pytest: not found",
            "bash: line 1: pytest: command not found",
            "ModuleNotFoundError: No module named 'sqlglot'",
            "/usr/bin/python3: No module named pytest",
            "python3: can't open file '/w/t.py': [Errno 2] No such file or directory",
            "Error: Cannot find module './src/index.js'",
        ] {
            assert!(unloaded(err), "{err}");
        }
        for err in [
            "",
            "AssertionError: 3 != 6",
            "KeyError: 'unit not found in table'",
            "ImportError: cannot import name 'slice_sql' from 'sqlglot.generator'",
        ] {
            assert!(!unloaded(err), "{err}");
        }
    }

    /// A body's command for a line the human said comes out wrong, which prints what they want
    /// before any work, is told at once that it most often runs another case.
    #[test]
    fn a_check_for_the_humans_case_that_prints_the_answer_before_any_work_is_questioned() {
        let (dir, mut cx) = world();
        std::fs::write(dir.path().join("strict.sh"), "echo 6\n").unwrap();
        let line = "Still wrong. `total 1 2` must give `6`".to_owned();
        let mut list = List::of("Sum.\n").holding(&[line]);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh 1 2", "gives": "6"}"#,
            true,
        );
        assert!(
            o.content.contains("The human said this case comes out wrong, and your command prints what they want before any work"),
            "{}",
            o.content
        );
        // A line of the ask's own is not questioned so.
        let mut own = List::of("Sum.\n\n## Done when\nD1: `total 1 2` must give `6`\n");
        let o = call_in(
            &mut own,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh 1 2", "gives": "6"}"#,
            true,
        );
        assert!(!o.content.contains("The human said"), "{}", o.content);
    }

    /// A check that pipes its command into `tail` ends well whatever the command did. A line
    /// that must stay is not held to an error message as if it were output: its command failed
    /// on the code as found, so the line is held as a failure is, and work that makes the
    /// command run is said once and refuses nothing. A reader wrote such a line for the very
    /// input an ask was about, and a correct fix failed its gate twice.
    #[test]
    fn a_piped_check_that_failed_is_held_as_a_failure_not_as_its_last_line() {
        let (dir, mut cx) = world();
        let script = |text: &str| std::fs::write(dir.path().join("strict.sh"), text).unwrap();
        script(
            "echo 'ParseError: no such clause' >&2\necho '  PARTITION BY EXCLUDING a' >&2\nexit 1\n",
        );
        let ask = "Be kinder.\n\n## Done when\nD1: `x` must stay as it is today\n";
        let mut list = List::of(ask);
        let o = call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh x 2>&1 | tail -1"}"#,
            true,
        );
        assert!(
            !o.is_error
                && o.content
                    .contains("your command fails:\nPARTITION BY EXCLUDING a\n"),
            "{}",
            o.content
        );
        assert!(
            matches!(list.checks()[0].must, Must::Fail(_)),
            "{:?}",
            list.checks()
        );
        // The work makes the command run: said once, and the landing is not refused.
        script("echo 'PARTITION BY EXCLUDING a'\n");
        let judged = list.judge(&registry(), &mut cx);
        assert_eq!(
            (judged.failures.len(), judged.moved.len()),
            (0, 1),
            "{judged:?}"
        );
        // The other way round is a break: it printed, and now its command fails behind the pipe.
        script("echo 5\n");
        let mut list = List::of(ask);
        call_in(
            &mut list,
            &mut cx,
            r#"{"line": "D1", "command": "sh strict.sh x 2>&1 | tail -1"}"#,
            true,
        );
        assert_eq!(list.checks()[0].must, Must::Print("5".into()));
        script("echo 'ValueError: bad unit' >&2\nexit 1\n");
        let judged = list.judge(&registry(), &mut cx);
        assert!(
            judged.failures[0].contains(
                "printed `5` on the code as you found it and fails (ValueError: bad unit) on your work"
            ),
            "{judged:?}"
        );
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

    /// A check runs its line's case. One that only prints words it holds, or that never names
    /// the case's values, is refused before anything runs; one that runs the case, in the
    /// command or from a file it names, is taken. The first is what a body handed in on the
    /// board when two lines of its list could not both be true: `format.__doc__` for a line
    /// about `format(90)`, and the gate landed it.
    #[test]
    fn a_check_runs_its_lines_case() {
        let (dir, mut cx) = world();
        let mut list = List::of(ASK);
        let hand =
            |list: &mut List, cx: &mut Context, line: &str, command: &str, gives: Option<&str>| {
                let mut input = serde_json::json!({"line": line, "command": command});
                if let Some(g) = gives {
                    input["gives"] = g.into();
                }
                call_in(list, cx, &input.to_string(), true)
            };
        for words in [
            "echo 3",
            r#"node -e "console.log('3')""#,
            r#"python3 -c "import calc; print('3')""#,
        ] {
            let o = hand(&mut list, &mut cx, "D1", words, Some("3"));
            assert!(
                o.is_error && o.content.contains("only prints words"),
                "{words}: {}",
                o.content
            );
        }
        let o = hand(&mut list, &mut cx, "D1", "sh calc.sh 4 -1", Some("3"));
        assert!(
            o.is_error && o.content.contains("never names `2` of its case"),
            "{}",
            o.content
        );
        let o = hand(&mut list, &mut cx, "D2", "grep -c total calc.sh", None);
        assert!(
            o.is_error && o.content.contains("never names `4`"),
            "{}",
            o.content
        );
        assert!(
            list.checks().is_empty(),
            "a refused check is held for nothing"
        );
        // The case in the command, or in a script the command runs.
        let o = hand(&mut list, &mut cx, "D1", "sh calc.sh 1 '' 2", Some("3"));
        assert!(!o.is_error, "{}", o.content);
        std::fs::write(dir.path().join("four.sh"), "sh calc.sh 4\n").unwrap();
        let o = hand(&mut list, &mut cx, "D2", "sh four.sh", None);
        assert!(!o.is_error, "{}", o.content);
        // A line with no case is held to nothing but a command that runs something.
        let o = hand(&mut list, &mut cx, "D3", "echo clear", None);
        assert!(
            o.is_error && o.content.contains("only prints words"),
            "{}",
            o.content
        );
        let o = hand(&mut list, &mut cx, "D3", "grep -c total calc.sh", None);
        assert!(!o.is_error, "{}", o.content);

        let found = dir.path();
        // The board's own two lines, as the human wrote them: no backticks, no `must`.
        let docstring = r#"python3 -c "import durations; print(durations.format.__doc__)""#;
        for line in ["format(90) gives '90m'", "format(90) still gives '1h30m'"] {
            assert!(
                runs_its_case("D2", Some(line), docstring, found).is_err(),
                "{line}"
            );
            let runs_it = r#"python3 -c "from durations import format; print(format(90))""#;
            assert!(
                runs_its_case("D2", Some(line), runs_it, found).is_ok(),
                "{line}"
            );
        }
        let line = "`format(90)` must stay as it is today";
        assert!(
            runs_its_case(
                "D2",
                Some(line),
                r#"python3 -c "import d; print(d.format.__doc__)""#,
                found
            )
            .is_err()
        );
        for runs_it in [
            r#"python3 -c "import d; print(d.format(90))""#,
            "cargo run -q -- 90",
        ] {
            assert!(
                runs_its_case("D2", Some(line), runs_it, found).is_ok(),
                "{runs_it}"
            );
        }
        // Another input than the line's is not its case; a call spelt otherwise is.
        let slice = "`ARRAY_SLICE(x, 2, 4)` read as duckdb must stay as it is today";
        let other =
            r#"python3 -c "import sqlglot; print(sqlglot.transpile('ARRAY_SLICE(x, 1, 10)')[0])""#;
        assert!(runs_its_case("D1", Some(slice), other, found).is_err());
        let shift = "`rightLogShift(matrix([[-1, 2]]), 0)` must give `[[4294967295, 2]]`";
        let aliased =
            r#"node -e "const m = math.matrix; console.log(math.rightLogShift(m([[-1,2]]),0))""#;
        assert!(runs_its_case("D1", Some(shift), aliased, found).is_ok());
    }

    /// What only prints words is read narrowly: anything that may work something out runs.
    #[test]
    fn a_command_that_only_prints_words_is_read_narrowly() {
        for words in [
            "echo -5",
            r#"echo "D9: mod(-7, 3) plain numbers""#,
            r"printf 'x\n'",
            r#"node -e "console.log('[4, 16]')""#,
            r#"python3 -c "import sqlglot; print('SLICE(x, 4, 1)')""#,
            r#"/usr/bin/python3 -c 'print("a", 3)'"#,
        ] {
            assert!(prints_only_words(words), "{words}");
        }
        for runs in [
            "echo $((1 + 2))",
            "echo 3 | sh calc.sh",
            "echo `date`",
            r#"python3 -c "print(total([1, 2]))""#,
            r#"python3 -c "import d; print(d.format.__doc__)""#,
            r#"python3 -c "print(f'{1 + 2}')""#,
            r#"node -e "console.log(JSON.stringify([1]))""#,
            "sh calc.sh 1 2",
            "python3 check.py",
        ] {
            assert!(!prints_only_words(runs), "{runs}");
        }
        assert_eq!(values("format(90)"), ["90"]);
        assert_eq!(values("mod(-7, -3.5)"), ["-7", "-3.5"]);
        assert_eq!(values("SELECT INTERVAL '1.5 Day'"), ["1.5 day"]);
        assert_eq!(values("total(['1', '', '2'])"), ["1", "2"]);
        assert_eq!(values("it's x - 1"), ["1"]);
    }
}
