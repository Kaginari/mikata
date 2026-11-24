//! The sandbox: landlock and seccomp on Linux, Seatbelt on macOS, the network policy.
//!
//! A command runs inside a [`Policy`]: it may write only the world and its own temporary
//! directory, read only the system's directories, the world and the toolchains in the home —
//! never a credential — and reach the network only when the policy says so.
//!
//! - **Linux**: the command is started through `mikata-guard`, which restricts itself with
//!   landlock (the file rules) and seccomp (no sockets without network; no `ptrace`, `mount`,
//!   `unshare`, `bpf`, `io_uring`, … ever), then execs the command. No namespaces are needed, so
//!   it works where unprivileged user namespaces are restricted. Where they are not, the guard
//!   also gives the command a `/tmp` of its own ([`Policy::tmp_at`]): a folder of its scratch
//!   mounted there for it alone, so a scratch file kept in `/tmp` needs nobody's approval and
//!   touches nothing of the machine's.
//! - **macOS**: the command is started through `sandbox-exec` with a generated Seatbelt profile.
//!
//! [`Sandbox::status`] says what is actually in force, or `none — <reason>`: containment is
//! never claimed.

use std::path::{Path, PathBuf};
use std::process::Command;

pub mod dirs;
mod env;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
mod macos;

pub use env::{git, is_secret, scrubbed_env};

/// The logins a forge is reached with, relative to the home: read by the harness's read-only
/// probe and by a typed outward act the human approved, and by nothing else.
pub const FORGE_LOGINS: &[&str] = &[
    ".ssh",
    ".netrc",
    ".git-credentials",
    ".config/git/credentials",
    ".config/gh",
];

/// Credential paths under the home: each covers itself and all beneath it. No sandbox rule names
/// them or a folder above them, Seatbelt hides them as well, and the shell classifier refuses
/// commands that name them. Landlock cannot hide a path inside a tree it reads, so the toolchain
/// trees that hold one — `.cargo`, `.cache` — are granted part by part ([`HOME_TOOLCHAINS`]).
pub const HOME_CREDENTIALS: &[&str] = &[
    ".ssh",
    ".aws",
    ".gnupg",
    ".password-store",
    ".netrc",
    ".git-credentials",
    ".git-credential-cache",
    ".npmrc",
    ".pypirc",
    ".docker",
    ".kube",
    ".config/gh",
    ".config/gcloud",
    ".azure",
    ".cargo/credentials",
    ".cargo/credentials.toml",
    // git's `credential.helper store` file, when it lives beside git's XDG config.
    ".config/git/credentials",
    "Library/Keychains",
    // uv's `uv auth` logins, beside the interpreters and tools the toolchains name.
    ".local/share/uv/credentials",
    // Hugging Face's login tokens — its model cache, `hub`, stays readable — and git's
    // credential-cache socket.
    ".cache/huggingface/token",
    ".cache/huggingface/stored_tokens",
    ".cache/git",
];

/// Credential files anywhere on the system.
pub const SYSTEM_CREDENTIALS: &[&str] = &["/etc/shadow", "/etc/gshadow", "/etc/sudoers"];

