//! An evolution's do-nothing controls, run on demand, and its judgement.
//!
//! No scheduler runs the controls beside the window: the human asks for them with
//! `mikatad evolution judge <swap>`. Each window run still waiting on its control is run again —
//! the same ask, by the same body, under the same build, through the model its window run rode
//! and not whichever the configuration names by then — in a scratch copy of the world whose
//! named files are put back as they were before the swap (the bytes kept beside the ledger), and
//! appended as a control row stamped with the before state it found and left. Then the swap is
//! judged ([`mikata_ledger::evolution`]). Keep or rollback is only proposed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mikata_config::{Config, Paths};
use mikata_ledger::evolution::state_now;
use mikata_ledger::evolution::{self as ledger, Judgement};
use mikata_ledger::{Build, Journal};
use mikata_provider::{Caller, anthropic_prices};
use mikata_races::Race;
use mikata_tools::{Context, NoHuman, registry};
use mikata_world::Roster;
use mikata_world::evolution::{Facts, Stamp, fingerprint, restore_before};

use crate::beats::Beat;
use crate::gate::{Author, EndGate};
use crate::model::Model;
use crate::run::{Ending, Limits, Run, Setup};
use crate::solo::{configure, solo};

/// How the model a window run rode is reached again for its control: given the provider's name
/// and the model's id that run was stamped with, the model — or why the configuration no longer
/// reaches it.
pub type Reach<'a> = &'a dyn Fn(&str, &str) -> Result<Arc<dyn Model>, String>;

/// What runs the controls: how the model a window run rode is reached, the configuration, the
/// live world's usage ledger and the session the controls' spend is counted under, how a
/// control's tools' context is made ready over its scratch world (the sandbox), and where a line
/// is said before anything is spent.
pub struct Controller<'a> {
    /// The live world's usage ledger: every control's call is written there and counted against
    /// the human's caps.
    pub usage: Journal<mikata_ledger::UsageRow>,
    /// The session the controls' spend is counted under.
    pub session: String,
    /// Says a line to the human as soon as it is known.
    pub say: &'a dyn Fn(&str),
    /// Reaches the model a control rides: the one its window run rode.
    pub model: Reach<'a>,
    /// The world's configuration.
    pub config: &'a Config,
    /// The context made ready over the scratch world, given its root.
    pub ready: fn(Context, &Path) -> Context,
    /// The attic the window's runs kept their starts in: the world's own when `None`.
    pub attic: Option<&'a Path>,
}

/// What a control rides: the model its window run rode, with the names that run's row carries.
struct Riding<'a> {
    /// The provider it is reached through, by the name the usage ledger writes.
    provider: &'a str,
    /// The id it answers to.
    id: &'a str,
    /// The model.
    model: Arc<dyn Model>,
}

/// Every model the runs in `waiting` rode, each reached once, before anything is said or spent.
///
/// # Errors
///
/// A run kept no model, its model cannot be reached again, or it is reached through another
/// provider than the run's.
fn reach_all<'r>(
    waiting: &[&'r ledger::Run],
    c: &Controller<'_>,
) -> Result<Vec<Riding<'r>>, String> {
    let mut reached: Vec<Riding<'r>> = Vec::new();
    for w in waiting {
        if reached
            .iter()
            .any(|r| r.provider == w.provider && r.id == w.model)
        {
            continue;
        }
        if w.model == mikata_ledger::UNKNOWN || w.provider == mikata_ledger::UNKNOWN {
            return Err(format!(
                "task {}: its window run kept no model, so its control cannot ride the same one; \
                 judge this window by hand",
                w.task
            ));
        }
        let model = (c.model)(&w.provider, &w.model).map_err(|e| {
            format!(
                "task {}: its window run rode {} through {}, and no control can ride the same \
                 ({e}); judge this window by hand",
                w.task, w.model, w.provider
            )
        })?;
        // A control's row names what it rode. Reached through another provider it would be no
        // control of its run, and the run would wait, and spend, at every judging.
        if model.provider() != w.provider {
            return Err(format!(
                "task {}: its window run rode {} through {}, and it is reached through {} now; \
                 judge this window by hand",
                w.task,
                w.model,
                w.provider,
                model.provider()
            ));
        }
        reached.push(Riding {
            provider: &w.provider,
            id: &w.model,
            model,
        });
    }
    Ok(reached)
}

