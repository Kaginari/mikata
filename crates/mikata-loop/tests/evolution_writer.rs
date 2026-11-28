//! An evolution end to end: a Mind change swapped in writes its swap row, every ask of its body
//! after it writes a run row, the do-nothing controls run on demand on the before state, and the
//! judgement — keep or rollback — replays from the chained ledger. The window's tasks carry the
//! issue's do-nothing fixtures: a review where no finding is right, and an ask where no recall hit
//! is right.

#![allow(
    clippy::unnecessary_wraps,
    reason = "scripted replies are Results: the script also holds errors"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mikata_config::{Config, Paths};
use mikata_ledger::Journal;
use mikata_ledger::evolution::{self, EVOLUTION_WINDOW, EvolutionRow, Gate, Judgement};
use mikata_loop::evolution::{Controller, Reach, reach, run_controls};
use mikata_loop::{Beat, EndGate, Limits, Model, Run, ScriptedModel, Setup, solo};
use mikata_provider::{Caller, Error, Response, anthropic_prices};
use mikata_races::Race;
use mikata_tools::{Context, Sandbox, registry};
use mikata_world::evolution::{Arm, Facts, Plan, Staged, stamp_for, swap_at};
use serde_json::json;

const MIND: &str = ".mikata/minds/careful/README.md";
const OLD: &str = "# Careful\n\nRead before writing.\n";
const NEW: &str = "# Careful\n\nRead before writing. File a finding only with a witness.\n";
const DOC: &str = ".mikata/summoner/summoner/README.md";
const NOTE: &str = "The deploy pipeline needs DOCKER_HOST exported before compose runs";

/// The kinds of task in a window, each with the right output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    /// A fix the gate checks: `out-<i>.txt` must read `fixed`.
    Fix,
    /// Nothing to change: no change is the right output.
    Nothing,
    /// A review of a sound file: no finding is the right output.
    NoFinding,
    /// An ask whose words no note in memory holds: no recall hit is the right output.
    NoRecall,
}

fn kind(i: usize) -> Kind {
    [Kind::Fix, Kind::Nothing, Kind::NoFinding, Kind::NoRecall][i % 4]
}

fn ask(i: usize) -> String {
    match kind(i) {
        Kind::Fix => format!("Make out-{i}.txt say fixed."),
        Kind::Nothing => format!("Check that task {i} needs no change."),
        Kind::NoFinding => {
            format!("Review sound-{i}.rs. Write any finding, with its witness, to findings-{i}.md.")
        }
        Kind::NoRecall => format!("Tell me whether sound-{i}.rs compiles as it stands."),
    }
}

/// The right output for task `i`, or — `wrong` — what a body that always does something does.
fn script(i: usize, wrong: bool) -> Vec<Result<Response, Error>> {
    let write = |path: String, content: &str| {
        ScriptedModel::tool("t1", "write", json!({"path": path, "content": content}))
    };
    match (kind(i), wrong) {
        (Kind::Fix, false) => vec![
            write(format!("out-{i}.txt"), "fixed\n"),
            ScriptedModel::text("wrote it"),
        ],
        (Kind::Fix, true) => vec![
            write(format!("out-{i}.txt"), "broken\n"),
            ScriptedModel::text("done"),
            ScriptedModel::text("still done"),
        ],
        (Kind::NoFinding, true) => vec![
            write(format!("findings-{i}.md"), "maybe an overflow\n"),
            ScriptedModel::text("filed one"),
            ScriptedModel::text("still filed"),
        ],
        (Kind::Nothing | Kind::NoFinding | Kind::NoRecall, _) => {
            vec![ScriptedModel::text("nothing to do: it already holds")]
        }
    }
}