/// Toolchains and caches in the home that builds read, on every platform. A tree that holds a
/// credential is named part by part — Landlock cannot hide a path inside a tree it reads — so
/// only what builds use is named, and a cache missing here is unreadable rather than a hole.
pub const HOME_TOOLCHAINS: &[&str] = &[
    // Not `.cargo` whole: `credentials.toml` holds the crates.io tokens. Cargo reads its
    // binaries, the registry's index and sources, git dependencies and its config; its locks and
    // usage database (`.package-cache`, `.global-cache`) are written, which a read grant does
    // not give anyway, and cargo goes on without them. Its config can hold a `[registry] token`,
    // and each git dependency's `git/db/*/config` its remote URL, which may embed a token: cargo
    // cannot build without them, so that risk is accepted and they stay readable.
    ".cargo/bin",
    ".cargo/registry",
    ".cargo/git",
    ".cargo/config.toml",
    ".cargo/config",
    ".cargo/env",
    ".rustup",
    // `.local` only in part: `.local/share/keyrings` holds credentials.
    ".local/bin",
    ".local/lib",
    ".local/share/pnpm",
    ".local/go-current",
    ".npm",
    // `.nvm` only in part: each installed Node's `bin`, `lib` and `include` are added when the
    // policy is made ([`Policy::new`]), never its `etc/npmrc`, which can hold registry tokens.
    ".pnpm-store",
    // Not `.cache` whole: `huggingface/token` and git's credential cache live there. The named
    // caches of Python (uv, pip, Poetry, pre-commit), Go, Node (corepack under `node`, pnpm,
    // node-gyp, TypeScript, Yarn, Playwright's browsers), Deno, Bazel, sccache, Hugging Face's
    // models, and the Kaginari bench's interpreters.
    ".cache/uv",
    ".cache/pip",
    ".cache/pypoetry",
    ".cache/pre-commit",
    ".cache/go-build",
    ".cache/golangci-lint",
    ".cache/node",
    ".cache/pnpm",
    ".cache/node-gyp",
    ".cache/typescript",
    ".cache/yarn",
    ".cache/ms-playwright",
    ".cache/deno",
    ".cache/bazel",
    ".cache/sccache",
    ".cache/huggingface/hub",
    ".cache/kaginari-bench",
    ".pyenv",
    // uv's interpreters and installed tools — the targets behind `.local/bin`'s links — but not
    // `.local/share/uv` whole: its `credentials/` holds the logins `uv auth` keeps.
    ".local/share/uv/python",
    ".local/share/uv/tools",
    ".bun",
    ".deno",
    // git cannot run without its config, and `.gitconfig` can hold an `http.extraHeader`
    // bearer token: that risk is accepted. Of `.config/git`, only the config, ignore and
    // attributes files: its `credentials` is git's credential store.
    ".gitconfig",
    ".config/git/config",
    ".config/git/ignore",
    ".config/git/attributes",
    // Of `~/go` (GOPATH), the installed binaries and the module cache: `go/src` holds the
    // human's own projects and their `.env` files.
    "go/bin",
    "go/pkg",
    // Not `.gradle` and `.m2` whole: `gradle.properties`, `settings.xml` and
    // `settings-security.xml` hold repository passwords.
    ".gradle/caches",
    ".gradle/wrapper",
    ".m2/repository",
];

/// What of each Node version nvm installed a command may read: never its `etc`.
const NVM_PARTS: &[&str] = &["bin", "lib", "include"];

/// The system directories a command may read (and execute from).
const SYSTEM_READ: &[&str] = &[
    "/usr",
    "/bin",
    "/sbin",
    "/lib",
    "/lib32",
    "/lib64",
    "/libx32",
    "/etc",
    "/opt",
    "/proc",
    "/sys",
    "/dev",
    "/nix",
    "/snap",
    "/System",
    "/Library",
    "/Applications",
    "/private/etc",
    "/private/var/db",
    "/var/db",
    "/var/select",
];

/// Device files a command may write.
const DEVICES: &[&str] = &[
    "/dev/null",
    "/dev/zero",
    "/dev/full",
    "/dev/tty",
    "/dev/random",
    "/dev/urandom",
];

/// What a sandboxed command may touch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Read and written: the world and the command's temporary directory.
    pub write: Vec<PathBuf>,
    /// Read (and executed from) only.
    pub read: Vec<PathBuf>,
    /// Paths never readable even under a readable directory (applied where the platform can
    /// deny inside an allowed tree, Seatbelt; on Linux they are simply never allowed).
    pub hidden: Vec<PathBuf>,
    /// Sockets allowed.
    pub network: bool,
    /// Rules left out because they resolve — through a link — onto, inside or above a credential
    /// path: a home toolchain's, or a loan's ([`Self::lend`]).
    pub refused: Vec<PathBuf>,
    /// The command is killed when the thread that started it ends (Linux): a long-lived process
    /// such as an MCP server never outlives the harness.
    pub die_with_parent: bool,
    /// The directory the command sees as `/tmp`, mounted there for it alone (Linux, where
    /// [`Sandbox::private_tmp`] holds): its scratch files land here, and the machine's `/tmp`
    /// is out of its reach. `None`: `/tmp` is the machine's, which no rule lets it touch.
    pub tmp_at: Option<PathBuf>,
}

