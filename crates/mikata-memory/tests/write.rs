//! The write rule: a body's words land personal and proposed; only a route's proof grounds them,
//! and every proof is refused without the evidence its route needs.

use std::fs;

use mikata_memory::{Proof, Route, Status, Tier, fold, ground, propose, read};

#[test]
fn every_proof_is_refused_without_its_evidence() {
    assert!(
        Proof::checked("log.md: gate PASS", &[], &["a.rs".into()]).is_err(),
        "no verify line ran"
    );
    assert!(
        Proof::checked("log.md: gate PASS", &["cargo test".into()], &[]).is_err(),
        "nothing changed"
    );
    assert_eq!(
        Proof::checked(
            "log.md: gate PASS",
            &["cargo test".into()],
            &["a.rs".into()]
        )
        .unwrap()
        .route(),
        Route::Checked
    );
    assert!(Proof::accepted("verdict", false).is_err());
    let anchors = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert!(
        Proof::corroborated("slime-a", "slime-a", &anchors(&["x"]), &anchors(&["y"])).is_err(),
        "no self-corroboration"
    );
    assert!(
        Proof::corroborated(
            "slime-a",
            "slime-b",
            &anchors(&["src/x.rs"]),
            &anchors(&["src/x.rs"])
        )
        .is_err(),
        "one anchor is one piece of evidence"
    );
    assert!(Proof::corroborated("slime-a", "slime-b", &anchors(&[]), &anchors(&["y"])).is_err());
    assert_eq!(
        Proof::corroborated(
            "slime-a",
            "slime-b",
            &anchors(&["src/x.rs"]),
            &anchors(&["log.md#4"])
        )
        .unwrap()
        .route(),
        Route::Corroborated
    );
    assert_eq!(
        Proof::confirmed("the human, in session s1").route(),
        Route::Confirmed
    );
}

