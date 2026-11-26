//! The Linux side: the guard restricts its own process, then execs the command.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use landlock::{
    ABI, Access, AccessFs, CompatLevel, Compatible, LandlockStatus, Ruleset, RulesetAttr,
    RulesetCreatedAttr, RulesetStatus, path_beneath_rules,
};
use rustix::process::{Resource, Rlimit, getrlimit, setrlimit};
use seccompiler::{
    BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter,
    SeccompRule, TargetArch,
};

/// The newest landlock ABI the guard asks for; older kernels enforce what they know.
const ABI_ASKED: ABI = ABI::V5;

/// What the guard restricts itself to.
#[derive(Debug, Default)]
pub struct Rules {
    /// Readable and executable.
    pub read: Vec<PathBuf>,
    /// Readable and writable.
    pub write: Vec<PathBuf>,
    /// Sockets allowed.
    pub network: bool,
}

/// Syscalls no sandboxed command may make: debugging or reading other processes, changing
/// mounts or namespaces, loading code into the kernel, the kernel's own keyring, `io_uring`
/// (which would bypass the socket rule), and leaving the command's process group (`setsid`,
/// `setpgid`), so that killing the group ends every process the command started.
const ALWAYS_DENIED: &[libc::c_long] = &[
    libc::SYS_ptrace,
    libc::SYS_process_vm_readv,
    libc::SYS_process_vm_writev,
    libc::SYS_mount,
    libc::SYS_umount2,
    libc::SYS_pivot_root,
    libc::SYS_chroot,
    libc::SYS_unshare,
    libc::SYS_setns,
    libc::SYS_keyctl,
    libc::SYS_add_key,
    libc::SYS_request_key,
    libc::SYS_bpf,
    libc::SYS_perf_event_open,
    libc::SYS_kexec_load,
    libc::SYS_kexec_file_load,
    libc::SYS_init_module,
    libc::SYS_finit_module,
    libc::SYS_delete_module,
    libc::SYS_reboot,
    libc::SYS_swapon,
    libc::SYS_swapoff,
    libc::SYS_open_by_handle_at,
    libc::SYS_userfaultfd,
    libc::SYS_io_uring_setup,
    libc::SYS_io_uring_enter,
    libc::SYS_io_uring_register,
    libc::SYS_acct,
    libc::SYS_quotactl,
    libc::SYS_setsid,
    libc::SYS_setpgid,
];

/// Processes the user may run while a command is in the guard: bounds a fork bomb. The kernel
/// counts every process of the user, so the cap is generous.
const MAX_PROCESSES: u64 = 8192;
/// The largest file a command may write.
const MAX_FILE_BYTES: u64 = 4 << 30;

/// Namespace flags a `clone` may not carry: new namespaces widen the kernel's attack surface.
const NAMESPACE_FLAGS: [libc::c_int; 7] = [
    libc::CLONE_NEWNS,
    libc::CLONE_NEWCGROUP,
    libc::CLONE_NEWUTS,
    libc::CLONE_NEWIPC,
    libc::CLONE_NEWUSER,
    libc::CLONE_NEWPID,
    libc::CLONE_NEWNET,
];

