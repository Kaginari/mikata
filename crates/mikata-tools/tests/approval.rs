//! The approval unit as the tools hold it: on, an outward or destructive act is put to the human;
//! switched off by the human, it runs without asking and is journaled so, never as approved.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mikata_races::{Race, ToolClass};
use mikata_sandbox::Sandbox;
use mikata_tools::gate::Scripted;
use mikata_tools::{Answer, Ask, Context, call, registry};

fn root(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("mikata-approval-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("world/src")).unwrap();
    fs::create_dir_all(root.join("outside")).unwrap();
    root
}

/// A deploy post: its acts write under the deploy root, outside the world, and are asked.
fn deploy(root: &std::path::Path, gate: Arc<Scripted>) -> Context {
    Context::new(Race::Kijin, root.join("world"), &["src"])
        .with_body("kijin-deploy")
        .with_deploy_roots(&[root.join("outside")])
        .with_gate(gate)
        .with_home("/h")
        .with_sandbox(Sandbox::None("these tests check the gate".into()))
        .with_unguarded(true)
}

fn bash(cx: &mut Context, command: &str) -> mikata_tools::Outcome {
    let args = serde_json::json!({ "command": command }).to_string();
    call(&registry(), "bash", &args, cx)
}

#[test]
fn on_an_outward_act_is_put_to_the_human() {
    let root = root("on");
    let marker = root.join("outside/marker");
    let gate = Arc::new(Scripted::new([Answer::Deny("not now".into())]));
    let mut cx = deploy(&root, gate.clone());
    assert!(cx.approval_on(), "on unless the human switches it off");
    let o = bash(&mut cx, &format!("echo hi > {}", marker.display()));
    assert!(o.is_error && o.content.contains("not now"), "{o:?}");
    assert_eq!(gate.asked().len(), 1);
    assert!(!marker.exists());
}

#[test]
fn off_an_outward_act_runs_without_asking_and_says_so() {
    let root = root("off");
    let marker = root.join("outside/marker");
    let gate = Arc::new(Scripted::new([]));
    let mut cx = deploy(&root, gate.clone());
    cx.set_approval_on(false);
    let o = bash(&mut cx, &format!("echo hi > {}", marker.display()));
    assert!(!o.is_error, "{o:?}");
    assert_eq!(fs::read_to_string(&marker).unwrap(), "hi\n");
    assert_eq!(gate.asked().len(), 0, "nobody was asked");
    assert!(
        o.content
            .contains("run without asking: the approval unit is off"),
        "{}",
        o.content
    );
    assert!(
        !o.content.contains("approved by the human"),
        "{}",
        o.content
    );
}

#[test]
fn off_the_humans_own_gate_still_asks() {
    let root = root("human");
    let gate = Arc::new(Scripted::new([Answer::Deny("no".into())]));
    let mut cx = deploy(&root, gate.clone());
    cx.set_approval_on(false);
    let ask = Ask {
        body: "mikata".into(),
        race: Race::Slime,
        class: ToolClass::Destructive,
        act: "land with tests changed".into(),
        why: Vec::new(),
        outside: Vec::new(),
        fingerprint: None,
    };
    assert!(matches!(cx.human_gate().ask(&ask), Answer::Deny(_)));
    assert_eq!(gate.asked().len(), 1);
}
