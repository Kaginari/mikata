//! `mikatad memory status`: each creature's unproven note events, named for the human; it fails
//! while any is found.

#[path = "support/isolated.rs"]
mod isolated;

use std::fs;

fn status(world: &std::path::Path, json: bool) -> (Option<i32>, String) {
    let mut args = vec!["memory", "status", "--world"];
    let w = world.to_str().unwrap();
    args.push(w);
    if json {
        args.push("--json");
    }
    let out = isolated::bin(env!("CARGO_BIN_EXE_mikatad"))
        .args(&args)
        .env_remove("ANTHROPIC_API_KEY")
        .env("XDG_STATE_HOME", world.join("state"))
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn a_hand_edit_is_named_and_fails_the_status() {
    let world = std::env::temp_dir().join(format!("mikata-memory-status-{}", std::process::id()));
    let _ = fs::remove_dir_all(&world);
    let dir = world.join(".mikata/memory/creature");
    fs::create_dir_all(&dir).unwrap();
    assert_eq!(
        status(&world, false),
        (Some(0), "no creature has memory here yet\n".into())
    );
    fs::write(
        dir.join("slime-a.jsonl"),
        concat!(
            r#"{"id":"e1","time":1,"note":"n1","by":"record","kind":"create","text":"done","tags":[],"owner":"slime-a","polarity":"support","salience":0.5,"confidence":0.5,"source":null,"links":[]}"#,
            "\n",
            r#"{"id":"e2","time":2,"note":"n1","by":"record","kind":"status","status":"grounded"}"#,
            "\n",
        ),
    )
    .unwrap();
    let (code, out) = status(&world, false);
    assert_eq!(code, Some(1));
    assert!(out.contains("\nslime-a: frozen — 1 note event(s)"), "{out}");
    assert!(
        out.contains("event e2  note n1: a standing claimed by a plain record"),
        "{out}"
    );
    let (code, out) = status(&world, true);
    assert_eq!(code, Some(1));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v[0]["creature"], "slime-a");
    assert_eq!(v[0]["unproven"][0]["event"], "e2");
    let _ = fs::remove_dir_all(&world);
}

/// A note the gate retired is listed for the human with its undo; `wake` restores it, and a body is
/// never let wake one.
#[test]
fn a_retired_note_is_listed_and_the_human_wakes_it() {
    let world = std::env::temp_dir().join(format!("mikata-memory-wake-{}", std::process::id()));
    let _ = fs::remove_dir_all(&world);
    let dir = world.join(".mikata/memory/creature");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("slime-a.jsonl"),
        concat!(
            r#"{"id":"e1","time":1,"note":"n1","by":"record","kind":"create","text":"skip the fixture","tags":[],"owner":"slime-a","polarity":"support","salience":0.5,"confidence":0.5,"source":null,"links":[]}"#,
            "\n",
            r#"{"id":"e2","time":2,"note":"n1","by":"human","kind":"status","status":"grounded"}"#,
            "\n",
            r#"{"id":"e3","time":3,"note":"n1","by":"gate","proof":"log.md two","kind":"status","status":"dormant"}"#,
            "\n",
        ),
    )
    .unwrap();
    let (code, out) = status(&world, false);
    assert_eq!(code, Some(0), "a retirement waits on nobody: {out}");
    assert!(
        out.contains("retired, kept dormant: n1 — skip the fixture (undo: mikatad memory wake n1)"),
        "{out}"
    );
    let bin = env!("CARGO_BIN_EXE_mikatad");
    let body = isolated::bin(bin)
        .args(["memory", "wake", "n1", "--world", world.to_str().unwrap()])
        .env("MIKATA_BODY", "slime-a")
        .output()
        .unwrap();
    assert_eq!(body.status.code(), Some(2), "a body never wakes a note");
    let human = isolated::bin(bin)
        .args(["memory", "wake", "n1", "--world", world.to_str().unwrap()])
        .env_remove("MIKATA_BODY")
        .output()
        .unwrap();
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    let (_, out) = status(&world, false);
    assert_eq!(
        out,
        "memory: 0 note(s) shared and grounded; personal: 0 proposed, 1 grounded\nslime-a: clear\n"
    );
    let _ = fs::remove_dir_all(&world);
}