/// One control's task: the ask its window run was given, by which body, from which tree, and
/// what it rides.
struct Task<'a> {
    /// The body that ran the window run, and runs its control.
    body: &'a str,
    /// The ask, as the window run was given it.
    ask: &'a str,
    /// The attic's tree the window run started from.
    start: &'a str,
    /// What the control rides.
    riding: &'a Riding<'a>,
}

/// The directories a scratch copy of the world leaves out: build output and the instruments.
const UNCOPIED: [&str; 2] = ["target", "node_modules"];

/// Copies `from` into `to`: links as links, never followed; build output, the instruments and the
/// scratch left behind.
pub(crate) fn copy_world(from: &Path, to: &Path, top: bool) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for e in fs::read_dir(from)? {
        let e = e?;
        let name = e.file_name();
        let name_s = name.to_string_lossy();
        if top && UNCOPIED.contains(&name_s.as_ref()) {
            continue;
        }
        let kind = e.file_type()?;
        let dest = to.join(&name);
        if kind.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(fs::read_link(e.path())?, &dest)?;
        } else if kind.is_dir() {
            if from.file_name().is_some_and(|n| n == ".mikata")
                && matches!(name_s.as_ref(), "instruments" | "tmp")
            {
                continue;
            }
            copy_world(&e.path(), &dest, false)?;
        } else {
            fs::copy(e.path(), &dest)?;
        }
    }
    Ok(())
}

/// The gate's facts from a run's journal: its last verdict, a pass on the first try, and the
/// holes it found (*tests intact*).
/// Whether the run's last gate saw no change, or the run met no gate at all.
fn last_gate_unchanged(journal: &Journal<Beat>) -> bool {
    let rows = journal.read().map(|r| r.rows).unwrap_or_default();
    rows.iter()
        .rev()
        .find_map(|b| match b {
            Beat::Gate { changed, .. } => Some(changed.is_empty()),
            _ => None,
        })
        .unwrap_or(true)
}

fn facts_of(journal: &Journal<Beat>) -> Facts {
    use mikata_ledger::evolution::Gate;
    let rows = journal.read().map(|r| r.rows).unwrap_or_default();
    let mut verdicts = Vec::new();
    let mut holes = 0u32;
    for b in rows {
        if let Beat::Gate {
            failures, verdict, ..
        } = b
        {
            holes = holes.saturating_add(
                u32::try_from(
                    failures
                        .iter()
                        .filter(|f| f.starts_with("tests intact:"))
                        .count(),
                )
                .unwrap_or(u32::MAX),
            );
            verdicts.push(match verdict.as_str() {
                "PASS" => Gate::Pass,
                "NOT_VERIFIED" => Gate::NotVerified,
                _ => Gate::Fail,
            });
        }
    }
    let gate = verdicts.last().copied();
    Facts {
        gate,
        first_try: verdicts.len() == 1 && gate == Some(Gate::Pass),
        holes_found: holes,
    }
}

