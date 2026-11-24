//! Units in a live session ([`mikata_loop::units`]): a native unit goes off and comes back while
//! the session runs, a unit's veto stops a call and its note reaches the body, and none of it
//! moves the cached prefix.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mikata_kernel::{
    Activation, Band, Change, Event, Grant, Manifest, Point, Reply, Unit, UnitError,
};
use mikata_ledger::Journal;
use mikata_loop::units::Cx;
use mikata_loop::{Beat, Limits, Run, ScriptedModel, Setup, solo};
use mikata_provider::{Caller, anthropic_prices};
use mikata_races::Race;
use mikata_tools::{Sandbox, registry};
use serde_json::json;

struct World {
    root: PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("mikata-units-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("world")).unwrap();
        Self { root }
    }

    fn world(&self) -> PathBuf {
        self.root.join("world")
    }

    fn journal(&self) -> Journal<Beat> {
        Journal::new(self.root.join("run.jsonl"))
    }

    fn beats(&self) -> Vec<Beat> {
        self.journal().read().unwrap().rows
    }

    fn run(&self, model: Arc<ScriptedModel>) -> Run {
        let (body, cx) = solo(&self.world(), "claude-sonnet-5-5");
        let cx = cx
            .with_sandbox(Sandbox::None("units tests run unguarded".into()))
            .with_unguarded(true);
        Run::new(
            body,
            Limits::for_race(Race::Summoner),
            Setup {
                model,
                tools: registry(),
                cx,
                journal: self.journal(),
                ledger: None,
                prices: anthropic_prices().unwrap(),
                caller: Caller {
                    session: "s".into(),
                    run: "r".into(),
                    body: "summoner".into(),
                    race: "summoner".into(),
                    commission: None,
                    prefix: 0,
                },
            },
        )
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn trees(beats: &[Beat]) -> Vec<String> {
    beats
        .iter()
        .filter_map(|b| match b {
            Beat::Tree { tool_use_id, .. } => Some(tool_use_id.clone()),
            _ => None,
        })
        .collect()
}

fn write(id: &str, path: &str) -> Result<mikata_provider::Response, mikata_provider::Error> {
    ScriptedModel::tool(id, "write", json!({"path": path, "content": "x\n"}))
}

/// A unit a test loads: it vetoes any write to `secret.txt` and notes every call it sees.
struct Guard(Manifest);

impl Guard {
    fn new() -> Self {
        Self(
            Manifest::new("guard", "1")
                .at(Point::ToolBefore, Band::Outer)
                .at(Point::ToolAfter, Band::Outer)
                .grant(Grant::Advise)
                .grant(Grant::Veto),
        )
    }
}

impl Unit<Cx> for Guard {
    fn manifest(&self) -> &Manifest {
        &self.0
    }

    fn activate(&mut self, _act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
        Ok(())
    }

    fn on(&mut self, _cx: &mut Cx, event: &Event) -> Reply {
        match event {
            Event::ToolBefore { input, .. } if input["path"] == "secret.txt" => Reply::Veto {
                why: "secret.txt is not to be written".into(),
            },
            Event::ToolAfter { name, .. } => Reply::Note {
                text: format!("saw {name}"),
            },
            _ => Reply::Pass,
        }
    }
}

#[tokio::test]
async fn a_native_unit_goes_off_and_comes_back_while_the_session_runs() {
    let w = World::new("off-on");
    let model = Arc::new(ScriptedModel::new([
        write("t1", "a.txt"),
        ScriptedModel::text("done"),
        write("t2", "b.txt"),
        ScriptedModel::text("done"),
        write("t3", "c.txt"),
        ScriptedModel::text("done"),
    ]));
    let mut run = w.run(model.clone());
    run.ask("write a", &mut |_| {}).await;
    assert_eq!(trees(&w.beats()), ["t1"], "on by default: the tree is read");
    assert!(
        !w.beats().iter().any(|b| matches!(b, Beat::Units { .. })),
        "the default composition journals nothing"
    );

    run.change_units(Change::Unload("returns".into()));
    run.ask("write b", &mut |_| {}).await;
    assert_eq!(trees(&w.beats()), ["t1"], "off: no tree read for t2");
    assert!(w.beats().iter().any(
        |b| matches!(b, Beat::Units { op: mikata_kernel::Op::Unload, id, .. } if id == "returns")
    ));

    run.change_units(Change::Load(
        mikata_loop::units::native("returns").unwrap(),
        Vec::new(),
    ));
    run.ask("write c", &mut |_| {}).await;
    assert_eq!(trees(&w.beats()), ["t1", "t3"], "on again");
    assert!(run.units().active().iter().any(|(id, _)| *id == "returns"));

    // The body was told each change, and the prefix never moved.
    let said = format!("{:?}", model.requests()[2].messages);
    assert!(said.contains("the unit `returns` is off"), "{said}");
    assert!(
        !w.beats().iter().any(|b| matches!(b, Beat::Prefix { .. })),
        "a change of units never moves the cached prefix"
    );
    let first = &model.requests()[0];
    assert!(model.requests().iter().all(|r| r.system == first.system));
}

#[tokio::test]
async fn a_units_veto_stops_the_call_and_its_note_reaches_the_body() {
    let w = World::new("veto");
    let model = Arc::new(ScriptedModel::new([
        write("t1", "secret.txt"),
        write("t2", "open.txt"),
        ScriptedModel::text("done"),
    ]));
    let mut run = w.run(model.clone());
    run.change_units(Change::Load(
        Box::new(Guard::new()),
        vec![Grant::Advise, Grant::Veto],
    ));
    run.ask("write two files", &mut |_| {}).await;
    assert!(!w.world().join("secret.txt").exists(), "the veto held");
    assert!(w.world().join("open.txt").exists(), "the other call ran");
    let after_veto = format!("{:?}", model.requests()[1].messages);
    assert!(
        after_veto.contains("secret.txt is not to be written (said by the unit `guard`)"),
        "{after_veto}"
    );
    let after_note = format!("{:?}", model.requests()[2].messages);
    assert!(after_note.contains("[guard] saw write"), "{after_note}");
    assert!(w.beats().iter().any(
        |b| matches!(b, Beat::Units { op: mikata_kernel::Op::Load, id, .. } if id == "guard")
    ));
}

#[tokio::test]
async fn a_reply_above_the_grant_is_read_as_a_pass_and_journaled() {
    let w = World::new("grant");
    let model = Arc::new(ScriptedModel::new([
        write("t1", "secret.txt"),
        ScriptedModel::text("done"),
    ]));
    let mut run = w.run(model.clone());
    // Granted advice only: the veto is dropped.
    run.change_units(Change::Load(Box::new(Guard::new()), vec![Grant::Advise]));
    run.ask("write it", &mut |_| {}).await;
    assert!(
        w.world().join("secret.txt").exists(),
        "a veto above the grant does not hold"
    );
    assert!(w.beats().iter().any(
        |b| matches!(b, Beat::Units { op: mikata_kernel::Op::Dropped, id, .. } if id == "guard")
    ));
}

/// The catalog holding one declared plugin, `id`, run as `command args`.
fn catalog(
    id: &str,
    command: &str,
    args: Vec<String>,
    grants: &[&str],
) -> mikata_loop::units::Catalog {
    mikata_loop::units::Catalog {
        plugins: vec![mikata_config::Plugin {
            id: id.into(),
            command: command.into(),
            args,
            env: Vec::new(),
            grants: grants.iter().map(|g| (*g).to_owned()).collect(),
            // The body's race alone, as the human's file names it (`mikata`): a plugin joins the
            // body, not the race its tools' context runs as.
            races: vec![Race::Summoner],
            enabled: true,
        }],
        hook_ms: 2000,
        host: None,
    }
}

fn bash(id: &str, command: &str) -> Result<mikata_provider::Response, mikata_provider::Error> {
    ScriptedModel::tool(id, "bash", json!({"command": command}))
}

#[tokio::test]
async fn a_plugin_loaded_mid_session_vetoes_a_force_push_and_unloads() {
    if std::process::Command::new("python3")
        .arg("-V")
        .output()
        .is_err()
    {
        eprintln!("skipped: no python3 on PATH to run the example plugin");
        return;
    }
    let w = World::new("plugin-veto");
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/plugins/no_force_push.py")
        .canonicalize()
        .unwrap();
    let model = Arc::new(ScriptedModel::new([
        bash("t1", "echo before"),
        ScriptedModel::text("done"),
        bash("t2", "git push --force origin main"),
        bash("t3", "echo git push --force"),
        ScriptedModel::text("done"),
        bash("t4", "git push -f origin main"),
        ScriptedModel::text("done"),
    ]));
    let mut run = w.run(model.clone()).with_catalog(catalog(
        "no-force-push",
        "python3",
        vec![script.display().to_string()],
        &["veto"],
    ));
    run.ask("first", &mut |_| {}).await;
    // Loaded while the session runs, between asks.
    run.switch_unit("plugin:no-force-push", true).unwrap();
    run.ask("push it", &mut |_| {}).await;
    let vetoed = format!("{:?}", model.requests()[3].messages);
    assert!(
        vetoed.contains("said by the unit `plugin:no-force-push`"),
        "{vetoed}"
    );
    let echoed = format!("{:?}", model.requests()[4].messages);
    assert!(
        echoed.contains("git push --force"),
        "an echo is no push: {echoed}"
    );
    // Unloaded while the session runs: the next force push is not the plugin's to stop.
    run.switch_unit("plugin:no-force-push", false).unwrap();
    run.ask("push again", &mut |_| {}).await;
    let after = format!("{:?}", model.requests()[6].messages.last());
    assert!(!after.contains("said by the unit"), "{after}");
    assert!(
        after.contains("the human gate did not approve"),
        "the harness's own gate still holds: {after}"
    );
    let ops: Vec<(mikata_kernel::Op, String)> = w
        .beats()
        .iter()
        .filter_map(|b| match b {
            Beat::Units { op, id, .. } => Some((*op, id.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        ops,
        [
            (mikata_kernel::Op::Load, "plugin:no-force-push".to_owned()),
            (mikata_kernel::Op::Unload, "plugin:no-force-push".to_owned()),
        ]
    );
    assert!(!w.beats().iter().any(|b| matches!(b, Beat::Prefix { .. })));
}

#[tokio::test]
async fn a_plugin_that_crashes_is_quarantined_and_the_session_goes_on() {
    let w = World::new("plugin-crash");
    let log = w.root.join("plugin.log");
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../mikata-tools/tests/fixtures/fake-plugin.sh")
        .canonicalize()
        .unwrap();
    let model = Arc::new(ScriptedModel::new([
        ScriptedModel::tool("t1", "crash", json!({})),
        ScriptedModel::tool("t2", "crash", json!({})),
        ScriptedModel::tool("t3", "crash", json!({})),
        bash("t4", "echo still here"),
        ScriptedModel::text("done"),
    ]));
    let mut run = w.run(model.clone()).with_catalog(catalog(
        "fake",
        "bash",
        vec![script.display().to_string(), log.display().to_string()],
        &["advise", "veto"],
    ));
    run.switch_unit("plugin:fake", true).unwrap();
    let ending = run.ask("go", &mut |_| {}).await;
    assert!(
        matches!(ending, mikata_loop::Ending::Answered(_)),
        "{ending:?}"
    );
    assert!(w.beats().iter().any(
        |b| matches!(b, Beat::Units { op: mikata_kernel::Op::Quarantine, id, .. } if id == "plugin:fake")
    ));
    assert!(
        run.units()
            .active()
            .iter()
            .all(|(id, _)| *id != "plugin:fake")
    );
    let told = format!("{:?}", model.requests()[4].messages);
    assert!(told.contains("still here"), "{told}");
}

#[tokio::test]
async fn memory_off_keeps_no_lesson_and_on_again_keeps_it() {
    use mikata_memory::{Tier, fold, read};
    let w = World::new("memory-off");
    let lesson = |text: &str| ScriptedModel::text(&format!("Done.\n\n- @U colony {text}"));
    let model = Arc::new(ScriptedModel::new([
        lesson("the tests need no network"),
        lesson("run the build before answering"),
    ]));
    let mut run = w.run(model.clone());
    let kept = |w: &World| {
        fold(
            &read(&w.world(), &Tier::Personal("summoner".into()))
                .map(|r| r.rows)
                .unwrap_or_default(),
        )
        .values()
        .map(|n| n.text.clone())
        .collect::<Vec<_>>()
    };
    run.change_units(Change::Unload(mikata_loop::units::MEMORY.into()));
    run.ask("first", &mut |_| {}).await;
    assert!(kept(&w).is_empty(), "off: nothing kept: {:?}", kept(&w));
    run.switch_unit(mikata_loop::units::MEMORY, true).unwrap();
    run.ask("second", &mut |_| {}).await;
    assert_eq!(kept(&w), ["run the build before answering"]);
}

/// The owner's example: the native memory off and a third party's on, from the human's file. One
/// session keeps a lesson through the plugin; the next is shown it in its preface; the native
/// memory keeps nothing.
#[tokio::test]
async fn a_plugin_memory_stands_in_for_the_native_one() {
    if std::process::Command::new("python3")
        .arg("-V")
        .output()
        .is_err()
    {
        eprintln!("skipped: no python3 on PATH to run the example plugin");
        return;
    }
    use mikata_memory::{Tier, fold, read};
    let w = World::new("plugin-memory");
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/plugins/notes_memory.py")
        .canonicalize()
        .unwrap();
    let session = |model: Arc<ScriptedModel>| {
        let mut run = w.run(model).with_catalog(catalog(
            "notes",
            "python3",
            vec![script.display().to_string()],
            &["advise"],
        ));
        run.change_units(Change::Unload(mikata_loop::units::MEMORY.into()));
        run.switch_unit("plugin:notes", true).unwrap();
        run
    };
    let first = Arc::new(ScriptedModel::new([ScriptedModel::text(
        "Done.\n\n- @U colony the parser tests need the duckdb dialect loaded first",
    )]));
    session(first)
        .ask("fix the parser tests", &mut |_| {})
        .await;
    let second = Arc::new(ScriptedModel::new([ScriptedModel::text("ok")]));
    session(second.clone())
        .ask("the parser tests fail again", &mut |_| {})
        .await;
    let said = format!("{:?}", second.requests()[0].messages);
    assert!(
        said.contains("[plugin:notes] What earlier sessions learned (the notes plugin):\\n- the parser tests need the duckdb dialect loaded first"),
        "{said}"
    );
    let native = read(&w.world(), &Tier::Personal("summoner".into())).map(|r| fold(&r.rows).len());
    assert_eq!(native.unwrap_or(0), 0, "the native memory kept nothing");
}
