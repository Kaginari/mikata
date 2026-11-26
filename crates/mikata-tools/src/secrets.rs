//! Secret files: the files in a world that hold what must never reach a model — `.env`, keys,
//! kubeconfigs, Terraform's state and variables — held by the `secrets` unit.
//!
//! A file is a secret when its path matches the patterns: the build's ([`DEFAULTS`]), then the
//! human's global `secrets.files` (which may un-mark one with `!`), then `secrets.also` from any
//! file (which only adds). A pattern with no `/` matches a file's name at any depth; one with a
//! `/` matches its path from the world's root. The last pattern that matches decides, as in a
//! `.gitignore`.
//!
//! While the unit is on:
//!
//! - `read` gives a secret file's shape and never its values: each `KEY=value` or `key: value`
//!   line as `KEY = ‹secret›`, every other line as `‹secret line›`. The body can write code that
//!   reads the variables it names, and never learns what they hold.
//! - No tool writes, moves or deletes one; `grep` and `index` pass over them.
//! - A command does not read them: the sandbox puts an empty file in each one's place
//!   (`/dev/null`, mounted over it in the command's own mount namespace on Linux; denied by
//!   Seatbelt on macOS).
//! - Every value the harness read from them is struck from every tool result before the model
//!   reads it: a second line, for a value a command prints from somewhere else.
//!
//! The scan runs as a session opens, over every file of the world but the trees no project keeps
//! a secret in ([`SKIPPED`]) — what git ignores included, since that is where secrets are kept.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};

/// The build's patterns: what is a secret wherever a project keeps it.
pub const DEFAULTS: &[&str] = &[
    // Environment files, but the examples a project commits.
    ".env",
    ".env.*",
    "*.env",
    "!.env.example",
    "!.env.sample",
    "!.env.template",
    "!.env.dist",
    "!*.env.example",
    // Keys and certificates.
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "*.jks",
    "*.keystore",
    "id_rsa",
    "id_rsa.*",
    "id_dsa",
    "id_ecdsa",
    "id_ecdsa.*",
    "id_ed25519",
    "id_ed25519.*",
    "!*.pub",
    "age.txt",
    "*.age",
    // Clusters and clouds.
    "kubeconfig",
    "*.kubeconfig",
    "talosconfig",
    "credentials.json",
    "service-account*.json",
    ".vault-token",
    // Terraform's state and the values it is given; SOPS-encrypted ones stay readable.
    "*.tfstate",
    "*.tfstate.*",
    "*.tfvars",
    "!*.tfvars.example",
    "!*.sops.tfvars",
    // Logins of package managers and tools.
    ".netrc",
    ".npmrc",
    ".pypirc",
    ".git-credentials",
    ".dockercfg",
];

/// Trees the scan never enters: what a build, a package manager or git keeps.
pub const SKIPPED: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".terraform",
    ".mikata",
    ".mnemorph",
];

/// The most files the scan looks at, and the largest secret file it reads for values.
const SCAN_FILES: usize = 200_000;
const READ_BYTES: u64 = 1 << 20;

/// The shortest value struck from results: shorter ones (`true`, a port) are words, not secrets.
pub const MIN_VALUE: usize = 8;

/// What a struck value reads as.
pub const STRUCK: &str = "‹secret›";

/// One pattern: what it matches, whether it un-marks, whether it reads a whole path.
#[derive(Debug, Clone)]
struct Pattern {
    matcher: GlobMatcher,
    negate: bool,
    path: bool,
}

/// A world's secret files and the values they hold.
#[derive(Debug, Clone, Default)]
pub struct Secrets {
    world: PathBuf,
    patterns: Vec<Pattern>,
    files: Vec<PathBuf>,
    values: Vec<String>,
}

impl Secrets {
    /// No secret: nothing is held.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// The patterns of `world`: the build's, then `human`'s (a `!` un-marks), then `also`'s
    /// (a `!` there is dropped: a world adds, never takes away). Nothing is read yet.
    #[must_use]
    pub fn patterns(world: &Path, human: &[String], also: &[String]) -> Self {
        let mut patterns = Vec::new();
        let human = human.iter().map(|s| (s.as_str(), true));
        let also = also.iter().map(|s| (s.as_str(), false));
        for (p, may_negate) in DEFAULTS.iter().map(|s| (*s, true)).chain(human).chain(also) {
            let p = p.trim();
            let (negate, body) = match p.strip_prefix('!') {
                Some(rest) if may_negate => (true, rest),
                Some(_) => continue,
                None => (false, p),
            };
            let body = body.trim_start_matches("./");
            if body.is_empty() {
                continue;
            }
            let path = body.contains('/');
            let Ok(glob) = Glob::new(body.trim_start_matches('/')) else {
                continue;
            };
            patterns.push(Pattern {
                matcher: glob.compile_matcher(),
                negate,
                path,
            });
        }
        Self {
            world: world.to_path_buf(),
            patterns,
            files: Vec::new(),
            values: Vec::new(),
        }
    }

