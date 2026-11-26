//! The run's units: the parts of the loop that can be switched on and off, and swapped, while a
//! session runs ([`mikata_kernel`], `docs/experimental/plugins.md`).
//!
//! A unit is shown an [`Event`] at a point of the loop and answers a [`Reply`]. What it does
//! besides answering, it does to a [`Cx`]: the run lends it the world's path, and takes back the
//! beats it wants journaled and the notes it wants said to the body. The default composition is
//! every native unit on; a session journals a `units` beat only for what differs from it, so a
//! run with the default composition journals what it journaled before units existed.

use std::path::{Path, PathBuf};

use mikata_kernel::{Activation, Band, Event, Manifest, Point, Reply, SafePoint, Unit, UnitError};

use crate::beats::Beat;

/// What the run lends its units at a point: where the world is, and what they leave for the run
/// to journal and to say.
#[derive(Debug, Default)]
pub struct Cx {
    /// The world the run works in.
    pub world: PathBuf,
    /// Beats a unit asks the run to journal, in order.
    pub beats: Vec<Beat>,
}

impl Cx {
    /// A context for the world at `world`.
    #[must_use]
    pub fn new(world: &Path) -> Self {
        Self {
            world: world.to_path_buf(),
            beats: Vec::new(),
        }
    }
}

/// A unit ready to load, with what the human grants it.
pub type Granted = (Box<dyn Unit<Cx>>, Vec<mikata_kernel::Grant>);

/// The run's registry of units.
pub type Registry = mikata_kernel::Registry<Cx>;

/// A native unit by its id, as the default composition and a `+<id>` in `--units` load it.
#[must_use]
pub fn native(id: &str) -> Option<Box<dyn Unit<Cx>>> {
    match id {
        Returns::ID => Some(Box::new(Returns::default())),
        MEMORY | READER | GATE | HOLD | CHALLENGE | SECRETS | APPROVAL => {
            Some(switch(id, SafePoint::Ask, false))
        }
        FOLD | STRESS => Some(switch(id, SafePoint::Batch, false)),
        WHAT_CHANGED => Some(switch(id, SafePoint::Ask, true)),
        PLAN | PLAN_LITE => Some(Box::new(Switch::new(
            Manifest::new(id, env!("CARGO_PKG_VERSION"))
                .safe_point(SafePoint::Session)
                .frame(mikata_kernel::FramePolicy::Frame),
        ))),
        _ => None,
    }
}

/// A [`Switch`] for the native unit `id`, switched at `at`; `world_may` when a world file may
/// switch it off.
fn switch(id: &str, at: SafePoint, world_may: bool) -> Box<dyn Unit<Cx>> {
    let manifest = Manifest::new(id, env!("CARGO_PKG_VERSION")).safe_point(at);
    Box::new(Switch::new(if world_may {
        manifest.tightens_when_off()
    } else {
        manifest
    }))
}

/// What changed ([`crate::changes`]): after a landing in light-coding, the body's own checks run
/// on the code before and after, said to the human. Nothing of it reaches the body, so a world file
/// may switch it off.
pub const WHAT_CHANGED: &str = "what_changed";

/// The reader ([`crate::lines`]): a second model that reads an ask with no `Done when` list and
/// writes one, when the human names a reader. Off, the body writes its own.
pub const READER: &str = "reader";

/// The stress signals ([`crate::stress`]): warnings said to the body when its acts circle, and the
/// checkpoint a doomed loop ends in. Off, the meters still read and journal, and nothing is said
/// or stopped by them.
pub const STRESS: &str = "stress";

/// Memory ([`mikata_memory`]): what the ask recalls, the corrections held from it, and every
/// note the run writes — claims, lessons, credit and retirement, a stop's account, a drain's and
/// a salvage's words. Off, the ask recalls nothing and the run writes nothing to memory. Off as
/// the session opens, the prompt does not ask for `@U` lessons ([`crate::solo::LESSONS`]);
/// switched off later, it still asks (the frame never changes) and they are not kept. It
/// switches between asks. Off loosens what a held correction guards, so a world file may not
/// switch it off.
pub const MEMORY: &str = "memory";

