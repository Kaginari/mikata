//! What changed, by the body's own checks: they are run again on a copy of the finished work and
//! on the same copy put back to the tree as the ask found it. The world itself is never run in.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use mikata_loop::changes;
use mikata_races::Race;
use mikata_tools::{Attic, Context, Sandbox, registry};

struct Ground {
    root: PathBuf,
}

impl Ground {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("mikata-changes-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("world")).unwrap();
        fs::write(root.join("world/lib.py"), "def f(x):\n    return x + 1\n").unwrap();
        Self { root }
    }

    fn world(&self) -> PathBuf {
        self.root.join("world").canonicalize().unwrap()
    }

    fn attic(&self) -> Attic {
        Attic::at(&self.root.join("attic"), &self.world()).expect("git is installed")
    }

    fn cx(&self) -> Context {
        Context::new(Race::Slime, self.world(), &["."])
            .with_sandbox(Sandbox::None("these tests run unguarded".into()))
            .with_unguarded(true)
    }
}

impl Drop for Ground {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn python() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// The work of every test here: `f` answers differently, and `g` is new.
const WORK: &str = "def f(x):\n    return x + 2\n\ndef g():\n    return 'new'\n";

#[test]
fn each_check_is_sorted_by_what_it_prints_on_the_two_trees() {
    if !python() {
        eprintln!("no python3 here: skipped");
        return;
    }
    let g = Ground::new("sorted");
    let world = g.world();
    let attic = g.attic();
    let start = attic.snapshot("before the ask").unwrap();
    fs::write(world.join("lib.py"), WORK).unwrap();
    let differs = format!(
        r#"cd {} && python3 -c "import lib; print('asked: f(1)'); print('f(1) =', lib.f(1))""#,
        world.display()
    );
    let checks = [
        differs.as_str(),
        r#"python3 -c "print('steady')""#,
        r#"python3 -c "import lib; print(lib.g())""#,
        r#"python3 -c "import os; print(os.getpid())""#,
        r#"python3 -c "raise SystemExit(3)""#,
    ];
    let r = changes::compare(&world, &g.cx(), &attic, &start, &checks, &registry());
    assert_eq!(r.why_not, None, "{r:?}");
    assert_eq!(
        (
            r.taken,
            r.same,
            r.only_now,
            r.unsteady,
            r.failed_now,
            r.not_reached
        ),
        (5, 1, 1, 1, 1, 0),
        "{r:?}"
    );
    assert_eq!(r.differ.len(), 1, "{r:?}");
    assert_eq!(r.differ[0].command, differs);
    assert_eq!(r.differ[0].before, "asked: f(1)\nf(1) = 2");
    assert_eq!(r.differ[0].now, "asked: f(1)\nf(1) = 3");
    assert_eq!(
        changes::said(&r, "Mikata").unwrap(),
        "what changed: of Mikata's last 5 checks, 1 prints differently than on the code you gave it \
         (1 runs only on the finished work, 1 fails on the finished work, 1 does not print the same \
         twice):\n\n  asked: f(1)\n- f(1) = 2\n+ f(1) = 3\n\nthe 1 check whole: .mikata/tmp/what-changed.md"
    );
}

#[test]
fn the_world_itself_is_never_run_in_nor_written() {
    if !python() {
        eprintln!("no python3 here: skipped");
        return;
    }
    let g = Ground::new("untouched");
    let world = g.world();
    let attic = g.attic();
    let start = attic.snapshot("before the ask").unwrap();
    fs::write(world.join("lib.py"), WORK).unwrap();
    // A check that leaves a file behind, and names its world in full.
    let leaves = format!(
        r#"cd {w} && python3 -c "import lib; open('{w}/made.txt', 'w').write('x'); print(lib.f(1))""#,
        w = world.display()
    );
    let r = changes::compare(
        &world,
        &g.cx(),
        &attic,
        &start,
        &[leaves.as_str()],
        &registry(),
    );
    assert_eq!(r.differ.len(), 1, "{r:?}");
    assert_eq!(
        (r.differ[0].before.as_str(), r.differ[0].now.as_str()),
        ("2", "3")
    );
    assert!(
        !world.join("made.txt").exists(),
        "the check ran in the copy"
    );
    assert!(
        !world.join("__pycache__").exists(),
        "nothing was imported in the world"
    );
    assert_eq!(fs::read_to_string(world.join("lib.py")).unwrap(), WORK);
    // The copy is gone with the step.
    let left: Vec<_> = fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("mikata-changes-{}-", std::process::id()))
        })
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn no_check_is_nothing_to_say() {
    let g = Ground::new("none");
    let world = g.world();
    let attic = g.attic();
    let start = attic.snapshot("before the ask").unwrap();
    let r = changes::compare(&world, &g.cx(), &attic, &start, &[], &registry());
    assert_eq!(r, changes::Report::default());
    assert_eq!(changes::said(&r, "Mikata"), None);
}

