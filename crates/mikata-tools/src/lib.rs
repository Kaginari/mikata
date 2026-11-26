//! The tools: read, index, write, edit, glob, grep, bash, git; the shell classifier; secret
//! injection; the MCP client and the tools it offers; the plugin host.

pub mod attic;
mod beneath;
pub mod classify;
pub mod files;
pub mod fingerprint;
mod forgive;
pub mod gate;
mod index;
pub mod mcp;
pub mod patch;
mod pinned;
pub mod plugin;
pub mod secrets;
pub mod shell;
pub mod tool;
mod tripwire;

pub use attic::{Act, Attic, Conflict, Held, Restore, Snapshot, State};
pub use classify::{
    Place, Push, Rule, Verdict, classify, classify_acts, classify_reach, remote_name,
};
pub use fingerprint::Fingerprint;
pub use gate::{Answer, Ask, Gate, NoHuman, Pending};
/// The logins a forge is reached with, relative to the home.
pub use mikata_sandbox::FORGE_LOGINS;
/// What a sandboxed command may touch, for a harness process the tools do not start.
pub use mikata_sandbox::Policy;
/// The sandbox a context runs commands in, as the context takes it.
pub use mikata_sandbox::Sandbox;
/// The human's state and config directories, refused to an unisolated test.
pub use mikata_sandbox::dirs;
/// A `git` command started without the harness's secrets.
pub use mikata_sandbox::git;
/// Close the harness's `/proc` files (its environment, where provider keys live) to other processes.
pub use mikata_sandbox::harden_self;
/// Whether an environment variable is a secret the harness keeps from its children.
pub use mikata_sandbox::is_secret;
pub use pinned::remove_created;
pub use secrets::Secrets;
pub use shell::{model_call, stash_depth, stdout_of};
pub use tool::{
    Context, Grant, Meta, Outcome, OwnerOf, ReadOf, ReadSpan, SLASH_TMP, Tool, call, ignored,
    registry, retired,
};
pub use tripwire::{kijin_witness, queue_witness, usage_witness};

/// Write a file beneath `world` whole or not at all, following no symlink on the way: for what
/// the harness itself leaves in a world's scratch, where a body's commands may have put a link.
///
/// # Errors
///
/// The path leaves the world, crosses a link, or cannot be written.
pub fn write_beneath(
    world: &std::path::Path,
    path: &std::path::Path,
    content: &[u8],
) -> std::io::Result<()> {
    beneath::write(world, path, content)
}
