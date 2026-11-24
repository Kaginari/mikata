//! The plugin host: a third-party plugin is an MCP server that also speaks `mikata/*`.
//!
//! A plugin is a program the human's global file names (`[plugins.<id>]`), started through the
//! MCP client ([`crate::mcp::client`]) in the same sandbox an MCP server runs in. Its `initialize`
//! answer says what it hooks: `capabilities.experimental.mikata = { "version": 1, "points": [...],
//! "grants": [...], "id": ..., "plugin_version": ... }`; a server without it is a plain MCP
//! server, not a plugin. Its manifest comes from that answer, cut down to what the human grants.
//!
//! Each hook is a `mikata/hook` request whose params are the [`Event`] as JSON and whose result
//! is a [`Reply`], waited on at most the hook deadline. A missed deadline, a crash, a malformed
//! reply or a reply above the grant counts a [`Strike`] and is read as [`Reply::Pass`]; at the
//! third strike the plugin has failed ([`Plugin::failed`]), is ended at once and is called no
//! more. An unload is the `mikata/unload` notification, then the process group is ended after
//! [`UNLOAD_GRACE`]. With the `tools` grant, the plugin's tools join the MCP catalog as one more
//! server ([`Declared::server`]), through the four doors, so the frame does not change.
//!
//! The loop wraps a [`Plugin`] as a kernel unit; this module knows nothing of the loop. Calls are
//! blocking, as every MCP call is.

use std::path::Path;
use std::time::Duration;

use mikata_kernel::{Band, FramePolicy, NOTE_CAP, Point, SafePoint};
/// The kernel's vocabulary a plugin speaks: what it is shown, what it answers, what it is granted
/// and the manifest it is loaded with.
pub use mikata_kernel::{Event, Grant, Manifest, Reply};
use mikata_races::Race;
use serde_json::{Value, json};

use crate::mcp::client::{Conn, Fault};
use crate::mcp::{Server, Timeouts, clean, lawful_server_name};
use crate::tool::Context;

/// The version of the `mikata` capability this host speaks.
pub const PROTOCOL: u64 = 1;
/// The deadline on one hook call, unless the human's file says (`plugins.timeouts.hook_ms`).
pub const HOOK_DEADLINE: Duration = Duration::from_secs(2);
/// How long a plugin may take to start and shake hands.
pub const START_DEADLINE: Duration = Duration::from_secs(30);
/// How long an unloaded plugin may take to end by itself before its process group is ended.
pub const UNLOAD_GRACE: Duration = Duration::from_secs(1);
/// The strikes in one session that fail a plugin.
pub const STRIKES: u32 = 3;
/// The prefix of a plugin's unit id: `plugin:no-force-push`.
pub const UNIT_PREFIX: &str = "plugin:";
/// The longest version a plugin may give itself, in characters.
pub const VERSION_CHARS: usize = 32;

/// A plugin as the human's global file declares it: what is started, and what it is granted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// Its id: the table's name in the file, `no-force-push`.
    pub id: String,
    /// The program.
    pub command: String,
    /// Its arguments.
    pub args: Vec<String>,
    /// Environment variables passed to it by name; every other secret is scrubbed.
    pub env: Vec<String>,
    /// What the human grants it. Its own ask is cut down to these.
    pub grants: Vec<Grant>,
    /// The bodies whose sessions it joins.
    pub races: Vec<Race>,
}

impl Declared {
    /// Its unit id in a composition: `plugin:<id>`.
    #[must_use]
    pub fn unit_id(&self) -> String {
        format!("{UNIT_PREFIX}{}", self.id)
    }

    /// The MCP server it runs as: the same program, arguments, passed variables and races, no
    /// network, its tools at the default class. Its hooks and, with the `tools` grant, its tools
    /// are both served by this program.
    #[must_use]
    pub fn as_server(&self) -> Server {
        let mut server = Server::new(&self.id, &self.command, &self.races);
        server.args.clone_from(&self.args);
        server.env.clone_from(&self.env);
        server.purpose = format!("tools of the plugin `{}`", self.id);
        // A plugin's program is a file the human named by its absolute path — the script an
        // interpreter runs, or the program itself — and lives outside the world: its folder is
        // lent to it to read, and nothing else is.
        server.reads = std::iter::once(&self.command)
            .chain(&self.args)
            .map(Path::new)
            .filter(|p| p.is_absolute() && p.is_file())
            .filter_map(Path::parent)
            .map(Path::to_path_buf)
            .collect();
        server.reads.dedup();
        server
    }