/// The end gate ([`crate::gate`]): the pinned verify lines and the test-tampering checks an answer
/// that changed files is judged by, with one chance to fix a failure. Off, an answer lands
/// unjudged. It switches between asks, by the human alone.
pub const GATE: &str = "gate";

/// The held `Done when` list ([`crate::done`]): a write refused while a line has no check, and the
/// list's checks run as part of the gate's verdict. Off, the list is the body's notes and nothing
/// holds it. By the human alone.
pub const HOLD: &str = "hold";

/// The context fold ([`crate::compact`]): the oldest exchanges folded once the context nears its
/// line. Off, the context grows to the stress line, where the run drains or compacts as before.
/// It switches between tool batches, by the human alone.
pub const FOLD: &str = "fold";

/// Secret files ([`mikata_tools::secrets`]): `.env`, keys, kubeconfigs, Terraform's state and
/// variables, and what the human's `secrets.files` and a world's `secrets.also` add. On, `read`
/// and `index` give a secret file's keys and never a value, no tool writes or searches one, a
/// command finds it empty, and every value read from one is struck from every result before the
/// model reads it. Off, they are files like any other. It switches between asks, and only the
/// human may switch it off: a world is not trusted to open its own secrets to a model.
pub const SECRETS: &str = "secrets";

/// The human's approval ([`mikata_tools::Gate`]): every outward or destructive act — a push, a
/// pull request, a deploy, an apply, a publish, a delete outside the world — is put to the human
/// on the approval card before it runs. Off, they run without asking, as approved: with the
/// network, and with the secret files as they are. The guard's refusals, the shelf and the end
/// gate's word on a landing that changed tests stand. It switches between asks, and only the
/// human may switch it off: their global file or their own command, never a world's file.
pub const APPROVAL: &str = "approval";

/// The Dark Elf challenge ([`crate::challenge`]): a landing in a domain the roster routes is held
/// for review. Off, it lands at once. By the human alone.
pub const CHALLENGE: &str = "challenge";

/// The plan ([`crate::solo::PLAN_BULLETS`]): the prompt's two bullets that have the body write
/// down what done means and its paths before the code, and run each line before it answers. It is
/// the prompt's, so it is decided when the session opens, by the human's global file alone, and
/// never switched while a session runs. **Off by default** ([`OFF_BY_DEFAULT`]): stage 13
/// (2026-11-25) found no pass it earned — qwen on prose asks 3 of 12 with it, 5 of 12 without;
/// Claude Haiku 5.5 18 of 18 either way at 84% of its calls — so a session asks for it by name.
pub const PLAN: &str = "plan";

/// The build's own profiles: a composition for a kind of work, named, each one earned by a screen.
/// A profile the human's global file names the same goes over the build's.
///
/// - `planned`: the full plan on — what done means and its paths before the code.
/// - `prose`: the plan, lite, on — the outputs a prose ask states, copied and checked; for a small
///   body on prose asks, where a held `Done when` list passed them in earlier screens.
/// - `small-fix`: the plan off; the default since stage 13, kept so a profile named in a global
///   file or a script still reads.
pub const PROFILES: &[(&str, &str)] = &[
    ("planned", "+plan"),
    ("prose", "+plan_lite"),
    ("small-fix", "-plan"),
];

/// The build's profile `name`, as a composition.
#[must_use]
pub fn profile(name: &str) -> Option<mikata_config::Composition> {
    let (_, list) = PROFILES.iter().find(|(n, _)| *n == name)?;
    mikata_config::Composition::parse(list).ok()
}

/// The plan, lite ([`crate::solo::PLAN_LITE_BULLET`]): in the place of the plan's two bullets, one
/// that writes down and checks only the outputs the ask states, and no plan for a change that
/// states none. Off in the default composition; on as a session opens (`+plan_lite`), it takes the
/// plan's place, which goes off with it.
pub const PLAN_LITE: &str = "plan_lite";