/// A world holding the Mind as it was, the throne's doc with its verify lines, sound files to
/// review, and one note memory holds about something else.
struct World {
    root: PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "mikata-evolution-writer-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let w = root.join("world");
        for (path, text) in [
            (MIND, OLD),
            (
                DOC,
                "# summoner\n\n- **Race:** summoner\n- **Minds:** careful\n\
                 - **Verify:** `! grep -qs broken out-*.txt`\n\
                 - **Verify:** `! ls findings-*.md`\n",
            ),
        ] {
            fs::create_dir_all(w.join(path).parent().unwrap()).unwrap();
            fs::write(w.join(path), text).unwrap();
        }
        for i in 0..EVOLUTION_WINDOW {
            fs::write(
                w.join(format!("sound-{i}.rs")),
                "pub fn add(a: u8, b: u8) -> Option<u8> { a.checked_add(b) }\n",
            )
            .unwrap();
        }
        mikata_memory::propose(&w, "summoner", NOTE, Vec::new(), None).unwrap();
        pin(
            &w,
            "summoner",
            &["! grep -qs broken out-*.txt", "! ls findings-*.md"],
        );
        fs::write(root.join("global.toml"), "").unwrap();
        Self { root }
    }

    fn world(&self) -> PathBuf {
        self.root.join("world")
    }

    fn ledger(&self) -> Journal<EvolutionRow> {
        Journal::chained(self.root.join("state/evolution.jsonl"))
    }

    fn usage(&self) -> Journal<mikata_ledger::UsageRow> {
        Journal::chained(self.world().join(".mikata/instruments/usage.jsonl"))
    }

    fn config(&self) -> Config {
        Config::load(&Paths::with_global(
            self.root.join("global.toml"),
            &self.world(),
        ))
        .unwrap()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Fixtures run unguarded, with the attic beside the world.
fn ready(cx: Context, root: &Path) -> Context {
    let world = cx.world().to_path_buf();
    let cx = cx
        .with_sandbox(Sandbox::None("fixtures run unguarded".into()))
        .with_unguarded(true);
    match mikata_tools::Attic::at(&root.join("attic"), &world) {
        Some(a) => cx.with_attic(a),
        None => cx,
    }
}

/// One ask of the throne in `w`, as a session runs it: its gate the doc's verify lines, its asks
/// stamped on the world's ledger. Returns its journal.
async fn ask_once(w: &World, i: usize, replies: Vec<Result<Response, Error>>) -> Journal<Beat> {
    ask_as(w, i, replies, "claude-sonnet-5-5").await
}

/// [`ask_once`], the session riding `model`: the one the human chose as it opened.
async fn ask_as(
    w: &World,
    i: usize,
    replies: Vec<Result<Response, Error>>,
    model: &str,
) -> Journal<Beat> {
    let world = w.world();
    let journal: Journal<Beat> = Journal::new(w.root.join(format!("run-{i}.jsonl")));
    let (body, cx) = solo(&world, model);
    let roster = mikata_world::Roster::load(&world);
    let setup = Setup {
        model: Arc::new(ScriptedModel::new(replies)),
        tools: registry(),
        cx: ready(cx, &w.root),
        journal: journal.clone(),
        ledger: None,
        prices: anthropic_prices().unwrap(),
        caller: Caller {
            session: "s".into(),
            run: format!("r{i}"),
            body: "summoner".into(),
            race: "summoner".into(),
            commission: None,
            prefix: 0,
        },
    };
    let mut run = Run::new(body, Limits::for_race(Race::Summoner), setup)
        .with_gate(EndGate {
            verify: roster.creatures["summoner"].verify.clone(),
            held: Vec::new(),
            by: Vec::new(),
            author: None,
            ..Default::default()
        })
        .with_evolution_ledger(w.ledger().path().to_path_buf());
    let mut quiet = |_| {};
    run.ask(&ask(i), &mut quiet).await;
    journal
}

fn swap(w: &World) {
    swap_at(
        &w.world(),
        &w.ledger(),
        &Plan {
            id: "careful-1".into(),
            creature: "summoner".into(),
            bodies: vec!["summoner".into()],
            arm: Arm::Mutate,
            files: vec![Staged {
                path: MIND.into(),
                after: Some(NEW.as_bytes().to_vec()),
                creates: false,
            }],
            proven_on: Vec::new(),
        },
    )
    .unwrap();
}

fn runs(w: &World) -> Vec<evolution::Run> {
    w.ledger()
        .read()
        .unwrap()
        .rows
        .into_iter()
        .filter_map(|r| match r {
            EvolutionRow::Run(r) => Some(r),
            EvolutionRow::Swap(_) | EvolutionRow::Settle(_) => None,
        })
        .collect()
}

/// The window's runs, then its controls run on demand by a body that always does something: the
/// evolution beats it on the fixes and the reviews where no finding is right, so it is kept; the
/// judgement replays from the ledger.
#[tokio::test]
async fn a_swapped_mind_is_stamped_controlled_and_judged() {
    let w = World::new("keep");
    swap(&w);
    for i in 0..EVOLUTION_WINDOW {
        let journal = ask_once(&w, i, script(i, false)).await;
        let beats = journal.read().unwrap().rows;
        if kind(i) == Kind::NoRecall {
            assert!(
                !beats.iter().any(|b| matches!(b, Beat::Recall { .. })),
                "task {i}: no note holds its words, so nothing is recalled"
            );
        }
    }
    let window = runs(&w);
    assert_eq!(
        window.len(),
        EVOLUTION_WINDOW,
        "every ask after the swap wrote its row"
    );
    assert!(window.iter().all(|r| !r.control
        && r.swap.as_deref() == Some("careful-1")
        && r.gate == Some(Gate::Pass)));
    // The controls: the same asks, by the same body, on the before state, in scratch worlds.
    let replies: Vec<_> = (0..EVOLUTION_WINDOW)
        .flat_map(|i| script(i, true))
        .collect();
    let config = w.config();
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let say = |l: &str| said.borrow_mut().push(l.to_owned());
    let attic = w.root.join("attic");
    let scripted: Arc<dyn Model> = Arc::new(ScriptedModel::new(replies));
    let model = |_: &str, _: &str| -> Result<Arc<dyn Model>, String> { Ok(Arc::clone(&scripted)) };
    let c = Controller {
        model: &model,
        config: &config,
        ready,
        attic: Some(&attic),
        usage: w.usage(),
        session: "judge-1".into(),
        say: &say,
    };
    let lines = run_controls(&w.world(), w.ledger().path(), "careful-1", &c)
        .await
        .unwrap();
    assert_eq!(lines.len(), EVOLUTION_WINDOW, "{lines:?}");
    assert!(
        said.borrow()[0].starts_with("10 control(s) to run through claude-sonnet-5-5"),
        "said before any is spent: {:?}",
        said.borrow()
    );
    // Every control's call is on the live world's usage ledger, under the judge's session.
    let spent = w.usage().read().unwrap().rows;
    assert_ne!(spent.len(), 0, "the controls spent on the live ledger");
    assert!(spent.iter().all(|r| r.session == "judge-1"));
    assert_eq!(
        fs::read_to_string(w.world().join(MIND)).unwrap(),
        NEW,
        "the controls never touch the world"
    );
    let controls: Vec<_> = runs(&w).into_iter().filter(|r| r.control).collect();
    assert_eq!(controls.len(), EVOLUTION_WINDOW);
    for (i, c) in controls.iter().enumerate() {
        let want = match kind(i) {
            Kind::Fix | Kind::NoFinding => Gate::Fail,
            Kind::Nothing | Kind::NoRecall => Gate::Pass,
        };
        assert_eq!(c.gate, Some(want), "control {i}");
    }
    // Run again, nothing waits: no model is called.
    let rerun = run_controls(&w.world(), w.ledger().path(), "careful-1", &c)
        .await
        .unwrap();
    assert_eq!(
        rerun,
        Vec::<String>::new(),
        "nothing waits: no model is called"
    );
    let head = w.ledger().head().unwrap();
    let read = evolution::read(w.ledger().path(), head.as_ref()).unwrap();
    let judged = evolution::judge(&read, "careful-1", mikata_config::MIN_NET_WINS);
    let Judgement::Keep(lines) = &judged else {
        panic!("{judged}");
    };
    // Three fixes and two reviews won, nothing lost.
    assert_eq!(lines[0].meter, "netWins");
    assert!((lines[0].reading - 5.0).abs() < 1e-9, "{judged}");
    let to = lines[0].to.unwrap();
    let again = evolution::replay(
        w.ledger().path(),
        "careful-1",
        to,
        head.as_ref(),
        mikata_config::MIN_NET_WINS,
    )
    .unwrap();
    assert_eq!(
        again, judged,
        "the ledger read again gives the same judgement"
    );
}

/// A control that does nothing faces the task the evolved run faced: it starts from the tree that
/// run started from. Task 0 starts broken and its run fixes it, so its control finds the file still
/// broken and does not land; on the finished world it found it fixed and "landed". (The other fixes
/// start from no file at all, which the throne's check, "nothing reads broken", passes either way:
/// a check that a do-nothing run passes cannot tell the two worlds apart.)
#[tokio::test]
async fn a_do_nothing_control_starts_where_the_run_started() {
    let w = World::new("fair-control");
    swap(&w);
    fs::write(w.world().join("out-0.txt"), "broken\n").unwrap();
    for i in 0..EVOLUTION_WINDOW {
        ask_once(&w, i, script(i, false)).await;
    }
    assert!(
        runs(&w).iter().all(|r| r.start.is_some()),
        "every window run kept its start"
    );
    let replies: Vec<_> = (0..EVOLUTION_WINDOW)
        .map(|_| ScriptedModel::text("nothing to do: it already holds"))
        .collect();
    let config = w.config();
    let say = |_: &str| {};
    let attic = w.root.join("attic");
    let scripted: Arc<dyn Model> = Arc::new(ScriptedModel::new(replies));
    let model = |_: &str, _: &str| -> Result<Arc<dyn Model>, String> { Ok(Arc::clone(&scripted)) };
    let c = Controller {
        model: &model,
        config: &config,
        ready,
        attic: Some(&attic),
        usage: w.usage(),
        session: "judge-fair".into(),
        say: &say,
    };
    run_controls(&w.world(), w.ledger().path(), "careful-1", &c)
        .await
        .unwrap();
    let controls: Vec<_> = runs(&w).into_iter().filter(|r| r.control).collect();
    assert_eq!(controls.len(), EVOLUTION_WINDOW);
    for (i, c) in controls.iter().enumerate() {
        let want = if i == 0 { Gate::Fail } else { Gate::Pass };
        assert_eq!(c.gate, Some(want), "control {i} ({:?})", kind(i));
    }
}

/// The fixtures where doing nothing is right, run with the right output and with a body that
/// always does something: the same gate word every time, and only the right output lands.
#[tokio::test]
async fn the_do_nothing_fixtures_are_deterministic() {
    let w = World::new("fixtures");
    for i in (0..EVOLUTION_WINDOW).filter(|&i| matches!(kind(i), Kind::NoFinding | Kind::NoRecall))
    {
        for wrong in [false, true] {
            let mut words = Vec::new();
            for round in 0..2 {
                let at = World::new(&format!("fixture-{i}-{wrong}-{round}"));
                let beats = ask_once(&at, i, script(i, wrong))
                    .await
                    .read()
                    .unwrap()
                    .rows;
                let gate = beats.iter().rev().find_map(|b| match b {
                    Beat::Gate { verdict, .. } => Some(verdict.clone()),
                    _ => None,
                });
                assert!(
                    !beats.iter().any(|b| matches!(b, Beat::Recall { .. })),
                    "{i}: no recall hit"
                );
                words.push(gate);
            }
            assert_eq!(words[0], words[1], "task {i} ({wrong}) is deterministic");
            let landed = words[0].as_deref().is_none_or(|v| v == "PASS");
            let want = !(wrong && kind(i) == Kind::NoFinding);
            assert_eq!(
                landed, want,
                "task {i}, always-something {wrong}: {words:?}"
            );
        }
    }
    // The no-recall fixture is not empty by accident: an ask in the note's own words recalls it.
    let journal: Journal<Beat> = Journal::new(w.root.join("hit.jsonl"));
    let (body, cx) = solo(&w.world(), "claude-sonnet-5-5");
    let setup = Setup {
        model: Arc::new(ScriptedModel::new([ScriptedModel::text("noted")])),
        tools: registry(),
        cx: ready(cx, &w.root),
        journal: journal.clone(),
        ledger: None,
        prices: anthropic_prices().unwrap(),
        caller: Caller {
            session: "s".into(),
            run: "hit".into(),
            body: "summoner".into(),
            race: "summoner".into(),
            commission: None,
            prefix: 0,
        },
    };
    let mut run = Run::new(body, Limits::for_race(Race::Summoner), setup);
    let mut quiet = |_| {};
    run.ask(
        "What does the deploy pipeline need exported before compose runs?",
        &mut quiet,
    )
    .await;
    assert!(
        journal
            .read()
            .unwrap()
            .rows
            .iter()
            .any(|b| matches!(b, Beat::Recall { anchors, .. } if !anchors.is_empty())),
        "the note is recalled by its own words"
    );
}

/// A spent budget cap runs no control: the judge says so, and nothing is written.
#[tokio::test]
async fn a_spent_cap_runs_no_control() {
    let w = World::new("capped");
    swap(&w);
    for i in 0..EVOLUTION_WINDOW {
        ask_once(&w, i, script(i, false)).await;
    }
    fs::write(w.root.join("global.toml"), "[budget]\nday = 10\n").unwrap();
    w.usage()
        .append(&mikata_ledger::UsageRow {
            ts: mikata_ledger::Timestamp::now(),
            session: "earlier".into(),
            run: "r".into(),
            body: "summoner".into(),
            race: "summoner".into(),
            commission: None,
            model: "m".into(),
            provider: "p".into(),
            usage: mikata_ledger::Usage::default(),
            prefix: 0,
            wt: 50,
            usd: None,
            reported_cost: None,
            upstream: None,
            message_id: None,
            estimated: false,
            envelope: None,
            purpose: None,
        })
        .unwrap();
    let config = w.config();
    let attic = w.root.join("attic");
    let scripted: Arc<dyn Model> = Arc::new(ScriptedModel::new([]));
    let model = |_: &str, _: &str| -> Result<Arc<dyn Model>, String> { Ok(Arc::clone(&scripted)) };
    let c = Controller {
        model: &model,
        config: &config,
        ready,
        attic: Some(&attic),
        usage: w.usage(),
        session: "judge-2".into(),
        say: &|_| {},
    };
    let e = run_controls(&w.world(), w.ledger().path(), "careful-1", &c)
        .await
        .unwrap_err();
    assert!(e.contains("cap is spent"), "{e}");
    assert!(runs(&w).iter().all(|r| !r.control), "no control written");
}

/// A window of two models, as when the human chooses another for a session: each control rides
/// the model its own run rode, reached once, whatever the configuration names by then. Every pair
/// shares its model, so the swap is judged on the change alone.
#[tokio::test]
async fn each_control_rides_the_model_its_run_rode() {
    let w = World::new("two-models");
    swap(&w);
    let rode = |i: usize| {
        if i.is_multiple_of(2) {
            "claude-sonnet-5-5"
        } else {
            "claude-haiku-5-5"
        }
    };
    for i in 0..EVOLUTION_WINDOW {
        ask_as(&w, i, script(i, false), rode(i)).await;
    }
    let replies: Vec<_> = (0..EVOLUTION_WINDOW)
        .flat_map(|i| script(i, true))
        .collect();
    let config = w.config();
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let say = |l: &str| said.borrow_mut().push(l.to_owned());
    let attic = w.root.join("attic");
    let scripted = Arc::new(ScriptedModel::new(replies));
    let asked = std::cell::RefCell::new(Vec::<(String, String)>::new());
    let model = |provider: &str, id: &str| -> Result<Arc<dyn Model>, String> {
        asked
            .borrow_mut()
            .push((provider.to_owned(), id.to_owned()));
        let reached: Arc<dyn Model> = scripted.clone();
        Ok(reached)
    };
    let c = Controller {
        model: &model,
        config: &config,
        ready,
        attic: Some(&attic),
        usage: w.usage(),
        session: "judge-two-models".into(),
        say: &say,
    };
    let lines = run_controls(&w.world(), w.ledger().path(), "careful-1", &c)
        .await
        .unwrap();
    assert_eq!(lines.len(), EVOLUTION_WINDOW, "{lines:?}");
    assert_eq!(
        *asked.borrow(),
        [
            ("scripted".to_owned(), "claude-sonnet-5-5".to_owned()),
            ("scripted".to_owned(), "claude-haiku-5-5".to_owned()),
        ],
        "each model the window rode is reached once, by the names its rows carry"
    );
    assert!(
        said.borrow()[0]
            .starts_with("10 control(s) to run through claude-sonnet-5-5, claude-haiku-5-5"),
        "{:?}",
        said.borrow()
    );
    // Every call of a control named the model its own run rode.
    let named: Vec<String> = scripted.requests().into_iter().map(|r| r.model).collect();
    let expected: Vec<&str> = (0..EVOLUTION_WINDOW)
        .flat_map(|i| std::iter::repeat_n(rode(i), script(i, true).len()))
        .collect();
    assert_eq!(named, expected);
    // And every row says what it rode: a run and its control alike.
    let (window, controls): (Vec<_>, Vec<_>) = runs(&w).into_iter().partition(|r| !r.control);
    assert_eq!(
        (window.len(), controls.len()),
        (EVOLUTION_WINDOW, EVOLUTION_WINDOW)
    );
    for (i, (run, control)) in window.iter().zip(&controls).enumerate() {
        assert_eq!(run.task, control.task, "pair {i}");
        for row in [run, control] {
            assert_eq!(
                (row.model.as_str(), row.provider.as_str()),
                (rode(i), "scripted"),
                "pair {i}, control {}",
                row.control
            );
        }
    }
    let head = w.ledger().head().unwrap();
    let read = evolution::read(w.ledger().path(), head.as_ref()).unwrap();
    let judged = evolution::judge(&read, "careful-1", mikata_config::MIN_NET_WINS);
    let Judgement::Keep(lines) = &judged else {
        panic!("{judged}");
    };
    assert!((lines[0].reading - 5.0).abs() < 1e-9, "{judged}");
}

/// A model that answers to another provider's name, and is never called.
struct Elsewhere;

impl Model for Elsewhere {
    fn provider(&self) -> &'static str {
        "elsewhere"
    }

    fn complete<'a>(
        &'a self,
        _: &'a mikata_provider::Request,
        _: &'a mut (dyn FnMut(mikata_provider::Event) + Send),
    ) -> mikata_loop::model::Completion<'a> {
        Box::pin(async { Err(Error::Invalid("a control never rides it".into())) })
    }
}