    /// The patterns of `world`, and its secret files found and read: what [`Self::patterns`]
    /// marks, among every file but under [`SKIPPED`].
    #[must_use]
    pub fn scan(world: &Path, human: &[String], also: &[String]) -> Self {
        let mut s = Self::patterns(world, human, also);
        let walk = ignore::WalkBuilder::new(world)
            .standard_filters(false)
            .hidden(false)
            .follow_links(false)
            .filter_entry(|e| {
                !(e.file_type().is_some_and(|t| t.is_dir())
                    && SKIPPED.iter().any(|d| e.file_name() == *d))
            })
            .build();
        for entry in walk.flatten().take(SCAN_FILES) {
            if entry.file_type().is_some_and(|t| t.is_file()) && s.is_secret(entry.path()) {
                s.files.push(entry.path().to_path_buf());
            }
        }
        s.files.sort();
        let mut values: Vec<String> = s
            .files
            .iter()
            .filter_map(|f| read_capped(f))
            .flat_map(|text| values_of(&text))
            .collect();
        values.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        values.dedup();
        s.values = values;
        s
    }

    /// Whether `path` (absolute, or from the world's root) is a secret by the patterns: the last
    /// that matches decides. A path outside the world is not this world's to hold.
    #[must_use]
    pub fn is_secret(&self, path: &Path) -> bool {
        let rel = if path.is_absolute() {
            match path.strip_prefix(&self.world) {
                Ok(r) => r,
                Err(_) => return false,
            }
        } else {
            path
        };
        let Some(name) = rel.file_name() else {
            return false;
        };
        let mut secret = false;
        for p in &self.patterns {
            let hit = if p.path {
                p.matcher.is_match(rel)
            } else {
                p.matcher.is_match(name)
            };
            if hit {
                secret = !p.negate;
            }
        }
        secret
    }

    /// The secret files the scan found, absolute.
    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// How many values are held.
    #[must_use]
    pub fn values(&self) -> usize {
        self.values.len()
    }

    /// `text` with every held value struck.
    #[must_use]
    pub fn strike<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if !self.values.iter().any(|v| text.contains(v.as_str())) {
            return Cow::Borrowed(text);
        }
        let mut out = text.to_owned();
        // Longest first: a value inside a longer one never leaves the longer one's tail.
        for v in &self.values {
            if out.contains(v.as_str()) {
                out = out.replace(v.as_str(), STRUCK);
            }
        }
        Cow::Owned(out)
    }

    /// What `read` gives of the secret file at `path`: its shape, never a value.
    #[must_use]
    pub fn view(&self, path: &Path, shown: &str) -> String {
        let Some(text) = read_capped(path) else {
            return format!(
                "`{shown}` is a secret file: its contents are never read to you, and it could not be read for its shape."
            );
        };
        let mut lines = vec![format!(
            "`{shown}` is a secret file (the secrets unit): its values are never read to you. Its shape, \
             so code can use the names it holds:"
        )];
        for line in text.lines().take(400) {
            lines.push(shape(line));
        }
        lines.join("\n")
    }
}

/// The file at `path`, as text, if it is a regular file no larger than [`READ_BYTES`].
fn read_capped(path: &Path) -> Option<String> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > READ_BYTES {
        return None;
    }
    std::fs::read(path)
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// A line's key, when it is `KEY=value`, `export KEY=value` or `key: value`, and its value.
fn key_value(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    if t.starts_with('#') || t.starts_with("//") {
        return None;
    }
    let t = t.strip_prefix("export ").unwrap_or(t).trim_start();
    let at = t.find(['=', ':'])?;
    let key = t[..at]
        .trim()
        .trim_start_matches("- ")
        .trim_matches(['"', '\'']);
    let ok = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    ok.then(|| (key, t[at + 1..].trim()))
}

/// A line as `read` shows it: its key and never its value.
fn shape(line: &str) -> String {
    let indent = &line[..line.len() - line.trim_start().len()];
    if line.trim().is_empty() {
        return String::new();
    }
    match key_value(line) {
        Some((key, "")) => format!("{indent}{key}:"),
        Some((key, _)) => format!("{indent}{key} = {STRUCK}"),
        None => format!("{indent}‹secret line›"),
    }
}