impl Policy {
    /// The policy for a command in `world` with `home`, writing temporary files under `tmp`.
    #[must_use]
    pub fn new(world: &Path, home: &Path, tmp: &Path, network: bool) -> Self {
        let credentials: Vec<PathBuf> = HOME_CREDENTIALS.iter().map(|c| home.join(c)).collect();
        // A toolchain path the human made a link resolves elsewhere, and the guard grants where
        // it resolves: a rule landing on, inside or above a credential is dropped.
        let resolved: Vec<PathBuf> = credentials
            .iter()
            .flat_map(|c| [c.clone(), resolve(c)])
            .collect();
        let (toolchains, refused): (Vec<PathBuf>, Vec<PathBuf>) = HOME_TOOLCHAINS
            .iter()
            .map(|t| home.join(t))
            .chain(nvm_versions(home))
            .partition(|t| {
                let at = resolve(t);
                !resolved
                    .iter()
                    .any(|c| at.starts_with(c) || c.starts_with(&at))
            });
        let mut read: Vec<PathBuf> = SYSTEM_READ.iter().map(PathBuf::from).collect();
        read.extend(toolchains);
        let mut write = vec![world.to_path_buf(), tmp.to_path_buf()];
        write.extend(DEVICES.iter().map(PathBuf::from));
        let mut hidden = credentials;
        hidden.extend(SYSTEM_CREDENTIALS.iter().map(PathBuf::from));
        Self {
            write,
            read,
            hidden,
            network,
            refused,
            die_with_parent: false,
            tmp_at: None,
        }
    }

    /// Let the command read `path` besides, and never write it: a folder outside its world that
    /// the harness lends it, named by its resolved path. A loan is the folder it was when it
    /// was lent: one that leads elsewhere now, because a link stands in its place or above it,
    /// is no loan. Nor is one that lands on, inside or above a credential. Either is left out,
    /// and kept in `refused`.
    pub fn lend(&mut self, path: &Path) {
        let at = resolve(path);
        let credential = self
            .hidden
            .iter()
            .flat_map(|c| [c.clone(), resolve(c)])
            .any(|c| at.starts_with(&c) || c.starts_with(&at));
        if credential || at != path {
            self.refused.push(path.to_path_buf());
        } else if !self.read.contains(&at) {
            self.read.push(at);
        }
    }

    /// The paths of `list` that exist, resolved; rules cannot name what is not there.
    #[must_use]
    pub fn existing(list: &[PathBuf]) -> Vec<PathBuf> {
        list.iter().filter_map(|p| p.canonicalize().ok()).collect()
    }
}

/// `path` resolved as far as it exists: its links followed, the missing rest kept as written.
fn resolve(path: &Path) -> PathBuf {
    let mut missing = Vec::new();
    let mut at = path;
    loop {
        if let Ok(real) = at.canonicalize() {
            return missing.iter().rev().fold(real, |p, part| p.join(part));
        }
        match (at.parent(), at.file_name()) {
            (Some(parent), Some(name)) => {
                missing.push(name.to_owned());
                at = parent;
            }
            _ => return path.to_path_buf(),
        }
    }
}

/// Each Node version nvm installed: its `bin`, `lib` and `include`.
fn nvm_versions(home: &Path) -> Vec<PathBuf> {
    let Ok(versions) = std::fs::read_dir(home.join(".nvm/versions/node")) else {
        return Vec::new();
    };
    let mut parts: Vec<PathBuf> = versions
        .flatten()
        .flat_map(|v| NVM_PARTS.iter().map(move |p| v.path().join(p)))
        .collect();
    parts.sort();
    parts
}

