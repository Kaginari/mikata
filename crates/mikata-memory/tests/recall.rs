//! Recall: meaning first, a seat for the dissent, what was cut said, silence when nothing is asked.

use mikata_memory::{Note, Polarity, Recalled, Relation, Status, Weights, recall};

const DAY: u64 = 86_400_000;
const NOW: u64 = 400 * DAY;

fn note(id: &str, text: &str, polarity: Polarity, status: Status) -> Note {
    Note {
        id: id.into(),
        text: text.into(),
        tags: vec![],
        owner: None,
        polarity,
        salience: 0.5,
        confidence: 0.5,
        status,
        source: None,
        proof: None,
        born: NOW - DAY,
        links: vec![],
        lent: vec![],
        evidence: vec![],
        ..Note::default()
    }
}

fn ids(r: &Recalled) -> Vec<&str> {
    r.anchors.iter().map(|h| h.note.id.as_str()).collect()
}

#[test]
fn a_query_of_no_content_words_recalls_nothing() {
    let notes = vec![(
        note(
            "n",
            "the parser drops the alias",
            Polarity::Support,
            Status::Grounded,
        ),
        Relation::Own,
    )];
    assert_eq!(
        recall("what is the of it", &notes, NOW, 6, &Weights::default()),
        Recalled::default()
    );
}

/// The only correcting note ranks below `k` and is still recalled: the dissent keeps a seat.
#[test]
fn the_only_correction_ranks_below_k_and_is_still_recalled() {
    let mut notes: Vec<(Note, Relation)> = (0..6)
        .map(|i| {
            let mut n = note(
                &format!("s{i}"),
                "the tokenizer splits interval strings on whitespace",
                Polarity::Support,
                Status::Grounded,
            );
            n.salience = 1.0;
            n.confidence = 1.0;
            (n, Relation::Own)
        })
        .collect();
    let mut correction = note(
        "fix",
        "interval strings with fractions are not split by the tokenizer",
        Polarity::Revise,
        Status::Grounded,
    );
    correction.salience = 0.0;
    correction.confidence = 0.1;
    notes.push((correction, Relation::Shared));
    // A memory holds more than the question: notes about other things.
    for i in 0..20 {
        notes.push((
            note(
                &format!("other{i}"),
                &format!("the optimizer pushes predicate {i} down through joins"),
                Polarity::Support,
                Status::Grounded,
            ),
            Relation::Shared,
        ));
    }
    let r = recall(
        "tokenizer interval strings",
        &notes,
        NOW,
        3,
        &Weights::default(),
    );
    assert_eq!(r.anchors.len(), 3);
    assert!(
        ids(&r).contains(&"fix"),
        "the correction holds the dissent's seat: {:?}",
        ids(&r)
    );
    assert_eq!(r.cut, 4, "the cut are counted");
    assert_eq!(r.cut_ids.len(), 3, "at most three of them named");
}

/// The best hit's meaning is a poor match: however high its priors, it stays under the floor.
#[test]
fn a_poor_match_stays_under_the_floor_however_high_its_priors() {
    // "parser" is in every note: it tells them apart by nothing, so it means little.
    let mut notes: Vec<(Note, Relation)> = (0..8)
        .map(|i| {
            (
                note(
                    &format!("n{i}"),
                    &format!("parser note number {i} about {i}"),
                    Polarity::Support,
                    Status::Grounded,
                ),
                Relation::Shared,
            )
        })
        .collect();
    let mut loud = note("loud", "parser", Polarity::Support, Status::Grounded);
    loud.salience = 1.0;
    loud.confidence = 1.0;
    loud.born = NOW;
    notes.push((loud, Relation::Own));
    let r = recall("parser", &notes, NOW, 6, &Weights::default());
    assert!(!ids(&r).contains(&"loud"), "{:?}", ids(&r));
    assert!(
        r.anchors
            .iter()
            .all(|h| h.meaning >= Weights::default().floor)
    );
}

