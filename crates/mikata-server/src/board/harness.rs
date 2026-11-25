//! `GET /api/w/<world>/units`: the harness a session in the world would open with — every native
//! unit with its version and whether the configuration has it on, every plugin the human's global
//! file declares, and the profiles a session may ask for (the build's, then the file's). The
//! board's Harness Builder reads it; what it composes goes back as `POST …/sessions`'s `units`.

use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

/// A unit as the builder shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BoardUnit {
    /// Its id: `memory`, or `plugin:<id>`.
    pub id: String,
    /// `native` or `plugin`.
    pub kind: String,
    /// Its version, for a native unit; empty for a plugin, whose version is its own program's.
    pub version: String,
    /// Whether a session opened now would have it on.
    pub on: bool,
    /// Part of the prompt: decided as a session opens, never switched while it runs.
    pub frame: bool,
    /// Switching it off only removes checks the body faces: a world's file may not.
    pub tightens: bool,
}

/// A named composition a session may ask for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BoardProfile {
    /// Its name: `small-fix`.
    pub name: String,
    /// Its list, as `--units` reads it: `-plan`.
    pub units: String,
    /// `build` or `global`: the human's global file's goes over the build's of the same name.
    pub from: String,
}

/// The world's harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BoardUnits {
    /// The native units, in the order the default composition loads them, then the plugins.
    pub units: Vec<BoardUnit>,
    /// The profiles, the build's first.
    pub profiles: Vec<BoardProfile>,
    /// What could not be read: the configuration, or its plugins.
    pub holes: Vec<String>,
}

/// The harness of the world at `world`.
#[must_use]
pub fn list(world: &Path) -> BoardUnits {
    let mut holes = Vec::new();
    let config = match mikata_loop::Config::load(&mikata_loop::Paths::resolve(world)) {
        Ok(c) => Some(c),
        Err(e) => {
            holes.push(format!("configuration: {e}"));
            None
        }
    };
    let on = |id: &str| {
        config
            .as_ref()
            .and_then(|c| c.unit(id))
            .is_none_or(|u| u.on)
    };
    let mut units: Vec<BoardUnit> = mikata_loop::units::NATIVE
        .iter()
        .filter_map(|id| mikata_loop::units::native(id))
        .map(|u| {
            let m = u.manifest();
            BoardUnit {
                id: m.id.clone(),
                kind: "native".into(),
                version: m.version.clone(),
                on: on(&m.id),
                frame: mikata_loop::units::Catalog::default().unit_frame(&m.id),
                tightens: m.tightens_when_off,
            }
        })
        .collect();
    if let Some(c) = &config {
        match c.plugins() {
            Ok(p) => units.extend(p.plugins.iter().map(|p| BoardUnit {
                id: p.unit_id(),
                kind: "plugin".into(),
                version: String::new(),
                on: p.enabled,
                frame: false,
                tightens: false,
            })),
            Err(e) => holes.push(format!("plugins: {e}")),
        }
    }
    let mut profiles: Vec<BoardProfile> = mikata_loop::units::PROFILES
        .iter()
        .map(|(name, list)| BoardProfile {
            name: (*name).to_owned(),
            units: (*list).to_owned(),
            from: "build".into(),
        })
        .collect();
    for (name, list) in config
        .as_ref()
        .map(mikata_loop::Config::profiles)
        .unwrap_or_default()
    {
        profiles.retain(|p| p.name != name);
        profiles.push(BoardProfile {
            name,
            units: list,
            from: "global".into(),
        });
    }
    BoardUnits {
        units,
        profiles,
        holes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_world_with_no_file_lists_every_native_unit_on_and_the_build_s_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let h = list(dir.path());
        let ids: Vec<&str> = h.units.iter().map(|u| u.id.as_str()).collect();
        for id in mikata_loop::units::NATIVE {
            assert!(ids.contains(id), "{id} listed");
        }
        assert!(
            h.units
                .iter()
                .all(|u| u.kind == "native" && !u.version.is_empty())
        );
        let plan = h.units.iter().find(|u| u.id == "plan").unwrap();
        assert!(plan.frame, "the plan is the prompt's");
        assert!(
            h.profiles
                .iter()
                .any(|p| p.name == "small-fix" && p.units == "-plan")
        );
    }
}
