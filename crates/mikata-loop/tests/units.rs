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