/// Give this process, and everything it starts, a `/tmp` of its own: `dir` mounted there, in a
/// mount namespace nothing else shares. The machine's `/tmp` is then out of reach from inside,
/// and nothing outside sees the mount: a command that keeps a scratch file in `/tmp`, as most
/// do, keeps it in `dir`.
///
/// A mount namespace takes a user namespace. The process keeps its user and its group, each
/// mapped to itself, and holds no capability once it execs a program; the seccomp filter that
/// follows refuses any further namespace or mount. It works only where the kernel lets an
/// unprivileged process make a user namespace and mount in it, which the harness asks first with
/// a probe ([`crate::Sandbox::private_tmp`]): it is never tried on a machine that refused it.
///
/// # Errors
///
/// What the kernel refused. The process may then stand in a namespace it cannot use, so the
/// caller runs nothing.
pub fn private_tmp(dir: &Path) -> Result<(), String> {
    use rustix::mount::{MountPropagationFlags, mount_bind, mount_change};
    use rustix::thread::{UnshareFlags, unshare_unsafe};
    let (uid, gid) = (
        rustix::process::getuid().as_raw(),
        rustix::process::getgid().as_raw(),
    );
    // SAFETY: the guard is a single thread here, before it execs, and holds nothing — no open
    // directory stream, no lock, no other thread's view of the mounts — that a new user and
    // mount namespace would leave dangling.
    #[allow(
        unsafe_code,
        reason = "unshare(2) has no safe wrapper; the guard is single-threaded here"
    )]
    unsafe { unshare_unsafe(UnshareFlags::NEWUSER | UnshareFlags::NEWNS) }
        .map_err(|e| format!("a namespace of its own: {e}"))?;
    let map =
        |file: &str, text: String| std::fs::write(file, text).map_err(|e| format!("{file}: {e}"));
    // An unprivileged process may map its group only once it can no longer change its groups.
    map("/proc/self/setgroups", "deny".into())?;
    map("/proc/self/uid_map", format!("{uid} {uid} 1"))?;
    map("/proc/self/gid_map", format!("{gid} {gid} 1"))?;
    // Nothing mounted from here on spreads back to the machine's own mounts.
    mount_change(
        "/",
        MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
    )
    .map_err(|e| format!("its mounts made private: {e}"))?;
    mount_bind(dir, "/tmp").map_err(|e| format!("{} on /tmp: {e}", dir.display()))
}

/// What the status line says when the command has a `/tmp` of its own.
pub const PRIVATE_TMP: &str = "private /tmp";

/// Put an empty file in `path`'s place for this process and what it starts: `/dev/null` mounted
/// over it, in the mount namespace [`private_tmp`] made (and only there: its mounts are
/// private). Read, it is empty; written, the bytes go nowhere. The seccomp filter that follows
/// refuses any mount, so the command cannot take it off.
///
/// # Errors
///
/// What the kernel refused; the caller runs nothing then.
pub fn hide(path: &Path) -> Result<(), String> {
    rustix::mount::mount_bind("/dev/null", path)
        .map_err(|e| format!("{} kept from the command: {e}", path.display()))
}

/// Lower a limit to `cap`, never raise it.
fn cap(resource: Resource, cap: u64) -> Result<(), String> {
    let now = getrlimit(resource);
    let lower = |v: Option<u64>| Some(v.map_or(cap, |v| v.min(cap)));
    setrlimit(
        resource,
        Rlimit {
            current: lower(now.current),
            maximum: lower(now.maximum),
        },
    )
    .map_err(|e| format!("limit {resource:?}: {e}"))
}

/// Restrict this process: limits, landlock (which also sets no-new-privileges), then seccomp. (it also sets no-new-privileges), then seccomp.
/// Returns the status line for `status`.
///
/// # Errors
///
/// A message naming what could not be enforced; the caller must not run the command then.
pub fn restrict(rules: &Rules) -> Result<String, String> {
    cap(Resource::Core, 0)?;
    cap(Resource::Fsize, MAX_FILE_BYTES)?;
    cap(Resource::Nproc, MAX_PROCESSES)?;
    let (landlock, abi) = landlock(rules)?;
    // Below ABI 3 landlock cannot refuse `truncate(2)` on a path it only lets be read: seccomp
    // refuses the call instead (`ftruncate` and `O_TRUNC` need a descriptor opened for writing,
    // which landlock does check).
    seccomp(rules.network, abi < 3)?;
    Ok(format!(
        "{landlock} + seccomp + limits, network {}",
        if rules.network { "on" } else { "off" }
    ))
}

