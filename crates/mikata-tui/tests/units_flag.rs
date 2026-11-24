//! `mikata run --units`: the composition one run asks for, read beside the command, and refused
//! anywhere but on `run`.

use mikata_loop::config::{Composition, UnitChange};
use mikata_tui::cli::{Command, parse, units};

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

/// `run` takes a composition, and the command reads as it did without one.
#[test]
fn run_takes_a_composition() {
    let a = args(&["run", "go", "--units", "-memory,+plugin:no-force-push"]);
    assert!(matches!(parse(&a), Ok(Command::Run { ask: Some(ref s), .. }) if s == "go"));
    let c = units(&a).unwrap().unwrap();
    assert_eq!(
        c.changes(),
        [
            UnitChange {
                id: "memory".into(),
                on: false
            },
            UnitChange {
                id: "plugin:no-force-push".into(),
                on: true
            },
        ]
    );
    assert_eq!(units(&args(&["run", "go"])), Ok(None));
    assert_eq!(
        units(&args(&["run", "--units", "", "go"])),
        Ok(Some(Composition::default()))
    );
}

/// A composition that does not read, a second `--units`, one with no list, or one on another
/// command is refused before anything runs.
#[test]
fn a_composition_is_refused_where_it_does_not_belong() {
    for (a, says) in [
        (&["run", "go", "--units", "memory"][..], "+<id>"),
        (&["run", "go", "--units"][..], "--units needs a list"),
        (
            &["run", "go", "--units", "-memory", "--units", "-reader"][..],
            "given once",
        ),
        (&["--units", "-memory"][..], "--units goes with run"),
        (
            &["status", "--units", "-memory"][..],
            "--units goes with run",
        ),
    ] {
        let e = parse(&args(a)).unwrap_err();
        assert!(e.contains(says), "{a:?}: {e}");
    }
}