/// The human settles a contested fact, shown and asked at the terminal: a body is never let do
/// it, piped input is no answer, `n` settles nothing, an unknown id is refused, and `y` files the
/// settle in the trusted facts under the state directory.
#[test]
fn the_human_settles_a_fact_at_the_terminal_and_a_body_cannot() {
    use mikata_server::world::Facts;
    use mikata_server::world::facts::{Event, from_unsaid};
    let root = std::env::temp_dir().join(format!("mikata-onto-settle-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let world = root.join("world");
    let state = root.join("state");
    fs::create_dir_all(world.join("src")).unwrap();
    let said = from_unsaid("src/ build=cargo test", "slime-a", None).unwrap();
    let Event::Support { id, .. } = &said else {
        panic!("{said:?}");
    };
    let id = id.clone();
    Facts::record_in(Some(&state), &world, &said).unwrap();
    let w = world.to_str().unwrap();
    let bin = env!("CARGO_BIN_EXE_mikatad");
    let detached = |args: &[&str], body: Option<&str>| {
        let mut c = if cfg!(target_os = "linux") {
            let mut c = isolated::bin("setsid");
            c.args(["-w", bin]);
            c
        } else {
            isolated::bin(bin)
        };
        c.args(args)
            .env("XDG_STATE_HOME", &state)
            .env_remove("MIKATA_BODY")
            .stdin(std::process::Stdio::null());
        if let Some(b) = body {
            c.env("MIKATA_BODY", b);
        }
        c.output().unwrap()
    };
    let settled = || Facts::load_in(Some(&state), &world).facts[&id].settled;
    let body = detached(&["onto", "settle", &id, "--world", w], Some("slime-a"));
    assert_eq!(body.status.code(), Some(2));
    let unknown = detached(&["onto", "settle", "nope", "--world", w], None);
    assert_eq!(unknown.status.code(), Some(2));
    let piped = detached(&["onto", "settle", &id, "--world", w], None);
    assert_eq!(piped.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&piped.stderr).contains("terminal"),
        "{piped:?}"
    );
    assert!(!settled(), "no terminal, no settle");
    #[cfg(target_os = "linux")]
    {
        use std::io::Write as _;
        let at_tty = |input: &str| {
            let line = format!("{bin} onto settle '#{id}' --world '{w}'");
            let mut child = isolated::bin("script")
                .args(["-qec", &line, "/dev/null"])
                .env("XDG_STATE_HOME", &state)
                .env_remove("MIKATA_BODY")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdin = child.stdin.take().unwrap();
            // Typed once the question is up: the terminal reads it, not the pipe.
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = stdin.write_all(input.as_bytes());
            drop(stdin);
            child.wait_with_output().unwrap()
        };
        let no = at_tty("n\n");
        assert!(
            String::from_utf8_lossy(&no.stdout).contains("build=cargo test"),
            "the fact is shown: {no:?}"
        );
        assert!(!settled(), "`n` settles nothing: {no:?}");
        let yes = at_tty("y\n");
        assert!(
            String::from_utf8_lossy(&yes.stdout).contains(&format!("settled #{id}")),
            "{yes:?}"
        );
        assert!(settled(), "{yes:?}");
    }
    let _ = fs::remove_dir_all(&root);
}

/// A proposed claim is listed with its vouch; the human confirms it, and it is grounded and
/// shared by the confirmed route. A body is never let, and an id no creature proposes is refused.
#[test]
fn the_human_confirms_a_proposed_claim_and_a_body_cannot() {
    use mikata_server::world::memory::{Status, Tier, fold, propose, read};
    let world = std::env::temp_dir().join(format!("mikata-memory-confirm-{}", std::process::id()));
    let _ = fs::remove_dir_all(&world);
    fs::create_dir_all(&world).unwrap();
    let claim = propose(&world, "slime-a", "run uv before pytest", vec![], None).unwrap();
    let id = claim.note.clone();
    let (_, out) = status(&world, false);
    assert!(
        out.contains(&format!(
            "proposed: {id} — run uv before pytest (vouch: mikatad memory confirm {id})"
        )),
        "{out}"
    );
    let (_, json) = status(&world, true);
    assert!(
        json.contains(&format!(
            r#""proposed":[{{"id":"{id}","text":"run uv before pytest"}}]"#
        )),
        "{json}"
    );
    assert!(
        !json.contains("vouch"),
        "no claim among the pathologies: {json}"
    );
    fs::create_dir_all(world.join(".mikata/memory/creature/stray")).unwrap();
    let w = world.to_str().unwrap();
    let bin = env!("CARGO_BIN_EXE_mikatad");
    let run = |args: &[&str], body: Option<&str>| {
        let mut c = isolated::bin(bin);
        c.args(args).args(["--world", w]);
        match body {
            Some(b) => c.env("MIKATA_BODY", b),
            None => c.env_remove("MIKATA_BODY"),
        };
        c.output().unwrap()
    };
    assert_eq!(
        run(&["memory", "confirm", &id], Some("slime-a"))
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        run(&["memory", "confirm", "n0"], None).status.code(),
        Some(2)
    );
    let human = run(&["memory", "confirm", &id], None);
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    let shared = fold(&read(&world, &Tier::Shared).unwrap().rows);
    assert_eq!(shared[&id].status, Status::Grounded);
    assert!(
        shared[&id]
            .proof
            .as_deref()
            .unwrap()
            .starts_with("confirmed: ")
    );
    let _ = fs::remove_dir_all(&world);
}

/// `memory status` says when no claim of a light-coding session can be grounded, and the line to add.
#[test]
fn status_says_when_the_session_card_has_no_verify_line() {
    let world = std::env::temp_dir().join(format!("mikata-memory-noverify-{}", std::process::id()));
    let _ = fs::remove_dir_all(&world);
    fs::create_dir_all(world.join(".mikata/summoner/summoner")).unwrap();
    fs::create_dir_all(world.join(".mikata/memory/creature")).unwrap();
    fs::write(
        world.join(".mikata/summoner/summoner/README.md"),
        "# summoner\n\n- **Race:** summoner\n",
    )
    .unwrap();
    fs::write(world.join("go.mod"), "module x\n").unwrap();
    let (_, out) = status(&world, false);
    assert!(
        out.contains("note: summoner's card") && out.contains("`go test ./...`"),
        "{out}"
    );
    let _ = fs::remove_dir_all(&world);
}

/// `mikatad <args> --world <world>`, as the human (`body` unset) or from inside a body's command.
fn mikatad(world: &std::path::Path, args: &[&str], body: Option<&str>) -> std::process::Output {
    let mut c = isolated::bin(env!("CARGO_BIN_EXE_mikatad"));
    c.args(args).args(["--world", world.to_str().unwrap()]);
    match body {
        Some(b) => c.env("MIKATA_BODY", b),
        None => c.env_remove("MIKATA_BODY"),
    };
    c.output().unwrap()
}

/// A world whose journal's last entry answered `Make out.txt say fixed`.
fn world_that_answered(name: &str) -> std::path::PathBuf {
    let world = std::env::temp_dir().join(format!("mikata-memory-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&world);
    fs::create_dir_all(world.join(".mikata")).unwrap();
    fs::write(
        world.join(".mikata/log.md"),
        "\n---\n\n### [2026-11-22T10:00:00Z] summoner — gate: PASS, unchecked\n\
         - **Asked:** Make out.txt say fixed\n- **Changed:** `out.txt`\n\
         - **Unchecked:** no verify line ran\n- **By:** the end gate\n",
    )
    .unwrap();
    world
}

/// Saying an answer was wrong is the human's act: a body's command is refused and writes
/// nothing, and so is a correction of nothing — no words, or no ask to say them of.
#[test]
fn a_correction_is_the_humans_and_names_what_it_corrects() {
    let world = world_that_answered("wrong-refused");
    let journal = || fs::read_to_string(world.join(".mikata/log.md")).unwrap();
    let body = mikatad(
        &world,
        &["memory", "wrong", "it still says broken"],
        Some("summoner"),
    );
    assert_eq!(body.status.code(), Some(2));
    assert_eq!(
        mikatad(&world, &["memory", "wrong"], None).status.code(),
        Some(2)
    );
    assert!(!journal().contains("wrong:"), "{}", journal());
    // A journal with no entry that says what it was asked, and no ask named.
    fs::write(world.join(".mikata/log.md"), "# Journal\n").unwrap();
    let unnamed = mikatad(&world, &["memory", "wrong", "it still says broken"], None);
    assert_eq!(unnamed.status.code(), Some(2));
    let why = String::from_utf8_lossy(&unnamed.stderr).into_owned();
    assert!(why.contains("--ask"), "{why}");
    let _ = fs::remove_dir_all(&world);
}

/// The human says an answer was wrong: their words go to the journal and are kept as a correction
/// under the ask the journal's last entry answers, shared and grounded, linking what was learned
/// under that ask; `--ask` names another ask by its first line.
#[test]
fn the_human_says_an_answer_was_wrong() {
    use mikata_server::world::memory::{Polarity, Status, Tier, fold, propose_under, read};
    let world = world_that_answered("wrong");
    let learned = propose_under(
        &world,
        "summoner",
        "out.txt is written by the build",
        vec!["law".into()],
        Some("Make out.txt say fixed"),
        vec![],
    )
    .unwrap();
    // In as many words as they type.
    let human = mikatad(
        &world,
        &["memory", "wrong", "it", "still", "says", "broken"],
        None,
    );
    let out = String::from_utf8_lossy(&human.stdout).into_owned();
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    assert!(
        out.contains("under the ask Make out.txt say fixed"),
        "{out}"
    );
    assert!(
        out.contains(&format!(
            "it corrects 1 note(s) learned under that ask: {}",
            learned.note
        )),
        "{out}"
    );
    let journal = fs::read_to_string(world.join(".mikata/log.md")).unwrap();
    for line in [
        "] the human — wrong: Make out.txt say fixed\n",
        "- **Asked:** Make out.txt say fixed\n- **Said:** it still says broken\n",
    ] {
        assert!(journal.contains(line), "{line}: {journal}");
    }
    let shared = fold(&read(&world, &Tier::Shared).unwrap().rows);
    assert_eq!(shared.len(), 1, "{shared:?}");
    let kept = shared.values().next().unwrap();
    assert_eq!(kept.text, "it still says broken");
    assert_eq!(
        (kept.status, kept.polarity),
        (Status::Grounded, Polarity::Revise)
    );
    assert_eq!(kept.asked.as_deref(), Some("Make out.txt say fixed"));
    assert_eq!(kept.links.len(), 1);
    assert_eq!(kept.links[0].note, learned.note);
    assert!(
        kept.proof
            .as_deref()
            .unwrap()
            .contains("the human — wrong:"),
        "it cites the journal entry that holds the human's word: {kept:?}"
    );
    // Memory still resolves whole: a human's word needs no other proof.
    assert_eq!(status(&world, false).0, Some(0));
    let other = mikatad(
        &world,
        &[
            "memory",
            "wrong",
            "--ask",
            "Sort the report",
            "ties come out in any order",
        ],
        None,
    );
    assert!(other.status.success());
    let shared = fold(&read(&world, &Tier::Shared).unwrap().rows);
    let second = shared
        .values()
        .find(|n| n.asked.as_deref() == Some("Sort the report"))
        .unwrap_or_else(|| panic!("{shared:?}"));
    assert_eq!(second.text, "ties come out in any order");
    assert!(
        second.links.is_empty(),
        "nothing was learned under that ask"
    );
    let _ = fs::remove_dir_all(&world);
}
