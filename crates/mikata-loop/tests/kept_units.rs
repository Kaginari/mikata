//! The human's own unit changes kept for one world, never in the global file: a unit switched
//! off that only tightens in the world's local file; a guard switched off, or a unit switched on,
//! in the human's own file for that world, out of its reach; a plugin's, the session's alone.

use std::fs;
use std::path::PathBuf;

use mikata_config::{Config, Layer, Paths};
use mikata_loop::units::{APPROVAL, MEMORY, SECRETS, WHAT_CHANGED, keep, kept_in};

fn paths(name: &str) -> (PathBuf, Paths) {
    let root = std::env::temp_dir().join(format!("mikata-kept-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("world/.mikata")).unwrap();
    fs::create_dir_all(root.join("global")).unwrap();
    let paths = Paths::with_global(root.join("global/config.toml"), &root.join("world"))
        .with_human(root.join("state/worlds/w/config.toml"));
    (root, paths)
}

#[test]
fn each_change_is_kept_where_its_world_may_hold_it() {
    let (root, paths) = paths("where");
    let own = paths.human.clone().unwrap();
    let place = |id: &str, on: bool| kept_in(&paths, id, on).map(|(f, _)| f);
    // Off, and it only tightens: the world's local file may hold it.
    assert_eq!(place(WHAT_CHANGED, false), Some(paths.local.clone()));
    // A guard switched off, or any unit switched on: the human's own, out of the world's reach.
    for id in [SECRETS, APPROVAL, MEMORY] {
        assert_eq!(place(id, false), Some(own.clone()), "{id} off");
    }
    assert_eq!(place(WHAT_CHANGED, true), Some(own.clone()));
    // A plugin's, or no unit's, is the session's alone.
    assert_eq!(place("plugin:no-force-push", false), None);
    assert_eq!(place("no-such-unit", false), None);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_kept_change_is_what_the_next_session_reads_and_the_global_file_is_untouched() {
    let (root, paths) = paths("round-trip");
    fs::write(&paths.global, "# mine\n[units]\nfold = true\n").unwrap();
    let global_before = fs::read_to_string(&paths.global).unwrap();
    keep(&paths, APPROVAL, false).unwrap();
    keep(&paths, WHAT_CHANGED, false).unwrap();
    assert_eq!(fs::read_to_string(&paths.global).unwrap(), global_before);
    let c = Config::load(&paths).unwrap();
    let switch = |id: &str| c.units().into_iter().find(|s| s.id == id).unwrap();
    let approval = switch(APPROVAL);
    assert!(!approval.on && approval.origin.layer == Layer::Human && !approval.from_world());
    let changed = switch(WHAT_CHANGED);
    assert!(!changed.on && changed.origin.layer == Layer::Local);
    assert!(
        c.holes().is_empty(),
        "nothing kept is refused: {:?}",
        c.holes()
    );
    let _ = fs::remove_dir_all(root);
}
