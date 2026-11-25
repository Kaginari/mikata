//! The `todo` unit: a list the body keeps when it wants one, with steps the harness checks.
//!
//! The plan made the body write what done means before any code, every ask; stage 13 found it
//! earned no pass and cost calls, and it went off by default. Maki offers the other way round: a
//! `todo_write` tool the model reaches for when the work has several steps (on mathjs, qwen used
//! it in 3 runs of 10). Its list is a note to itself: nothing holds a step marked done to being
//! done. Here a step may carry a `check` — one command that shows it done — and the harness runs it
//! the moment the step is marked completed. A check that fails leaves the step in progress, with
//! what the command said; one that passes is said so. The list costs no call the body did not
//! choose to make, and a step it calls done has been shown done.
//!
//! Off by default (`--units +todo`, `--profile todo`): a screen decides whether it earns a place.

use mikata_races::ToolClass;
use mikata_tools::{Context, Outcome, Tool, call};
use serde::Deserialize;
use serde_json::{Value, json};

/// The tool's name, and the unit's id.
pub const TODO: &str = "todo";

/// The prompt's line for the unit, in the place of a plan.
pub const HINT: &str = "- For work of three steps or more, keep a todo list with the todo tool and \
update it after each step, never all at the end. Give a step a `check` when one command can show \
it done: the harness runs it as you mark the step completed, and a check that fails leaves the \
step in progress. A small change needs no list.";

/// The most steps a list holds.
const MOST: usize = 30;
/// The characters of a check's output said back.
const SHOWN: usize = 300;
/// How long one check may run, in milliseconds.
const EACH_MS: u64 = 60_000;

/// The `todo` tool as the model sees it. The loop runs the call ([`Todos::update`]).
pub struct TodoTool;

impl Tool for TodoTool {
    fn name(&self) -> &'static str {
        TODO
    }

    fn description(&self) -> &'static str {
        "Keep your todo list for work of three steps or more: send the whole list each time (it replaces the last one). Mark a step in_progress as you start it and completed when it is done, after each step. A step may carry `check`: one shell command whose success shows the step done (a test, a script that prints the expected value and exits 0). The harness runs it when you mark the step completed; if it fails, the step stays in_progress and you are told what it said."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": "The whole list, in order.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {"type": "string", "description": "The step."},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "completed", "cancelled"]},
                            "check": {"type": "string", "description": "Optional: one command that exits 0 when the step is done."}
                        },
                        "required": ["content", "status"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["todos"],
            "additionalProperties": false
        })
    }

    fn class(&self) -> ToolClass {
        ToolClass::Read
    }

    fn example(&self) -> Value {
        json!({"todos": [
            {"content": "reproduce the bug", "status": "completed", "check": "npm test -- --grep 'radix'"},
            {"content": "fix the parser", "status": "in_progress"}
        ]})
    }

    fn run(&self, _: Value, _: &mut Context) -> Outcome {
        Outcome::error("the todo list is the loop's to keep")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    todos: Vec<Step>,
}

/// One step of the list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// What the step is.
    pub content: String,
    /// `pending`, `in_progress`, `completed` or `cancelled`.
    pub status: String,
    /// One command whose success shows the step done.
    #[serde(default)]
    pub check: Option<String>,
}

/// The body's list, as the harness keeps it, and the checks that held.
#[derive(Debug, Default)]
pub struct Todos {
    steps: Vec<Step>,
    /// The steps whose check passed, by their content and command: a list sent again does not
    /// run a passed check again.
    held: Vec<(String, String)>,
    /// How many checks ran, and how many failed.
    pub ran: u32,
    /// See [`Todos::ran`].
    pub failed: u32,
}

const STATUSES: [&str; 4] = ["pending", "in_progress", "completed", "cancelled"];

impl Todos {
    /// The steps as they stand.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// The list the body sent: each step newly marked completed with a check has it run (with
    /// `tools`' bash, in `cx`), and stays in progress when it fails.
    pub fn update(&mut self, input: &str, tools: &[Box<dyn Tool>], cx: &mut Context) -> Outcome {
        let args: Args = match serde_json::from_str(input) {
            Ok(a) => a,
            Err(e) => {
                return Outcome::error(format!(
                    "the todo list did not read ({e}); send {{\"todos\": [{{\"content\", \"status\", \"check\"?}}]}}"
                ));
            }
        };
        if args.todos.len() > MOST {
            return Outcome::error(format!("a list holds at most {MOST} steps"));
        }
        if let Some(bad) = args
            .todos
            .iter()
            .find(|s| !STATUSES.contains(&s.status.as_str()))
        {
            return Outcome::error(format!(
                "`{}` is no status: pending, in_progress, completed or cancelled",
                bad.status
            ));
        }
        let mut steps = args.todos;
        let mut said = Vec::new();
        for step in &mut steps {
            let Some(command) = step.check.clone().filter(|c| !c.trim().is_empty()) else {
                continue;
            };
            if step.status != "completed" {
                continue;
            }
            let key = (step.content.clone(), command.clone());
            if self.held.contains(&key) {
                continue;
            }
            let args = json!({"command": command, "timeout_ms": EACH_MS});
            let o = call(tools, "bash", &args.to_string(), cx);
            self.ran += 1;
            if o.is_error {
                self.failed += 1;
                "in_progress".clone_into(&mut step.status);
                said.push(format!(
                    "✘ {}: `{command}` failed — {}. It stays in_progress.",
                    step.content,
                    tail(&o.content)
                ));
            } else {
                self.held.push(key);
                said.push(format!("✔ {}: `{command}` passed.", step.content));
            }
        }
        self.steps = steps;
        let mut out = self.summary();
        for s in said {
            out.push('\n');
            out.push_str(&s);
        }
        Outcome::ok(out, ToolClass::Read)
    }