/// The body `name`'s run over the scratch world `world`: Summoner solo, or a living creature with
/// its own prompt, soil and verify lines; its gate denies every act that asks.
fn body_run(
    world: &Path,
    trust: &Path,
    root: &Path,
    name: &str,
    riding: &Riding<'_>,
    c: &Controller<'_>,
    journal: Journal<Beat>,
) -> Result<Run, String> {
    let roster = Roster::load(world);
    let (body, cx, gate) = if name == "summoner" {
        let (body, cx) = solo(world, riding.id);
        let card = roster.creatures.get(name);
        let doc = card.map_or_else(
            || ".mikata/summoner/summoner/README.md".to_owned(),
            mikata_world::Creature::doc,
        );
        let verify = card.map(|cr| cr.verify.clone()).unwrap_or_default();
        let gate = EndGate::pinned(trust, name, &doc, &verify, c.config.verify_secs(), None);
        (body, cx, gate)
    } else {
        let creature = roster
            .creatures
            .get(name)
            .filter(|cr| !cr.retired)
            .ok_or_else(|| format!("{name} is no living creature of the world"))?;
        let cx = crate::commission::context_for(world, &roster, name)
            .ok_or_else(|| format!("{name} has no soil"))?;
        let body = crate::run::Body {
            name: name.to_owned(),
            race: creature.race,
            model: riding.id.to_owned(),
            system: crate::commission::prompt(
                world,
                &roster,
                name,
                "human",
                (crate::Law::On, false),
            ),
        };
        let gate = EndGate::pinned(
            trust,
            name,
            &creature.doc(),
            &creature.verify,
            c.config.verify_secs(),
            Some(Author {
                name: name.to_owned(),
                roster: roster.clone(),
                collective: false,
            }),
        );
        (body, cx, gate)
    };
    let race = body.race;
    let (cx, mut limits) = configure(
        c.config,
        cx.with_gate(Arc::new(NoHuman)),
        Limits::for_race(race),
    );
    // The attic lives in the scratch root, removed with it: never left under the state directory.
    let cx = match mikata_tools::Attic::at(&root.join("attic"), world) {
        Some(a) => cx.with_attic(a),
        None => cx,
    };
    let cx = (c.ready)(cx, root);
    // What the human's caps leave, read afresh: the control stops at it.
    let (caps, capped, _, left) = crate::open::read_caps(c.config, &c.usage, &c.session)?;
    limits.budget_wt = left.map(|b| b.wt);
    limits.budget_cap = left.map(|b| b.cap);
    let mut tools = registry();
    tools.push(Box::new(crate::wear_tool::WearTool));
    let setup = Setup {
        model: Arc::clone(&riding.model),
        tools,
        cx,
        journal,
        ledger: Some(c.usage.clone()),
        prices: anthropic_prices().map_err(|e| format!("prices: {e}"))?,
        caller: Caller {
            session: c.session.clone(),
            run: format!("control-{name}"),
            body: name.to_owned(),
            race: if race == Race::Summoner {
                "summoner".into()
            } else {
                race.name().into()
            },
            commission: None,
            prefix: 0,
        },
    };
    let run = Run::new(body, limits, setup)
        .with_gate(gate)
        // A control is stamped by its runner, never by the scratch world's own ledger.
        .with_evolution_ledger(root.join("no-ledger.jsonl"));
    Ok(if capped { run.with_caps(caps) } else { run })
}

