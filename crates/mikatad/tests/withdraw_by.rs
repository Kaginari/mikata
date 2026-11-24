//! `mikatad memory withdraw --by <unit@version>`: the human takes back every note one unit wrote.

#[path = "support/isolated.rs"]
mod isolated;

use std::fs;
use std::path::Path;

use mikata_server::world::memory::{Status, Tier, UnitStamp, as_unit, fold, propose, read};

fn said(world: &Path, args: &[&str], body: Option<&str>) -> (bool, String, String) {
    let mut c = isolated::bin(env!("CARGO_BIN_EXE_mikatad"));
    c.args(args).args(["--world", world.to_str().unwrap()]);
    match body {
        Some(b) => c.env("MIKATA_BODY", b),
        None => c.env_remove("MIKATA_BODY"),
    };
    let o = c.output().unwrap();
    (
        o.status.success(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// The human withdraws a unit's notes by its stamp, written to the journal; a body may not, a
/// stamp that does not read is refused, and a second withdrawal finds nothing awake.
#[test]
fn the_human_withdraws_a_unit_s_notes_by_its_stamp() {
    let world = std::env::temp_dir().join(format!("mikata-withdraw-by-{}", std::process::id()));
    let _ = fs::remove_dir_all(&world);
    fs::create_dir_all(world.join(".mikata")).unwrap();
    fs::write(world.join(".mikata/log.md"), "\n").unwrap();
    let stamp: UnitStamp = "plugin:noisy@0.2.0".parse().unwrap();
    let note = as_unit(&stamp, || {
        propose(&world, "summoner", "skip the slow tests", vec![], None).unwrap()
    })
    .note;
    let other = propose(&world, "summoner", "the lexer is in src/", vec![], None)
        .unwrap()
        .note;
    let journal = || fs::read_to_string(world.join(".mikata/log.md")).unwrap();
    let before = journal();

    let (ok, _, err) = said(
        &world,
        &["memory", "withdraw", "--by", "plugin:noisy@0.2.0"],
        Some("summoner"),
    );
    assert!(
        !ok && err.contains("the human's act, never a body's"),
        "{err}"
    );
    let (ok, _, err) = said(&world, &["memory", "withdraw", "--by", "noisy"], None);
    assert!(!ok && err.contains("no unit's stamp"), "{err}");
    let (ok, _, err) = said(&world, &["memory", "withdraw", "--by"], None);
    assert!(!ok && err.contains("--by needs one unit's stamp"), "{err}");
    let (ok, _, err) = said(&world, &["memory", "withdraw", "x", "--by", "a@1"], None);
    assert!(!ok && err.contains("not both"), "{err}");
    assert_eq!(journal(), before, "a refusal writes nothing");

    let (ok, out, err) = said(
        &world,
        &["memory", "withdraw", "--by", "plugin:noisy@0.2.0"],
        None,
    );
    assert!(ok, "{err}");
    assert!(out.contains(&format!("withdrawn: {note} in ")), "{out}");
    assert!(out.contains("1 note(s)"), "{out}");
    assert!(
        journal().contains("the human — withdrawn: the notes of plugin:noisy@0.2.0"),
        "{}",
        journal()
    );
    let notes = fold(
        &read(&world, &Tier::Personal("summoner".into()))
            .unwrap()
            .rows,
    );
    assert_eq!(notes[&note].status, Status::Dormant);
    assert_eq!(notes[&other].status, Status::Proposed);

    let (ok, _, err) = said(
        &world,
        &["memory", "withdraw", "--by", "plugin:noisy@0.2.0"],
        None,
    );
    assert!(!ok && err.contains("is awake"), "{err}");
    let _ = fs::remove_dir_all(&world);
}