    /// The server whose tools join the MCP catalog, when the human grants `tools`: added to the
    /// session's MCP servers when the session starts, so its tools pass the four doors.
    #[must_use]
    pub fn server(&self) -> Option<Server> {
        self.grants
            .contains(&Grant::Tools)
            .then(|| self.as_server())
    }
}

/// The grant a configuration names: `observe`, `advise`, `veto`, `stop` or `tools`.
#[must_use]
pub fn grant_named(name: &str) -> Option<Grant> {
    match name {
        "observe" => Some(Grant::Observe),
        "advise" => Some(Grant::Advise),
        "veto" => Some(Grant::Veto),
        "stop" => Some(Grant::Stop),
        "tools" => Some(Grant::Tools),
        _ => None,
    }
}

/// The plugins' servers joined to the MCP servers: a plugin with the `tools` grant whose id an
/// MCP server already names is refused, and the returned lines say so.
#[must_use]
pub fn join_servers(servers: &mut Vec<Server>, plugins: &[Declared]) -> Vec<String> {
    let mut holes = Vec::new();
    for server in plugins.iter().filter_map(Declared::server) {
        if servers.iter().any(|s| s.name == server.name) {
            holes.push(format!(
                "plugin `{}`: its tools are not offered — an MCP server has the same name",
                server.name
            ));
        } else {
            servers.push(server);
        }
    }
    holes
}

/// What went wrong with one hook call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offence {
    /// It did not answer within the deadline.
    Late,
    /// It is gone: closed, flooding or no longer reading.
    Crashed,
    /// It answered something that is no reply, refused the hook, or answered a reply that does
    /// not count at the point.
    Malformed,
    /// It answered a reply its grant does not allow.
    AboveGrant,
}

/// One strike against a plugin: what went wrong, how many strikes it now has, and whether that
/// failed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strike {
    /// What went wrong.
    pub fault: Offence,
    /// What went wrong, in a sentence naming the plugin.
    pub why: String,
    /// Its strikes this session, this one counted.
    pub count: u32,
    /// Whether this strike failed it: it is called no more.
    pub failed: bool,
}

/// A running plugin: its manifest, cut to the human's grant, and its connection.
pub struct Plugin {
    id: String,
    manifest: Manifest,
    conn: Option<Conn>,
    deadline: Duration,
    strikes: u32,
    failed: Option<String>,
}

impl std::fmt::Debug for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plugin")
            .field("id", &self.id)
            .field("strikes", &self.strikes)
            .finish_non_exhaustive()
    }
}

impl Plugin {
    /// Start `declared` in the sandbox of `cx`, working in `scratch`, shake hands within
    /// [`START_DEADLINE`], and read its manifest; each hook waits at most `deadline`.
    ///
    /// # Errors
    ///
    /// The program did not start (as an MCP server would not), the handshake failed, or the
    /// answer carries no lawful `mikata` capability.
    pub fn start(
        declared: &Declared,
        cx: &Context,
        scratch: &Path,
        deadline: Duration,
    ) -> Result<Self, String> {
        Self::start_within(declared, cx, scratch, deadline, START_DEADLINE)
    }

    /// As [`Plugin::start`], the handshake waited on at most `start`.
    ///
    /// # Errors
    ///
    /// As [`Plugin::start`].
    pub fn start_within(
        declared: &Declared,
        cx: &Context,
        scratch: &Path,
        deadline: Duration,
        start: Duration,
    ) -> Result<Self, String> {
        if !lawful_server_name(&declared.id) {
            return Err(format!("plugin `{}`: no lawful id", clean(&declared.id)));
        }
        let timeouts = Timeouts {
            start,
            list: start,
            call: start,
        };
        let conn = Conn::start(&declared.as_server(), cx, scratch, timeouts)
            .map_err(|e| format!("plugin `{}`: {e}", declared.id))?;
        let manifest = manifest(declared, conn.handshake())?;
        Ok(Self {
            id: declared.id.clone(),
            manifest,
            conn: Some(conn),
            deadline,
            strikes: 0,
            failed: None,
        })
    }

