use std::collections::BTreeSet;

use super::*;
use crate::manifest::{Band, Manifest, SafePoint};

/// What the probes write into: the registrations alive now, and what happened, in order.
#[derive(Default)]
struct Cx {
    live: BTreeSet<String>,
    log: Vec<String>,
}

/// A unit that registers itself, provides what its manifest names, and answers one reply.
struct Probe {
    manifest: Manifest,
    reply: Reply,
    panic_on: Option<Point>,
    fail_start: bool,
    broken: Option<String>,
}

fn probe(manifest: Manifest) -> Probe {
    Probe {
        manifest,
        reply: Reply::Pass,
        panic_on: None,
        fail_start: false,
        broken: None,
    }
}

impl Probe {
    fn replying(mut self, reply: Reply) -> Self {
        self.reply = reply;
        self
    }

    fn boxed(self) -> Box<dyn Unit<Cx>> {
        Box::new(self)
    }
}

impl Unit<Cx> for Probe {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn activate(&mut self, act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
        let id = act.id().to_owned();
        for key in self.manifest.provides.clone() {
            act.provide(&key, format!("{id}:{key}"))?;
        }
        let back = id.clone();
        act.register(
            |cx| cx.live.insert(id.clone()),
            move |cx| {
                cx.live.remove(&back);
                cx.log.push(format!("undo {back}"));
            },
        );
        if self.fail_start {
            return Err(UnitError::new("no disk"));
        }
        Ok(())
    }

    fn on(&mut self, cx: &mut Cx, event: &Event) -> Reply {
        cx.log
            .push(format!("{} {}", self.manifest.id, event.point().name()));
        assert!(self.panic_on != Some(event.point()), "a probe that panics");
        self.reply.clone()
    }

    fn settle(&mut self, cx: &mut Cx) {
        cx.log.push(format!("settle {}", self.manifest.id));
    }

    fn failed(&self) -> Option<String> {
        self.broken.clone()
    }
}

fn unit(id: &str) -> Manifest {
    Manifest::new(id, "1").safe_point(SafePoint::Batch)
}

fn load(reg: &mut Registry<Cx>, cx: &mut Cx, p: Probe, grants: &[Grant]) -> Vec<UnitsRow> {
    reg.request(Change::Load(p.boxed(), grants.to_vec()));
    reg.reconcile(SafePoint::Session, cx)
}

fn unload(reg: &mut Registry<Cx>, cx: &mut Cx, id: &str) -> Vec<UnitsRow> {
    reg.request(Change::Unload(id.into()));
    reg.reconcile(SafePoint::Session, cx)
}

fn ops(rows: &[UnitsRow]) -> Vec<(Op, &str)> {
    rows.iter().map(|r| (r.op, r.id.as_str())).collect()
}

fn ask() -> Event {
    Event::AskBegin { ask: "go".into() }
}

fn tool_before() -> Event {
    Event::ToolBefore {
        id: "t".into(),
        name: "bash".into(),
        input: serde_json::Value::Null,
    }
}

fn order_at(reg: &mut Registry<Cx>, cx: &mut Cx, event: &Event) -> Vec<String> {
    reg.dispatch(event.point(), cx, event)
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn a_load_registers_and_an_unload_takes_every_registration_back() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let rows = load(
        &mut reg,
        &mut cx,
        probe(
            unit("memory")
                .provides("recall")
                .at(Point::AskBegin, Band::Native),
        ),
        &[],
    );
    assert_eq!(ops(&rows), [(Op::Load, "memory")]);
    assert_eq!(rows[0].digest, "memory@1");
    assert_eq!(reg.active(), [("memory", "1")]);
    assert_eq!(
        reg.service::<String>("recall").map(String::as_str),
        Some("memory:recall")
    );
    assert!(cx.live.contains("memory"));

    let rows = unload(&mut reg, &mut cx, "memory");
    assert_eq!(ops(&rows), [(Op::Unload, "memory")]);
    assert_eq!(rows[0].digest, "");
    assert!(reg.active().is_empty() && cx.live.is_empty());
    assert!(reg.service::<String>("recall").is_none());
    assert_eq!(cx.log, ["settle memory", "undo memory"]);

    let rows = unload(&mut reg, &mut cx, "memory");
    assert_eq!(ops(&rows), [(Op::Refused, "memory")]);
}