#[test]
fn dormant_and_tombstoned_notes_are_never_recalled_and_the_own_ranks_first() {
    let text = "the generator quotes reserved identifiers";
    let notes = vec![
        (
            note("shared", text, Polarity::Support, Status::Grounded),
            Relation::Shared,
        ),
        (
            note("own", text, Polarity::Support, Status::Grounded),
            Relation::Own,
        ),
        (
            note("asleep", text, Polarity::Support, Status::Dormant),
            Relation::Own,
        ),
        (
            note("gone", text, Polarity::Support, Status::Tombstoned),
            Relation::Own,
        ),
        (
            note(
                "other",
                "nothing about that at all",
                Polarity::Support,
                Status::Grounded,
            ),
            Relation::Own,
        ),
    ];
    let r = recall(
        "generator reserved identifiers",
        &notes,
        NOW,
        6,
        &Weights::default(),
    );
    assert_eq!(ids(&r), ["own", "shared"]);
    assert_eq!(r.cut, 0);
}

/// The seats are the page's: the k − 2 best agreeing notes, the best dissent, then the fill — so
/// agreeing notes keep their seats when dissents outnumber them.
#[test]
fn the_seats_keep_k_minus_two_for_agreement_and_one_for_dissent() {
    let text = "the planner inlines common table expressions";
    let mut notes: Vec<(Note, Relation)> = (0..3)
        .map(|i| {
            let mut n = note(&format!("c{i}"), text, Polarity::Counter, Status::Grounded);
            n.salience = 1.0;
            (n, Relation::Own)
        })
        .collect();
    notes.push((
        note("s", text, Polarity::Support, Status::Grounded),
        Relation::Shared,
    ));
    for i in 0..20 {
        notes.push((
            note(
                &format!("x{i}"),
                &format!("unrelated note {i}"),
                Polarity::Support,
                Status::Grounded,
            ),
            Relation::Shared,
        ));
    }
    let r = recall(
        "planner inlines table expressions",
        &notes,
        NOW,
        3,
        &Weights::default(),
    );
    let got = ids(&r);
    assert!(
        got.contains(&"s"),
        "the agreeing note keeps its seat: {got:?}"
    );
    assert_eq!(got.len(), 3);
}

/// Another creature's proposed note is its guess: never weighed. The asker's own is.
#[test]
fn only_the_askers_own_proposed_notes_are_weighed() {
    let text = "the dialect quotes identifiers with backticks";
    let notes = vec![
        (
            note("mine", text, Polarity::Support, Status::Proposed),
            Relation::Own,
        ),
        (
            note("theirs", text, Polarity::Support, Status::Proposed),
            Relation::Shared,
        ),
        (
            note("kin", text, Polarity::Support, Status::Proposed),
            Relation::Race,
        ),
        (
            note(
                "other",
                "nothing like that",
                Polarity::Support,
                Status::Grounded,
            ),
            Relation::Shared,
        ),
    ];
    let r = recall(
        "dialect quotes identifiers backticks",
        &notes,
        NOW,
        6,
        &Weights::default(),
    );
    assert_eq!(ids(&r), ["mine"]);
}