/// The native units the default composition leaves off: asked for by name.
pub const OPTIONAL: &[&str] = &[PLAN_LITE];

/// The units a session opens without unless the human's ask or global file switches them on.
pub const OFF_BY_DEFAULT: &[&str] = &[PLAN, PLAN_LITE];

/// The ids of the native units, in the order the default composition loads them.
pub const NATIVE: &[&str] = &[
    Returns::ID,
    MEMORY,
    WHAT_CHANGED,
    READER,
    STRESS,
    PLAN,
    GATE,
    HOLD,
    FOLD,
    CHALLENGE,
    SECRETS,
    APPROVAL,
];

/// A native unit whose part of the loop is still the run's own code: the run asks whether it is
/// on ([`Registry::active`]) where that code runs. It hooks no point and says nothing.
pub struct Switch(Manifest);

impl Switch {
    /// A switch with this manifest.
    #[must_use]
    pub fn new(manifest: Manifest) -> Self {
        Self(manifest)
    }
}

impl Unit<Cx> for Switch {
    fn manifest(&self) -> &Manifest {
        &self.0
    }

    fn activate(&mut self, _act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
        Ok(())
    }

    fn on(&mut self, _cx: &mut Cx, _event: &Event) -> Reply {
        Reply::Pass
    }
}

/// Whether a world file may switch the native unit `id` off: its manifest says switching it off
/// loosens nothing the human relies on (it only reads, or only adds refusals of its own kind).
/// Any other unit is switched off by the human alone: a world is not trusted to drop a guard.
#[must_use]
pub fn tightens_when_off(id: &str) -> bool {
    native(id).is_some_and(|u| u.manifest().tightens_when_off)
}

/// The registry holding the default composition: every native unit on but those
/// [`OFF_BY_DEFAULT`].
#[must_use]
pub fn default_registry(cx: &mut Cx) -> Registry {
    let mut registry = Registry::new();
    for id in NATIVE.iter().filter(|id| !OFF_BY_DEFAULT.contains(id)) {
        if let Some(unit) = native(id) {
            registry.request(mikata_kernel::Change::Load(unit, Vec::new()));
        }
    }
    // The default composition is the build's own: nothing of it is journaled.
    let _ = registry.reconcile(SafePoint::Session, cx);
    registry
}

/// Where the run finds a unit by its id: the native units, and the plugins the human's global
/// configuration declares.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    /// The plugins declared, enabled or not.
    pub plugins: Vec<mikata_config::Plugin>,
    /// How long a plugin's hook may take, in milliseconds.
    pub hook_ms: u64,
    /// Where plugins run: set by the run when it takes the catalog.
    pub host: Option<crate::plugins::Host>,
}

impl Catalog {
    /// Whether the native unit `id` is part of the prompt ([`mikata_kernel::FramePolicy::Frame`]).
    #[must_use]
    pub fn unit_frame(&self, id: &str) -> bool {
        native(id).is_some_and(|u| u.manifest().frame == mikata_kernel::FramePolicy::Frame)
    }

    /// Whether a unit goes by `id`, without starting it.
    #[must_use]
    pub fn knows(&self, id: &str) -> bool {
        NATIVE.contains(&id)
            || OPTIONAL.contains(&id)
            || id
                .strip_prefix("plugin:")
                .is_some_and(|name| self.plugins.iter().any(|p| p.id == name))
    }

    /// The unit `id` (`returns`, `plugin:<id>`), ready to load, and what the human grants it.
    ///
    /// # Errors
    ///
    /// No native unit and no declared plugin goes by that id, or the plugin cannot be started.
    pub fn unit(&self, id: &str) -> Result<Granted, String> {
        if let Some(unit) = native(id) {
            return Ok((unit, Vec::new()));
        }
        let Some(name) = id.strip_prefix("plugin:") else {
            return Err(format!("there is no unit `{id}`"));
        };
        let Some(plugin) = self.plugins.iter().find(|p| p.id == name) else {
            return Err(format!(
                "the plugin `{name}` is not declared in the human's global configuration"
            ));
        };
        crate::plugins::unit(plugin, self.hook_ms, self.host.as_ref())
    }
}