#[test]
fn a_change_waits_for_the_safe_point_its_units_allow() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let m = Manifest::new("gate", "1").safe_point(SafePoint::Ask);
    reg.request(Change::Load(probe(m).boxed(), vec![]));
    assert_eq!(reg.reconcile(SafePoint::Batch, &mut cx), []);
    assert_eq!(reg.pending(), 1);
    assert_eq!(
        ops(&reg.reconcile(SafePoint::Ask, &mut cx)),
        [(Op::Load, "gate")]
    );

    // A batch unit that a session unit depends on leaves only when the session unit may.
    load(&mut reg, &mut cx, probe(unit("base").provides("k")), &[]);
    let late = Manifest::new("late", "1")
        .needs("k")
        .safe_point(SafePoint::Session);
    load(&mut reg, &mut cx, probe(late), &[]);
    reg.request(Change::Unload("base".into()));
    assert_eq!(reg.reconcile(SafePoint::Ask, &mut cx), []);
    assert_eq!(
        ops(&reg.reconcile(SafePoint::Session, &mut cx)),
        [(Op::Unload, "late"), (Op::Unload, "base")]
    );
}

#[test]
fn hooks_run_by_band_then_by_edges_then_by_id() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let at = |m: Manifest, band| m.at(Point::AskBegin, band);
    load(
        &mut reg,
        &mut cx,
        probe(at(unit("zeta"), Band::Native)),
        &[],
    );
    load(
        &mut reg,
        &mut cx,
        probe(at(unit("plugin:a"), Band::Outer)),
        &[],
    );
    load(
        &mut reg,
        &mut cx,
        probe(at(unit("alpha"), Band::Native).after("zeta")),
        &[],
    );
    load(&mut reg, &mut cx, probe(at(unit("gate"), Band::Guard)), &[]);
    load(
        &mut reg,
        &mut cx,
        probe(at(unit("beta"), Band::Native)),
        &[],
    );
    load(&mut reg, &mut cx, probe(unit("quiet")), &[]);
    assert_eq!(
        order_at(&mut reg, &mut cx, &ask()),
        ["gate", "beta", "zeta", "alpha", "plugin:a"]
    );
    assert_eq!(
        reg.digest(),
        "alpha@1,beta@1,gate@1,plugin:a@1,quiet@1,zeta@1"
    );
}

#[test]
fn a_unit_needing_a_service_waits_for_it_and_runs_after_its_provider() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let reader = unit("a-reader")
        .needs("recall")
        .at(Point::AskBegin, Band::Native);
    assert_eq!(
        ops(&load(&mut reg, &mut cx, probe(reader), &[])),
        [(Op::Load, "a-reader")]
    );
    assert_eq!(reg.fibers(), [("a-reader", "1", Fiber::Inactive)]);
    assert_eq!(order_at(&mut reg, &mut cx, &ask()), Vec::<String>::new());

    let memory = unit("memory")
        .provides("recall")
        .at(Point::AskBegin, Band::Native);
    load(&mut reg, &mut cx, probe(memory), &[]);
    assert_eq!(reg.active(), [("memory", "1"), ("a-reader", "1")]);
    assert_eq!(order_at(&mut reg, &mut cx, &ask()), ["memory", "a-reader"]);
}

#[test]
fn an_unload_takes_its_dependents_out_first_newest_registration_first() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    load(
        &mut reg,
        &mut cx,
        probe(unit("memory").provides("recall")),
        &[],
    );
    load(
        &mut reg,
        &mut cx,
        probe(unit("reader").needs("recall")),
        &[],
    );
    load(
        &mut reg,
        &mut cx,
        probe(unit("stamps").after("reader")),
        &[],
    );
    load(&mut reg, &mut cx, probe(unit("other")), &[]);
    cx.log.clear();

    let rows = unload(&mut reg, &mut cx, "memory");
    assert_eq!(
        ops(&rows),
        [
            (Op::Unload, "stamps"),
            (Op::Unload, "reader"),
            (Op::Unload, "memory")
        ]
    );
    assert_eq!(
        rows[0].why.as_deref(),
        Some("it depends on memory, which was unloaded")
    );
    assert_eq!(
        cx.log,
        [
            "settle stamps",
            "undo stamps",
            "settle reader",
            "undo reader",
            "settle memory",
            "undo memory"
        ]
    );
    assert_eq!(reg.active(), [("other", "1")]);
}