/// A correction is recalled where the note it corrects would be: written by another creature's
/// failed run, in a failure line's words, it still takes the relation of the note it links to and
/// matches the query that note matches — the premise of its head start.
#[test]
fn a_correction_stands_where_the_note_it_corrects_stands() {
    use mikata_memory::{Anchor, Proof, Relation, gather, ground, propose, recall, revise};
    let w = std::env::temp_dir().join(format!("mikata-lend-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&w);
    std::fs::create_dir_all(&w).unwrap();
    let habit = ground(
        &w,
        propose(
            &w,
            "slime-a",
            "the tokenizer keeps interval fractions whole",
            vec![],
            None,
        )
        .unwrap(),
        &Proof::confirmed("the human"),
    )
    .unwrap()
    .note;
    for text in [
        "the planner caches joins",
        "the lexer is pure",
        "dates parse in utc",
    ] {
        ground(
            &w,
            propose(&w, "slime-z", text, vec![], None).unwrap(),
            &Proof::confirmed("h"),
        )
        .unwrap();
    }
    std::thread::sleep(std::time::Duration::from_millis(3));
    // Another creature's failed run corrects it, in its failure's words.
    let correction = revise(
        &w,
        "slime-b",
        &["traits hold: test_round_trip failed".into()],
        &[Anchor {
            id: habit.clone(),
            personal: false,
            dissent: false,
            cited: true,
        }],
        "log.md fail",
    )
    .unwrap()
    .remove(0);
    let memory = gather(&w, "slime-a", |_| false).unwrap();
    let rel = memory
        .notes
        .iter()
        .find(|(n, _)| n.id == correction)
        .unwrap()
        .1;
    assert_eq!(
        rel,
        Relation::Own,
        "the correction stands where the note it corrects stands"
    );
    let r = recall(
        "why does the tokenizer split interval fractions",
        &memory.notes,
        mikata_memory::store::now_ms(),
        6,
        &mikata_memory::Weights::default(),
    );
    assert!(
        r.anchors.iter().any(|h| h.note.id == correction),
        "recalled with the note it corrects"
    );
    assert!(
        r.anchors.iter().any(|h| h.note.id == habit),
        "the note itself is not diluted"
    );
    let stored = mikata_memory::fold(
        &mikata_memory::read(&w, &mikata_memory::Tier::Shared)
            .unwrap()
            .rows,
    );
    assert_eq!(
        stored[&correction].tags,
        ["failure"],
        "nothing lent is stored"
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// Against the 100-odd notes real runs wrote: each task's ask, over all of them, should find its
/// own notes. A fixed saturation lets nearly every note clear the floor once memory is this large;
/// the saturation that grows with the notes keeps most of what clears relevant (#196).
/// A lesson is put in the words of what was found, not of what was asked. Kept with the first
/// line of the ask it was learned under, it is recalled when that ask comes again; the same
/// words kept without it are not.
#[test]
fn a_note_is_recalled_for_the_ask_it_was_learned_under() {
    use mikata_memory::{gather, propose_on, propose_under};
    let w = std::env::temp_dir().join(format!("mikata-recall-asked-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&w);
    std::fs::create_dir_all(&w).unwrap();
    let ask = "DuckDB: support NEAREST top-k joins\n\nA join may name how many rows it keeps.";
    let lesson = "pushdown stops at the first source it cannot fill";
    let learned = propose_under(
        &w,
        "summoner",
        lesson,
        vec!["law".into()],
        ask.lines().next(),
        vec![],
    )
    .unwrap();
    assert_eq!(
        learned.asked.as_deref(),
        Some("DuckDB: support NEAREST top-k joins")
    );
    let other = propose_on(
        &w,
        "summoner",
        "an index is counted from one in this dialect",
        vec!["law".into()],
        None,
        vec![],
    )
    .unwrap();
    let memory = gather(&w, "summoner", |_| false).unwrap();
    let r = recall(ask, &memory.notes, NOW, 6, &Weights::default());
    assert_eq!(ids(&r), [learned.note.as_str()], "{r:?}");
    // The words are borrowed for the match alone: the note says what it said.
    assert_eq!(r.anchors[0].note.text, lesson);
    // Another ask does not find it by them, and a note kept under no ask borrows nothing.
    let elsewhere = recall(
        "Oracle: alternative quoting rejects most delimiters",
        &memory.notes,
        NOW,
        6,
        &Weights::default(),
    );
    assert!(elsewhere.anchors.is_empty(), "{elsewhere:?}");
    assert!(
        memory
            .notes
            .iter()
            .any(|(n, _)| n.id == other.note && n.lent.is_empty())
    );
    let _ = std::fs::remove_dir_all(&w);
}

fn fresh_world(name: &str) -> std::path::PathBuf {
    let w = std::env::temp_dir().join(format!("mikata-recall-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&w);
    std::fs::create_dir_all(&w).unwrap();
    w
}

const ASKED: &str = "Support NEAREST top-k joins";

/// A first failure corrects no note: nothing had been recalled. It is remembered all the same,
/// under the ask that failed, and the ask that comes again is told it in the correction's seat.
/// Before, such a failure left nothing behind.
#[test]
fn a_first_failure_is_remembered_under_its_ask() {
    use mikata_memory::{gather, lines, revise, revise_under};
    let w = fresh_world("first-failure");
    let failed =
        ["verify: `pytest -q` failed: tests/test_join.py::test_nearest_keeps_k".to_owned()];
    let verdict = "log.md ### [1] summoner — gate: FAIL";
    // With no ask to keep it under and nothing recalled, a failure has nowhere to stand.
    let unkept = revise(&w, "summoner", &failed, &[], verdict).unwrap();
    assert!(unkept.is_empty(), "{unkept:?}");
    let made = revise_under(&w, "summoner", &failed, &[], verdict, Some(ASKED)).unwrap();
    assert_eq!(made.len(), 1, "one note for the one rule that failed");
    let memory = gather(&w, "summoner", |_| false).unwrap();
    let again = format!("# {ASKED}\n\nA join may say how many rows it keeps.");
    let r = recall(&again, &memory.notes, NOW, 6, &Weights::default());
    assert_eq!(ids(&r), [made[0].as_str()], "{r:?}");
    let told = lines(&r, &memory.personal);
    assert!(
        told[0].starts_with(&format!("@M {} personal grounded revise: verify:", made[0])),
        "{told:?}"
    );
    assert!(
        !told.iter().any(|l| l.contains("dissent: none selected")),
        "a correction was selected: {told:?}"
    );
    let other = recall(
        "Oracle: alternative quoting rejects most delimiters",
        &memory.notes,
        NOW,
        6,
        &Weights::default(),
    );
    assert!(other.anchors.is_empty(), "{other:?}");
    assert!(
        mikata_memory::status(&w)
            .unwrap()
            .iter()
            .all(|(_, f)| f.is_empty())
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// A session that stops at a limit leaves an account under its ask: that it stopped, why, and
/// what it had changed. It doubts no note.
#[test]
fn a_stop_is_remembered_under_its_ask() {
    use mikata_memory::{gather, stopped};
    let w = fresh_world("stopped");
    let verdict = "log.md ### [1] summoner — stopped: steps: 200 taken";
    assert_eq!(
        stopped(&w, "summoner", "steps: 200 taken", &[], verdict, None).unwrap(),
        None,
        "with no ask to keep it under, nothing is written"
    );
    let id = stopped(
        &w,
        "summoner",
        "steps: 200 taken",
        &["src/join.js".into(), "test/join.test.js".into()],
        verdict,
        Some(ASKED),
    )
    .unwrap()
    .unwrap();
    let memory = gather(&w, "summoner", |_| false).unwrap();
    let r = recall(ASKED, &memory.notes, NOW, 6, &Weights::default());
    assert_eq!(ids(&r), [id.as_str()], "{r:?}");
    let note = &r.anchors[0].note;
    assert_eq!(
        note.text,
        "an earlier session was asked this and stopped before anything landed (steps: 200 \
         taken); it had changed `src/join.js`, `test/join.test.js`"
    );
    assert_eq!(
        (note.status, note.polarity),
        (Status::Grounded, Polarity::Revise)
    );
    assert!(note.links.is_empty(), "a stop is laid at no note's door");
    // Resumed and stopped again: the later stop stands, the earlier one sleeps.
    let id = stopped(
        &w,
        "summoner",
        "clock: 600 s",
        &["src/join.js".into()],
        "log.md ### [1b] summoner — stopped: clock: 600 s",
        Some(ASKED),
    )
    .unwrap()
    .unwrap();
    let memory = gather(&w, "summoner", |_| false).unwrap();
    let r = recall(ASKED, &memory.notes, NOW, 6, &Weights::default());
    assert_eq!(
        ids(&r),
        [id.as_str()],
        "one stop stands under an ask: {r:?}"
    );
    // The ask lands at last: the stop was a state, and is out of recall; a failure under the
    // same ask is a lesson, and stays.
    let failed = mikata_memory::revise_under(
        &w,
        "summoner",
        &["verify: tests/join.test.js failed".to_owned()],
        &[],
        "log.md ### [2] summoner — gate: FAIL",
        Some(ASKED),
    )
    .unwrap();
    let landed = "log.md ### [3] summoner — gate: PASS";
    assert_eq!(
        mikata_memory::answered(&w, "summoner", ASKED, landed).unwrap(),
        std::slice::from_ref(&id)
    );
    let memory = gather(&w, "summoner", |_| false).unwrap();
    let r = recall(ASKED, &memory.notes, NOW, 6, &Weights::default());
    assert_eq!(ids(&r), [failed[0].as_str()], "{r:?}");
    assert!(
        mikata_memory::answered(&w, "summoner", ASKED, landed)
            .unwrap()
            .is_empty(),
        "nothing left to settle"
    );
    assert!(
        mikata_memory::status(&w)
            .unwrap()
            .iter()
            .all(|(_, f)| f.is_empty())
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// The human says an answer was wrong. Their words are shared, grounded, and kept under the ask:
/// they link what a session learned under it and are recalled ahead of it, for the ask that comes
/// again and wherever those notes are recalled.
#[test]
fn the_humans_word_corrects_what_was_learned_under_an_ask() {
    use mikata_memory::{
        Tier, corrected, fold, gather, learned_under, propose_on, propose_under, read,
    };
    let w = fresh_world("corrected");
    let believed = propose_under(
        &w,
        "summoner",
        "src/join.js the planner keeps the k nearest rows by sorting the whole input",
        vec!["law".into()],
        Some(ASKED),
        vec![],
    )
    .unwrap();
    let elsewhere = propose_on(
        &w,
        "summoner",
        "an index is counted from one in this dialect",
        vec!["law".into()],
        None,
        vec![],
    )
    .unwrap();
    // A lesson about the repository, learned under the same ask: no part of the answer.
    let place = propose_under(
        &w,
        "summoner",
        "the tests of the planner run with make test-join",
        vec!["colony".into()],
        Some(ASKED),
        vec![],
    )
    .unwrap();
    assert_eq!(
        learned_under(&w, ASKED).unwrap(),
        [(believed.note.clone(), true)]
    );
    assert!(corrected(&w, "  ", ASKED, "log.md ### [2] human — wrong").is_err());
    let (id, linked) = corrected(
        &w,
        "wrong: with ties the join returns more than k rows",
        ASKED,
        "log.md ### [2] human — wrong: Support NEAREST top-k joins",
    )
    .unwrap();
    assert_eq!(linked, std::slice::from_ref(&believed.note));
    let shared = fold(&read(&w, &Tier::Shared).unwrap().rows);
    let note = &shared[&id];
    assert_eq!(
        (note.status, note.polarity),
        (Status::Grounded, Polarity::Revise)
    );
    assert_eq!(note.owner, None, "the world's, not a creature's");
    assert_eq!(note.asked.as_deref(), Some(ASKED));
    let memory = gather(&w, "summoner", |_| false).unwrap();
    // The ask again: the correction first, the belief it speaks of marked as corrected, and the
    // lesson about the repository as it was.
    let r = recall(ASKED, &memory.notes, NOW, 6, &Weights::default());
    assert_eq!(ids(&r)[0], id.as_str(), "{r:?}");
    let told = mikata_memory::lines(&r, &memory.personal);
    assert!(
        told.contains(&format!(
            "@M {} personal proposed support: the tests of the planner run with make test-join",
            place.note
        )),
        "{told:?}"
    );
    let told: Vec<String> = told
        .into_iter()
        .filter(|l| !l.contains(&place.note))
        .collect();
    assert_eq!(
        told,
        [
            format!(
                "@M {id} shared grounded revise: wrong: with ties the join returns more than k rows"
            ),
            format!(
                "@M {} personal proposed support corrected: src/join.js the planner keeps the k \
                 nearest rows by sorting the whole input",
                believed.note
            ),
        ]
    );
    // And wherever the note it corrects is recalled, by that note's own words.
    let by_words = recall(
        "how does the planner sort its input rows",
        &memory.notes,
        NOW,
        6,
        &Weights::default(),
    );
    assert!(ids(&by_words).contains(&id.as_str()), "{by_words:?}");
    assert!(!ids(&by_words).contains(&elsewhere.note.as_str()));
    assert!(
        mikata_memory::status(&w)
            .unwrap()
            .iter()
            .all(|(_, f)| f.is_empty())
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// Measured on a carried campaign's own notes (`fixtures/carried-recall.json`: one body, ten
/// tasks in one codebase, three rounds on one memory, 88 notes). A task that comes again must be
/// shown a note its own earlier run wrote. Matched by their words alone, the notes of a long ask
/// lose their seats to other tasks' notes that share the codebase's words; kept with the first
/// line of the ask they were learned under, every one of the twenty returning asks finds its own.
#[test]
fn a_task_that_comes_again_finds_its_own_notes_among_a_codebases() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/carried-recall.json")).unwrap();
    let order: Vec<&str> = data["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    let ask = |task: &str| data["asks"][task].as_str().unwrap();
    let first_line = |task: &str| {
        ask(task)
            .lines()
            .map(|l| l.trim().trim_start_matches('#').trim())
            .find(|l| !l.is_empty())
            .unwrap()
            .to_owned()
    };
    // Where a run stands in the campaign: its round, then its task's place in the order.
    let at = |round: u64, task: &str| (round, order.iter().position(|t| *t == task).unwrap());
    let written: Vec<(u64, &str, Note)> = data["notes"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let (round, task) = (n["round"].as_u64().unwrap(), n["task"].as_str().unwrap());
            let note = Note {
                tags: n["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|t| t.as_str().unwrap().to_owned())
                    .collect(),
                owner: Some("summoner".into()),
                salience: 0.6,
                born: NOW - DAY + i as u64,
                ..note(
                    &format!("n{i:03}"),
                    n["text"].as_str().unwrap(),
                    Polarity::Support,
                    Status::Proposed,
                )
            };
            (round, task, note)
        })
        .collect();
    assert!(written.len() >= 80, "{}", written.len());
    // For each run of rounds 2 and 3: recall over what was written before it, and whether a
    // note of its own task is among the anchors.
    let found = |kept_with_their_ask: bool| {
        let (mut own_shown, mut cases, mut own_seats, mut seats) = (0, 0, 0, 0);
        for round in [2, 3] {
            for task in &order {
                let before: Vec<(Note, Relation)> = written
                    .iter()
                    .filter(|(r, t, _)| at(*r, t) < at(round, task))
                    .map(|(_, t, n)| {
                        let mut n = n.clone();
                        if kept_with_their_ask {
                            n.lent.push(first_line(t));
                        }
                        (n, Relation::Own)
                    })
                    .collect();
                let own: Vec<&str> = written
                    .iter()
                    .filter(|(r, t, _)| t == task && at(*r, t) < at(round, task))
                    .map(|(_, _, n)| n.id.as_str())
                    .collect();
                let r = recall(ask(task), &before, NOW, 6, &Weights::default());
                let shown = ids(&r);
                let mine = shown.iter().filter(|i| own.contains(i)).count();
                cases += 1;
                own_shown += usize::from(mine > 0);
                own_seats += mine;
                seats += shown.len();
            }
        }
        (own_shown, cases, own_seats, seats)
    };
    let (by_words, cases, words_own, words_seats) = found(false);
    let (with_ask, _, ask_own, ask_seats) = found(true);
    assert_eq!(cases, 20);
    assert!(
        by_words < cases,
        "by their words alone every ask found its own note: the fixture no longer shows the fault"
    );
    assert_eq!(
        with_ask, cases,
        "kept with its ask, a note is found when the ask comes again"
    );
    // And it takes the seats other tasks' notes held: twice as many of the six are its own.
    assert!(
        ask_own * words_seats >= 2 * words_own * ask_seats,
        "own seats {ask_own} of {ask_seats} with the ask, {words_own} of {words_seats} by words"
    );
}

#[test]
fn a_large_memory_does_not_flood_the_floor() {
    use mikata_memory::text::{bm25, content_words};
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/bench-recall.json")).unwrap();
    let notes: Vec<(&str, Vec<String>)> = data["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["task"].as_str().unwrap(),
                content_words(n["text"].as_str().unwrap()),
            )
        })
        .collect();
    let docs: Vec<Vec<String>> = notes.iter().map(|n| n.1.clone()).collect();
    assert!(docs.len() >= 100, "{}", docs.len());
    let w = Weights::default();
    let precision = |c: f64| {
        let (mut right, mut all) = (0_u32, 0_u32);
        for (task, ask) in data["asks"].as_object().unwrap() {
            let terms = content_words(ask.as_str().unwrap());
            for (owner, doc) in &notes {
                let raw = bm25(&terms, doc, &docs);
                if w.meaning * raw / (raw + c) >= w.floor {
                    all += 1;
                    right += u32::from(owner == task);
                }
            }
        }
        f64::from(right) / f64::from(all.max(1))
    };
    let fixed = precision(w.saturation[2]);
    let grown = precision(w.saturation_for(4, docs.len()));
    assert!(fixed < 0.15, "a fixed saturation floods: {fixed}");
    assert!(grown > 0.5, "the grown one holds: {grown}");
    // A lone note is judged as it always was.
    assert!((w.saturation_for(4, 1) - w.saturation[2]).abs() < 1e-9);
    assert!(w.saturation_for(4, 100) > w.saturation_for(2, 100));
}
