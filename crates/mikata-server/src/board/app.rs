//! The board as an app: a window of its own, no tabs and no address bar, from a browser of the
//! Chromium family (Chrome, Edge, Brave, Chromium, Vivaldi) in its app mode — `--app=<page>`.
//!
//! The page handed over is the board's launcher file ([`super::Board::launch`]), as `--open`
//! hands the default browser: the key is in the file, never on the command line, which every
//! local user can read while the browser starts. A browser already running takes the window
//! into its own process.

use std::path::{Path, PathBuf};

use super::browser::Browser;

/// A browser that can open an app window: its program, and the kind of browser it is (a snap
/// reads its launcher from its own directory).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppBrowser {
    /// The program to run.
    pub program: PathBuf,
    /// Where its launcher goes.
    pub kind: Browser,
}

/// The programs tried on Linux, in order, by name on the `PATH`.
const LINUX: &[&str] = &[
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    "microsoft-edge",
    "microsoft-edge-stable",
    "brave-browser",
    "vivaldi",
];

/// The applications tried on macOS, in order: each bundle's program.
const MACOS: &[&str] = &[
    "Google Chrome.app/Contents/MacOS/Google Chrome",
    "Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "Brave Browser.app/Contents/MacOS/Brave Browser",
    "Chromium.app/Contents/MacOS/Chromium",
    "Vivaldi.app/Contents/MacOS/Vivaldi",
];

/// What the human is told when no browser can open an app window.
pub const NO_APP_BROWSER: &str = "no browser here opens an app window (Chrome, Edge, Brave, \
                                  Chromium or Vivaldi): opened as a page in your browser instead";

/// The first browser that opens app windows: on macOS among `apps` (the `/Applications`
/// folders), elsewhere on `path` (the `PATH`).
#[must_use]
pub fn find_in(path: Option<&std::ffi::OsStr>, apps: &[PathBuf]) -> Option<AppBrowser> {
    if cfg!(target_os = "macos") {
        return MACOS.iter().find_map(|rel| {
            apps.iter()
                .map(|dir| dir.join(rel))
                .find(|p| p.is_file())
                .map(|program| AppBrowser {
                    program,
                    kind: Browser::Plain,
                })
        });
    }
    let dirs: Vec<PathBuf> = path.map_or_else(Vec::new, |p| std::env::split_paths(p).collect());
    LINUX.iter().find_map(|name| {
        dirs.iter()
            .filter(|d| d.is_absolute())
            .map(|d| d.join(name))
            .find(|p| executable(p))
            .map(|program| {
                let kind = snap_of(&program).map_or(Browser::Plain, Browser::Snap);
                AppBrowser { program, kind }
            })
    })
}

/// The first browser that opens app windows on this machine.
#[must_use]
pub fn find() -> Option<AppBrowser> {
    let mut apps = vec![PathBuf::from("/Applications")];
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        apps.push(home.join("Applications"));
    }
    find_in(std::env::var_os("PATH").as_deref(), &apps)
}

/// Whether `p` is a file this user may run.
fn executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

/// The snap a program runs, when it is one: `/snap/bin/<snap>` or a link that resolves there.
fn snap_of(program: &Path) -> Option<String> {
    let real = std::fs::canonicalize(program).unwrap_or_else(|_| program.to_path_buf());
    for p in [program, real.as_path()] {
        if let Ok(rest) = p.strip_prefix("/snap/bin") {
            let name = rest.to_str()?.split('.').next()?.to_owned();
            return Some(name);
        }
    }
    None
}

/// The command that opens the launcher `file` as an app window of `browser`. Only the file's
/// path is on the command line; the key is inside it.
#[must_use]
pub fn opener(browser: &AppBrowser, file: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(&browser.program);
    cmd.arg(format!("--app=file://{}", file.display()))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_names_the_file_and_never_a_key() {
        let b = AppBrowser {
            program: PathBuf::from("/usr/bin/chromium"),
            kind: Browser::Plain,
        };
        let cmd = opener(&b, Path::new("/state/open-abc.html"));
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["--app=file:///state/open-abc.html"]);
        assert!(!args.iter().any(|a| a.contains("#k=")));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn the_first_browser_on_the_path_is_found_and_a_plain_file_is_not() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = std::env::temp_dir().join(format!("mikata-app-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = std::env::join_paths([&dir]).unwrap();
        assert_eq!(find_in(Some(&path), &[]), None, "nothing there");
        // Not executable: not a browser.
        std::fs::write(dir.join("chromium"), "").unwrap();
        assert_eq!(find_in(Some(&path), &[]), None);
        std::fs::set_permissions(dir.join("chromium"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
        std::fs::write(dir.join("microsoft-edge"), "").unwrap();
        std::fs::set_permissions(
            dir.join("microsoft-edge"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        let found = find_in(Some(&path), &[]).unwrap();
        assert_eq!(found.program, dir.join("chromium"), "the order chooses");
        assert_eq!(found.kind, Browser::Plain);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
