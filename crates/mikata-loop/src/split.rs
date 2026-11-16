//! Light-coding's split: an ask that names two or more separate projects of the world runs one
//! light-coding body a project, at once, each with its project's folder as its root. Five sqlglot
//! copies asked as one: one body fixed 2.5 of 5 on average, 3.5 given 400 steps; one body a
//! project fixed 5 of 5 in two rounds of two (bench step 22).

use std::path::Path;

/// The files that make a folder a project of its own.
const PROJECT_FILES: &[&str] = &[
    "pyproject.toml",
    "setup.py",
    "Cargo.toml",
    "package.json",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Gemfile",
    "composer.json",
    "mix.exs",
    "deno.json",
];

/// The folders at the world's root that `ask` names as a word and that hold a project file of
/// their own, in name order. A link is no folder of the world.
#[must_use]
pub fn projects(world: &Path, ask: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(world) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .filter(|name| {
            PROJECT_FILES
                .iter()
                .any(|f| world.join(name).join(f).is_file())
        })
        .filter(|name| names_word(ask, name))
        .collect();
    out.sort();
    out
}

/// What a part's body is asked: its project, the world it now stands in, and the whole ask for
/// context.
#[must_use]
pub fn scoped(ask: &str, project: &str) -> String {
    format!(
        "Your part of this ask: the project in `{project}/`, and only that. Your repository root \
         is that folder: a path the ask writes under `{project}/` is relative to your root \
         without it, and its code and tests run from your root. The ask's other projects are \
         other bodies', working at the same time; do not look for them.\n\nThe whole ask, for \
         context:\n\n{ask}"
    )
}

/// Whether `text` names `word` as a word: `alpha`, `alpha/`, `(alpha)`, not `alpha-2` or
/// `betalpha`.
#[must_use]
pub fn names_word(text: &str, word: &str) -> bool {
    let inner = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-');
    !word.is_empty()
        && text.match_indices(word).any(|(i, _)| {
            !inner(text[..i].chars().next_back()) && !inner(text[i + word.len()..].chars().next())
        })
}

#[cfg(test)]
mod tests {
    use super::{names_word, projects};

    #[test]
    fn a_project_is_a_named_folder_with_its_own_project_file() {
        let world = std::env::temp_dir().join(format!("mikata-split-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&world);
        for (dir, file) in [("a-1", "pyproject.toml"), ("b", "Cargo.toml"), ("docs", "")] {
            std::fs::create_dir_all(world.join(dir)).unwrap();
            if !file.is_empty() {
                std::fs::write(world.join(dir).join(file), "").unwrap();
            }
        }
        assert_eq!(
            projects(&world, "fix a-1/ and (b), see docs/"),
            ["a-1", "b"]
        );
        assert_eq!(projects(&world, "fix a-10 and bb"), Vec::<String>::new());
        assert!(names_word("in `03-alter/`", "03-alter"));
        assert!(!names_word("03-alter-drop", "03-alter"));
        let _ = std::fs::remove_dir_all(&world);
    }
}