#[test]
fn a_refused_load_leaves_nothing() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    load(
        &mut reg,
        &mut cx,
        probe(unit("memory").provides("recall")),
        &[],
    );
    load(&mut reg, &mut cx, probe(unit("reader").needs("echo")), &[]);
    let before = (reg.fibers().len(), reg.digest(), cx.live.clone());

    let refusals = [
        (unit("memory"), "it is already loaded"),
        (
            unit("twin").provides("recall"),
            "`recall` is already provided by memory",
        ),
        (
            unit("lost").after("nowhere"),
            "it runs after nowhere, which is not loaded",
        ),
        (unit("self").after("self"), "it runs after itself"),
        (
            unit("echo").provides("echo").after("reader"),
            "it closes a cycle among echo, reader",
        ),
        (
            unit("dup")
                .at(Point::Preface, Band::Native)
                .at(Point::Preface, Band::Outer),
            "it hooks preface twice",
        ),
    ];
    for (m, why) in refusals {
        let id = m.id.clone();
        let rows = load(&mut reg, &mut cx, probe(m), &[]);
        assert_eq!(ops(&rows), [(Op::Refused, id.as_str())]);
        assert_eq!(rows[0].why.as_deref(), Some(why));
    }

    // One that fails to start, after registering and providing, takes it all back.
    let mut failing = probe(unit("flaky").provides("weather"));
    failing.fail_start = true;
    let rows = load(&mut reg, &mut cx, failing, &[]);
    assert_eq!(ops(&rows), [(Op::Refused, "flaky")]);
    assert_eq!(rows[0].why.as_deref(), Some("no disk"));
    assert!(reg.service::<String>("weather").is_none());
    assert_eq!((reg.fibers().len(), reg.digest(), cx.live.clone()), before);
}

#[test]
fn a_unit_that_does_not_provide_what_it_names_is_refused() {
    struct Idle(Manifest);
    impl Unit<Cx> for Idle {
        fn manifest(&self) -> &Manifest {
            &self.0
        }
        fn activate(&mut self, act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
            act.provide("other", 1_u8)
        }
        fn on(&mut self, _: &mut Cx, _: &Event) -> Reply {
            Reply::Pass
        }
    }
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    reg.request(Change::Load(
        Box::new(Idle(unit("idle").provides("k"))),
        vec![],
    ));
    let rows = reg.reconcile(SafePoint::Batch, &mut cx);
    assert_eq!(
        rows[0].why.as_deref(),
        Some("idle provides `other`, which its manifest does not name")
    );
}

#[test]
fn replies_are_cut_to_the_grant_and_to_where_they_count() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let asks_veto = |id: &str| {
        unit(id)
            .at(Point::ToolBefore, Band::Outer)
            .at(Point::AskBegin, Band::Outer)
            .grant(Grant::Veto)
            .grant(Grant::Advise)
    };
    let veto = Reply::Veto { why: "no".into() };
    // Asked for veto, granted only advise: its veto is a pass.
    load(
        &mut reg,
        &mut cx,
        probe(asks_veto("a")).replying(veto.clone()),
        &[Grant::Advise],
    );
    // Granted veto, but a veto does not count when an ask begins.
    load(
        &mut reg,
        &mut cx,
        probe(asks_veto("b")).replying(veto.clone()),
        &[Grant::Veto],
    );
    assert_eq!(reg.grants("a"), Some(&[Grant::Advise][..]));

    let replies = reg.dispatch(Point::ToolBefore, &mut cx, &tool_before());
    assert_eq!(
        replies,
        [
            ("a".to_owned(), Reply::Pass),
            ("b".to_owned(), veto.clone())
        ]
    );
    let replies = reg.dispatch(Point::AskBegin, &mut cx, &ask());
    assert!(replies.iter().all(|(_, r)| *r == Reply::Pass));
    let rows = reg.take_rows();
    assert_eq!(
        ops(&rows),
        [(Op::Dropped, "a"), (Op::Dropped, "a"), (Op::Dropped, "b")]
    );
    assert_eq!(
        rows[0].why.as_deref(),
        Some("a veto at tool_before needs the veto grant")
    );
    assert_eq!(
        rows[2].why.as_deref(),
        Some("a veto does not count at ask_begin")
    );
    assert_eq!(reg.take_rows(), []);
}

