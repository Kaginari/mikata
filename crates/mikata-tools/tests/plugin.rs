//! The plugin host against a fake plugin (`fixtures/fake-plugin.sh`) and, when `python3` is on
//! the path, the example plugin of `examples/plugins/`: the handshake, a veto, a note, a missed
//! deadline, a crash, a malformed reply, a reply above the grant, three strikes and an unload.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mikata_kernel::{Band, Event, Grant, Point, Reply};
use mikata_races::Race;
use mikata_tools::plugin::{Declared, Offence, Plugin, STRIKES};
use mikata_tools::{Context, NoHuman, Sandbox};
use serde_json::{Value, json};

/// The deadline on a hook in these tests: the fake's slow answer takes a second.
const DEADLINE: Duration = Duration::from_millis(300);

struct World {
    root: PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("mikata-plugin-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("world")).unwrap();
        Self { root }
    }

    fn log(&self) -> PathBuf {
        self.root.join("log")
    }

    fn lines(&self) -> Vec<String> {
        std::fs::read_to_string(self.log())
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn pid(&self) -> u32 {
        self.lines()
            .iter()
            .find_map(|l| l.strip_prefix("pid ")?.parse().ok())
            .expect("the plugin logs its pid")
    }

    fn cx(&self) -> Context {
        Context::new(Race::Slime, self.root.join("world"), &["."])
            .with_sandbox(Sandbox::None("the test runs unguarded".into()))
            .with_unguarded(true)
            .with_gate(Arc::new(NoHuman))
            .with_body("slime-scout")
    }

    fn start(&self, mode: Option<&str>, grants: &[Grant]) -> Result<Plugin, String> {
        let mut args = vec![fixture().display().to_string(), self.log().display().to_string()];
        args.extend(mode.map(str::to_owned));
        let declared = Declared {
            id: "fake".into(),
            command: "bash".into(),
            args,
            env: Vec::new(),
            grants: grants.to_vec(),
            races: vec![Race::Slime],
        };
        Plugin::start_within(
            &declared,
            &self.cx(),
            &self.root.join("scratch"),
            DEADLINE,
            Duration::from_secs(10),
        )
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-plugin.sh")
}

fn bash(id: &str, command: &str) -> Event {
    Event::ToolBefore {
        id: id.into(),
        name: "bash".into(),
        input: json!({ "command": command }),
    }
}

fn named(name: &str) -> Event {
    Event::ToolBefore {
        id: "t".into(),
        name: name.into(),
        input: Value::Null,
    }
}

fn running(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

#[test]
fn the_handshake_gives_a_manifest_cut_to_the_grant() {
    let w = World::new("handshake");
    let p = w.start(None, &[Grant::Veto, Grant::Tools]).unwrap();
    let m = p.manifest();
    assert_eq!(m.stamp(), "plugin:fake@0.1.0");
    assert_eq!(
        m.points,
        vec![
            (Point::ToolBefore, Band::Outer),
            (Point::AnswerLand, Band::Outer)
        ]
    );
    // It asked for advise, veto and stop; the human granted veto and tools.
    assert_eq!(m.grants, vec![Grant::Veto]);
}

#[test]
fn a_plain_mcp_server_is_no_plugin() {
    let w = World::new("plain");
    let err = w.start(Some("plain"), &[Grant::Veto]).unwrap_err();
    assert!(err.contains("plain MCP server"), "{err}");
}

#[test]
fn a_veto_and_a_note_are_answered_and_a_pass_is_a_pass() {
    let w = World::new("answers");
    let mut p = w.start(None, &[Grant::Advise, Grant::Veto]).unwrap();
    assert_eq!(
        p.on(&named("veto")),
        (
            Reply::Veto {
                why: "not this one".into()
            },
            None
        )
    );
    let (note, strike) = p.on(&named("note"));
    assert_eq!(strike, None);
    // The escape is cleaned out of what the body will read.
    assert_eq!(
        note,
        Reply::Note {
            text: "mind the tests [31m".into()
        }
    );
    assert_eq!(p.on(&named("read")), (Reply::Pass, None));
    let landed = Event::AnswerLand {
        answer: "done".into(),
    };
    assert_eq!(p.on(&landed), (Reply::Pass, None));
    // A point it does not hook is not sent.
    assert_eq!(
        p.on(&Event::AskBegin { ask: "x".into() }),
        (Reply::Pass, None)
    );
    assert!(w.lines().iter().all(|l| !l.starts_with("hook ask_begin")));
    assert_eq!(p.strikes(), 0);
    assert_eq!(p.failed(), None);
}

#[test]
fn a_reply_above_the_grant_or_out_of_place_or_malformed_is_a_pass_and_a_strike() {
    let w = World::new("above");
    let mut p = w.start(None, &[Grant::Advise]).unwrap();
    let (reply, strike) = p.on(&named("veto"));
    assert_eq!(reply, Reply::Pass);
    let strike = strike.unwrap();
    assert_eq!(strike.fault, Offence::AboveGrant);
    assert!(strike.why.contains("needs the veto grant"), "{}", strike.why);
    assert_eq!((strike.count, strike.failed), (1, false));

    let (reply, strike) = p.on(&named("garbage"));
    assert_eq!(reply, Reply::Pass);
    assert_eq!(strike.unwrap().fault, Offence::Malformed);
    assert_eq!(p.strikes(), 2);
    // The plugin still answers after both.
    assert_eq!(
        p.on(&named("note")).0,
        Reply::Note {
            text: "mind the tests [31m".into()
        }
    );
}

#[test]
fn a_stop_where_it_does_not_count_is_malformed() {
    let w = World::new("stop");
    let mut p = w.start(None, &[Grant::Stop]).unwrap();
    let (reply, strike) = p.on(&named("stop"));
    assert_eq!(reply, Reply::Pass);
    let strike = strike.unwrap();
    assert_eq!(strike.fault, Offence::Malformed);
    assert!(strike.why.contains("does not count"), "{}", strike.why);
}

#[test]
fn a_missed_deadline_is_a_strike_and_the_plugin_answers_again() {
    let w = World::new("late");
    let mut p = w.start(None, &[Grant::Veto]).unwrap();
    let began = Instant::now();
    let (reply, strike) = p.on(&named("slow"));
    assert!(began.elapsed() < Duration::from_millis(900));
    assert_eq!(reply, Reply::Pass);
    assert_eq!(strike.unwrap().fault, Offence::Late);
    // The fake answers one message at a time: once its slow answer is out, that late pass is
    // passed over and the next hook is answered as asked.
    std::thread::sleep(Duration::from_millis(900));
    assert_eq!(
        p.on(&named("veto")).0,
        Reply::Veto {
            why: "not this one".into()
        }
    );
}

#[test]
fn three_strikes_fail_the_plugin_and_end_it() {
    let w = World::new("strikes");
    let mut p = w.start(None, &[Grant::Veto]).unwrap();
    let pid = w.pid();
    for n in 1..=STRIKES {
        let (reply, strike) = p.on(&named("slow"));
        assert_eq!(reply, Reply::Pass);
        let strike = strike.unwrap();
        assert_eq!(strike.count, n);
        assert_eq!(strike.failed, n == STRIKES);
    }
    let why = p.failed().unwrap();
    assert!(why.contains("3 strikes"), "{why}");
    assert!(!running(pid), "a failed plugin is ended at once");
    // It is called no more.
    let hooks = w.lines().iter().filter(|l| l.starts_with("hook")).count();
    assert_eq!(p.on(&named("veto")), (Reply::Pass, None));
    assert_eq!(
        w.lines().iter().filter(|l| l.starts_with("hook")).count(),
        hooks
    );
}

#[test]
fn a_crash_is_a_strike_at_every_hook_until_it_fails() {
    let w = World::new("crash");
    let mut p = w.start(None, &[Grant::Veto]).unwrap();
    let (reply, strike) = p.on(&named("crash"));
    assert_eq!(reply, Reply::Pass);
    assert_eq!(strike.unwrap().fault, Offence::Crashed);
    let began = Instant::now();
    for _ in 1..STRIKES {
        assert_eq!(p.on(&named("veto")).1.unwrap().fault, Offence::Crashed);
    }
    // A gone plugin is not waited on.
    assert!(began.elapsed() < DEADLINE);
    assert!(p.failed().is_some());
}

#[test]
fn an_unload_is_sent_and_the_plugin_ends() {
    let w = World::new("unload");
    let mut p = w.start(None, &[Grant::Veto]).unwrap();
    let pid = w.pid();
    p.unload();
    assert!(w.lines().contains(&"unload".to_owned()));
    assert!(!running(pid));
    // A second unload does nothing.
    p.unload();
    assert_eq!(p.on(&named("veto")), (Reply::Pass, None));
}

#[test]
fn a_plugin_that_ignores_its_unload_is_ended_after_a_second() {
    let w = World::new("deaf");
    let mut p = w.start(Some("deaf"), &[Grant::Veto]).unwrap();
    let pid = w.pid();
    let began = Instant::now();
    p.unload();
    let took = began.elapsed();
    assert!(took >= Duration::from_millis(900), "{took:?}");
    assert!(took < Duration::from_secs(5), "{took:?}");
    assert!(w.lines().contains(&"unload".to_owned()));
    assert!(!running(pid));
}

/// The example plugin, run by `python3` when it is on the path.
#[test]
fn the_python_example_vetoes_a_force_push() {
    let Some(python) = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|d| d.join("python3"))
            .find(|p| p.is_file())
    }) else {
        eprintln!("skipped: no python3 on the PATH, so the Python example plugin is not run");
        return;
    };
    let w = World::new("python");
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins");
    let declared = Declared {
        id: "no-force-push".into(),
        command: python.display().to_string(),
        args: vec![
            example
                .join("no_force_push.py")
                .canonicalize()
                .unwrap()
                .display()
                .to_string(),
        ],
        env: Vec::new(),
        grants: vec![Grant::Advise, Grant::Veto],
        races: vec![Race::Slime],
    };
    let mut p = Plugin::start(
        &declared,
        &w.cx(),
        &w.root.join("scratch"),
        Duration::from_secs(5),
    )
    .unwrap();
    assert_eq!(p.manifest().id, "plugin:no-force-push");
    assert_eq!(p.manifest().points, vec![(Point::ToolBefore, Band::Outer)]);
    assert_eq!(p.manifest().grants, vec![Grant::Veto]);
    for command in [
        "git push --force origin main",
        "cd repo && git push -f",
        "git push origin +main --force-with-lease",
        "git -C repo push -uf origin main",
    ] {
        let (reply, strike) = p.on(&bash("t1", command));
        assert_eq!(strike, None, "{command}");
        assert!(matches!(reply, Reply::Veto { .. }), "{command}: {reply:?}");
    }
    for command in [
        "git push origin main",
        "git status -f",
        "git fetch --force",
        "echo git push --force",
    ] {
        assert_eq!(
            p.on(&bash("t2", command)),
            (Reply::Pass, None),
            "{command}"
        );
    }
    assert_eq!(p.on(&named("read")), (Reply::Pass, None));
    let began = Instant::now();
    p.unload();
    assert!(began.elapsed() < Duration::from_secs(2));
}