/// A check leaves compiled Python beside its source, keyed by the source's size and its time to
/// the second. A file put back at the same size within that second would be read from the other
/// tree's cache: each tree is run in a copy of its own, so it is not.
#[test]
fn a_change_of_the_same_size_within_the_second_is_still_seen() {
    if !python() {
        eprintln!("no python3 here: skipped");
        return;
    }
    let g = Ground::new("same-size");
    let world = g.world();
    let attic = g.attic();
    let start = attic.snapshot("before the ask").unwrap();
    // One character differs, and the body's own run has left its cache in the world.
    fs::write(world.join("lib.py"), "def f(x):\n    return x + 2\n").unwrap();
    let check = r#"python3 -c "import lib; print(lib.f(1))""#;
    assert!(
        Command::new("bash")
            .args(["-c", check])
            .current_dir(&world)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        world.join("__pycache__").exists(),
        "the body's run cached it"
    );
    let r = changes::compare(&world, &g.cx(), &attic, &start, &[check], &registry());
    assert_eq!(r.differ.len(), 1, "{r:?}");
    assert_eq!(
        (r.differ[0].before.as_str(), r.differ[0].now.as_str()),
        ("2", "3")
    );
}

/// A kept run, asked again what it would have been shown: its world as it was left, its attic,
/// the tree its ask began on, and its journal, whose shell commands are sorted as the loop sorts
/// them. No model call. Run by hand, with the four paths named in the environment:
///
/// ```sh
/// MIKATA_REPLAY_WORLD=… MIKATA_REPLAY_ATTIC=… MIKATA_REPLAY_START=<tree> MIKATA_REPLAY_JOURNAL=… \
///   tools/test-env.sh cargo nextest run -p mikata-loop --test changes --run-ignored only \
///   --no-capture replay
/// ```
#[test]
#[ignore = "needs a kept run: see its comment"]
fn replay_what_a_kept_run_would_have_been_shown() {
    let var = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("set {name}"));
    let world = PathBuf::from(var("MIKATA_REPLAY_WORLD"));
    let attic = Attic::at(&PathBuf::from(var("MIKATA_REPLAY_ATTIC")), &world).expect("git");
    let start = mikata_tools::Snapshot {
        tree: var("MIKATA_REPLAY_START"),
        skipped: Vec::new(),
    };
    let mut checks = changes::Checks::default();
    for line in fs::read_to_string(var("MIKATA_REPLAY_JOURNAL"))
        .unwrap()
        .lines()
    {
        let Ok(beat) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        for block in beat["content"].as_array().into_iter().flatten() {
            if block["type"] == "tool_use" && block["name"] == "bash" {
                checks.saw(block["input"]["command"].as_str());
            }
        }
    }
    let cx = Context::new(Race::Slime, &world, &["."])
        .with_sandbox(Sandbox::None("a replay by hand runs unguarded".into()))
        .with_unguarded(true);
    let newest = checks.newest(changes::MOST);
    let r = changes::compare(&world, &cx, &attic, &start, &newest, &registry());
    println!(
        "taken {} same {} differ {} only_now {} failed_now {} unsteady {} not_reached {} in {} ms\n",
        r.taken,
        r.same,
        r.differ.len(),
        r.only_now,
        r.failed_now,
        r.unsteady,
        r.not_reached,
        r.ms
    );
    println!("{}", changes::said(&r, "Mikata").unwrap_or_default());
}
