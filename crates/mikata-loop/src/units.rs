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

/// The run's registry of units.
pub type Registry = mikata_kernel::Registry<Cx>;

/// A native unit by its id, as the default composition and a `+<id>` in `--units` load it.
#[must_use]
pub fn native(id: &str) -> Option<Box<dyn Unit<Cx>>> {
    match id {
        Returns::ID => Some(Box::new(Returns::default())),
        _ => None,
    }
}

/// The ids of the native units, in the order the default composition loads them.
pub const NATIVE: &[&str] = &[Returns::ID];

/// Whether switching the native unit `id` off only takes away something the body faces — a
/// check, a refusal — so a world file may do it. A unit that only reads, or that the body leans
/// on, may be switched off by the human alone.
#[must_use]
pub fn tightens_when_off(id: &str) -> bool {
    native(id).is_some_and(|u| u.manifest().tightens_when_off)
}

/// The registry holding the default composition: every native unit on.
#[must_use]
pub fn default_registry(cx: &mut Cx) -> Registry {
    let mut registry = Registry::new();
    for id in NATIVE {
        if let Some(unit) = native(id) {
            registry.request(mikata_kernel::Change::Load(unit, Vec::new()));
        }
    }
    // The default composition is the build's own: nothing of it is journaled.
    let _ = registry.reconcile(SafePoint::Session, cx);
    registry
}

/// A run's units: what it lends them, and the registry holding them.
pub struct Units {
    /// What the run lends its units at a point.
    pub cx: Cx,
    /// The composition.
    pub registry: Registry,
}

impl Default for Units {
    /// The default composition. The run sets the world before every point.
    fn default() -> Self {
        let mut cx = Cx::default();
        let registry = default_registry(&mut cx);
        Self { cx, registry }
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
    fn the_default_composition_holds_every_native_unit() {
        let mut cx = Cx::default();
        let registry = default_registry(&mut cx);
        let active: Vec<&str> = registry.active().iter().map(|(id, _)| *id).collect();
        assert_eq!(active, NATIVE);
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