    /// Its manifest: the points it hooks, each in the outer band, and what it asked for cut to
    /// what the human grants.
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Its strikes this session.
    #[must_use]
    pub fn strikes(&self) -> u32 {
        self.strikes
    }

    /// Why it failed, once its third strike is counted: the kernel quarantines it on this.
    #[must_use]
    pub fn failed(&self) -> Option<String> {
        self.failed.clone()
    }

    /// One hook call: `event` sent as `mikata/hook`, its reply read within the deadline. A
    /// failure, or a reply its grant or the point does not allow, is read as [`Reply::Pass`] and
    /// counted as a strike; the third ends the plugin at once. A failed plugin, or a point it does
    /// not hook, answers [`Reply::Pass`] without a call. Text a reply carries is cleaned of
    /// control characters and cut to [`NOTE_CAP`] characters.
    pub fn on(&mut self, event: &Event) -> (Reply, Option<Strike>) {
        let point = event.point();
        if self.failed.is_some() || self.manifest.band_at(point).is_none() {
            return (Reply::Pass, None);
        }
        let Some(conn) = self.conn.as_mut() else {
            return (Reply::Pass, None);
        };
        let params = serde_json::to_value(event).unwrap_or(Value::Null);
        let answered = conn.ask(self.deadline, "mikata/hook", &params);
        let id = &self.id;
        let read = match answered {
            Ok(result) => self.read(point, result),
            Err(Fault::Late(_)) => Err((
                Offence::Late,
                format!(
                    "plugin `{id}` did not answer {} within {}ms",
                    point.name(),
                    self.deadline.as_millis()
                ),
            )),
            Err(Fault::Dead(e)) => Err((
                Offence::Crashed,
                format!("plugin `{id}` is gone at {}: {e}", point.name()),
            )),
            Err(Fault::Refused(e)) => Err((
                Offence::Malformed,
                format!("plugin `{id}` refused {}: {e}", point.name()),
            )),
        };
        match read {
            Ok(reply) => (reply, None),
            Err((fault, why)) => (Reply::Pass, Some(self.strike(fault, why))),
        }
    }

    /// Unload it: the `mikata/unload` notification, then its process group ended after at most
    /// [`UNLOAD_GRACE`]. A second unload, or one after it failed, does nothing.
    pub fn unload(&mut self) {
        self.end(UNLOAD_GRACE);
    }

    fn end(&mut self, grace: Duration) {
        if let Some(mut conn) = self.conn.take() {
            let grace = if conn.notify("mikata/unload", Some(&json!({}))).is_ok() && conn.alive() {
                grace
            } else {
                Duration::ZERO
            };
            conn.end(grace);
        }
    }

    /// A result read as a reply this plugin may give at `point`, or the strike it earns.
    fn read(&self, point: Point, result: Value) -> Result<Reply, (Offence, String)> {
        let id = &self.id;
        let reply = serde_json::from_value::<Reply>(result).map_err(|e| {
            (
                Offence::Malformed,
                format!(
                    "plugin `{id}` answered {} with no reply: {}",
                    point.name(),
                    clean(&e.to_string().chars().take(200).collect::<String>())
                ),
            )
        })?;
        if !reply.counts_at(point) {
            return Err((
                Offence::Malformed,
                format!(
                    "plugin `{id}` answered {} with a {}, which does not count there",
                    point.name(),
                    reply.kind()
                ),
            ));
        }
        if let Some(grant) = reply.needs()
            && !self.manifest.grants.contains(&grant)
        {
            return Err((
                Offence::AboveGrant,
                format!(
                    "plugin `{id}` answered {} with a {}, which needs the {} grant",
                    point.name(),
                    reply.kind(),
                    grant.name()
                ),
            ));
        }
        Ok(tidy(reply))
    }

    fn strike(&mut self, fault: Offence, why: String) -> Strike {
        self.strikes += 1;
        let failed = self.strikes >= STRIKES;
        if failed {
            self.failed = Some(format!("{STRIKES} strikes this session; the last: {why}"));
            // Ended at once: a failed plugin is no longer waited on.
            self.end(Duration::ZERO);
        }
        Strike {
            fault,
            why,
            count: self.strikes,
            failed,
        }
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        self.end(Duration::ZERO);
    }
}