/// The values of a secret file worth striking: each key's value, unquoted, and each line that is
/// one long word (a key's body, a token), of [`MIN_VALUE`] characters or more.
fn values_of(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let candidate = if let Some((_, v)) = key_value(line) {
            v.split(" #")
                .next()
                .unwrap_or(v)
                .trim()
                .trim_matches(['"', '\''])
                .to_owned()
        } else {
            let t = line.trim();
            if t.contains(char::is_whitespace) || t.starts_with("-----") {
                continue;
            }
            t.to_owned()
        };
        if candidate.chars().count() >= MIN_VALUE {
            out.push(candidate);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> tempfile::TempDir {
        let w = tempfile::tempdir().unwrap();
        let put = |rel: &str, text: &str| {
            let at = w.path().join(rel);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, text).unwrap();
        };
        put(
            ".env",
            "VAULT_TOKEN=hvs.s3cr3tT0kenValue\nexport DB_URL=\"postgres://u:pw@h/db\"\nPORT=8080\n# a note\n",
        );
        put(".env.example", "VAULT_TOKEN=\n");
        put("deploy/prod.tfvars", "api_key = \"abcd1234efgh5678\"\n");
        put("deploy/prod.sops.tfvars", "api_key = ENC[AES256]\n");
        put(
            "keys/server.pem",
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASC\n-----END PRIVATE KEY-----\n",
        );
        put("keys/id_ed25519.pub", "ssh-ed25519 AAAA public\n");
        put(
            "bootstrap/inventory.yml",
            "all:\n  vars:\n    root_password: hunter2hunter2\n",
        );
        put("node_modules/x/.env", "NOT_SCANNED=zzzzzzzzzzzz\n");
        put("src/main.rs", "fn main() {}\n");
        w
    }

    #[test]
    fn the_defaults_and_the_humans_patterns_mark_the_secret_files() {
        let w = world();
        let s = Secrets::scan(w.path(), &["bootstrap/inventory.yml".into()], &[]);
        let rel: Vec<String> = s
            .files()
            .iter()
            .map(|f| f.strip_prefix(w.path()).unwrap().display().to_string())
            .collect();
        assert_eq!(
            rel,
            [
                ".env",
                "bootstrap/inventory.yml",
                "deploy/prod.tfvars",
                "keys/server.pem"
            ]
        );
        // A file made after the scan is held by its name all the same.
        assert!(s.is_secret(&w.path().join("later/.env.production")));
        assert!(!s.is_secret(Path::new("src/main.rs")));
        // Outside the world, nothing is this world's.
        assert!(!s.is_secret(Path::new("/elsewhere/.env")));
    }

    #[test]
    fn a_world_adds_and_never_takes_away_while_the_human_may() {
        let w = world();
        let world_tries = Secrets::patterns(w.path(), &[], &["!.env".into(), "notes.txt".into()]);
        assert!(
            world_tries.is_secret(Path::new(".env")),
            "a world's ! is dropped"
        );
        assert!(world_tries.is_secret(Path::new("a/notes.txt")));
        let human = Secrets::patterns(w.path(), &["!*.tfvars".into()], &[]);
        assert!(!human.is_secret(Path::new("deploy/prod.tfvars")));
    }

    #[test]
    fn the_shape_names_the_keys_and_no_value_reaches_it() {
        let w = world();
        let s = Secrets::scan(w.path(), &[], &[]);
        let view = s.view(&w.path().join(".env"), ".env");
        assert!(view.contains("VAULT_TOKEN = ‹secret›"), "{view}");
        assert!(view.contains("DB_URL = ‹secret›"), "{view}");
        assert!(
            view.contains("‹secret line›"),
            "a comment is a line too: {view}"
        );
        assert!(!view.contains("hvs.") && !view.contains("postgres") && !view.contains("8080"));
    }

    #[test]
    fn the_values_are_struck_from_any_text_the_short_ones_left() {
        let w = world();
        let s = Secrets::scan(w.path(), &["bootstrap/inventory.yml".into()], &[]);
        let out = "token hvs.s3cr3tT0kenValue, url postgres://u:pw@h/db, key MIIEvQIBADANBgkqhkiG9w0BAQEFAASC, pw hunter2hunter2, port 8080";
        let struck = s.strike(out);
        for leaked in ["hvs.", "postgres://", "MIIEv", "hunter2", "zzzz"] {
            assert!(!struck.contains(leaked), "{leaked} in {struck}");
        }
        assert!(
            struck.contains("port 8080"),
            "a short value is a word: {struck}"
        );
        assert!(matches!(s.strike("nothing here"), Cow::Borrowed(_)));
    }
}