#[test]
fn a_claim_lands_proposed_and_a_route_grounds_it_in_both_tiers() {
    let w = std::env::temp_dir().join(format!("mikata-write-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(&w).unwrap();
    let claim = propose(
        &w,
        "slime-s",
        "the parser keeps interval fractions",
        vec!["law".into()],
        Some("src/parser.rs".into()),
    )
    .unwrap();
    let personal = Tier::Personal("slime-s".into());
    let mine = fold(&read(&w, &personal).unwrap().rows);
    let n = &mine[&claim.note];
    assert_eq!(
        (n.status, n.salience, n.confidence),
        (Status::Proposed, 0.6, 0.5)
    );
    assert!(
        fold(&read(&w, &Tier::Shared).unwrap().rows).is_empty(),
        "nothing unconfirmed is shared"
    );
    let proof = Proof::checked(
        "log.md: slime-s gate PASS",
        &["cargo build".into(), "cargo test -p parser".into()],
        &[
            "src/parser.rs".into(),
            "src/lexer.rs".into(),
            "README.md".into(),
        ],
    )
    .unwrap()
    .observes(
        &claim.text,
        claim.source.as_deref(),
        Some("cargo test -p parser"),
    )
    .unwrap();
    let grounded = ground(&w, claim, &proof).unwrap();
    let mine = fold(&read(&w, &personal).unwrap().rows);
    let shared = fold(&read(&w, &Tier::Shared).unwrap().rows);
    for n in [&mine[&grounded.note], &shared[&grounded.note]] {
        assert_eq!(n.status, Status::Grounded);
        assert!((n.confidence - 0.7).abs() < 1e-9, "a check grounds at 0.7");
        assert!(n.proof.as_deref().unwrap().starts_with(
            "checked: log.md: slime-s gate PASS (2 verify line(s) over 3 path(s)) — check: \
                 `cargo test -p parser`"
        ));
    }
    assert_eq!(shared[&grounded.note].owner.as_deref(), Some("slime-s"));
    let _ = fs::remove_dir_all(&w);
}

/// A run-ending failure: one grounded revise note per failed act, linking each recalled anchor —
/// 0.9 when cited, 0.4 when merely recalled — never a dissenting one; shared when any is shared.
#[test]
fn a_run_ending_failure_revises_what_it_rode_on() {
    use mikata_memory::{Anchor, Polarity, revise};
    let w = std::env::temp_dir().join(format!("mikata-revise-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(&w).unwrap();
    let anchor = |id: &str, personal: bool, dissent: bool, cited: bool| Anchor {
        id: id.into(),
        personal,
        dissent,
        cited,
    };
    // Nothing recalled, or only dissent: nothing written.
    assert_eq!(
        revise(&w, "slime-s", &["x".into()], &[], "log.md v").unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(
        revise(
            &w,
            "slime-s",
            &["x".into()],
            &[anchor("c", false, true, false)],
            "v"
        )
        .unwrap(),
        Vec::<String>::new()
    );
    let made = revise(
        &w,
        "slime-s",
        &[
            "traits hold: `cargo test` failed".into(),
            "right author: `lib/x` is not yours".into(),
        ],
        &[
            anchor("cited", true, false, true),
            anchor("bystander", false, false, false),
            anchor("dissent", false, true, false),
        ],
        "log.md ### [t] slime-s — gate: FAIL",
    )
    .unwrap();
    assert_eq!(made.len(), 2, "one per failed act");
    let shared = fold(&read(&w, &Tier::Shared).unwrap().rows);
    for id in &made {
        let n = &shared[id];
        assert_eq!((n.polarity, n.status), (Polarity::Revise, Status::Grounded));
        assert!((n.salience - 1.0).abs() < 1e-9 && (n.confidence - 0.9).abs() < 1e-9);
        let links: Vec<(&str, f64)> = n
            .links
            .iter()
            .map(|l| (l.note.as_str(), l.weight))
            .collect();
        assert_eq!(
            links,
            [("cited", 0.9), ("bystander", 0.4)],
            "never the dissent"
        );
    }
    // Only personal anchors: the revise lands in the owner's personal tier.
    let personal = revise(
        &w,
        "slime-s",
        &["x".into()],
        &[anchor("mine", true, false, false)],
        "v",
    )
    .unwrap();
    let mine = fold(&read(&w, &Tier::Personal("slime-s".into())).unwrap().rows);
    assert!(mine.contains_key(&personal[0]));
    let _ = fs::remove_dir_all(&w);
}

/// One failed rule is one revise note, however many lines it failed by: eight foreign paths are
/// one `right author` failure.
#[test]
fn one_rule_failed_on_many_lines_is_one_revise_note() {
    use mikata_memory::{Anchor, revise};
    let w = std::env::temp_dir().join(format!("mikata-revise-rules-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(&w).unwrap();
    let acts: Vec<String> = (0..8)
        .map(|i| format!("right author: `lib/x{i}.rs` is slime-b's, not slime-a's\nmore detail"))
        .collect();
    let made = revise(
        &w,
        "slime-a",
        &acts,
        &[Anchor {
            id: "n".into(),
            personal: false,
            dissent: false,
            cited: false,
        }],
        "log.md v",
    )
    .unwrap();
    assert_eq!(made.len(), 1);
    let n = &fold(&read(&w, &Tier::Shared).unwrap().rows)[&made[0]];
    assert!(
        n.text.starts_with("right author: `lib/x0.rs`"),
        "{}",
        n.text
    );
    assert!(
        !n.text.contains("more detail"),
        "only each line's first line"
    );
    let _ = fs::remove_dir_all(&w);
}

fn anchor(id: &str, personal: bool, cited: bool) -> mikata_memory::Anchor {
    mikata_memory::Anchor {
        id: id.into(),
        personal,
        dissent: false,
        cited,
    }
}

fn world_of(name: &str) -> std::path::PathBuf {
    let w = std::env::temp_dir().join(format!("mikata-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(&w).unwrap();
    w
}

fn grounded_note(w: &std::path::Path, owner: &str, text: &str) -> String {
    ground(
        w,
        propose(w, owner, text, vec![], None).unwrap(),
        &Proof::confirmed("the human"),
    )
    .unwrap()
    .note
}

/// A failure counts against a note only when what failed names what the note names: the first
/// doubts it, the second retires it. A note the failures never name is never touched, however
/// often it was cited.
#[test]
fn a_note_is_retired_only_by_failures_laid_at_its_door() {
    use mikata_memory::{retire, revise};
    let w = world_of("door");
    let bad = grounded_note(&w, "slime-s", "skip the fixture tests/fixtures/utf8.sql");
    let innocent = grounded_note(&w, "slime-s", "the lexer lives in src/lexer.rs");
    let anchors = [anchor(&bad, false, true), anchor(&innocent, false, true)];
    let fail = |v: &str| {
        revise(
            &w,
            "slime-s",
            &["traits hold: tests/fixtures/utf8.sql::test_round_trip failed".into()],
            &anchors,
            v,
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(3));
        retire(&w, "slime-s", &anchors, v).unwrap()
    };
    let first = fail("log.md one");
    assert_eq!((first.doubted, first.retired), (vec![bad.clone()], vec![]));
    let notes = fold(&read(&w, &Tier::Shared).unwrap().rows);
    assert!(
        (notes[&bad].confidence - 0.6).abs() < 1e-9,
        "doubted: 0.7 − 0.1"
    );
    let second = fail("log.md two");
    assert_eq!(
        (second.doubted, second.retired),
        (vec![], vec![bad.clone()])
    );
    let notes = fold(&read(&w, &Tier::Shared).unwrap().rows);
    assert_eq!(notes[&bad].status, Status::Dormant);
    assert_eq!(
        notes[&innocent].status,
        Status::Grounded,
        "the failures never named it"
    );
    assert!(
        (notes[&innocent].confidence - 0.7).abs() < 1e-9,
        "nor doubted it"
    );
    assert_eq!(
        mikata_memory::retired(&w, "slime-s").unwrap(),
        vec![(
            bad.clone(),
            "skip the fixture tests/fixtures/utf8.sql".to_owned()
        )]
    );
    assert!(
        mikata_memory::findings(&w, "slime-s").unwrap().is_empty(),
        "every event resolves"
    );
    let _ = fs::remove_dir_all(&w);
}

/// Passes are evidence too: a note cited by as many passing asks as failures stands. The human's
/// waking starts its evidence afresh, and a personal note's strikes are read from shared memory,
/// where `revise` wrote them.
#[test]
fn passes_outweigh_failures_and_a_waking_starts_clean() {
    use mikata_memory::{confirm, retire, revise, wake};
    let w = world_of("evidence");
    let mine = grounded_note(
        &w,
        "slime-s",
        "run uv before pytest in sqlglot/tests/conftest.py",
    );
    let shared = grounded_note(&w, "slime-o", "the dialects live in sqlglot/dialects/");
    let anchors = [anchor(&mine, true, true), anchor(&shared, false, false)];
    for v in ["log.md pass one", "log.md pass two"] {
        assert_eq!(confirm(&w, "slime-s", &anchors, v).unwrap(), 1);
    }
    let fail = |v: &str| {
        revise(
            &w,
            "slime-s",
            &["tests intact: sqlglot/tests/conftest.py could not import".into()],
            &anchors,
            v,
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(3));
        retire(&w, "slime-s", &anchors, v).unwrap()
    };
    assert_eq!(fail("log.md one").doubted, vec![mine.clone()]);
    assert_eq!(
        fail("log.md two").retired,
        Vec::<String>::new(),
        "two passes stand against two"
    );
    assert_eq!(
        fail("log.md three").retired,
        vec![mine.clone()],
        "three outnumber two"
    );
    let shared_copy = fold(&read(&w, &Tier::Shared).unwrap().rows);
    assert_eq!(
        shared_copy[&mine].status,
        Status::Dormant,
        "the shared copy is retired too"
    );
    let woke = wake(&w, &mine).unwrap();
    assert_eq!(
        woke,
        [
            ".mikata/memory/shared/notes.jsonl",
            ".mikata/memory/creature/slime-s.jsonl"
        ],
        "both copies of a grounded note wake"
    );
    let personal = Tier::Personal("slime-s".into());
    assert_eq!(
        fold(&read(&w, &personal).unwrap().rows)[&mine].status,
        Status::Grounded
    );
    std::thread::sleep(std::time::Duration::from_millis(3));
    let after = fail("log.md four");
    assert_eq!(
        (after.doubted, after.retired),
        (vec![mine.clone()], vec![]),
        "it starts clean"
    );
    assert!(wake(&w, "no-such-note").is_err());
    let _ = fs::remove_dir_all(&w);
}

/// Only a retirement is woken: a note put to sleep otherwise, or never grounded, keeps its standing
/// and its route. A proposed note can be struck, so a pass credits it too.
#[test]
fn wake_undoes_only_a_retirement_and_a_proposed_note_is_credited() {
    use mikata_memory::{Forget, confirm, forget, wake};
    let w = world_of("wake-only");
    let asleep = grounded_note(&w, "slime-s", "the cache lives in .cache/sqlglot");
    forget(&w, &Tier::Shared, &asleep, Forget::Dormant, "").unwrap();
    assert!(
        wake(&w, &asleep).is_err(),
        "put to sleep by housekeeping, not retired"
    );
    let guess = propose(
        &w,
        "slime-s",
        "maybe src/tokens.py:40 is wrong",
        vec![],
        None,
    )
    .unwrap()
    .note;
    assert!(
        wake(&w, &guess).is_err(),
        "never grounded: no route, no waking"
    );
    assert_eq!(
        confirm(&w, "slime-s", &[anchor(&guess, true, true)], "log.md pass").unwrap(),
        1,
        "a proposed note is credited"
    );
    let _ = fs::remove_dir_all(&w);
}

/// A proposed claim is found again by its id, for a route to ground later; a grounded one, or
/// another creature's, is not.
#[test]
fn a_proposed_claim_is_found_again_by_its_id() {
    use mikata_memory::proposed;
    let w = world_of("proposed");
    let c = propose(&w, "slime-s", "the API caps at 100 a minute", vec![], None).unwrap();
    let id = c.note.clone();
    let again = proposed(&w, "slime-s", &id).unwrap().unwrap();
    assert_eq!(
        (again.text.as_str(), again.owner.as_str()),
        ("the API caps at 100 a minute", "slime-s")
    );
    assert!(
        proposed(&w, "slime-o", &id).unwrap().is_none(),
        "another creature's"
    );
    ground(
        &w,
        again,
        &Proof::accepted("orc-o accepted it", true).unwrap(),
    )
    .unwrap();
    assert!(
        proposed(&w, "slime-s", &id).unwrap().is_none(),
        "grounded already"
    );
    let shared = fold(&read(&w, &Tier::Shared).unwrap().rows);
    assert!(
        (shared[&id].confidence - 0.6).abs() < 1e-9,
        "accepted grounds at 0.6"
    );
    let _ = fs::remove_dir_all(&w);
}

/// A checked proof bears only on claims about its diff — the path, a `file:line` in it, a
/// directory above it — that name a check that ran: the verify line, or a test it names. A claim
/// about another file or the world beyond, naming no check, or naming one that did not run, is
/// refused, and so is a proof bound to another claim.
#[test]
fn a_checked_proof_bears_only_on_claims_about_its_diff_that_name_a_check_that_ran() {
    let proof = Proof::checked(
        "log.md v",
        &[
            "python3 -m compileall -q src".into(),
            "python3 -m pytest tests/test_a.py::test_empty".into(),
        ],
        &["src/parser/mod.rs".into(), "src/a.rs".into()],
    )
    .unwrap();
    let test = Some("tests/test_a.py::test_empty");
    let bears =
        |text: &str, source: Option<&str>, check: Option<&str>| proof.observes(text, source, check);
    assert!(bears("src/a.rs:12 handles the empty case", None, test).is_ok());
    assert!(
        bears("the parser in src/parser keeps fractions", None, test).is_ok(),
        "a directory above"
    );
    assert!(
        bears("it keeps fractions", Some("src/a.rs"), test).is_ok(),
        "by its source"
    );
    assert!(
        bears(
            "src/a.rs compiles",
            None,
            Some("python3 -m compileall -q src")
        )
        .is_ok(),
        "the whole line"
    );
    let why = |r: Result<Proof, String>| r.unwrap_err();
    assert!(why(bears("src/b.rs handles the empty case", None, test)).contains("names no path"));
    assert!(why(bears("the API caps at 100 a minute", None, test)).contains("names no path"));
    assert!(
        why(bears("src/a.rs handles the empty case", None, None)).contains("names no check"),
        "a passing line proves nothing the claim does not bind it to"
    );
    assert!(
        why(bears(
            "src/a.rs handles the empty case",
            None,
            Some("test_other")
        ))
        .contains("`test_other` is no verify line that ran")
    );
    assert!(
        why(bears("src/a.rs handles the empty case", None, Some("-q")))
            .contains("no verify line that ran"),
        "a flag is no test"
    );
    assert!(
        Proof::confirmed("the human")
            .observes("anything at all", None, None)
            .is_ok(),
        "other routes observe no diff"
    );
    let w = world_of("covers");
    let world_claim = propose(&w, "slime-s", "the API caps at 100 a minute", vec![], None).unwrap();
    let refused = ground(&w, world_claim, &proof).unwrap_err();
    assert!(refused.to_string().contains("bear on"), "{refused}");
    let diff_claim = propose(
        &w,
        "slime-s",
        "src/a.rs:12 handles the empty case",
        vec![],
        None,
    )
    .unwrap();
    let other = propose(&w, "slime-s", "src/a.rs:20 keeps the order", vec![], None).unwrap();
    let bound = proof.observes(&diff_claim.text, None, test).unwrap();
    assert!(
        ground(&w, other, &bound).is_err(),
        "a proof bound to one claim grounds no other"
    );
    assert!(ground(&w, diff_claim, &bound).is_ok());
    let _ = fs::remove_dir_all(&w);
}

#[test]
fn a_claim_names_its_check_last() {
    use mikata_memory::named_check;
    assert_eq!(
        named_check("src/a.rs pops check=`tests/a.py::test_pop`"),
        ("src/a.rs pops", Some("tests/a.py::test_pop"))
    );
    assert_eq!(
        named_check("src/a.rs pops check=cargo test -p a"),
        ("src/a.rs pops", Some("cargo test -p a"))
    );
    assert_eq!(named_check("src/a.rs pops"), ("src/a.rs pops", None));
    assert_eq!(named_check("src/a.rs pops check="), ("src/a.rs pops", None));
}

/// A second creature that says the same from other evidence grounds both claims, each citing the
/// other — and only then: not on shared evidence, not when one is negated, not from itself, not
/// from no evidence.
#[test]
fn a_second_voice_from_other_evidence_corroborates() {
    use mikata_memory::{corroborate, propose_on};
    let ev = |p: &[&str]| p.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let w = world_of("corroborate");
    let first = propose_on(
        &w,
        "slime-a",
        "The lexer keeps interval fractions.",
        vec![],
        None,
        ev(&["src/lexer.rs"]),
    )
    .unwrap();
    let first_id = first.note.clone();
    // A second creature on the same evidence: no second voice.
    let same = propose_on(
        &w,
        "slime-b",
        "the lexer keeps interval fractions",
        vec![],
        None,
        ev(&["src/lexer.rs"]),
    )
    .unwrap();
    assert!(
        corroborate(&w, same, &[]).unwrap().is_err(),
        "one piece of evidence"
    );
    // Negated, or resting on nothing: no match.
    let not = propose_on(
        &w,
        "slime-c",
        "the lexer keeps no interval fractions",
        vec![],
        None,
        ev(&["tests/lexer.rs"]),
    )
    .unwrap();
    assert!(
        corroborate(&w, not, &[]).unwrap().is_err(),
        "negated alike or not at all"
    );
    let bare = propose_on(
        &w,
        "slime-c",
        "the lexer keeps interval fractions",
        vec![],
        None,
        vec![],
    )
    .unwrap();
    assert!(
        corroborate(&w, bare, &[]).unwrap().is_err(),
        "no evidence, no corroboration"
    );
    // Another creature, other evidence, the same words: both grounded.
    let second = propose_on(
        &w,
        "slime-d",
        "interval fractions: the lexer keeps them",
        vec![],
        None,
        ev(&["tests/test_lexer.py"]),
    )
    .unwrap();
    let Ok((grounded, by)) = corroborate(&w, second, &[]).unwrap() else {
        panic!("a second voice from other evidence corroborates");
    };
    assert_eq!(by, first_id);
    let shared = fold(&read(&w, &Tier::Shared).unwrap().rows);
    for (id, owner) in [(&grounded.note, "slime-d"), (&first_id, "slime-a")] {
        let mine = fold(&read(&w, &Tier::Personal(owner.into())).unwrap().rows);
        assert_eq!(mine[id].status, Status::Grounded, "{owner}");
        assert!(
            mine[id]
                .proof
                .as_deref()
                .unwrap()
                .starts_with("corroborated: "),
            "{owner}"
        );
    }
    assert_eq!(shared[&grounded.note].status, Status::Grounded);
    assert!(
        !shared.contains_key(&first_id),
        "one copy shared, so recall counts it once"
    );
    assert_eq!(
        shared[&grounded.note].evidence,
        ev(&["tests/test_lexer.py"]),
        "evidence travels"
    );
    let _ = fs::remove_dir_all(&w);
}

/// No second voice from the claim owner itself, nor from its own line — a parent restating its child's offered claim
/// is an echo — nor for a claim too short to match on more than chance.
#[test]
fn kin_and_short_claims_never_corroborate() {
    use mikata_memory::{corroborate, propose_on};
    let w = world_of("corroborate-kin");
    let say = |who: &str, text: &str, on: &str| {
        propose_on(&w, who, text, vec![], None, vec![on.to_owned()]).unwrap()
    };
    say(
        "slime-a",
        "the lexer keeps interval fractions",
        "src/lexer.rs",
    );
    let parent = say(
        "orc-o",
        "the lexer keeps interval fractions",
        "src/parser.rs",
    );
    assert!(
        corroborate(&w, parent, &["slime-a".into()])
            .unwrap()
            .is_err(),
        "an echo along the line"
    );
    // The same creature on other evidence, alone in its world: no second voice.
    let alone = world_of("corroborate-alone");
    for on in ["src/lexer.rs", "src/parser.rs"] {
        let c = propose_on(
            &alone,
            "slime-a",
            "the lexer keeps interval fractions",
            vec![],
            None,
            vec![on.to_owned()],
        )
        .unwrap();
        assert!(
            corroborate(&alone, c, &[]).unwrap().is_err(),
            "itself, on {on}"
        );
    }
    let _ = fs::remove_dir_all(&alone);
    say("slime-b", "tests pass", "src/a.rs");
    let short = say("slime-c", "tests pass", "src/b.rs");
    assert!(
        corroborate(&w, short, &[]).unwrap().is_err(),
        "two words match by chance"
    );
    let _ = fs::remove_dir_all(&w);
}

/// The human's act names the digest of what they read: a claim rewritten between the reading and
/// the act is refused and nothing is written; the claim as it now reads is vouched for once read.
#[test]
fn an_act_on_a_note_rewritten_since_it_was_read_is_refused() {
    use mikata_memory::{amend, changed, seen, vouch, wake_by};
    let w = world_of("rewritten");
    let claim = propose(&w, "slime-s", "the lexer owns quoting", vec![], None).unwrap();
    let read_as = seen(&claim.note, "the lexer owns quoting");
    amend(
        &w,
        "slime-s",
        &claim.note,
        "the parser owns quoting",
        vec![],
    )
    .unwrap();
    let file = w.join(".mikata/memory/creature/slime-s.jsonl");
    let before = fs::read_to_string(&file).unwrap();
    assert_eq!(
        vouch(&w, &claim.note, "the human's word", &read_as),
        Err(changed(&claim.note))
    );
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        before,
        "nothing written"
    );
    assert!(!w.join(".mikata/memory/shared/notes.jsonl").exists());
    let now = seen(&claim.note, "the parser owns quoting");
    assert_eq!(
        vouch(&w, &claim.note, "the human's word", &now),
        Ok("slime-s".to_owned())
    );

    // A retired note: woken only as it was read.
    let rows = [
        r#"{"id":"e1","time":1,"note":"n1","by":"record","kind":"create","text":"retry twice","tags":[],"owner":"slime-r","polarity":"support","salience":0.5,"confidence":0.5,"source":null,"links":[]}"#,
        r#"{"id":"e2","time":2,"note":"n1","by":"human","kind":"status","status":"grounded"}"#,
        r#"{"id":"e3","time":3,"note":"n1","by":"gate","proof":"log.md","kind":"status","status":"dormant"}"#,
    ];
    let tier = w.join(".mikata/memory/creature/slime-r.jsonl");
    fs::write(&tier, rows.join("\n") + "\n").unwrap();
    let e = wake_by(&w, "n1", "woken", &seen("n1", "retry once")).unwrap_err();
    assert_eq!(e.to_string(), changed("n1"));
    assert_eq!(fs::read_to_string(&tier).unwrap(), rows.join("\n") + "\n");
    assert!(wake_by(&w, "n1", "woken", &seen("n1", "retry twice")).is_ok());
    let _ = fs::remove_dir_all(&w);
}

/// Notes an evolution kept distils are put to sleep — dormant, their events kept — once each,
/// and only the owner's live personal notes.
#[test]
fn distilled_notes_go_to_sleep_once() {
    let world = world_of("distilled");
    let w = world.as_path();
    let a = mikata_memory::propose(w, "sly", "fractions stay exact", vec![], None)
        .unwrap()
        .note;
    let b = mikata_memory::propose(w, "sly", "lex before parsing", vec![], None)
        .unwrap()
        .note;
    let moved = mikata_memory::distilled(
        w,
        "sly",
        &[a.clone(), a.clone(), "n-not-there".into()],
        "sly-mutate-1",
    )
    .unwrap();
    assert_eq!(moved, vec![a.clone()]);
    let rows = mikata_memory::read(w, &mikata_memory::Tier::Personal("sly".into()))
        .unwrap()
        .rows;
    let notes = mikata_memory::fold(&rows);
    assert_eq!(notes[&a].status, mikata_memory::Status::Dormant);
    assert_eq!(notes[&b].status, mikata_memory::Status::Proposed);
    assert!(
        rows.iter().any(|e| e.note == a
            && e.proof
                .as_deref()
                .is_some_and(|p| p.contains("sly-mutate-1"))),
        "the why is kept"
    );
    assert!(
        mikata_memory::distilled(w, "sly", &[a], "again")
            .unwrap()
            .is_empty(),
        "asleep already: moved once"
    );
}

/// A claim whose check names a test file, not a verify line. A test that stood before the session,
/// its diff leaving it as it was, ran under the verify line the gate passed: the claim is grounded
/// now. A test the session itself wrote proves little of its author's claim: the claim is held,
/// and a later gate grounds it once that test has stood unchanged through another session's work.
/// A later session that changes the test again keeps it waiting; one whose test moved since stays
/// proposed for good. On the bench, every claim Claude Haiku 5.5 wrote over ten sqlglot fixes
/// cited a test it had just written, and none could be grounded.
#[test]
fn a_claim_on_a_test_its_session_wrote_is_grounded_by_a_later_gate() {
    use mikata_memory::{Bears, test_named};
    let w = world_of("held");
    fs::create_dir_all(w.join("tests")).unwrap();
    fs::create_dir_all(w.join("sqlglot")).unwrap();
    fs::write(w.join("sqlglot/a.py"), "x = 1\n").unwrap();
    fs::write(w.join("tests/test_a.py"), "def test_keep(): pass\n").unwrap();
    fs::write(w.join("tests/test_old.py"), "def test_x(): pass\n").unwrap();
    let line = "python3 .mikata/bench/no_regressions.py".to_owned();
    let first = Proof::checked(
        "log.md first",
        std::slice::from_ref(&line),
        &["sqlglot/a.py".into(), "tests/test_a.py".into()],
    )
    .unwrap();
    let claim = "sqlglot/a.py keeps the collation";
    // Written this session: held. Stood before: grounded now, by path or by dotted module.
    assert_eq!(
        first.bears(&w, claim, None, Some("tests/test_a.py::TestA::test_keep")),
        Ok(Bears::Later {
            test: "tests/test_a.py".into()
        })
    );
    for stood in [
        "tests/test_old.py",
        "python3 -m unittest tests.test_old.TestOld.test_x",
    ] {
        let Ok(Bears::Now(p)) = first.bears(&w, claim, None, Some(stood)) else {
            panic!("{stood}: a test that stood before the session grounds now");
        };
        let g = ground(
            &w,
            propose(&w, "summoner", claim, vec![], None).unwrap(),
            &p,
        )
        .unwrap();
        let notes = fold(&read(&w, &Tier::Personal("summoner".into())).unwrap().rows);
        assert!(
            notes[&g.note]
                .proof
                .as_deref()
                .unwrap()
                .contains("standing before the session")
        );
    }
    // The verify line itself still grounds now, and a claim about no changed path never does.
    assert!(matches!(
        first.bears(&w, claim, None, Some(line.as_str())),
        Ok(Bears::Now(_))
    ));
    assert!(
        first
            .bears(&w, "the API caps at 100", None, Some("tests/test_a.py"))
            .unwrap_err()
            .contains("names no path")
    );
    assert!(
        first
            .bears(&w, claim, None, Some("tests/test_gone.py::t"))
            .unwrap_err()
            .contains("nor a test file of the world")
    );
    assert_eq!(
        test_named(&w, "pytest tests/test_a.py::T::t -q"),
        Some("tests/test_a.py".into())
    );
    assert_eq!(
        test_named(&w, "tests.test_old"),
        Some("tests/test_old.py".into())
    );
    assert_eq!(test_named(&w, "../etc/passwd"), None);
    assert_eq!(test_named(&w, "-x"), None);
    let _ = fs::remove_dir_all(&w);
}

/// A held claim waits on its test: a later session that changes the test again keeps it waiting,
/// one that leaves it as it was grounds it, citing both gates' line and the test, and one whose
/// test moved since leaves it proposed for good. Only the checked route settles anything.
#[test]
fn a_held_claim_is_grounded_once_its_test_stood_through_a_later_gate() {
    use mikata_memory::{hold, settle};
    let w = world_of("settle");
    fs::create_dir_all(w.join("tests")).unwrap();
    fs::write(w.join("tests/test_a.py"), "def test_keep(): pass\n").unwrap();
    let line = "python3 .mikata/bench/no_regressions.py".to_owned();
    let claim = "sqlglot/a.py keeps the collation";

    // Held: proposed, waiting on its test with the hash it has now.
    let held = propose(&w, "summoner", claim, vec![], None).unwrap();
    let id = held.note.clone();
    hold(
        &w,
        &held,
        "tests/test_a.py",
        "tests/test_a.py::TestA::test_keep",
    )
    .unwrap();
    let standing = |id: &str| {
        let notes = fold(&read(&w, &Tier::Personal("summoner".into())).unwrap().rows);
        let n = &notes[id];
        (n.status, n.pending.as_ref().map(|h| h.test.clone()))
    };
    assert_eq!(
        standing(&id),
        (Status::Proposed, Some("tests/test_a.py".into()))
    );
    // A route other than checked settles nothing.
    assert_eq!(
        settle(&w, "summoner", &Proof::confirmed("the human"))
            .unwrap()
            .grounded
            .len(),
        0
    );
    // A later session that changes the test again: still waiting.
    let again = Proof::checked(
        "log.md second",
        std::slice::from_ref(&line),
        &["tests/test_a.py".into()],
    )
    .unwrap();
    let s = settle(&w, "summoner", &again).unwrap();
    assert_eq!((s.grounded.len(), s.waiting, s.moved), (0, 1, 0));
    // A later session that leaves it alone: grounded, citing both.
    let later = Proof::checked(
        "log.md third",
        std::slice::from_ref(&line),
        &["sqlglot/b.py".into()],
    )
    .unwrap();
    let s = settle(&w, "summoner", &later).unwrap();
    assert_eq!(s.grounded, vec![id.clone()]);
    assert_eq!(standing(&id), (Status::Grounded, None));
    let notes = fold(&read(&w, &Tier::Personal("summoner".into())).unwrap().rows);
    let cites = notes[&id].proof.clone().unwrap();
    assert!(
        cites.contains("log.md third") && cites.contains("unchanged since the claim was held"),
        "{cites}"
    );
    assert!(
        fold(&read(&w, &Tier::Shared).unwrap().rows).contains_key(&id),
        "grounded is shared"
    );
    // Its test moved before any later gate: it stays proposed, and is counted as moved.
    let moved = propose(
        &w,
        "summoner",
        "sqlglot/a.py reads COLLATE once",
        vec![],
        None,
    )
    .unwrap();
    let moved_id = moved.note.clone();
    hold(&w, &moved, "tests/test_a.py", "tests/test_a.py").unwrap();
    fs::write(w.join("tests/test_a.py"), "def test_keep(): assert True\n").unwrap();
    let s = settle(&w, "summoner", &later).unwrap();
    assert_eq!((s.grounded.len(), s.waiting, s.moved), (0, 0, 1));
    assert_eq!(standing(&moved_id).0, Status::Proposed);
    let _ = fs::remove_dir_all(&w);
}