/// What a client of the session (the server's `units/list` and `units/change`) shares with the
/// run: the changes asked, taken at the run's next point, and the composition as last listed.
#[derive(Debug, Default)]
pub struct Desk {
    /// Changes asked and not yet taken: a unit's id, and on or off.
    pub asked: std::collections::VecDeque<(String, bool)>,
    /// The units, `(id, version, state)`, as the run last listed them.
    pub listed: Vec<(String, String, String)>,
    /// The composition's fingerprint, as the run last listed it.
    pub digest: String,
    /// Changes the run could not take, and why, newest last.
    pub refused: Vec<String>,
}

/// The desk a run shares with its clients.
pub type UnitDesk = std::sync::Arc<std::sync::Mutex<Desk>>;

/// A run's units: what it lends them, the registry holding them, and where it finds more.
pub struct Units {
    /// What the run lends its units at a point.
    pub cx: Cx,
    /// The composition.
    pub registry: Registry,
    /// Where units are found by id.
    pub catalog: Catalog,
    /// Units a resumed journal held that wait for the catalog to be found: plugins.
    pub wanted: Vec<String>,
    /// What the run shares with its clients.
    pub desk: UnitDesk,
}

impl Default for Units {
    /// The default composition. The run sets the world before every point.
    fn default() -> Self {
        let mut cx = Cx::default();
        let registry = default_registry(&mut cx);
        Self {
            cx,
            registry,
            catalog: Catalog::default(),
            wanted: Vec::new(),
            desk: UnitDesk::default(),
        }
    }
}

impl Units {
    /// The composition a resumed journal last held, rebuilt over the default without journaling
    /// it again: the last `units` beat's fingerprint names every unit that was on. A native unit
    /// is loaded or unloaded at once; a plugin waits in [`Units::wanted`] for the catalog.
    pub fn replay(&mut self, beats: &[Beat]) {
        let Some(digest) = beats.iter().rev().find_map(|b| match b {
            Beat::Units { digest, .. } => Some(digest.clone()),
            _ => None,
        }) else {
            return;
        };
        // A unit of the prompt follows the prompt the resume makes, never the journal.
        let frame = |id: &str| {
            native(id).is_some_and(|u| u.manifest().frame == mikata_kernel::FramePolicy::Frame)
        };
        let held: Vec<String> = digest
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|stamp| {
                stamp
                    .rsplit_once('@')
                    .map_or(stamp, |(id, _)| id)
                    .to_owned()
            })
            .filter(|id| !frame(id))
            .collect();
        let active: Vec<String> = self
            .registry
            .active()
            .iter()
            .map(|(id, _)| (*id).to_owned())
            .filter(|id| !frame(id))
            .collect();
        for id in active.iter().filter(|id| !held.contains(id)) {
            self.registry
                .request(mikata_kernel::Change::Unload(id.clone()));
        }
        for id in held.iter().filter(|id| !active.contains(id)) {
            match native(id) {
                Some(unit) => self
                    .registry
                    .request(mikata_kernel::Change::Load(unit, Vec::new())),
                None => self.wanted.push(id.clone()),
            }
        }
        let _ = self.registry.reconcile(SafePoint::Session, &mut self.cx);
    }
}

/// The tools whose writes land a tree.
const WRITE_TOOLS: [&str; 6] = ["write", "edit", "multiedit", "patch", "move", "delete"];

/// The tree reading ([`crate::returns`]): the tree an ask's landed writes left, and how many times
/// it has stood, journaled as a `tree` beat after each landed write. It only reads, and says
/// nothing to the body.
pub struct Returns {
    manifest: Manifest,
    returns: crate::returns::Returns,
}

impl Default for Returns {
    fn default() -> Self {
        Self {
            manifest: Self::manifest_of(),
            returns: crate::returns::Returns::default(),
        }
    }
}

