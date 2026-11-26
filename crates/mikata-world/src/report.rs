//! The world's report folder: where a check may write what it leaves behind — test reports,
//! coverage, snapshots — since a check never writes the tree. Prepared when a session opens:
//! `.mikata/report/` made, and it and the common test leftovers ignored through git's local
//! exclude file, which is never committed, so the project's own `.gitignore` stays the human's.

use std::fs;
use std::path::Path;

/// The report folder, relative to the world.
pub const REPORT: &str = ".mikata/report";

/// What a check run commonly leaves behind: a verify run may write these. The gate's view of a
/// body's own changes does not read the exclude file, so these hide nothing a body writes.
pub const LEFTOVERS: &[&str] = &[
    ".mikata/report/",
    "__pycache__/",
    ".pytest_cache/",
    // Anchored at the root: a source directory named `coverage` deeper down stays the project's.
    "/.coverage",
    "/htmlcov/",
    "/coverage/",
    "/.nyc_output/",
    ".mypy_cache/",
    ".ruff_cache/",
];

/// This machine's own files in the world, kept out of git locally: the human's local
/// configuration, which the session writes its changes to.
pub const LOCAL_FILES: &[&str] = &["/.mikata/config.local.toml"];

/// Whether `rel`, a path relative to the world, is one of [`LEFTOVERS`]: a folder of that name
/// anywhere on the path, or for a line anchored with `/`, that file or folder at the root. What
/// a check left behind is no change to the code.
#[must_use]
pub fn is_leftover(rel: &Path) -> bool {
    let parts: Vec<&str> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    LEFTOVERS.iter().any(|line| {
        let name = line.trim_matches('/');
        // Anchored at the root: the path is it, or lies in it.
        if line.starts_with('/') {
            return parts.first().is_some_and(|first| *first == name);
        }
        // `.mikata/report/` is a path of its own, not one name.
        if name.contains('/') {
            return rel.starts_with(name);
        }
        // A folder of that name anywhere above the file.
        parts[..parts.len().saturating_sub(1)].contains(&name)
    })
}

/// Make the report folder and ignore it and the common test leftovers locally. Lines already in
/// git's exclude file are not added twice; a world with no git, or whose git files are links, gets
/// the folder only.
///
/// # Errors
///
/// The folder or the exclude file cannot be written.
pub fn prepare(world: &Path) -> std::io::Result<()> {
    fs::create_dir_all(world.join(REPORT))?;
    let info = world.join(".git").join("info");
    let linked = |p: &Path| fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink());
    if !world.join(".git").is_dir() || linked(&world.join(".git")) || linked(&info) {
        return Ok(());
    }
    let exclude = info.join("exclude");
    if linked(&exclude) {
        return Ok(());
    }
    fs::create_dir_all(&info)?;
    if linked(&info) {
        return Ok(());
    }
    let had = fs::read_to_string(&exclude).unwrap_or_default();
    let missing: Vec<&str> = LEFTOVERS
        .iter()
        .chain(LOCAL_FILES)
        .copied()
        .filter(|l| !had.lines().any(|h| h.trim() == *l))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    let mut next = had;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(
        "# mikata: what checks leave behind, and this machine's config (local, never committed)\n",
    );
    for l in missing {
        next.push_str(l);
        next.push('\n');
    }
    // Written new beside it, then renamed over it: a link put in its place is replaced, never
    // followed.
    let temp = info.join(".exclude.mikata");
    let _ = fs::remove_file(&temp);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .and_then(|mut f| std::io::Write::write_all(&mut f, next.as_bytes()))
        .and_then(|()| fs::rename(&temp, &exclude))
}

#[cfg(test)]
mod tests {
    use super::is_leftover;
    use std::path::Path;

    /// What a check leaves behind is told from the code by where it lies: a cache folder
    /// anywhere, a coverage file or folder at the root only.
    #[test]
    fn a_leftover_is_told_from_the_code() {
        for left in [
            "__pycache__/calc.cpython-312.pyc",
            "pkg/sub/__pycache__/m.pyc",
            ".pytest_cache/v/cache/lastfailed",
            ".mypy_cache/3.12/a.json",
            ".coverage",
            "htmlcov/index.html",
            "coverage/lcov.info",
            ".nyc_output/x.json",
            ".mikata/report/junit.xml",
        ] {
            assert!(is_leftover(Path::new(left)), "{left}");
        }
        for code in [
            "calc.py",
            "src/coverage/report.py",
            "docs/__pycache__",
            "pkg/.coverage",
            ".mikata/tmp/plan.md",
            "htmlcov.py",
        ] {
            assert!(!is_leftover(Path::new(code)), "{code}");
        }
    }
}
