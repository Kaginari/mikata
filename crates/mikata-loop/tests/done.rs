//! The ask's own `Done when` list, held by the harness (`mikata_loop::done`): a study's replay of
//! it over kept trees, with no model. The list's own tests are beside its code, and the run's are
//! in `solo.rs`.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mikata_loop::done::List;
use mikata_loop::open::{Bound, open_with};
use mikata_loop::{Config, Paths, ScriptedModel};
use mikata_provider::Block;
use mikata_races::Race;
use mikata_tools::{Context, Sandbox, registry};

/// Every text the model was sent in its request number `i`, as one.
fn sent(model: &ScriptedModel, i: usize) -> String {
    model.requests()[i]
        .messages
        .iter()
        .flat_map(|m| &m.content)
        .filter_map(|b| match b {
            Block::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A session opened as the human opens one is light-coding's: its body holds the `check` tool,
/// and a reader named in the human's own file reads each ask apart before the body sees it. Its
/// list goes after the ask, with how the harness holds it.
#[tokio::test]
async fn a_light_coding_session_holds_the_check_tool_and_the_humans_reader() {
    let root = std::env::temp_dir().join(format!("mikata-done-open-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let w = root.join("world");
    fs::create_dir_all(w.join(".mikata")).unwrap();
    fs::write(w.join("total.sh"), "echo 3\n").unwrap();
    fs::write(
        root.join("global.toml"),
        "[models]\nreader = \"a-strong-reader\"\n",
    )
    .unwrap();
    let model = Arc::new(ScriptedModel::new([
        ScriptedModel::text("D1: `sh total.sh` must stay as it is today"),
        ScriptedModel::text("it is as you want it"),
    ]));
    let config = Config::load(&Paths::with_global(root.join("global.toml"), &w)).unwrap();
    assert_eq!(config.reader().as_deref(), Some("a-strong-reader"));
    let attic = root.join("attic");
    let mut opened = open_with(
        &w,
        &config,
        Bound {
            model: model.clone(),
            model_id: "scripted-model",
            attic: Some(&attic),
        },
        Arc::new(mikata_tools::NoHuman),
        Arc::new(|_: &mikata_loop::Seen| {}),
        |_| None,
    )
    .unwrap();
    opened
        .session
        .run()
        .ask("Leave total.sh alone.", &mut |_| {})
        .await;
    let requests = model.requests();
    assert_eq!(requests[0].model, "a-strong-reader");
    assert!(requests[0].tools.is_empty(), "a reader holds no tool");
    assert_eq!(sent(&model, 0), "Leave total.sh alone.");
    assert!(requests[1].tools.iter().any(|t| t.name == "check"));
    let told = sent(&model, 1);
    assert!(
        told.contains(
            "Leave total.sh alone.\n\n## Done when\nD1: `sh total.sh` must stay as it is today\n\nThe ask holds a `## Done when` list (D1)"
        ),
        "{told}"
    );
    let _ = fs::remove_dir_all(&root);
}

fn unguarded(world: &PathBuf) -> Context {
    Context::new(Race::Slime, world, &["."])
        .with_sandbox(Sandbox::None("a replay by hand runs unguarded".into()))
        .with_unguarded(true)
}

/// What a list would have said of trees a study kept. `MIKATA_HELD_ASK` is a file holding an ask
/// with its `## Done when` list; `MIKATA_HELD_CHECKS` a JSON file of the checks to hand in, each
/// `{"line", "command"}` with `"gives"` where the line states its output; `MIKATA_HELD_START` the
/// tree as the ask found it, where the checks are handed in; `MIKATA_HELD_TREES` the trees to
/// judge, `:` between. Every tree is run in: hand it copies.
#[test]
#[ignore = "a study's replay: it needs kept trees"]
fn replay_a_list_over_kept_trees() {
    let var = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"));
    let ask = fs::read_to_string(var("MIKATA_HELD_ASK")).unwrap();
    let checks: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(var("MIKATA_HELD_CHECKS")).unwrap()).unwrap();
    let start = PathBuf::from(var("MIKATA_HELD_START"))
        .canonicalize()
        .unwrap();
    let mut list = List::of(&ask);
    println!(
        "{}\n",
        list.note()
            .unwrap_or_else(|| "the ask holds no list".to_owned())
    );
    let mut cx = unguarded(&start);
    let mut refused = 0;
    for check in &checks {
        let o = list.hand_in(
            &check.to_string(),
            true,
            |_| Err("a replay hands every check in on the starting tree".to_owned()),
            &registry(),
            &mut cx,
        );
        refused += usize::from(o.is_error);
        println!(
            "{} {}: {}",
            if o.is_error { "REFUSED" } else { "held   " },
            check["line"].as_str().unwrap_or("?"),
            o.content.replace('\n', " | ")
        );
    }
    println!("\n{} checks handed in, {refused} refused", checks.len());
    for tree in var("MIKATA_HELD_TREES")
        .split(':')
        .filter(|t| !t.is_empty())
    {
        let world = PathBuf::from(tree).canonicalize().unwrap();
        let mut cx = unguarded(&world);
        let judged = list.judge(&registry(), &mut cx);
        println!(
            "\n== {tree}\n{} of {} lines hold ({} but for letter case or blanks); {} failures",
            judged.hold,
            list.checks().len(),
            judged.loosely,
            judged.failures.len()
        );
        for f in &judged.failures {
            println!("  {f}");
        }
    }
}