impl Returns {
    /// The unit's id.
    pub const ID: &'static str = "returns";

    fn manifest_of() -> Manifest {
        Manifest::new(Self::ID, env!("CARGO_PKG_VERSION"))
            .at(Point::AskBegin, Band::Native)
            .at(Point::ToolBefore, Band::Native)
            .at(Point::ToolAfter, Band::Native)
            .safe_point(SafePoint::Batch)
            .tightens_when_off()
    }
}

impl Unit<Cx> for Returns {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn activate(&mut self, _act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
        Ok(())
    }

    fn on(&mut self, cx: &mut Cx, event: &Event) -> Reply {
        match event {
            Event::AskBegin { .. } => self.returns = crate::returns::Returns::default(),
            Event::ToolBefore { name, input, .. } if WRITE_TOOLS.contains(&name.as_str()) => {
                self.returns.before(&cx.world, name, &input_text(input));
            }
            Event::ToolAfter { id, wrote, .. } => {
                let wrote: Vec<PathBuf> = wrote.iter().map(PathBuf::from).collect();
                if let Some(tree) = self.returns.after(&cx.world, id, &wrote) {
                    cx.beats.push(tree);
                }
            }
            _ => {}
        }
        Reply::Pass
    }
}

/// A tool call's input as the tools read it: the text the model sent, which [`Event`] carries as
/// JSON when it is JSON and as a string when it is not.
#[must_use]
pub fn input_text(input: &serde_json::Value) -> String {
    match input {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// A tool call's input as [`Event`] carries it: JSON when the model sent JSON, else its text.
#[must_use]
pub fn input_value(input: &str) -> serde_json::Value {
    serde_json::from_str(input).unwrap_or_else(|_| serde_json::Value::String(input.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_composition_holds_every_native_unit_but_those_off_by_default() {
        let mut cx = Cx::default();
        let registry = default_registry(&mut cx);
        let mut active: Vec<&str> = registry.active().iter().map(|(id, _)| *id).collect();
        let mut native: Vec<&str> = NATIVE
            .iter()
            .copied()
            .filter(|id| !OFF_BY_DEFAULT.contains(id))
            .collect();
        assert!(!active.contains(&PLAN), "the plan is off unless asked for");
        active.sort_unstable();
        native.sort_unstable();
        assert_eq!(active, native);
    }

    #[test]
    fn a_string_input_goes_back_as_it_came() {
        assert_eq!(input_text(&input_value("not json")), "not json");
        assert_eq!(
            input_text(&input_value(r#"{"path":"a"}"#)),
            r#"{"path":"a"}"#
        );
    }

    #[test]
    fn the_tree_unit_reads_a_landed_write_and_says_nothing() {
        let w = std::env::temp_dir().join(format!("mikata-units-returns-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&w);
        std::fs::create_dir_all(&w).unwrap();
        let mut cx = Cx::new(&w);
        let mut registry = default_registry(&mut cx);
        let input = serde_json::json!({"path": "a.txt", "content": "x"});
        let before = Event::ToolBefore {
            id: "t1".into(),
            name: "write".into(),
            input: input.clone(),
        };
        assert!(
            registry
                .dispatch(Point::ToolBefore, &mut cx, &before)
                .iter()
                .all(|(_, r)| *r == Reply::Pass)
        );
        std::fs::write(w.join("a.txt"), "x").unwrap();
        let after = Event::ToolAfter {
            id: "t1".into(),
            name: "write".into(),
            input,
            content: "wrote a.txt".into(),
            is_error: false,
            wrote: vec![w.join("a.txt").display().to_string()],
        };
        registry.dispatch(Point::ToolAfter, &mut cx, &after);
        assert!(
            matches!(cx.beats.as_slice(), [Beat::Tree { tool_use_id, .. }] if tool_use_id == "t1"),
            "{:?}",
            cx.beats
        );
        let _ = std::fs::remove_dir_all(&w);
    }
}
