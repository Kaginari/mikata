//! What a world's working session is held to, read from its files for a client to show: the
//! session card's verify line (or the test command to add when it has none), the body's `Done
//! when` list in its plan, and the branch the repository is on. The terminal app's panels and
//! the board's app read it alike.

use std::path::Path;

use serde::Serialize;

use crate::roster::Roster;

/// A world's desk: what its session is held to.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Desk {
    /// The session card's verify line: its commands; empty when the card has none.
    pub verify: Vec<String>,
    /// The test command the project's manifest names, suggested only when `verify` is empty.
    pub suggested: Option<String>,
    /// The lines of the body's `## Done when` list (`.mikata/tmp/plan.md`), as written.
    pub done: Vec<String>,
    /// The branch the repository is on, when it is on one.
    pub branch: Option<String>,
}

/// Read `world`'s desk: file reads only.
#[must_use]
pub fn desk(world: &Path) -> Desk {
    let verify = Roster::load(world)
        .creatures
        .get("summoner")
        .map(|c| c.verify.clone())
        .unwrap_or_default();
    let suggested = if verify.is_empty() {
        crate::verify::suggest(world)
    } else {
        None
    };
    Desk {
        verify,
        suggested,
        done: done_when(world),
        branch: branch(world),
    }
}

/// The lines under `## Done when` in the body's plan (`.mikata/tmp/plan.md`): each non-empty line
/// up to the next heading, its list mark taken off; none when there is no plan.
#[must_use]
pub fn done_when(world: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(world.join(".mikata/tmp/plan.md")) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('#') {
            inside = l
                .trim_start_matches('#')
                .trim()
                .eq_ignore_ascii_case("done when");
            continue;
        }
        if inside && !l.is_empty() {
            lines.push(l.trim_start_matches(['-', '*', ' ']).trim().to_owned());
        }
    }
    lines.truncate(40);
    lines
}

/// The branch `world`'s repository is on, read from `.git/HEAD`; none when detached or absent.
#[must_use]
pub fn branch(world: &Path) -> Option<String> {
    let head = std::fs::read_to_string(world.join(".git/HEAD")).ok()?;
    head.trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_desk_reads_the_plan_the_branch_and_nothing_else() {
        let w = std::env::temp_dir().join(format!("mikata-desk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&w);
        std::fs::create_dir_all(w.join(".mikata/tmp")).unwrap();
        std::fs::create_dir_all(w.join(".git")).unwrap();
        assert_eq!(desk(&w).done, Vec::<String>::new(), "no plan, no lines");
        std::fs::write(
            w.join(".mikata/tmp/plan.md"),
            "# Plan\n\nread it\n\n## Done when\n\n- parse('1d') gives 1440\n* tests pass\n\n## Notes\n- not this\n",
        )
        .unwrap();
        std::fs::write(w.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        let d = desk(&w);
        assert_eq!(d.done, ["parse('1d') gives 1440", "tests pass"]);
        assert_eq!(d.branch.as_deref(), Some("main"));
        std::fs::write(
            w.join(".git/HEAD"),
            "4b825dc642cb6eb9a060e54bf8d69288fbee4904\n",
        )
        .unwrap();
        assert_eq!(branch(&w), None, "detached");
        let _ = std::fs::remove_dir_all(&w);
    }
}