    /// `Todos: 2 completed (1 checked), 1 in progress, 1 pending`.
    #[must_use]
    pub fn summary(&self) -> String {
        let count = |status: &str| self.steps.iter().filter(|s| s.status == status).count();
        let checked = self
            .steps
            .iter()
            .filter(|s| {
                s.status == "completed"
                    && s.check
                        .as_ref()
                        .is_some_and(|c| self.held.contains(&(s.content.clone(), c.clone())))
            })
            .count();
        let mut parts = Vec::new();
        let done = count("completed");
        if done > 0 {
            parts.push(if checked > 0 {
                format!("{done} completed ({checked} checked)")
            } else {
                format!("{done} completed")
            });
        }
        for (status, word) in [
            ("in_progress", "in progress"),
            ("pending", "pending"),
            ("cancelled", "cancelled"),
        ] {
            let n = count(status);
            if n > 0 {
                parts.push(format!("{n} {word}"));
            }
        }
        if parts.is_empty() {
            "Todos cleared".to_owned()
        } else {
            format!("Todos: {}", parts.join(", "))
        }
    }
}

/// The end of what a command said, on one line, cut short.
fn tail(text: &str) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let n = one.chars().count();
    if n <= SHOWN {
        one
    } else {
        format!("…{}", one.chars().skip(n - SHOWN).collect::<String>())
    }
}

/// `system` with the todo line ([`HINT`]) before the prompt's last bullet of how it works.
#[must_use]
pub fn with_todo(system: &str) -> String {
    let anchor = "- If the ask is ambiguous";
    let mut out = Vec::new();
    let mut said = false;
    for line in system.split('\n') {
        if !said && line.starts_with(anchor) {
            out.push(HINT);
            said = true;
        }
        out.push(line);
    }
    if !said {
        out.push(HINT);
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cx(dir: &std::path::Path) -> Context {
        // The test's own shell, unguarded: what is held is the list, not the sandbox.
        Context::new(mikata_races::Race::Slime, dir, &["."]).with_unguarded(true)
    }

    #[test]
    fn a_completed_step_with_a_check_is_run_and_one_that_fails_stays_in_progress() {
        let dir = tempfile::tempdir().unwrap();
        let tools = mikata_tools::registry();
        let mut cx = cx(dir.path());
        let mut todos = Todos::default();
        let o = todos.update(
            &json!({"todos": [
                {"content": "true holds", "status": "completed", "check": "true"},
                {"content": "false fails", "status": "completed", "check": "false"},
                {"content": "next", "status": "pending"}
            ]})
            .to_string(),
            &tools,
            &mut cx,
        );
        assert!(!o.is_error, "{}", o.content);
        assert!(o.content.contains("✔ true holds"), "{}", o.content);
        assert!(o.content.contains("✘ false fails"), "{}", o.content);
        assert_eq!(todos.steps()[1].status, "in_progress");
        assert!(
            o.content
                .starts_with("Todos: 1 completed (1 checked), 1 in progress, 1 pending"),
            "{}",
            o.content
        );
        // Sent again, a check that held is not run again.
        let ran = todos.ran;
        todos.update(
            &json!({"todos": [{"content": "true holds", "status": "completed", "check": "true"}]})
                .to_string(),
            &tools,
            &mut cx,
        );
        assert_eq!(todos.ran, ran);
        assert_eq!((todos.ran, todos.failed), (2, 1));
    }

    #[test]
    fn a_list_that_does_not_read_is_refused_and_nothing_runs() {
        let dir = tempfile::tempdir().unwrap();
        let tools = mikata_tools::registry();
        let mut todos = Todos::default();
        let o = todos.update(
            r#"{"todos":[{"content":"x","status":"done"}]}"#,
            &tools,
            &mut cx(dir.path()),
        );
        assert!(
            o.is_error && o.content.contains("no status"),
            "{}",
            o.content
        );
        assert_eq!(todos.ran, 0);
    }

    #[test]
    fn the_hint_goes_before_the_last_bullet_once() {
        let system = "How you work:\n- Read.\n- If the ask is ambiguous, ask back.";
        let with = with_todo(system);
        assert_eq!(with.matches(HINT).count(), 1);
        assert!(with.find(HINT) < with.find("- If the ask is ambiguous"));
    }
}
