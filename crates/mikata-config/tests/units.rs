//! `[units]`, `[plugins.*]` and `--units`: which parts of the loop run, who may say so, and how a
//! command line asks for another composition.

use std::fs;
use std::path::PathBuf;

use mikata_config::{
    Composition, Config, HOOK_MS, Layer, Paths, UnitChange, global_only, lawful_unit_id,
};
use mikata_races::Race;

/// A world and a global directory under a fresh temporary root.
struct Ground {
    root: PathBuf,
}

impl Ground {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("mikata-units-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("world/.mikata")).unwrap();
        fs::create_dir_all(root.join("global")).unwrap();
        Self { root }
    }

    fn paths(&self) -> Paths {
        Paths::with_global(
            self.root.join("global/config.toml"),
            &self.root.join("world"),
        )
    }

    fn global(&self, text: &str) -> &Self {
        fs::write(self.paths().global, text).unwrap();
        self
    }

    fn tracked(&self, text: &str) -> &Self {
        fs::write(self.paths().tracked, text).unwrap();
        self
    }

    fn local(&self, text: &str) -> &Self {
        fs::write(self.paths().local, text).unwrap();
        self
    }

    fn load(&self) -> Result<Config, String> {
        Config::load(&self.paths())
    }
}

impl Drop for Ground {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The default composition is every unit on: no switch is set until a file sets one.
#[test]
fn no_unit_is_switched_until_a_file_says() {
    let g = Ground::new("none");
    let c = g.load().unwrap();
    assert_eq!(c.units(), []);
    assert!(c.unit("memory").is_none());
    let p = c.plugins().unwrap();
    assert_eq!(p.plugins, []);
    assert_eq!(p.hook_ms, HOOK_MS);
}

/// The human's global file switches a unit either way, and the switch names its file.
#[test]
fn the_global_file_switches_units_either_way_with_their_origin() {
    let g = Ground::new("global");
    g.global("[units]\nmemory = false\nwhat_changed = true\n");
    let c = g.load().unwrap();
    let memory = c.unit("memory").unwrap();
    assert!(!memory.on);
    assert_eq!(memory.origin.layer, Layer::Global);
    assert!(!memory.from_world());
    assert!(c.unit("what_changed").unwrap().on);
    assert_eq!(
        c.units().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        ["memory", "what_changed"]
    );
    let explained = c.explain("units");
    assert_eq!(explained.len(), 2);
    assert!(explained.iter().all(|e| e.origin.layer == Layer::Global));
    assert_eq!(c.holes(), []);
}

/// A world's files may switch a unit off, and the switch says it came from the world, so the loop
/// can honour it only for a unit that tightens; switching one on there is a hole, and the global
/// file's word stands.
#[test]
fn a_world_file_may_only_switch_a_unit_off() {
    let g = Ground::new("world");
    g.global("[units]\nmemory = false\n");
    g.tracked("[units]\nmemory = true\nwhat_changed = false\n");
    g.local("[units]\nreader = false\nwhat_changed = true\n");
    let c = g.load().unwrap();
    assert!(
        !c.unit("memory").unwrap().on,
        "the global file's off stands"
    );
    let wc = c.unit("what_changed").unwrap();
    assert!(!wc.on);
    assert_eq!(wc.origin.layer, Layer::Tracked);
    assert!(wc.from_world());
    let reader = c.unit("reader").unwrap();
    assert!(!reader.on);
    assert_eq!(reader.origin.layer, Layer::Local);
    assert!(reader.from_world());
    let holes: Vec<_> = c
        .holes()
        .iter()
        .filter(|h| h.why.contains("only switch a unit off"))
        .map(|h| (h.key.as_str(), h.origin.layer))
        .collect();
    assert_eq!(
        holes,
        [
            ("units.memory", Layer::Tracked),
            ("units.what_changed", Layer::Local)
        ]
    );
    assert!(!global_only("units.memory"));
}

/// A unit's id is a plain lawful word; a plugin is not switched in `[units]`, and a switch is a
/// boolean.
#[test]
fn a_unit_id_is_lawful_and_a_switch_is_a_boolean() {
    let g = Ground::new("lawful");
    g.global("[units]\n\"plugin:x\" = false\n");
    let e = g.load().unwrap_err();
    assert!(
        e.contains("units.plugin:x") && e.contains("[plugins.<id>]"),
        "{e}"
    );
    g.global("[units]\nMemory = false\n");
    assert!(g.load().is_err());
    g.global("[units]\nmemory = \"off\"\n");
    let e = g.load().unwrap_err();
    assert!(e.contains("true or false"), "{e}");
    for id in ["memory", "what_changed", "done-when", "a1"] {
        assert!(lawful_unit_id(id), "{id}");
    }
    for id in ["", "1a", "_a", "Memory", "a.b", "plugin:x", &"a".repeat(33)] {
        assert!(!lawful_unit_id(id), "{id}");
    }
}

/// A plugin is read whole from the global file: its command, arguments, environment, grants in
/// their order, races, and whether it is enabled; the deadline on its hooks is the human's too.
#[test]
fn a_plugin_is_read_from_the_global_file() {
    let g = Ground::new("plugin");
    g.global(
        "[plugins.no-force-push]\ncommand = \"python3\"\nargs = [\"/p/no_force_push.py\"]\n\
         env = [\"HOME\"]\ngrants = [\"veto\", \"advise\"]\nraces = [\"slime\", \"mikata\"]\n\n\
         [plugins.watcher]\ncommand = \"watch\"\nenabled = false\n\n\
         [plugins.timeouts]\nhook_ms = 500\n",
    );
    let c = g.load().unwrap();
    let p = c.plugins().unwrap();
    assert_eq!(p.hook_ms, 500);
    assert_eq!(p.plugins.len(), 2);
    let nfp = &p.plugins[0];
    assert_eq!(nfp.id, "no-force-push");
    assert_eq!(nfp.unit_id(), "plugin:no-force-push");
    assert_eq!(nfp.command, "python3");
    assert_eq!(nfp.args, ["/p/no_force_push.py"]);
    assert_eq!(nfp.env, ["HOME"]);
    assert_eq!(nfp.grants, ["advise", "veto"], "in the grants' own order");
    assert_eq!(nfp.races, [Race::Slime, Race::Summoner]);
    assert!(nfp.enabled);
    let declared = nfp.declared();
    assert_eq!(declared.unit_id(), "plugin:no-force-push");
    assert_eq!(
        declared.grants,
        [
            mikata_tools::plugin::Grant::Advise,
            mikata_tools::plugin::Grant::Veto
        ]
    );
    assert_eq!(declared.races, nfp.races);
    assert_eq!(declared.args, nfp.args);
    let watcher = &p.plugins[1];
    assert_eq!(watcher.grants, ["observe"], "none named: observe alone");
    assert_eq!(watcher.races, []);
    assert!(!watcher.enabled);
    let explained = c.explain("plugins.no-force-push");
    assert_eq!(explained.len(), 5);
    assert!(explained.iter().all(|e| e.origin.layer == Layer::Global));
}

/// A world's file never loads a plugin, widens its grants or moves its deadline: each key is a
/// hole, and the global file's plugin is read as it wrote it.
#[test]
fn a_world_file_never_loads_a_plugin_or_widens_a_grant() {
    let g = Ground::new("plugin-world");
    g.global("[plugins.watcher]\ncommand = \"watch\"\ngrants = [\"observe\"]\n");
    g.tracked(
        "[plugins.evil]\ncommand = \"sh\"\n\n[plugins.watcher]\ngrants = [\"veto\", \"stop\"]\n",
    );
    g.local("[plugins.watcher]\nenabled = true\nraces = [\"slime\"]\n\n[plugins.timeouts]\nhook_ms = 60000\n");
    let c = g.load().unwrap();
    let p = c.plugins().unwrap();
    assert_eq!(p.plugins.len(), 1);
    assert_eq!(p.plugins[0].grants, ["observe"]);
    assert_eq!(p.plugins[0].races, []);
    assert_eq!(p.hook_ms, HOOK_MS);
    let mut holes: Vec<_> = c
        .holes()
        .iter()
        .filter(|h| h.why.contains("a plugin is a program"))
        .map(|h| (h.origin.layer, h.key.as_str()))
        .collect();
    holes.sort_unstable();
    assert_eq!(
        holes,
        [
            (Layer::Tracked, "plugins.evil.command"),
            (Layer::Tracked, "plugins.watcher.grants"),
            (Layer::Local, "plugins.timeouts.hook_ms"),
            (Layer::Local, "plugins.watcher.enabled"),
            (Layer::Local, "plugins.watcher.races"),
        ]
    );
    for key in [
        "plugins.x.command",
        "plugins.x.grants",
        "plugins.timeouts.hook_ms",
    ] {
        assert!(global_only(key), "{key}");
    }
}

/// What does not read is an error naming its file: an unknown grant, an unlawful id, a deadline
/// out of bounds, a plugin with no command, a race that is none.
#[test]
fn a_plugin_that_does_not_read_is_an_error() {
    let g = Ground::new("plugin-bad");
    for (text, says) in [
        (
            "[plugins.x]\ncommand = \"c\"\ngrants = [\"root\"]\n",
            "no grant `root`",
        ),
        ("[plugins.X]\ncommand = \"c\"\n", "a plugin's id"),
        ("[plugins.timeouts]\nhook_ms = 0\n", "from 1 to"),
        ("[plugins.timeouts]\nhook_ms = 60001\n", "from 1 to"),
        ("[plugins.x]\nenabled = true\n", ""),
        ("[plugins.x]\ncommand = \"c\"\nraces = [\"dragon\"]\n", ""),
        ("[plugins.x]\ncommand = 3\n", "must be a string"),
    ] {
        g.global(text);
        let e = match g.load() {
            Err(e) => e,
            Ok(c) => c.plugins().unwrap_err(),
        };
        assert!(e.contains(says), "{text}: {e}");
        if !says.is_empty() && !says.contains("string") {
            assert!(e.contains("global"), "names its file: {e}");
        }
    }
    g.global("[plugins.x]\nenabled = true\n");
    assert_eq!(
        g.load().unwrap().plugins().unwrap_err(),
        "plugins.x: no command"
    );
    g.global("[plugins.x]\ncommand = \"c\"\nraces = [\"dragon\"]\n");
    assert_eq!(
        g.load().unwrap().plugins().unwrap_err(),
        "plugins.x.races: `dragon` is no race"
    );
}

/// `--units` reads as a list of changes in the order given, and says back as it was asked.
#[test]
fn a_composition_reads_from_the_command_line() {
    let c = Composition::parse("-memory, -what_changed ,+plugin:no-force-push").unwrap();
    assert_eq!(
        c.changes(),
        [
            UnitChange {
                id: "memory".into(),
                on: false
            },
            UnitChange {
                id: "what_changed".into(),
                on: false
            },
            UnitChange {
                id: "plugin:no-force-push".into(),
                on: true
            },
        ]
    );
    assert_eq!(c.changes()[2].plugin(), Some("no-force-push"));
    assert_eq!(c.changes()[0].plugin(), None);
    assert_eq!(c.to_string(), "-memory,-what_changed,+plugin:no-force-push");
    assert_eq!(c.to_string().parse::<Composition>(), Ok(c));
    assert!(Composition::parse("").unwrap().is_empty());
    assert!(Composition::parse("  ").unwrap().is_empty());
}

/// Anything else is refused with what a unit looks like.
#[test]
fn a_composition_that_does_not_read_is_refused() {
    for (text, says) in [
        ("memory", "+<id> to switch it on or -<id>"),
        ("-memory,,+reader", "an empty item"),
        ("-memory,", "an empty item"),
        ("*memory", "+<id>"),
        ("-", "is no unit's id"),
        ("+Memory", "is no unit's id"),
        ("+plugin:", "is no unit's id"),
        ("+plugin:No", "is no unit's id"),
        ("+plugin:timeouts", "is no unit's id"),
        ("+mcp:x", "is no unit's id"),
        ("-memory,+memory", "`memory` is named twice"),
        ("é", "+<id>"),
    ] {
        let e = Composition::parse(text).unwrap_err();
        assert!(e.starts_with("--units"), "{text}: {e}");
        assert!(e.contains(says), "{text}: {e}");
    }
}