/// A reply's text cleaned of control characters and cut to [`NOTE_CAP`] characters.
fn tidy(reply: Reply) -> Reply {
    let cut = |text: String| clean(&text).chars().take(NOTE_CAP).collect::<String>();
    match reply {
        Reply::Pass => Reply::Pass,
        Reply::Note { text } => Reply::Note { text: cut(text) },
        Reply::Text { text } => Reply::Text { text: cut(text) },
        Reply::Veto { why } => Reply::Veto { why: cut(why) },
        Reply::Stop { why } => Reply::Stop { why: cut(why) },
    }
}

/// The manifest an `initialize` answer declares, cut to what the human grants `declared`.
///
/// # Errors
///
/// No `mikata` capability (a plain MCP server), another version of it, or one whose points or
/// grants are no lists of names.
pub fn manifest(declared: &Declared, handshake: &Value) -> Result<Manifest, String> {
    let id = &declared.id;
    let Some(cap) = handshake["capabilities"]["experimental"]
        .get("mikata")
        .filter(|c| c.is_object())
    else {
        return Err(format!(
            "plugin `{id}` declares no `mikata` capability: it is a plain MCP server, not a plugin"
        ));
    };
    if cap["version"].as_u64() != Some(PROTOCOL) {
        return Err(format!(
            "plugin `{id}` speaks version {} of the `mikata` capability; this host speaks {PROTOCOL}",
            clean(
                &cap["version"]
                    .to_string()
                    .chars()
                    .take(20)
                    .collect::<String>()
            )
        ));
    }
    let names = |key: &str| -> Result<Vec<String>, String> {
        match &cap[key] {
            Value::Null => Ok(Vec::new()),
            Value::Array(items) => items
                .iter()
                .map(|v| {
                    v.as_str().map(str::to_owned).ok_or_else(|| {
                        format!("plugin `{id}`: `mikata.{key}` holds something that is no name")
                    })
                })
                .collect(),
            _ => Err(format!("plugin `{id}`: `mikata.{key}` is no list")),
        }
    };
    let version: String = cap["plugin_version"]
        .as_str()
        .map(clean)
        .map(|v| v.trim().chars().take(VERSION_CHARS).collect())
        .filter(|v: &String| !v.is_empty())
        .unwrap_or_else(|| "0".to_owned());
    let mut manifest = Manifest::new(declared.unit_id(), version)
        .safe_point(SafePoint::Ask)
        .frame(FramePolicy::Tail);
    // A point or a grant this host does not know is passed over: a newer plugin still loads.
    for name in names("points")? {
        if let Some(point) = Point::ALL.into_iter().find(|p| p.name() == name)
            && manifest.band_at(point).is_none()
        {
            manifest = manifest.at(point, Band::Outer);
        }
    }
    let asked: Vec<Grant> = names("grants")?
        .iter()
        .filter_map(|g| grant_named(g))
        .collect();
    for grant in [
        Grant::Observe,
        Grant::Advise,
        Grant::Veto,
        Grant::Stop,
        Grant::Tools,
    ] {
        if asked.contains(&grant) && declared.grants.contains(&grant) {
            manifest = manifest.grant(grant);
        }
    }
    if manifest.grants.is_empty() {
        // Nothing asked is granted: it still sees what it hooks, and every reply is a pass.
        manifest = manifest.grant(Grant::Observe);
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(grants: &[Grant]) -> Declared {
        Declared {
            id: "no-force-push".into(),
            command: "python3".into(),
            args: vec!["/p/no_force_push.py".into()],
            env: vec!["TOKEN".into()],
            grants: grants.to_vec(),
            races: vec![Race::Slime],
        }
    }

    #[test]
    fn a_plugins_server_reads_its_own_program_folder_and_no_other() {
        let dir = std::env::temp_dir().join(format!("mikata-plugin-reads-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("observe.py");
        std::fs::write(&script, "").unwrap();
        let mut d = declared(&[Grant::Observe]);
        d.args = vec![
            script.display().to_string(),
            "/not/a/file.jsonl".into(),
            "-v".into(),
        ];
        assert_eq!(d.as_server().reads, [dir.clone()]);
        d.args.clear();
        assert!(
            d.as_server().reads.is_empty(),
            "a bare program on PATH lends nothing"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn hello(cap: &Value) -> Value {
        json!({"protocolVersion": "2025-06-18", "capabilities": {"experimental": {"mikata": cap}}})
    }

    #[test]
    fn the_handshake_is_cut_to_the_human_grant() {
        let cap = json!({
            "version": 1,
            "id": "no-force-push",
            "plugin_version": "0.2.0",
            "points": ["tool_before", "answer_land", "not_a_point", "tool_before"],
            "grants": ["advise", "veto", "stop", "shout"],
        });
        let m = manifest(&declared(&[Grant::Veto, Grant::Tools]), &hello(&cap)).unwrap();
        assert_eq!(m.id, "plugin:no-force-push");
        assert_eq!(m.version, "0.2.0");
        assert_eq!(
            m.points,
            vec![
                (Point::ToolBefore, Band::Outer),
                (Point::AnswerLand, Band::Outer)
            ]
        );
        assert_eq!(m.grants, vec![Grant::Veto]);
        assert_eq!(m.safe_point, SafePoint::Ask);
        assert_eq!(m.frame, FramePolicy::Tail);
        assert!(!m.tightens_when_off);
    }

    #[test]
    fn nothing_granted_leaves_it_observing() {
        let cap = json!({"version": 1, "points": ["ask_begin"], "grants": ["veto"]});
        let m = manifest(&declared(&[Grant::Advise]), &hello(&cap)).unwrap();
        assert_eq!(m.grants, vec![Grant::Observe]);
        assert_eq!(m.version, "0");
    }

    #[test]
    fn a_plain_server_or_another_version_is_no_plugin() {
        let d = declared(&[Grant::Veto]);
        let plain = json!({"capabilities": {"tools": {}}});
        assert!(
            manifest(&d, &plain)
                .unwrap_err()
                .contains("plain MCP server")
        );
        let v2 = hello(&json!({"version": 2}));
        assert!(manifest(&d, &v2).unwrap_err().contains("version 2"));
        let bad = hello(&json!({"version": 1, "points": "tool_before"}));
        assert!(manifest(&d, &bad).unwrap_err().contains("no list"));
        let worse = hello(&json!({"version": 1, "grants": [3]}));
        assert!(manifest(&d, &worse).unwrap_err().contains("no name"));
    }

    #[test]
    fn a_version_is_cleaned_and_capped() {
        let cap = json!({"version": 1, "plugin_version": format!("1\u{1b}[31m{}", "9".repeat(80))});
        let m = manifest(&declared(&[]), &hello(&cap)).unwrap();
        assert!(!m.version.contains('\u{1b}'));
        assert_eq!(m.version.chars().count(), VERSION_CHARS);
    }

    #[test]
    fn only_the_tools_grant_joins_the_catalog_and_a_taken_name_is_refused() {
        assert_eq!(declared(&[Grant::Veto]).server(), None);
        let with = declared(&[Grant::Tools]);
        let server = with.server().unwrap();
        assert_eq!(server.name, "no-force-push");
        assert_eq!(server.args, vec!["/p/no_force_push.py".to_owned()]);
        assert_eq!(server.env, vec!["TOKEN".to_owned()]);
        assert!(!server.network);
        let mut servers = vec![Server::new("other", "x", &[])];
        let none: Vec<String> = Vec::new();
        assert_eq!(
            join_servers(&mut servers, std::slice::from_ref(&with)),
            none
        );
        assert_eq!(servers.len(), 2);
        let holes = join_servers(&mut servers, &[with]);
        assert_eq!(holes.len(), 1);
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn grants_are_read_by_their_names() {
        for g in [
            Grant::Observe,
            Grant::Advise,
            Grant::Veto,
            Grant::Stop,
            Grant::Tools,
        ] {
            assert_eq!(grant_named(g.name()), Some(g));
        }
        assert_eq!(grant_named("shout"), None);
    }

    #[test]
    fn reply_text_is_cleaned_and_capped() {
        let long = format!("\u{1b}[31m{}", "x".repeat(NOTE_CAP + 10));
        let Reply::Note { text } = tidy(Reply::Note { text: long }) else {
            panic!("a note stays a note");
        };
        assert!(!text.contains('\u{1b}'));
        assert_eq!(text.chars().count(), NOTE_CAP);
    }
}