#[test]
fn the_first_veto_ends_the_point_and_notes_are_capped() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let m = |id: &str| {
        unit(id)
            .at(Point::ToolBefore, Band::Native)
            .grant(Grant::Advise)
            .grant(Grant::Veto)
    };
    let long = "x".repeat(NOTE_CAP + 10);
    load(
        &mut reg,
        &mut cx,
        probe(m("a")).replying(Reply::Note { text: long }),
        &[Grant::Advise],
    );
    load(
        &mut reg,
        &mut cx,
        probe(m("b")).replying(Reply::Veto {
            why: "force push".into(),
        }),
        &[Grant::Veto],
    );
    load(
        &mut reg,
        &mut cx,
        probe(m("c")).replying(Reply::Note {
            text: "late".into(),
        }),
        &[Grant::Advise],
    );
    let replies = reg.dispatch(Point::ToolBefore, &mut cx, &tool_before());
    assert_eq!(replies.len(), 2);
    let Reply::Note { text } = &replies[0].1 else {
        panic!("a note")
    };
    assert_eq!(text.chars().count(), NOTE_CAP);
    assert_eq!(
        replies[1],
        (
            "b".to_owned(),
            Reply::Veto {
                why: "force push".into()
            }
        )
    );
    assert!(!cx.log.iter().any(|l| l.starts_with("c ")));
}

#[test]
fn a_hook_that_panics_is_quarantined_and_answers_pass() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let mut bad = probe(unit("bad").provides("k").at(Point::AskBegin, Band::Native));
    bad.panic_on = Some(Point::AskBegin);
    load(&mut reg, &mut cx, bad, &[]);
    load(&mut reg, &mut cx, probe(unit("child").needs("k")), &[]);
    load(
        &mut reg,
        &mut cx,
        probe(unit("good").at(Point::AskBegin, Band::Native)),
        &[],
    );

    let replies = reg.dispatch(Point::AskBegin, &mut cx, &ask());
    assert_eq!(
        replies,
        [
            ("bad".to_owned(), Reply::Pass),
            ("good".to_owned(), Reply::Pass)
        ]
    );
    let rows = reg.take_rows();
    assert_eq!(ops(&rows), [(Op::Unload, "child"), (Op::Quarantine, "bad")]);
    assert!(
        rows[1]
            .why
            .as_deref()
            .unwrap()
            .contains("panicked at ask_begin")
    );
    assert_eq!(cx.live, BTreeSet::from(["good".to_owned()]));
    assert!(reg.service::<String>("k").is_none());
    assert_eq!(
        reg.fibers(),
        [("good", "1", Fiber::Active), ("bad", "1", Fiber::Failed)]
    );
    // Quarantined: not called again, and not settled on its way out.
    assert_eq!(order_at(&mut reg, &mut cx, &ask()), ["good"]);
    assert!(!cx.log.contains(&"settle bad".to_owned()));
    // A fixed build of it may take its place.
    let rows = load(&mut reg, &mut cx, probe(unit("bad").provides("k")), &[]);
    assert_eq!(ops(&rows), [(Op::Load, "bad")]);
    assert_eq!(reg.fibers().len(), 2);
}

#[test]
fn a_unit_that_says_it_failed_is_quarantined() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    let mut tired = probe(
        unit("plugin:tired")
            .at(Point::AskBegin, Band::Outer)
            .grant(Grant::Advise),
    );
    tired.reply = Reply::Note { text: "hi".into() };
    tired.broken = Some("three strikes".into());
    load(&mut reg, &mut cx, tired, &[Grant::Advise]);
    let replies = reg.dispatch(Point::AskBegin, &mut cx, &ask());
    assert_eq!(replies, [("plugin:tired".to_owned(), Reply::Pass)]);
    let rows = reg.take_rows();
    assert_eq!(rows[0].op, Op::Quarantine);
    assert_eq!(rows[0].why.as_deref(), Some("three strikes"));
    assert_eq!(reg.quarantine("plugin:tired", "again", &mut cx), []);
    assert_eq!(
        ops(&unload(&mut reg, &mut cx, "plugin:tired")),
        [(Op::Unload, "plugin:tired")]
    );
    assert_eq!(reg.fibers(), []);
}