/// Runs, for swap `id` on the ledger at `at`, every control its window still waits on, and
/// appends each as a control row. Returns one line per control run.
///
/// Each control rides the model its window run rode, reached before a line is said or a call is
/// spent: a window may hold runs of several models, and each is paired with a control of its own.
///
/// # Errors
///
/// The swap cannot be judged, it was swapped in under another build, a window run kept no model
/// or rode one the configuration no longer reaches, a window run kept no ask, a scratch world
/// cannot be made or put back to the before state, or a row cannot be written.
pub async fn run_controls(
    world: &Path,
    at: &Path,
    id: &str,
    c: &Controller<'_>,
) -> Result<Vec<String>, String> {
    let read = ledger::read(at, None).map_err(|e| format!("the ledger cannot be read: {e}"))?;
    let (swap, waiting) = ledger::awaiting(&read, id).map_err(|r| format!("refused: {r}"))?;
    if waiting.is_empty() {
        return Ok(Vec::new());
    }
    if swap.build != Build::this() {
        return Err(rebuilt(swap));
    }
    let reached = reach_all(&waiting, c)?;
    let through = reached.iter().map(|r| r.id).collect::<Vec<_>>().join(", ");
    // What the controls may spend, said before any is spent; a spent cap runs none.
    let (_, _, _, left) = crate::open::read_caps(c.config, &c.usage, &c.session)?;
    match left {
        Some(l) if l.wt == 0 => {
            return Err(format!(
                "budget: the {} cap is spent, so no control runs; raise budget.{} to judge",
                l.cap, l.cap
            ));
        }
        Some(l) => (c.say)(&format!(
            "{} control(s) to run through {}: at most {} wt, what the {} cap leaves; each call is \
             written to the usage ledger, and a control the cap stops is not recorded",
            waiting.len(),
            through,
            l.wt,
            l.cap
        )),
        None => (c.say)(&format!(
            "{} control(s) to run through {}, as long as the window's asks each; no budget cap is \
             set, so the spend is bounded only by each body's steps — set budget.session to bound \
             it; each call is written to the usage ledger",
            waiting.len(),
            through
        )),
    }
    let mut lines = Vec::new();
    for (n, w) in waiting.iter().enumerate() {
        let ask = w
            .ask
            .as_deref()
            .filter(|a| fingerprint(a) == w.task)
            .ok_or_else(|| {
                format!(
                    "task {}: its ask was not kept, so no control can run",
                    w.task
                )
            })?;
        // The control faces the task the evolved run faced: the tree that run started from. A run
        // stamped before starts were kept has none, and on the finished world a control that does
        // nothing finds the work done.
        let start = w.start.as_deref().ok_or_else(|| {
            format!(
                "task {}: its window run kept no start, so its control would begin from the \
                 finished world; judge this window by hand",
                w.task
            )
        })?;
        let riding = reached
            .iter()
            .find(|r| r.provider == w.provider && r.id == w.model)
            .ok_or_else(|| format!("task {}: its model was not reached", w.task))?;
        let task = Task {
            body: w.body.as_str(),
            ask,
            start,
            riding,
        };
        let root =
            std::env::temp_dir().join(format!("mikata-control-{id}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let scratch = root.join(world.file_name().unwrap_or_else(|| "world".as_ref()));
        let ran = control(world, at, swap, &task, &root, &scratch, c).await;
        let _ = fs::remove_dir_all(&root);
        // The attic `solo` would have kept for the scratch world, should one have been made.
        if let Some(a) = mikata_tools::Attic::for_world(&scratch) {
            let _ = fs::remove_dir_all(a.dir());
        }
        let short: String = w.task.chars().take(12).collect();
        match ran? {
            Some(f) => lines.push(format!(
                "control of task {short} by {}: {}",
                w.body,
                f.gate.map_or("nothing landed", |g| match g {
                    ledger::Gate::Pass => "pass",
                    ledger::Gate::Fail => "fail",
                    ledger::Gate::NotVerified => "not verified",
                })
            )),
            None => lines.push(format!(
                "control of task {short} by {}: stopped — interrupted or at a budget cap — and \
                 not recorded",
                w.body
            )),
        }
    }
    Ok(lines)
}

/// One control: the scratch world made and put back to the before state, the ask run, the row
/// written. `None` when the run was interrupted.
async fn control(
    world: &Path,
    at: &Path,
    swap: &ledger::Swap,
    task: &Task<'_>,
    root: &Path,
    scratch: &Path,
    c: &Controller<'_>,
) -> Result<Option<Facts>, String> {
    let Task {
        body,
        ask,
        start,
        riding,
    } = task;
    copy_world(world, scratch, true).map_err(|e| format!("the scratch world: {e}"))?;
    let attic = match c.attic {
        Some(dir) => mikata_tools::Attic::at(dir, world),
        None => mikata_tools::Attic::for_world(world),
    }
    .ok_or("no attic holds the window run's start, so no control can begin from it")?;
    attic
        .export_into(start, scratch)
        .map_err(|e| format!("the window run's start: {e}"))?;
    restore_before(at, swap, scratch).map_err(|e| format!("the before state: {e}"))?;
    let state = state_now(scratch, swap).map_err(|e| format!("the before state: {e}"))?;
    if state != swap.before_state() {
        return Err("the scratch world does not hold the named files as they were before".into());
    }
    let journal: Journal<Beat> = Journal::new(root.join("control.jsonl"));
    // The scratch copy's cards are judged against the live world's pins: a copy has none.
    let mut run = body_run(scratch, world, root, body, riding, c, journal.clone())?;
    let stamp = Stamp {
        ledger: at.to_path_buf(),
        swap: swap.clone(),
        body: (*body).to_owned(),
        model: riding.id.to_owned(),
        provider: riding.model.provider().to_owned(),
        ask: (*ask).to_owned(),
        state,
        control: true,
        start: None,
    };
    let mut quiet = |_| {};
    // A control the human or a cap stopped says nothing of doing nothing: no row.
    if matches!(
        run.ask(ask, &mut quiet).await,
        Ending::Interrupted | Ending::Checkpoint { .. }
    ) {
        return Ok(None);
    }
    let mut facts = facts_of(&journal);
    // A control that changed nothing is judged by whether the task is done, as its window run
    // is: the gate passes no change, and on that pass a do-nothing control "landed" every task.
    if last_gate_unchanged(&journal)
        && let Some((checked, failures)) = run.verify_now()
    {
        use mikata_ledger::evolution::Gate;
        facts.gate = Some(if !checked {
            Gate::NotVerified
        } else if failures.is_empty() {
            Gate::Pass
        } else {
            Gate::Fail
        });
        facts.first_try = facts.gate == Some(Gate::Pass);
    }
    stamp
        .record(scratch, facts)
        .map_err(|e| format!("the control row: {e}"))?;
    Ok(Some(facts))
}

/// Why a swap from another build is not judged by this one.
fn rebuilt(swap: &ledger::Swap) -> String {
    format!(
        "swap `{}` built by {}; this build {}: re-swap to judge",
        swap.id,
        swap.build,
        Build::this()
    )
}

/// The model a window run rode, reached again for its control: the configuration's own when it
/// is still that one, else the one the run names, asked for as a session asks for a model when it
/// opens (`<provider>:<id>` over the configuration's own), through the same backends and keys.
///
/// # Errors
///
/// The model's backend is unknown, its key is not set, or the configuration reaches no model
/// `id` through `provider`.
pub fn reach(
    config: &Config,
    world: &Path,
    env: &dyn Fn(&str) -> Option<String>,
    provider: &str,
    id: &str,
) -> Result<Arc<dyn Model>, String> {
    let same = |model: &Arc<dyn Model>, backend: &mikata_config::Backend| {
        backend.model == id && model.provider() == provider
    };
    // The configuration's own first: most windows ride it, and it may be named another way.
    if let Ok((model, backend, ..)) = crate::open::model_of(config, world, env)
        && same(&model, &backend)
    {
        return Ok(model);
    }
    let named = format!("{provider}:{id}");
    let asked = |key: &str| {
        if key == "MIKATA_MODEL" {
            Some(named.clone())
        } else {
            env(key)
        }
    };
    let (model, backend, ..) = crate::open::model_of(config, world, &asked)?;
    if same(&model, &backend) {
        Ok(model)
    } else {
        Err(format!(
            "the configuration reaches no model `{id}` through `{provider}`"
        ))
    }
}

/// What judging on demand came to: the controls run, then the judgement.
pub struct Judged {
    /// One line per control run.
    pub controls: Vec<String>,
    /// The judgement, a proposal.
    pub judgement: Judgement,
}

/// `mikatad evolution judge <id>`: runs the controls swap `id` waits on in `world` — each through
/// the model its window run rode ([`reach`]), its key read from `env` — then judges it from the
/// world's ledger, held to its anchored head, on `evolution.min_net_wins`. No model is reached
/// when no control waits.
///
/// # Errors
///
/// The configuration does not load, there is no home for the ledger, a window run's model cannot
/// be reached, or a control cannot run.
pub fn judge(
    world: &Path,
    id: &str,
    env: &dyn Fn(&str) -> Option<String>,
    say: &dyn Fn(&str),
) -> Result<Judged, String> {
    let config = Config::load(&Paths::resolve(world))?;
    let at: PathBuf = mikata_world::evolution::ledger_of(world)
        .ok_or("no home to keep the evolution ledger under")?;
    let read = ledger::read(&at, None).map_err(|e| format!("the ledger cannot be read: {e}"))?;
    let awaiting = ledger::awaiting(&read, id).ok();
    if let Some((swap, _)) = awaiting.as_ref().filter(|(s, _)| s.build != Build::this()) {
        return Err(rebuilt(swap));
    }
    let waits = awaiting.is_some_and(|(_, w)| !w.is_empty());
    let controls = if waits {
        let model = |provider: &str, id: &str| reach(&config, world, env, provider, id);
        let c = Controller {
            model: &model,
            config: &config,
            ready: |cx, _| cx,
            attic: None,
            usage: crate::open::usage_ledger(world),
            session: format!(
                "evolution-judge-{id}-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_millis())
            ),
            say,
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("the runtime: {e}"))?;
        rt.block_on(run_controls(world, &at, id, &c))?
    } else {
        Vec::new()
    };
    let judgement = mikata_world::evolution::judge(world, id, config.evolution_min_net_wins())
        .map_err(|e| format!("the ledger cannot be read: {e}"))?;
    Ok(Judged {
        controls,
        judgement,
    })
}
