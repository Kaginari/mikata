//! A world's own files the session keeps its changes in: the local file in the world, made once,
//! and the human's own file for that world under the state directory, which holds what a world's
//! file may not and goes over every other file, for that world alone.

use std::fs;
use std::path::PathBuf;

use mikata_config::{Config, Layer, Paths, ensure_local, set_switch};

struct Ground {
    root: PathBuf,
}

impl Ground {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("mikata-world-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("world/.mikata")).unwrap();
        fs::create_dir_all(root.join("global")).unwrap();
        fs::create_dir_all(root.join("state/worlds/w")).unwrap();
        Self { root }
    }

    fn paths(&self) -> Paths {
        Paths::with_global(
            self.root.join("global/config.toml"),
            &self.root.join("world"),
        )
    }

    fn own(&self) -> PathBuf {
        self.root.join("state/worlds/w/config.toml")
    }
}

impl Drop for Ground {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn the_humans_own_file_for_a_world_holds_what_its_files_may_not() {
    let g = Ground::new("human");
    let paths = g.paths();
    fs::write(&paths.global, "[units]\nsecrets = true\n").unwrap();
    fs::write(
        &paths.local,
        "[units]\nsecrets = false\nwhat_changed = false\n",
    )
    .unwrap();
    fs::write(g.own(), "[units]\nsecrets = false\nmemory = false\n").unwrap();
    let c = Config::load(&paths.clone().with_human(g.own())).unwrap();
    let switch = |id: &str| c.units().into_iter().find(|s| s.id == id).unwrap();
    // The human's own file wins, and is no world's: what it switches off stands.
    let secrets = switch("secrets");
    assert!(!secrets.on);
    assert_eq!(secrets.origin.layer, Layer::Human);
    assert!(!secrets.from_world());
    assert!(!switch("memory").on);
    // What only the local file says stays the local file's.
    assert_eq!(switch("what_changed").origin.layer, Layer::Local);
    // Without the human's own file, the local word is a world's, which the loop refuses for a
    // guard.
    let other = Config::load(&paths).unwrap();
    let s = other
        .units()
        .into_iter()
        .find(|s| s.id == "secrets")
        .unwrap();
    assert!(s.from_world());
}

#[test]
fn the_local_file_is_made_once_and_a_kept_switch_keeps_the_comments() {
    let g = Ground::new("local-made");
    let paths = g.paths();
    assert!(ensure_local(&paths).unwrap());
    let made = fs::read_to_string(&paths.local).unwrap();
    assert!(made.starts_with("# This machine's configuration"), "{made}");
    fs::write(&paths.local, "# mine\n[units]\nfold = false # kept\n").unwrap();
    assert!(!ensure_local(&paths).unwrap(), "never made twice");
    set_switch(&paths.local, "units.what_changed", false).unwrap();
    let text = fs::read_to_string(&paths.local).unwrap();
    assert!(
        text.contains("# mine") && text.contains("fold = false # kept"),
        "{text}"
    );
    assert!(text.contains("what_changed = false"), "{text}");
    // The human's own file, with its folders, made on the first switch kept there.
    let own = g.root.join("state/worlds/new/config.toml");
    set_switch(&own, "units.approval", false).unwrap();
    assert!(
        fs::read_to_string(&own)
            .unwrap()
            .contains("approval = false")
    );
}