/// A window run whose model cannot be reached again — its key not set, or the same id through
/// another provider — gets no control, and neither does a run that kept no model: nothing is
/// said, spent or written, the judgement waits, and the human is told to judge the window.
#[tokio::test]
async fn a_run_whose_model_is_out_of_reach_gets_no_control() {
    let w = World::new("out-of-reach");
    swap(&w);
    for i in 0..EVOLUTION_WINDOW {
        ask_once(&w, i, script(i, false)).await;
    }
    let config = w.config();
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let say = |l: &str| said.borrow_mut().push(l.to_owned());
    let attic = w.root.join("attic");
    let unkeyed = |_: &str, _: &str| -> Result<Arc<dyn Model>, String> {
        Err("ANTHROPIC_API_KEY is not set".into())
    };
    let moved = |_: &str, _: &str| -> Result<Arc<dyn Model>, String> { Ok(Arc::new(Elsewhere)) };
    let cases: [(Reach<'_>, &str); 2] = [
        (&unkeyed, "ANTHROPIC_API_KEY is not set"),
        (&moved, "it is reached through elsewhere now"),
    ];
    for (model, why) in cases {
        let c = Controller {
            model,
            config: &config,
            ready,
            attic: Some(&attic),
            usage: w.usage(),
            session: "judge-out-of-reach".into(),
            say: &say,
        };
        let e = run_controls(&w.world(), w.ledger().path(), "careful-1", &c)
            .await
            .unwrap_err();
        assert!(
            e.contains("its window run rode claude-sonnet-5-5 through scripted")
                && e.contains(why)
                && e.contains("judge this window by hand"),
            "{e}"
        );
    }
    assert!(
        said.borrow().is_empty(),
        "no spend is announced: {:?}",
        said.borrow()
    );
    assert!(runs(&w).iter().all(|r| !r.control), "no control written");
    assert!(
        w.usage().read().map_or(true, |r| r.rows.is_empty()),
        "nothing spent"
    );
    let head = w.ledger().head().unwrap();
    let read = evolution::read(w.ledger().path(), head.as_ref()).unwrap();
    assert_eq!(
        evolution::judge(&read, "careful-1", mikata_config::MIN_NET_WINS),
        Judgement::Pending {
            runs: EVOLUTION_WINDOW,
            controls: 0
        }
    );
    // A run whose runner said no model: no model is even asked for.
    let old = World::new("no-model");
    swap(&old);
    stamp_for(&old.world(), old.ledger().path(), "summoner", &ask(0))
        .unwrap()
        .record(&old.world(), Facts::default())
        .unwrap();
    let never = |_: &str, _: &str| -> Result<Arc<dyn Model>, String> {
        panic!("a run that named no model asks for none")
    };
    let c = Controller {
        model: &never,
        config: &config,
        ready,
        attic: Some(&attic),
        usage: old.usage(),
        session: "judge-no-model".into(),
        say: &say,
    };
    let e = run_controls(&old.world(), old.ledger().path(), "careful-1", &c)
        .await
        .unwrap_err();
    assert!(
        e.contains("its window run kept no model") && e.contains("judge this window by hand"),
        "{e}"
    );
}

/// The model a window run rode is reached again by the names its row carries: the
/// configuration's own while it is still that one, another provider's when the run rode that, an
/// id that holds a colon whole. One the configuration no longer reaches is refused, and so is one
/// whose key is not set.
#[test]
fn a_runs_model_is_reached_by_the_names_its_row_carries() {
    let w = World::new("reach");
    fs::write(
        w.root.join("global.toml"),
        "[providers.openrouter]\nkind = \"openai\"\nbase_url = \"https://openrouter.invalid/api\"\n\
         key_env = \"OPENROUTER_API_KEY\"\n\n\
         [providers.vllm]\nkind = \"openai\"\nbase_url = \"http://127.0.0.1:9/v1\"\n\
         key_env = \"VLLM_API_KEY\"\n\n\
         [models]\nprovider = \"openrouter\"\ndefault = \"qwen/qwen3.5-35b\"\n",
    )
    .unwrap();
    let config = w.config();
    let keys = |k: &str| matches!(k, "OPENROUTER_API_KEY" | "VLLM_API_KEY").then(|| "k".to_owned());
    for (provider, id) in [
        ("openrouter", "qwen/qwen3.5-35b"),
        ("vllm", "qwen3-coder"),
        ("openrouter", "qwen/qwen3.5-35b:free"),
    ] {
        let model = reach(&config, &w.world(), &keys, provider, id)
            .unwrap_or_else(|e| panic!("{id} through {provider}: {e}"));
        assert_eq!(model.provider(), provider, "{id}");
    }
    let gone = reach(&config, &w.world(), &keys, "gone", "some-model")
        .err()
        .expect("no provider of that name");
    assert!(
        gone.contains("reaches no model `some-model` through `gone`"),
        "{gone}"
    );
    let only = |k: &str| (k == "OPENROUTER_API_KEY").then(|| "k".to_owned());
    let unkeyed = reach(&config, &w.world(), &only, "vllm", "qwen3-coder")
        .err()
        .expect("its key is not set");
    assert!(unkeyed.contains("VLLM_API_KEY"), "{unkeyed}");
}

/// `creature`'s verify lines pinned in `world`, as the human's `mikatad verify pin` would pin
/// them: the end gate runs only pinned lines.
fn pin(world: &std::path::Path, creature: &str, lines: &[&str]) {
    use mikata_world::pins;
    let lines: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
    pins::record(
        world,
        pins::row(pins::Act::Pin, creature, &lines, pins::HUMAN),
    )
    .unwrap();
}
