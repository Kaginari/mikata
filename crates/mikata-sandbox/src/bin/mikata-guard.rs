//! `mikata-guard [--read P]… [--write P]… [--rules FILE] [--tmp DIR] [--network] [--die-with-parent] -- PROGRAM ARGS…`
//!
//! `--rules` reads `r`/`w` and a path, each NUL-terminated, from FILE and removes it. `--tmp`
//! mounts DIR on `/tmp` for PROGRAM alone, in a mount namespace of its own, before anything is
//! restricted: the harness passes it only on a machine whose probe took it, and a guard the
//! kernel then refuses runs nothing.
//!
//! Restricts its own process with landlock and seccomp, then execs PROGRAM in its place, so the
//! command and everything it starts inherit the restriction. `--probe` restricts and prints the
//! status line instead of running anything. `--die-with-parent` has the kernel kill PROGRAM when
//! the thread that started the guard ends, so a long-lived server never outlives the harness. Exit 125: the restriction could not be enforced,
//! nothing ran; exit 127: PROGRAM could not be executed.

#[cfg(target_os = "linux")]
fn main() {
    use std::os::unix::process::CommandExt as _;
    use std::path::PathBuf;
    use std::process::{Command, exit};

    use mikata_sandbox::linux::{Rules, restrict};

    let mut rules = Rules::default();
    let mut probe = false;
    let mut die_with_parent = false;
    let mut tmp: Option<PathBuf> = None;
    let parent = rustix::process::getppid();
    let mut args = std::env::args_os().skip(1);
    let mut program: Vec<std::ffi::OsString> = Vec::new();
    while let Some(a) = args.next() {
        match a.to_str() {
            Some("--read") => rules.read.extend(args.next().map(PathBuf::from)),
            Some("--write") => rules.write.extend(args.next().map(PathBuf::from)),
            Some("--network") => rules.network = true,
            Some("--rules") => {
                let Some(file) = args.next().map(PathBuf::from) else {
                    eprintln!("mikata-guard: --rules needs a file");
                    exit(125);
                };
                if let Err(e) = read_rules(&file, &mut rules) {
                    eprintln!("mikata-guard: the rules file: {e}; nothing ran");
                    exit(125);
                }
            }
            Some("--tmp") => {
                tmp = args.next().map(PathBuf::from);
                if tmp.is_none() {
                    eprintln!("mikata-guard: --tmp needs a directory");
                    exit(125);
                }
            }
            Some("--probe") => probe = true,
            Some("--die-with-parent") => die_with_parent = true,
            Some("--") => {
                program.extend(args.by_ref());
                break;
            }
            _ => {
                eprintln!("mikata-guard: unknown argument {}", a.to_string_lossy());
                exit(125);
            }
        }
    }
    if die_with_parent {
        // Set before the parent can be checked, then checked: a parent gone in between is caught.
        if let Err(e) =
            rustix::process::set_parent_process_death_signal(Some(rustix::process::Signal::KILL))
        {
            eprintln!("mikata-guard: the parent-death signal: {e}; nothing ran");
            exit(125);
        }
        if rustix::process::getppid() != parent {
            exit(125);
        }
    }
    // The rules file is read and gone by now: the machine's `/tmp`, where it may have lain, can
    // go out of reach.
    if let Some(dir) = &tmp {
        if let Err(e) = mikata_sandbox::linux::private_tmp(dir) {
            eprintln!("mikata-guard: a /tmp of its own: {e}; nothing ran");
            exit(125);
        }
        // Landlock meets a rule on the way up from a file: from inside the mount that way ends
        // at the mounted directory itself, never at the folders above it where it came from. So
        // the directory is named in its own right.
        rules.write.push(dir.clone());
    }
    let status = match restrict(&rules) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mikata-guard: {e}; nothing ran");
            exit(125);
        }
    };
    if probe {
        if tmp.is_some() {
            println!("{status}, {}", mikata_sandbox::linux::PRIVATE_TMP);
        } else {
            println!("{status}");
        }
        return;
    }
    let Some((prog, rest)) = program.split_first() else {
        eprintln!("mikata-guard: no command after --");
        exit(125);
    };
    let err = Command::new(prog).args(rest).exec();
    eprintln!("mikata-guard: cannot run {}: {err}", prog.to_string_lossy());
    exit(127);
}

/// Read the rules file into `rules`, then remove it: it is read once, before any restriction.
#[cfg(target_os = "linux")]
fn read_rules(
    file: &std::path::Path,
    rules: &mut mikata_sandbox::linux::Rules,
) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt as _;
    let bytes = std::fs::read(file)?;
    std::fs::remove_file(file)?;
    let mut fields = bytes.split(|b| *b == 0);
    while let (Some(kind), Some(path)) = (fields.next(), fields.next()) {
        let path = std::path::PathBuf::from(std::ffi::OsStr::from_bytes(path));
        match kind {
            b"r" => rules.read.push(path),
            b"w" => rules.write.push(path),
            b"" => break,
            _ => return Err(std::io::Error::other("a rule is neither r nor w")),
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("mikata-guard runs on Linux only");
    std::process::exit(125);
}