fn landlock(rules: &Rules) -> Result<(String, i32), String> {
    let fail = |e: &dyn std::fmt::Display| format!("landlock: {e}");
    let status = Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(AccessFs::from_all(ABI_ASKED))
        .map_err(|e| fail(&e))?
        .create()
        .map_err(|e| fail(&e))?
        .add_rules(path_beneath_rules(
            &rules.read,
            AccessFs::from_read(ABI_ASKED),
        ))
        .map_err(|e| fail(&e))?
        .add_rules(path_beneath_rules(
            &rules.write,
            AccessFs::from_all(ABI_ASKED),
        ))
        .map_err(|e| fail(&e))?
        .restrict_self()
        .map_err(|e| fail(&e))?;
    if !status.no_new_privs {
        return Err("no-new-privileges could not be set".into());
    }
    let (abi, effective) = match status.landlock {
        LandlockStatus::Available {
            effective_abi,
            kernel_abi,
        } => (
            kernel_abi.map_or_else(|| format!("{}", effective_abi as i32), |k| k.to_string()),
            effective_abi as i32,
        ),
        LandlockStatus::NotEnabled => {
            return Err(
                "landlock is built in but not enabled (add it to the lsm= boot parameter)".into(),
            );
        }
        LandlockStatus::NotImplemented => {
            return Err("this kernel has no landlock (5.13 or newer is needed)".into());
        }
    };
    match status.ruleset {
        RulesetStatus::FullyEnforced => Ok((format!("landlock ABI {abi}"), effective)),
        RulesetStatus::PartiallyEnforced => Ok((
            format!("landlock ABI {abi} (of {} asked)", ABI_ASKED as i32),
            effective,
        )),
        RulesetStatus::NotEnforced => Err("landlock rules were not enforced".into()),
    }
}

fn seccomp(network: bool, no_truncate: bool) -> Result<(), String> {
    let mut denied: BTreeMap<i64, Vec<SeccompRule>> =
        ALWAYS_DENIED.iter().map(|s| ((*s), Vec::new())).collect();
    if no_truncate {
        denied.insert(libc::SYS_truncate, Vec::new());
    }
    if !network {
        // Every socket, Unix ones included: a host daemon's socket (docker, an agent) is a way
        // out. `socketpair` stays: it reaches nothing.
        denied.insert(libc::SYS_socket, Vec::new());
    }
    let clone_flags = NAMESPACE_FLAGS
        .iter()
        .map(|&flag| {
            #[allow(
                clippy::cast_sign_loss,
                reason = "the clone flags are positive bit masks"
            )]
            let flag = flag as u64;
            SeccompCondition::new(
                0,
                SeccompCmpArgLen::Qword,
                SeccompCmpOp::MaskedEq(flag),
                flag,
            )
            .and_then(|c| SeccompRule::new(vec![c]))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("seccomp: {e}"))?;
    denied.insert(libc::SYS_clone, clone_flags);
    let arch = TargetArch::try_from(std::env::consts::ARCH).map_err(|e| format!("seccomp: {e}"))?;
    apply(denied, libc::EPERM, arch)?;
    // `clone3` takes its flags in memory seccomp cannot read: it reports itself missing, and the
    // C library falls back to `clone`, whose flags the rule above sees.
    apply(
        BTreeMap::from([(libc::SYS_clone3, Vec::new())]),
        libc::ENOSYS,
        arch,
    )
}

fn apply(
    rules: BTreeMap<i64, Vec<SeccompRule>>,
    errno: libc::c_int,
    arch: TargetArch,
) -> Result<(), String> {
    #[allow(
        clippy::cast_sign_loss,
        reason = "errno values are small positive numbers"
    )]
    let errno = errno as u32;
    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(errno),
        arch,
    )
    .map_err(|e| format!("seccomp: {e}"))?;
    let program: BpfProgram = filter
        .try_into()
        .map_err(|e: seccompiler::BackendError| format!("seccomp: {e}"))?;
    seccompiler::apply_filter(&program).map_err(|e| format!("seccomp: {e}"))
}
