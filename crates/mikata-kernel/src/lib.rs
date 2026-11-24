//! The plugin kernel: units, their manifests, the registry with its undo stacks, dependency
//! order, safe-point reconcile and quarantine, and the hook vocabulary. No I/O.
//!
//! Every part of the loop beyond its core is a unit: memory, the gate's helpers, what-changed,
//! a third-party plugin. A unit says what it hooks in its [`Manifest`]; what a hook is shown and
//! what it may answer is plain data ([`Event`], [`Reply`]), so a plugin in another process is
//! shown the same JSON a native unit is. The plan of record is `docs/experimental/plugins.md`.

mod manifest;
mod vocabulary;

pub use manifest::{Band, FramePolicy, Manifest, SafePoint};
pub use vocabulary::{Event, Grant, NOTE_CAP, Point, Reply, Verdict};
