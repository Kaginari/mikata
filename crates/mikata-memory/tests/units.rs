//! Which unit wrote a note: its `unit@version` stamp, read from old rows as none, written while a
//! unit writes, and the human's withdrawal of everything one unit wrote.

use std::fs;
use std::path::{Path, PathBuf};

use mikata_memory::{
    Proof, Status, Tier, UnitStamp, as_unit, cache, fold, ground, propose, read, withdraw_by,
    written_by,
};

fn world_of(name: &str) -> PathBuf {
    let w = std::env::temp_dir().join(format!("mikata-units-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(&w).unwrap();
    w
}

fn status_in(w: &Path, tier: &Tier, note: &str) -> Option<Status> {
    fold(&read(w, tier).unwrap().rows)
        .get(note)
        .map(|n| n.status)
}

/// A stamp is `unit@version`, each part plain: it reads back as it was written, and anything else
/// is refused.
#[test]
fn a_stamp_is_unit_at_version() {
    let s: UnitStamp = "plugin:cognee@0.1.0".parse().unwrap();
    assert_eq!(s.as_str(), "plugin:cognee@0.1.0");
    assert_eq!(s, UnitStamp::new("plugin:cognee", "0.1.0").unwrap());
    assert_eq!(s.to_string(), "plugin:cognee@0.1.0");
    for bad in ["memory", "@1", "memory@", "memory@1@2", "mem ory@1", "memory@1 ", "mémoire@1"] {
        assert!(bad.parse::<UnitStamp>().is_err(), "{bad}");
    }
    assert!(UnitStamp::new(&"a".repeat(65), "1").is_err());
}

/// A row written before units has no stamp and reads as none; a row with one keeps it, and its
/// note says which unit created it. A row without a stamp is written without the field, as before.
#[test]
fn old_rows_read_and_unstamped_rows_are_written_as_before() {
    let w = world_of("old-rows");
    let file = w.join(Tier::Shared.file());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        concat!(
            r#"{"id":"1-a","time":1,"note":"n1","by":"human","proof":"log.md #1","kind":"create","text":"old","polarity":"support","salience":0.5,"confidence":0.5}"#,
            "\n",
            r#"{"id":"2-a","time":2,"note":"n2","by":"record","unit":"memory@1.0.0","kind":"create","text":"new","polarity":"support","salience":0.5,"confidence":0.5}"#,
            "\n"
        ),
    )
    .unwrap();
    let back = read(&w, &Tier::Shared).unwrap();
    assert_eq!(back.rows.len(), 2);
    assert_eq!(back.rows[0].unit, None);
    assert_eq!(back.rows[1].unit.as_deref(), Some("memory@1.0.0"));
    let notes = fold(&back.rows);
    assert_eq!(notes["n1"].unit, None);
    assert_eq!(notes["n2"].unit.as_deref(), Some("memory@1.0.0"));
    let plain = propose(&w, "slime-s", "plain words", vec![], None).unwrap();
    let text = fs::read_to_string(w.join(Tier::Personal("slime-s".into()).file())).unwrap();
    assert!(!text.contains("\"unit\""), "{text}");
    assert!(text.contains(&plain.note));
    let _ = fs::remove_dir_all(&w);
}

/// While a unit writes, every event it appends is stamped, through any writer; the stamp in force
/// before is put back after, a panic included, and other threads are never stamped.
#[test]
fn a_unit_stamps_what_it_writes_and_no_more() {
    let w = world_of("stamps");
    let memory: UnitStamp = "memory@1.0.0".parse().unwrap();
    let other: UnitStamp = "plugin:cognee@0.1.0".parse().unwrap();
    let personal = Tier::Personal("slime-s".into());
    let (a, b, c) = as_unit(&memory, || {
        let a = propose(&w, "slime-s", "the lexer lives in src/lexer.rs", vec![], None).unwrap();
        let b = as_unit(&other, || {
            propose(&w, "slime-s", "the parser lives in src/parser.rs", vec![], None).unwrap()
        });
        let c = propose(&w, "slime-s", "the tests live in tests/", vec![], None).unwrap();
        // Another thread writes as no unit.
        let w2 = w.clone();
        std::thread::spawn(move || propose(&w2, "slime-s", "unstamped", vec![], None).unwrap())
            .join()
            .unwrap();
        (a.note, b.note, c.note)
    });
    let d = propose(&w, "slime-s", "after", vec![], None).unwrap().note;
    let caught = std::panic::catch_unwind(|| as_unit::<()>(&other, || panic!("a unit failed")));
    assert!(caught.is_err());
    let e = propose(&w, "slime-s", "after the panic", vec![], None).unwrap().note;
    let notes = fold(&read(&w, &personal).unwrap().rows);
    let unit = |id: &str| notes[id].unit.clone();
    assert_eq!(unit(&a).as_deref(), Some("memory@1.0.0"));
    assert_eq!(unit(&b).as_deref(), Some("plugin:cognee@0.1.0"));
    assert_eq!(unit(&c).as_deref(), Some("memory@1.0.0"));
    assert_eq!(unit(&d), None);
    assert_eq!(unit(&e), None);
    let unstamped = notes.values().find(|n| n.text == "unstamped").unwrap();
    assert_eq!(unstamped.unit, None);
    let _ = fs::remove_dir_all(&w);
}

/// Withdrawing by a unit puts to sleep every note it wrote that is awake, in every tier its notes
/// reached — a grounding's copy in shared memory and a short tier's cache included — and nothing
/// else: another unit's notes and the notes no unit wrote stay as they were.
#[test]
fn withdraw_by_puts_every_note_a_unit_wrote_to_sleep() {
    let w = world_of("withdraw-by");
    let bad: UnitStamp = "plugin:noisy@0.2.0".parse().unwrap();
    let good: UnitStamp = "memory@1.0.0".parse().unwrap();
    let personal = Tier::Personal("slime-s".into());
    let short = Tier::Short("slime-s".into());
    let (grounded, proposed, cached) = as_unit(&bad, || {
        let g = ground(
            &w,
            propose(&w, "slime-s", "always skip the fixtures", vec![], None).unwrap(),
            &Proof::confirmed("the human"),
        )
        .unwrap()
        .note;
        let p = propose(&w, "slime-s", "never run the slow tests", vec![], None)
            .unwrap()
            .note;
        let c = cache(&w, "slime-s", "fixtures are slow").unwrap();
        (g, p, c)
    });
    let kept = as_unit(&good, || {
        propose(&w, "slime-s", "the lexer lives in src/lexer.rs", vec![], None).unwrap()
    })
    .note;
    let plain = propose(&w, "slime-s", "plain words", vec![], None)
        .unwrap()
        .note;
    assert_eq!(status_in(&w, &Tier::Shared, &grounded), Some(Status::Grounded));

    let mut found: Vec<(String, String)> = written_by(&w, &bad)
        .unwrap()
        .into_iter()
        .map(|x| (x.tier.file().display().to_string(), x.note))
        .collect();
    found.sort();
    let mut want = vec![
        (Tier::Shared.file().display().to_string(), grounded.clone()),
        (personal.file().display().to_string(), grounded.clone()),
        (personal.file().display().to_string(), proposed.clone()),
        (short.file().display().to_string(), cached.clone()),
    ];
    want.sort();
    assert_eq!(found, want);

    let gone = withdraw_by(&w, &bad, "log.md 2026-11-24 withdrawn").unwrap();
    assert_eq!(gone.len(), 4);
    for (tier, note) in [
        (&Tier::Shared, &grounded),
        (&personal, &grounded),
        (&personal, &proposed),
        (&short, &cached),
    ] {
        assert_eq!(status_in(&w, tier, note), Some(Status::Dormant), "{note}");
    }
    assert_eq!(status_in(&w, &personal, &kept), Some(Status::Proposed));
    assert_eq!(status_in(&w, &personal, &plain), Some(Status::Proposed));
    // The human's word is written, never a unit's, and it cites its proof.
    let rows = read(&w, &Tier::Shared).unwrap().rows;
    let last = rows.last().unwrap();
    assert_eq!(last.proof.as_deref(), Some("log.md 2026-11-24 withdrawn"));
    assert_eq!(last.unit, None);
    // A second withdrawal finds nothing awake.
    assert!(withdraw_by(&w, &bad, "log.md again").unwrap().is_empty());
    assert!(mikata_memory::findings(&w, "slime-s").unwrap().is_empty());
    let _ = fs::remove_dir_all(&w);
}
