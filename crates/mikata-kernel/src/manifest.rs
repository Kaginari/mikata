//! What a unit says about itself before it runs: where it hooks, what it provides and needs,
//! what it runs after, when it may change, whether it touches the frame and what it asks to be
//! granted.

use serde::{Deserialize, Serialize};

use crate::vocabulary::{Grant, Point};

/// A unit's description, read by the registry before the unit is activated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The unit's id: `memory` for a native unit, `plugin:cognee` for a plugin.
    pub id: String,
    /// The unit's version, written beside its id in the journal and in memory's `by` stamp.
    pub version: String,
    /// The points it hooks, each in a band.
    pub points: Vec<(Point, Band)>,
    /// The service keys it provides. One provider per key: a second is refused.
    pub provides: Vec<String>,
    /// The service keys it needs. It stays inactive until each is provided.
    pub needs: Vec<String>,
    /// Unit ids it runs after. Each must be loaded when this unit loads.
    pub after: Vec<String>,
    /// The earliest moment it may load or unload.
    pub safe_point: SafePoint,
    /// Whether it brings text to the frame.
    pub frame: FramePolicy,
    /// Whether switching it off only removes checks the body faces. When it does, a world file
    /// may not switch it off.
    pub tightens_when_off: bool,
    /// What it asks to be granted. The human's grant cuts this down.
    pub grants: Vec<Grant>,
}

impl Manifest {
    /// A manifest with no hooks, no services, no edges and no grants, changed only at an ask's
    /// boundary and leaving the frame alone; the builder methods below fill it in.
    #[must_use]
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Manifest {
            id: id.into(),
            version: version.into(),
            points: Vec::new(),
            provides: Vec::new(),
            needs: Vec::new(),
            after: Vec::new(),
            safe_point: SafePoint::Ask,
            frame: FramePolicy::None,
            tightens_when_off: false,
            grants: Vec::new(),
        }
    }

    /// Hooks `point` in `band`.
    #[must_use]
    pub fn at(mut self, point: Point, band: Band) -> Self {
        self.points.push((point, band));
        self
    }

    /// Provides the service `key`.
    #[must_use]
    pub fn provides(mut self, key: impl Into<String>) -> Self {
        self.provides.push(key.into());
        self
    }

    /// Needs the service `key`.
    #[must_use]
    pub fn needs(mut self, key: impl Into<String>) -> Self {
        self.needs.push(key.into());
        self
    }

    /// Runs after the unit `id`.
    #[must_use]
    pub fn after(mut self, id: impl Into<String>) -> Self {
        self.after.push(id.into());
        self
    }

    /// May load or unload no earlier than `at`.
    #[must_use]
    pub fn safe_point(mut self, at: SafePoint) -> Self {
        self.safe_point = at;
        self
    }

    /// Brings text to the frame as `frame` says.
    #[must_use]
    pub fn frame(mut self, frame: FramePolicy) -> Self {
        self.frame = frame;
        self
    }

    /// Switching it off only removes checks the body faces.
    #[must_use]
    pub fn tightens_when_off(mut self) -> Self {
        self.tightens_when_off = true;
        self
    }

    /// Asks for `grant`.
    #[must_use]
    pub fn grant(mut self, grant: Grant) -> Self {
        self.grants.push(grant);
        self
    }

    /// The band it hooks `point` in, or `None` when it does not hook it.
    #[must_use]
    pub fn band_at(&self, point: Point) -> Option<Band> {
        self.points
            .iter()
            .find(|(p, _)| *p == point)
            .map(|(_, b)| *b)
    }

    /// The `id@version` stamp the journal and memory's `by` carry.
    #[must_use]
    pub fn stamp(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
}

/// A hook's place within a point: guards run first, then native units, then plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Band {
    /// The checks that guard the run.
    Guard,
    /// The compiled-in units.
    Native,
    /// Third-party plugins.
    Outer,
}

/// The earliest moment a unit may load or unload, finest first: between tool batches, at an
/// ask's boundary, at a resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafePoint {
    /// Between tool batches; an ask's boundary and a resume are batch boundaries too.
    Batch,
    /// Before an ask begins; a resume is an ask's boundary too.
    Ask,
    /// At a resume only.
    Session,
}

impl SafePoint {
    /// Whether a unit that may change no earlier than `self` may change at `at`.
    #[must_use]
    pub fn allows(self, at: SafePoint) -> bool {
        at >= self
    }
}

/// Whether a unit brings text to the frame (the system prompt and the tool list), which stays
/// sealed for the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FramePolicy {
    /// It brings none.
    None,
    /// It reaches the body only as a context update in the uncached tail.
    Tail,
    /// It brings frame text, so it is held until the next epoch.
    Frame,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_manifest_round_trips_as_json() {
        let manifest = Manifest::new("plugin:cognee", "0.1.0")
            .at(Point::ToolBefore, Band::Outer)
            .provides("recall")
            .needs("journal")
            .after("memory")
            .safe_point(SafePoint::Batch)
            .frame(FramePolicy::Tail)
            .tightens_when_off()
            .grant(Grant::Advise);
        let value = serde_json::to_value(&manifest).unwrap();
        assert_eq!(value["points"], json!([["tool_before", "outer"]]));
        assert_eq!(value["safe_point"], "batch");
        assert_eq!(value["frame"], "tail");
        assert_eq!(serde_json::from_value::<Manifest>(value).unwrap(), manifest);
        assert_eq!(manifest.stamp(), "plugin:cognee@0.1.0");
        assert_eq!(manifest.band_at(Point::ToolBefore), Some(Band::Outer));
        assert_eq!(manifest.band_at(Point::Preface), None);
    }

    #[test]
    fn a_safe_point_allows_itself_and_every_coarser_one() {
        assert!(SafePoint::Batch.allows(SafePoint::Batch));
        assert!(SafePoint::Batch.allows(SafePoint::Session));
        assert!(SafePoint::Ask.allows(SafePoint::Ask));
        assert!(!SafePoint::Ask.allows(SafePoint::Batch));
        assert!(!SafePoint::Session.allows(SafePoint::Ask));
        assert!(Band::Guard < Band::Native && Band::Native < Band::Outer);
    }
}
