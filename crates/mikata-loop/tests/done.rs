//! The ask's own `Done when` list, held by the harness (`mikata_loop::done`): a study's replay of
//! it over kept trees, with no model. The list's own tests are beside its code, and the run's are
//! in `solo.rs`.

use std::fs;
use std::path::PathBuf;

use mikata_loop::done::List;
use mikata_races::Race;
use mikata_tools::{Context, Sandbox, registry};

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