#[test]
fn a_swap_keeps_the_dependents_and_a_refused_swap_keeps_the_old_unit() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    load(
        &mut reg,
        &mut cx,
        probe(unit("memory").provides("recall")),
        &[],
    );
    load(
        &mut reg,
        &mut cx,
        probe(unit("reader").needs("recall")),
        &[],
    );
    cx.log.clear();

    let v2 = Manifest::new("memory", "2").provides("recall");
    reg.request(Change::Swap("memory".into(), probe(v2).boxed(), vec![]));
    let rows = reg.reconcile(SafePoint::Ask, &mut cx);
    assert_eq!(ops(&rows), [(Op::Swap, "memory")]);
    assert_eq!(rows[0].why.as_deref(), Some("in place of memory@1"));
    assert_eq!(rows[0].digest, "memory@2,reader@1");
    assert_eq!(
        cx.log,
        [
            "settle reader",
            "undo reader",
            "settle memory",
            "undo memory"
        ]
    );
    assert_eq!(reg.active(), [("memory", "2"), ("reader", "1")]);

    let mut broken = probe(Manifest::new("memory", "3").provides("recall"));
    broken.fail_start = true;
    reg.request(Change::Swap("memory".into(), broken.boxed(), vec![]));
    let rows = reg.reconcile(SafePoint::Ask, &mut cx);
    assert_eq!(ops(&rows), [(Op::Refused, "memory")]);
    assert_eq!(reg.active(), [("memory", "2"), ("reader", "1")]);
    assert_eq!(cx.live.len(), 2);

    // A swap to a unit that no longer provides what the reader needs leaves it waiting.
    reg.request(Change::Swap(
        "memory".into(),
        probe(unit("plain")).boxed(),
        vec![],
    ));
    let rows = reg.reconcile(SafePoint::Ask, &mut cx);
    assert_eq!(ops(&rows), [(Op::Swap, "plain")]);
    assert_eq!(
        reg.fibers(),
        [
            ("plain", "1", Fiber::Active),
            ("reader", "1", Fiber::Inactive)
        ]
    );
}

#[test]
fn a_swap_to_another_id_unloads_what_ran_after_the_old_one() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    load(&mut reg, &mut cx, probe(unit("old")), &[]);
    load(&mut reg, &mut cx, probe(unit("next").after("old")), &[]);
    reg.request(Change::Swap(
        "old".into(),
        probe(unit("new")).boxed(),
        vec![],
    ));
    let rows = reg.reconcile(SafePoint::Batch, &mut cx);
    assert_eq!(ops(&rows), [(Op::Unload, "next"), (Op::Swap, "new")]);
    assert_eq!(reg.active(), [("new", "1")]);
    assert_eq!(cx.live, BTreeSet::from(["new".to_owned()]));
}

#[test]
fn rows_and_ops_are_written_in_snake_case() {
    let row = UnitsRow {
        op: Op::Quarantine,
        id: "plugin:x".into(),
        version: "0.1.0".into(),
        why: None,
        digest: String::new(),
    };
    let value = serde_json::to_value(&row).unwrap();
    assert_eq!(value["op"], "quarantine");
    assert_eq!(serde_json::from_value::<UnitsRow>(value).unwrap(), row);
    assert_eq!(serde_json::to_value(Fiber::Inactive).unwrap(), "inactive");
}

#[test]
fn a_swap_is_checked_against_the_units_that_stay() {
    let mut reg = Registry::new();
    let mut cx = Cx::default();
    load(&mut reg, &mut cx, probe(unit("old")), &[]);
    load(&mut reg, &mut cx, probe(unit("next").after("old")), &[]);
    // `next` leaves with `old`, so a newcomer that runs after it has nothing to run after.
    let newcomer = unit("new").after("next");
    reg.request(Change::Swap("old".into(), probe(newcomer).boxed(), vec![]));
    let rows = reg.reconcile(SafePoint::Batch, &mut cx);
    assert_eq!(ops(&rows), [(Op::Refused, "new")]);
    assert_eq!(
        rows[0].why.as_deref(),
        Some("it runs after next, which is not loaded")
    );
    assert_eq!(reg.active(), [("old", "1"), ("next", "1")]);
}
