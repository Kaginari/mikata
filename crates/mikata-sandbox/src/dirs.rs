//! The human's state and config directories, resolved in one place — and refused to a test that
//! runs unisolated.
//!
//! A test binary resolves these from its environment like the harness does. Run without an
//! isolated home (plain `cargo test` on a developer's machine), that is the developer's real
//! `~/.local/state/mikata`, which a test must never write. So in a debug build, a test harness
//! process (a binary cargo keeps under `target/<profile>/deps`, run by cargo, which sets
//! `CARGO_MANIFEST_DIR`) panics here unless its environment carries [`ISOLATED`], which
//! `tools/test-env.sh`, nextest's `tools/test-home.sh` and the tests' isolated spawns set. A release
//! build never compiles the check; an installed binary never lives under `deps`.

use std::path::{Path, PathBuf};

/// The marker an isolated test environment carries.
pub const ISOLATED: &str = "MIKATA_ISOLATED";

/// The human's state directory: `$XDG_STATE_HOME` when absolute, else `~/.local/state`.
///
/// # Panics
///
/// In a debug build, in a test harness process not marked [`ISOLATED`].
#[must_use]
pub fn state_home() -> Option<PathBuf> {
    xdg("XDG_STATE_HOME", ".local/state")
}

/// The human's config directory: `$XDG_CONFIG_HOME` when absolute, else `~/.config`.
///
/// # Panics
///
/// As [`state_home`].
#[must_use]
pub fn config_home() -> Option<PathBuf> {
    xdg("XDG_CONFIG_HOME", ".config")
}

fn xdg(var: &str, under_home: &str) -> Option<PathBuf> {
    refuse_unisolated();
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::home_dir().map(|h| h.join(under_home)))
}

/// Panic when this is a test process running unisolated: see the module's doc.
fn refuse_unisolated() {
    #[cfg(debug_assertions)]
    if unisolated_test(
        std::env::current_exe().ok().as_deref(),
        std::env::var_os("CARGO_MANIFEST_DIR").is_some(),
        std::env::var_os(ISOLATED).is_some(),
    ) {
        panic!(
            "tests must run isolated: use `just test-isolated` (or `cargo nextest run` in the \
             workspace, or `tools/test-env.sh <command>`) — this test would read and write the \
             real state and config directories"
        );
    }
}

/// Whether a process at `exe`, run by cargo (`manifest`), unmarked (`!isolated`), is a test that
/// must not reach the human's directories.
#[must_use]
pub fn unisolated_test(exe: Option<&Path>, manifest: bool, isolated: bool) -> bool {
    let harness = exe
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .is_some_and(|d| d == "deps");
    harness && manifest && !isolated
}

/// A world's id across the state directory: `<name>-<FNV-1a 64 of its path>`, as every record
/// kept for it there is named (the registry, a world's own human file).
#[must_use]
pub fn world_id(world: &Path) -> String {
    let h = world
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    let name = world
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!("{name}-{h:016x}")
}

/// The human's own file for one world: `<state>/mikata/worlds/<world id>/config.toml`, out of
/// the world's reach, where what they set for that world alone and a world's file may not hold —
/// a guard switched off — is kept. `None` with no state directory.
#[must_use]
pub fn world_config(world: &Path) -> Option<PathBuf> {
    state_home().map(|s| {
        s.join("mikata/worlds")
            .join(world_id(world))
            .join("config.toml")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_unmarked_test_harness_run_by_cargo_is_refused() {
        let test = Path::new("/w/target/debug/deps/escape-0123abcd");
        let app = Path::new("/w/target/debug/mikata");
        let installed = Path::new("/home/h/.local/bin/mikatad");
        assert!(unisolated_test(Some(test), true, false));
        assert!(!unisolated_test(Some(test), true, true), "marked isolated");
        assert!(
            !unisolated_test(Some(test), false, false),
            "not run by cargo"
        );
        assert!(!unisolated_test(Some(app), true, false), "`cargo run`");
        assert!(!unisolated_test(Some(installed), false, false));
        assert!(!unisolated_test(None, true, false));
    }
}

/// Mikata was called Mnemorph until November 2026: its folders under the human's state and
/// config homes, and a world's own folder, were `mnemorph` and `.mnemorph`. Each is moved once to
/// its new name, the first time Mikata runs, when the new one does not exist yet; both present,
/// nothing is touched. What was moved, said for the human, one line each.
#[must_use]
pub fn migrate_homes() -> Vec<String> {
    [state_home(), config_home()]
        .into_iter()
        .flatten()
        .filter_map(|home| moved(&home.join("mnemorph"), &home.join("mikata")))
        .collect()
}

/// A world's `.mnemorph` folder moved to `.mikata` ([`migrate_homes`]).
#[must_use]
pub fn migrate_world(world: &Path) -> Option<String> {
    moved(&world.join(".mnemorph"), &world.join(".mikata"))
}

/// `old` renamed to `new` when `old` is a folder (not a link) and nothing stands at `new`.
fn moved(old: &Path, new: &Path) -> Option<String> {
    let folder = std::fs::symlink_metadata(old).is_ok_and(|m| m.file_type().is_dir());
    if !folder || std::fs::symlink_metadata(new).is_ok() {
        return None;
    }
    Some(match std::fs::rename(old, new) {
        Ok(()) => format!(
            "moved {} to {} (Mnemorph is now Mikata)",
            old.display(),
            new.display()
        ),
        Err(e) => format!(
            "{} could not be moved to {}: {e}; move it by hand",
            old.display(),
            new.display()
        ),
    })
}

#[cfg(test)]
mod migrate_tests {
    use super::{migrate_world, moved};

    #[test]
    fn an_old_folder_moves_once_and_a_new_one_is_never_overwritten() {
        let root = std::env::temp_dir().join(format!("mikata-migrate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".mnemorph")).unwrap();
        std::fs::write(root.join(".mnemorph/log.md"), "kept\n").unwrap();
        assert!(migrate_world(&root).is_some_and(|s| s.contains("Mnemorph is now Mikata")));
        assert_eq!(
            std::fs::read_to_string(root.join(".mikata/log.md")).unwrap(),
            "kept\n"
        );
        assert!(!root.join(".mnemorph").exists());
        assert!(migrate_world(&root).is_none(), "once");
        // Both present: nothing is touched.
        std::fs::create_dir_all(root.join(".mnemorph")).unwrap();
        assert!(migrate_world(&root).is_none());
        assert!(root.join(".mnemorph").exists() && root.join(".mikata/log.md").exists());
        // A file of the old name is no folder of ours.
        std::fs::write(root.join("old"), "").unwrap();
        assert!(moved(&root.join("old"), &root.join("new")).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }
}
