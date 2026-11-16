//! Ownership: one function, the first rule that matches — on this repository's own world and on
//! worlds made to break it.

use std::fs;
use std::path::{Path, PathBuf};

use mikata_world::{Owner, Roster, Rule, explain, own, soil};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn creature(name: &str) -> Owner {
    Owner::Creature(name.into())
}

#[test]
fn this_repositorys_world_reads_whole() {
    let roster = Roster::cards(&repo());
    assert!(roster.problems.is_empty(), "{:?}", roster.problems);
    assert!(roster.creatures.len() >= 20, "{}", roster.creatures.len());
    assert_eq!(roster.loops(), [] as [std::string::String; 0]);
    assert_eq!(
        roster.chain("slime-wire"),
        ["orc-law", "elf-engine", "summoner"]
    );
    assert!(roster.is_above("summoner", "slime-wire"));
    assert!(!roster.is_above("slime-wire", "orc-law"));
}

#[test]
fn every_path_of_this_world_has_the_owner_its_docs_say() {
    let roster = Roster::cards(&repo());
    let cases = [
        ("crates/mikata-wire/src/lib.rs", creature("slime-wire")),
        ("crates/mikata-loop/src/run.rs", creature("slime-loop")),
        // No Slime's zone covers these: they fall to the Orc whose domain does.
        ("crates/mikata-config/src/lib.rs", creature("orc-engine")),
        ("packages/theme/src/tokens.css", creature("orc-clients")),
        (".cargo/config.toml", creature("slime-workspace")),
        (".github/ISSUE_TEMPLATE/bug.yml", Owner::Unowned),
        ("Cargo.toml", creature("slime-workspace")),
        ("tools/check-layers.mjs", creature("slime-workspace")),
        (".github/workflows/ci.yml", creature("kijin-ci")),
        (".mikata/slime/wire/README.md", creature("slime-wire")),
        (".mikata/shared/queue.jsonl", Owner::Harness),
        (".mikata/instruments/usage.jsonl", Owner::Harness),
        (".mikata/log.md", Owner::Harness),
        (".mikata/law.md", Owner::Human),
        (".mikata/config.toml", Owner::Human),
        (".mikata/dimension.toml", creature("summoner")),
        (".mikata/plan.md", creature("summoner")),
        (".mikata/planet.md", Owner::Unowned),
        (".mikata/tmp/summoner/x", creature("summoner")),
        (".mikata/tmp/evolution/7/README.md", Owner::Harness),
        // A path is read segment by segment: `..` never carries a claim out of its zone.
        ("crates/mikata-wire/../../.mikata/law.md", Owner::Human),
        (
            "crates/mikata-wire/../mikata-config/src/lib.rs",
            creature("orc-engine"),
        ),
        (
            ".mikata/shared/../slime/wire/README.md",
            creature("slime-wire"),
        ),
        ("crates//mikata-wire/./src/lib.rs", creature("slime-wire")),
        ("crates/mikata-wire/", creature("slime-wire")),
        ("../elsewhere/x.rs", Owner::Unowned),
        ("/.mikata/law.md", Owner::Unowned),
        ("", Owner::Unowned),
        (".", Owner::Unowned),
        (".mikata", Owner::Unowned),
    ];
    for (path, owner) in cases {
        assert_eq!(own(&roster, path), owner, "{path}");
    }
}

struct World(PathBuf);

impl World {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("mikata-own-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        World(root)
    }