/// How commands are contained on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sandbox {
    /// Through the `mikata-guard` helper at this path (Linux).
    Guard(PathBuf),
    /// Through `sandbox-exec` (macOS).
    Seatbelt(PathBuf),
    /// Not at all, and why.
    None(String),
}

impl Sandbox {
    /// Find the sandbox for this machine: `MIKATA_GUARD`, or `mikata-guard` beside the running
    /// binary, on Linux; `/usr/bin/sandbox-exec` on macOS.
    #[must_use]
    pub fn detect() -> Self {
        if cfg!(target_os = "linux") {
            let candidate = std::env::var_os("MIKATA_GUARD")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::current_exe()
                        .ok()
                        .and_then(|exe| exe.parent().map(|d| d.join("mikata-guard")))
                });
            match candidate {
                Some(p) if p.is_file() => Self::Guard(p),
                Some(p) => Self::None(format!("the guard helper is missing ({})", p.display())),
                None => Self::None("the guard helper cannot be located".into()),
            }
        } else if cfg!(target_os = "macos") {
            let p = PathBuf::from("/usr/bin/sandbox-exec");
            if p.is_file() {
                Self::Seatbelt(p)
            } else {
                Self::None("sandbox-exec is missing".into())
            }
        } else {
            Self::None(format!("no sandbox for {}", std::env::consts::OS))
        }
    }

    /// A command running `program` with `args` inside `policy`.
    #[must_use]
    pub fn command(&self, policy: &Policy, program: &str, args: &[&str]) -> Command {
        match self {
            Self::Guard(guard) => {
                let mut cmd = Command::new(guard);
                let read = Policy::existing(&policy.read);
                let write = Policy::existing(&policy.write);
                // Past a few hundred rules the command line would not hold them: they go in a
                // private file the guard reads, then removes, before it restricts itself.
                if let Some(file) = (read.len() + write.len() > RULES_ON_ARGV)
                    .then(|| rules_file(&read, &write))
                    .flatten()
                {
                    cmd.arg("--rules").arg(file);
                } else {
                    for p in read {
                        cmd.arg("--read").arg(p);
                    }
                    for p in write {
                        cmd.arg("--write").arg(p);
                    }
                }
                if policy.network {
                    cmd.arg("--network");
                }
                if policy.die_with_parent {
                    cmd.arg("--die-with-parent");
                }
                // Only on a machine whose probe took it, and only a directory that is there.
                if let Some(dir) = policy.tmp_at.as_ref().filter(|d| d.is_dir())
                    && self.private_tmp()
                {
                    cmd.arg("--tmp").arg(dir);
                }
                cmd.arg("--").arg(program).args(args);
                cmd
            }
            #[cfg(target_os = "macos")]
            Self::Seatbelt(exec) => {
                let mut cmd = Command::new(exec);
                cmd.arg("-p")
                    .arg(macos::profile(policy))
                    .arg(program)
                    .args(args);
                cmd
            }
            #[cfg(not(target_os = "macos"))]
            Self::Seatbelt(_) => {
                let mut cmd = Command::new(program);
                cmd.args(args);
                cmd
            }
            Self::None(_) => {
                let mut cmd = Command::new(program);
                cmd.args(args);
                cmd
            }
        }
    }

    /// Whether a command can be given a `/tmp` of its own on this machine ([`Policy::tmp_at`]):
    /// the guard is asked once, with a probe that makes the mount and restricts itself as a
    /// command would, and the answer is kept for the life of the process. Linux only, and only
    /// where the kernel lets an unprivileged process make a user namespace and mount in it;
    /// elsewhere the guard works as it always did, without namespaces.
    #[must_use]
    pub fn private_tmp(&self) -> bool {
        static PROBED: std::sync::Mutex<Vec<(PathBuf, bool)>> = std::sync::Mutex::new(Vec::new());
        let Self::Guard(guard) = self else {
            return false;
        };
        let mut probed = PROBED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((_, took)) = probed.iter().find(|(g, _)| g == guard) {
            return *took;
        }
        let took = probe_private_tmp(guard);
        probed.push((guard.clone(), took));
        took
    }

    /// What is in force, in one line — `landlock ABI 1 + seccomp, network off`, or
    /// `none — <reason>`. On Linux the guard restricts a probe and reports what the kernel did,
    /// and whether a command gets a `/tmp` of its own.
    #[must_use]
    pub fn status(&self, network: bool) -> String {
        match self {
            Self::Guard(guard) => {
                let mut probe = Command::new(guard);
                probe.arg("--probe").env_clear().envs(scrubbed_env());
                if network {
                    probe.arg("--network");
                }
                match probe.output() {
                    // A command the human let reach the network keeps the machine's `/tmp`.
                    Ok(out) if out.status.success() && !network && self.private_tmp() => {
                        format!(
                            "{}, private /tmp",
                            String::from_utf8_lossy(&out.stdout).trim()
                        )
                    }
                    Ok(out) if out.status.success() => {
                        String::from_utf8_lossy(&out.stdout).trim().to_owned()
                    }
                    Ok(out) => format!(
                        "none — the guard refused: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    ),
                    Err(e) => format!("none — the guard did not start: {e}"),
                }
            }
            Self::Seatbelt(_) => {
                format!("seatbelt, network {}", if network { "on" } else { "off" })
            }
            Self::None(reason) => format!("none — {reason}"),
        }
    }
}

/// Ask the guard at `guard` whether it can give a command a `/tmp` of its own: a probe that
/// mounts a fresh empty directory there and restricts itself. False on any refusal.
fn probe_private_tmp(guard: &Path) -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir =
        std::env::temp_dir().join(format!("mikata-tmp-probe-{}-{stamp:x}", std::process::id()));
    if std::fs::create_dir(&dir).is_err() {
        return false;
    }
    let out = Command::new(guard)
        .arg("--probe")
        .arg("--tmp")
        .arg(&dir)
        .env_clear()
        .envs(scrubbed_env())
        .output();
    let _ = std::fs::remove_dir(&dir);
    out.is_ok_and(|o| {
        o.status.success() && String::from_utf8_lossy(&o.stdout).contains("private /tmp")
    })
}

/// Make this process unreadable to the commands it starts: on Linux it becomes non-dumpable,
/// so `/proc/<pid>/environ` and its memory are closed to every non-root process — an unguarded
/// command's included, where landlock is not there to close them. `mikata` and `mikatad`
/// call it first. It writes no core dump and refuses a debugger's attach to anyone but root;
/// the process still reads its own `/proc/self`, and the programs it starts are dumpable again.
///
/// # Errors
///
/// When the kernel refuses.
pub fn harden_self() -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        rustix::process::set_dumpable_behavior(rustix::process::DumpableBehavior::NotDumpable)?;
    }
    Ok(())
}

/// The most rules passed on the guard's command line; past it they go in a file.
const RULES_ON_ARGV: usize = 256;

/// Write the rules to a fresh owner-only file outside every world: `r` or `w`, then a path,
/// each NUL-terminated. `None` when no such file could be made (the rules then go on argv).
fn rules_file(read: &[PathBuf], write: &[PathBuf]) -> Option<PathBuf> {
    use std::io::Write as _;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let path = std::env::temp_dir().join(format!("mikata-rules-{}-{stamp:x}", std::process::id()));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .ok()?;
    let mut bytes = Vec::new();
    for (kind, list) in [(b'r', read), (b'w', write)] {
        for p in list {
            bytes.extend([kind, 0]);
            bytes.extend(p.as_os_str().as_bytes());
            bytes.push(0);
        }
    }
    f.write_all(&bytes).ok()?;
    Some(path)
}