    fn doc(&self, race: &str, short: &str, body: &str) -> &Self {
        let dir = self.0.join(".mikata").join(race).join(short);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("README.md"), body).unwrap();
        self
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Lesson: a creature claiming the world directory itself swallowed every seat, the shared
/// queue and scratch. The order is the rule: harness soil, scratch and seats are asked first.
#[test]
fn a_creature_claiming_the_records_owns_none_of_the_seats_the_queue_or_scratch() {
    let w = World::new("greedy");
    w.doc("orc", "boss", "# orc-boss\n\n- **Race:** orc\n- **Reports to:** summoner\n- **Domain:** src/\n")
        .doc("slime", "greedy", "# slime-greedy\n\n- **Race:** slime\n- **Reports to:** orc-boss\n- **Territory:** .mikata/\n")
        .doc("slime", "small", "# slime-small\n\n- **Race:** slime\n- **Reports to:** orc-boss\n- **Territory:** src/small/\n");
    let roster = Roster::cards(&w.0);
    assert!(roster.problems.is_empty(), "{:?}", roster.problems);
    assert_eq!(
        own(&roster, ".mikata/slime/small/README.md"),
        creature("slime-small")
    );
    assert_eq!(
        own(&roster, ".mikata/orc/boss/README.md"),
        creature("orc-boss")
    );
    assert_eq!(own(&roster, ".mikata/shared/queue.jsonl"), Owner::Harness);
    assert_eq!(
        own(&roster, ".mikata/tmp/slime-small/x"),
        creature("slime-small")
    );
    assert_eq!(own(&roster, ".mikata/notes.md"), creature("slime-greedy"));
    assert_eq!(own(&roster, "src/small/a.rs"), creature("slime-small"));
    assert_eq!(own(&roster, "src/other.rs"), creature("orc-boss"));
    assert_eq!(own(&roster, "lib/x.rs"), Owner::Unowned);
}

/// Retired creatures own nothing; a tie on one prefix breaks by name; a loop is found.
#[test]
fn retired_ties_and_loops() {
    let w = World::new("edges");
    w.doc(
        "slime",
        "a",
        "# slime-a\n\n- **Race:** slime\n- **Reports to:** slime-b\n- **Territory:** src/\n",
    )
    .doc(
        "slime",
        "b",
        "# slime-b\n\n- **Race:** slime\n- **Reports to:** slime-a\n- **Territory:** src/\n",
    )
    .doc(
        "slime",
        "old",
        "# slime-old\n\n- **Race:** slime\n- **Territory:** lib/\n- **Retired:** yes\n",
    );
    let roster = Roster::cards(&w.0);
    assert_eq!(
        own(&roster, "src/x.rs"),
        creature("slime-a"),
        "a tie breaks by name, the same every time"
    );
    assert_eq!(own(&roster, "lib/x.rs"), Owner::Unowned);
    assert_eq!(own(&roster, ".mikata/slime/old/README.md"), Owner::Unowned);
    assert_eq!(roster.loops(), ["slime-a", "slime-b"]);
}

/// A doc that cannot be read is reported, never guessed; an Orc's ground is its Domain.
#[test]
fn a_doc_that_does_not_read_is_reported() {
    let w = World::new("bad");
    w.doc(
        "orc",
        "x",
        "# orc-x\n\n- **Race:** orc\n- **Territory:** src/\n",
    )
    .doc("slime", "y", "no heading\n")
    .doc("slime", "z", "# slime-z\n\n- **Race:** wizard\n");
    let roster = Roster::cards(&w.0);
    assert!(roster.creatures.is_empty());
    let problems = roster.problems.join("\n");
    assert!(
        problems.contains("names its ground as Domain"),
        "{problems}"
    );
    assert!(problems.contains("no `# name` heading"), "{problems}");
    assert!(problems.contains("no race `wizard`"), "{problems}");
    let w = World::new("prose");
    w.doc(
        "slime",
        "p",
        "# slime-p\n\n- **Race:** slime\n- **Territory:** every file at the root\n",
    );
    let problems = Roster::cards(&w.0).problems.join("\n");
    assert!(
        problems.contains("`every` is not one"),
        "prose is never a claim: {problems}"
    );
}

/// The rules between them: a nested zone, a post over a zone and the workspace, an Elf's lane
/// owning nothing, an ascended race's seat, the one proving ground, a header that ends at any
/// heading, and a file claim.
#[test]
fn nested_posts_lanes_ascended_and_files() {
    let w = World::new("rules");
    w.doc("slime", "outer", "# slime-outer\n\n- **Race:** slime\n- **Territory:** src/\n")
        .doc("slime", "inner", "# slime-inner\n\n- **Race:** slime\n- **Territory:** src/inner/\n### Later\n- **Territory:** lib/\n")
        .doc("kijin", "release", "# kijin-release\n\n- **Race:** kijin\n- **Post:** ./CHANGELOG.md src/release/\n")
        .doc("elf", "plan", "# elf-plan\n\n- **Race:** elf\n- **Territory:** docs/\n")
        .doc("high-orc", "big", "# high-orc-big\n\n- **Race:** high-orc\n- **Domain:** docs/\n")
        .doc("dark-elf", "shade", "# dark-elf-shade\n\n- **Race:** dark-elf\n")
        .doc("dark-elf", "abyss", "# dark-elf-abyss\n\n- **Race:** dark-elf\n");
    let roster = Roster::cards(&w.0);
    assert!(roster.problems.is_empty(), "{:?}", roster.problems);
    assert_eq!(own(&roster, "src/a.rs"), creature("slime-outer"));
    assert_eq!(
        own(&roster, "src/inner/a.rs"),
        creature("slime-inner"),
        "the longest zone wins"
    );
    assert_eq!(
        own(&roster, "lib/a.rs"),
        Owner::Unowned,
        "a field under a later heading is prose"
    );
    assert_eq!(
        own(&roster, "src/release/x.toml"),
        creature("kijin-release"),
        "a post before a zone"
    );
    assert_eq!(
        own(&roster, "CHANGELOG.md"),
        creature("kijin-release"),
        "a post before the workspace"
    );
    assert_eq!(
        own(&roster, "CHANGELOG.md.bak"),
        Owner::Unowned,
        "a file claim is the file alone"
    );
    assert_eq!(
        own(&roster, "docs/x.md"),
        creature("high-orc-big"),
        "an Elf's lane owns nothing"
    );
    assert_eq!(
        own(&roster, ".mikata/high-orc/big/README.md"),
        creature("high-orc-big")
    );
    assert_eq!(
        own(&roster, ".mikata/dark-elf/witness/x"),
        creature("dark-elf-abyss")
    );
    // No throne doc: the throne's files are nobody's until it has one.
    assert_eq!(own(&roster, ".mikata/plan.md"), Owner::Unowned);
}

/// A claim is a path inside the world, never one that climbs out or starts at the root.
#[test]
fn a_claim_outside_the_world_is_no_claim() {
    for bad in ["../other/", "/abs/", "/", "./", "src/../../x/", "every"] {
        let w = World::new("outside");
        w.doc(
            "slime",
            "x",
            &format!("# slime-x\n\n- **Race:** slime\n- **Territory:** {bad}\n"),
        );
        let roster = Roster::cards(&w.0);
        assert!(roster.creatures.is_empty(), "`{bad}` was taken as a claim");
        assert!(
            roster.problems[0].contains("is not one"),
            "{:?}",
            roster.problems
        );
    }
    let w = World::new("unicode");
    w.doc(
        "slime",
        "fr",
        "# slime-fr\n\n- **Race:** slime\n- **Territory:** docs/français/\n",
    );
    assert_eq!(
        own(&Roster::cards(&w.0), "docs/français/a.md"),
        creature("slime-fr")
    );
}

/// A doc that cannot be read is reported, and its change moves the fingerprint.
#[test]
fn a_doc_that_cannot_be_read_is_reported_and_fingerprinted() {
    let w = World::new("binary");
    w.doc(
        "slime",
        "ok",
        "# slime-ok\n\n- **Race:** slime\n- **Territory:** src/\n",
    );
    let before = Roster::cards(&w.0).fingerprint;
    let doc = w.0.join(".mikata/slime/bad/README.md");
    fs::create_dir_all(doc.parent().unwrap()).unwrap();
    fs::write(&doc, [0xff, 0xfe, 0x00]).unwrap();
    let roster = Roster::cards(&w.0);
    assert_ne!(roster.fingerprint, before);
    assert!(
        roster
            .problems
            .iter()
            .any(|p| p.contains("the doc cannot be read")),
        "{:?}",
        roster.problems
    );
    fs::write(&doc, [0xff, 0xfe, 0x01]).unwrap();
    assert_ne!(Roster::cards(&w.0).fingerprint, roster.fingerprint);
}

/// The soil handed to a creature claiming the records gives away what the earlier rules give
/// away: it agrees with `own` on every place the rules are about.
#[test]
fn a_greedy_creatures_soil_leaves_the_seats_the_queue_and_scratch() {
    let w = World::new("greedy-soil");
    w.doc(
        "orc",
        "boss",
        "# orc-boss\n\n- **Race:** orc\n- **Domain:** src/\n",
    )
    .doc(
        "slime",
        "greedy",
        "# slime-greedy\n\n- **Race:** slime\n- **Territory:** .mikata/ src/\n",
    )
    .doc(
        "slime",
        "small",
        "# slime-small\n\n- **Race:** slime\n- **Territory:** src/small/\n",
    )
    .doc(
        "kijin",
        "release",
        "# kijin-release\n\n- **Race:** kijin\n- **Post:** ./CHANGELOG.md src/small/release/\n",
    )
    .doc(
        "slime",
        "workspace",
        "# slime-workspace\n\n- **Race:** slime\n- **Territory:** tools/\n",
    );
    let roster = Roster::cards(&w.0);
    let greedy = soil(&roster, "slime-greedy").unwrap();
    for p in [
        ".mikata/shared/queue.jsonl",
        ".mikata/slime/small/README.md",
        ".mikata/tmp/slime-small/x",
        ".mikata/law.md",
    ] {
        assert!(!greedy.admits(p), "{p}");
    }
    assert!(greedy.admits(".mikata/notes.md"));
    assert!(greedy.admits(".mikata/tmp/slime-greedy/x"));
    let paths = [
        ".mikata/notes.md",
        ".mikata/shared/q",
        ".mikata/slime/small/README.md",
        ".mikata/slime/greedy/README.md",
        ".mikata/tmp/slime-small/x",
        ".mikata/tmp/slime-greedy/x",
        ".mikata/tmp/other/x",
        ".mikata/log.md",
        ".mikata/plan.md",
        "src/a.rs",
        "src/small/a.rs",
        "src/small/release/r.toml",
        "CHANGELOG.md",
        "README.md",
        "tools/x.mjs",
        ".mikata",
    ];
    for c in roster.living() {
        let s = soil(&roster, &c.name).unwrap();
        for p in paths {
            let e = explain(&roster, p);
            let writes = e.owner == Owner::Creature(c.name.clone()) && e.rule != Rule::Domain;
            assert_eq!(s.admits(p), writes, "{} on {p}: {e:?}, soil {s:?}", c.name);
        }
    }
}

/// The family a reader worries about: claims nested three deep with alternating owners, a file
/// claim inside another's directory, nested posts, a zone inside a post, a seat, scratch and the
/// proving ground inside zones, a retired creature's old claims. Every soil agrees with `own`.
#[test]
fn nested_claims_every_soil_agrees_with_own() {
    let w = World::new("nested");
    w.doc(
        "slime",
        "a",
        "# slime-a\n\n- **Race:** slime\n- **Territory:** src/ src/b/a/ .mikata/\n",
    )
    .doc(
        "slime",
        "b",
        "# slime-b\n\n- **Race:** slime\n- **Territory:** src/b/ src/x.rs .github/scripts/\n",
    )
    .doc(
        "kijin",
        "out",
        "# kijin-out\n\n- **Race:** kijin\n- **Post:** .github/ ./VERSION\n",
    )
    .doc(
        "kijin",
        "in",
        "# kijin-in\n\n- **Race:** kijin\n- **Post:** .github/workflows/\n",
    )
    .doc("dark-elf", "d", "# dark-elf-d\n\n- **Race:** dark-elf\n")
    .doc(
        "summoner",
        "summoner",
        "# summoner\n\n- **Race:** summoner\n",
    )
    .doc(
        "slime",
        "old",
        "# slime-old\n\n- **Race:** slime\n- **Territory:** src/b/a/old/\n- **Retired:** yes\n",
    );
    let roster = Roster::cards(&w.0);
    assert!(roster.problems.is_empty(), "{:?}", roster.problems);
    let paths = [
        "src/y.rs",
        "src/b/y.rs",
        "src/b/a/y.rs",
        "src/b/a/old/y.rs",
        "src/x.rs",
        "src/x.rs.bak",
        ".github/ci.yml",
        ".github/workflows/w.yml",
        ".github/scripts/s.sh",
        "VERSION",
        "README.md",
        ".mikata/notes.md",
        ".mikata/slime/b/README.md",
        ".mikata/slime/old/README.md",
        ".mikata/tmp/slime-b/x",
        ".mikata/tmp/ghost/x",
        ".mikata/tmp/",
        ".mikata/tmp/evolution/1/x",
        ".mikata/dark-elf/witness/x",
        ".mikata/dark-elf/d/README.md",
        ".mikata/plan.md",
        ".mikata/shared/q",
        ".mikata/log.md",
        ".mikata/law.md",
        "src/b/",
        ".mikata",
    ];
    for c in roster.living() {
        let s = soil(&roster, &c.name).unwrap();
        for p in paths {
            let e = explain(&roster, p);
            let writes = e.owner == Owner::Creature(c.name.clone()) && e.rule != Rule::Domain;
            assert_eq!(s.admits(p), writes, "{} on {p}: {e:?}, soil {s:?}", c.name);
        }
    }
    assert_eq!(
        explain(&roster, ".mikata/tmp/").claim.as_deref(),
        Some(".mikata/tmp/")
    );
}
