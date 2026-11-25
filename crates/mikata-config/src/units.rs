//! Units and plugins: which parts of the loop run, and which third-party programs join it.
//!
//! `[units]` switches native units (compiled in) on and off by id; the default composition is
//! every unit on. The human's global file may switch any unit either way. A world's files — the
//! tracked one and the local one, which whatever runs in the world may write — may only switch a
//! unit off, and the loop honours that only for a unit whose manifest says switching it off
//! tightens what the body faces ([`UnitSwitch::from_world`]): the configuration keeps the ids as
//! plain strings and does not know which units those are.
//!
//! `[plugins.<id>]` names a third-party plugin: a program started with its own reach, so every
//! `plugins.*` key is honoured only from the global file. `[plugins.timeouts] hook_ms` is the
//! deadline on each hook call.
//!
//! A run's command line may ask for another composition (`--units`), parsed here into a
//! [`Composition`]: a list of on and off changes the loop applies over the configured one.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use mikata_races::Race;

use crate::{Config, Layer, Origin};

/// What a plugin may be granted, weakest first: see events, add notes and preface text, veto a
/// call or an answer, stop an ask, and offer its own tools.
pub const GRANTS: [&str; 5] = ["observe", "advise", "veto", "stop", "tools"];

/// The deadline on one plugin hook call, in milliseconds, unless `plugins.timeouts.hook_ms` says.
pub const HOOK_MS: u64 = 2_000;

/// The longest `plugins.timeouts.hook_ms` may be: a hook runs inside a turn, and a minute is
/// already one the body waits through.
pub const HOOK_MS_CEILING: u64 = 60_000;

/// The prefix of a plugin's unit id: `plugin:no-force-push`.
pub const PLUGIN_PREFIX: &str = "plugin:";

/// The table name under `plugins` that holds the deadlines, which no plugin may take as its id.
const TIMEOUTS: &str = "timeouts";

/// Whether `id` is a lawful native unit id: a lower-case letter first, then lower-case letters,
/// digits, `_` and `-`, at most 32 characters (`memory`, `what_changed`).
#[must_use]
pub fn lawful_unit_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id.starts_with(|c: char| c.is_ascii_lowercase())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Whether `id` is a lawful plugin id: as an MCP server's name, lower-case letters, digits and
/// dashes, at most 24, and not `timeouts`, which names the deadlines' table.
#[must_use]
pub fn lawful_plugin_id(id: &str) -> bool {
    id != TIMEOUTS && mikata_tools::mcp::lawful_server_name(id)
}

/// One `[units]` switch as the merged configuration holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSwitch {
    /// The unit's id.
    pub id: String,
    /// Whether it is switched on.
    pub on: bool,
    /// The file that set it.
    pub origin: Origin,
}

impl UnitSwitch {
    /// Whether a world's file set it (tracked or local): such a switch is always off, since a
    /// world's file switching a unit on is a hole, and the loop honours it only for a unit whose
    /// manifest says switching it off tightens.
    #[must_use]
    pub fn from_world(&self) -> bool {
        matches!(self.origin.layer, Layer::Tracked | Layer::Local)
    }
}

/// A third-party plugin as the human's global file names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plugin {
    /// Its id: the table's name, `no-force-push`.
    pub id: String,
    /// The program started.
    pub command: String,
    /// Its arguments.
    pub args: Vec<String>,
    /// Environment variables passed to it by name, as for an MCP server.
    pub env: Vec<String>,
    /// What the human grants it, from [`GRANTS`], in that order and each once; `observe` alone
    /// when the file names none. A plugin's own ask is cut down to these.
    pub grants: Vec<String>,
    /// The bodies whose sessions it joins, as for an MCP server: none when none is named.
    pub races: Vec<Race>,
    /// Whether it is loaded: on unless the file says `enabled = false`.
    pub enabled: bool,
}

impl Plugin {
    /// Its unit id in a composition: `plugin:<id>`.
    #[must_use]
    pub fn unit_id(&self) -> String {
        format!("{PLUGIN_PREFIX}{}", self.id)
    }

    /// What the plugin host starts: the program, its arguments, passed variables and races, and
    /// the grants as the host reads them.
    #[must_use]
    pub fn declared(&self) -> mikata_tools::plugin::Declared {
        mikata_tools::plugin::Declared {
            id: self.id.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            grants: self
                .grants
                .iter()
                .filter_map(|g| mikata_tools::plugin::grant_named(g))
                .collect(),
            races: self.races.clone(),
        }
    }
}

/// Every plugin the human's global file names, and the deadline on their hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plugins {
    /// The plugins, by id.
    pub plugins: Vec<Plugin>,
    /// The deadline on one hook call, in milliseconds.
    pub hook_ms: u64,
}

impl Config {
    /// The profiles the human's global file names, each with its list as written, by name.
    #[must_use]
    pub fn profiles(&self) -> Vec<(String, String)> {
        self.leaves
            .iter()
            .filter_map(|(key, leaf)| {
                let name = key.strip_prefix("profiles.")?.strip_suffix(".units")?;
                Some((name.to_owned(), leaf.value.as_str()?.to_owned()))
            })
            .collect()
    }

    /// The composition the human's global file names `name` (`[profiles.<name>] units = "…"`),
    /// or `None` when it names none: a mix of units for a kind of work, asked for with
    /// `--profile <name>`.
    #[must_use]
    pub fn profile(&self, name: &str) -> Option<Composition> {
        let leaf = self.leaves.get(&format!("profiles.{name}.units"))?;
        Composition::parse(leaf.value.as_str()?).ok()
    }

    /// The `[units]` switches every layer set, by id, each with the file that set it last. A unit
    /// not listed is on (the default composition is every unit on).
    #[must_use]
    pub fn units(&self) -> Vec<UnitSwitch> {
        self.leaves
            .iter()
            .filter_map(|(k, l)| {
                let id = k.strip_prefix("units.")?;
                Some(UnitSwitch {
                    id: id.to_owned(),
                    on: l.value.as_bool()?,
                    origin: l.origin.clone(),
                })
            })
            .collect()
    }

    /// The switch for one unit, when a layer set one.
    #[must_use]
    pub fn unit(&self, id: &str) -> Option<UnitSwitch> {
        self.units().into_iter().find(|s| s.id == id)
    }

    /// The plugins the human's global file names, and the hook deadline.
    ///
    /// # Errors
    ///
    /// A plugin with no command or an unknown race (the ids, the grants and the deadline are
    /// checked when the configuration loads).
    pub fn plugins(&self) -> Result<Plugins, String> {
        let list = |key: &str| -> Vec<String> {
            self.get(key)
                .map(|l| crate::strings(&l.value).map(str::to_owned).collect())
                .unwrap_or_default()
        };
        let ids: BTreeSet<&str> = self
            .leaves
            .keys()
            .filter_map(|k| k.strip_prefix("plugins."))
            .filter_map(|k| k.split_once('.').map(|(id, _)| id))
            .filter(|id| *id != TIMEOUTS)
            .collect();
        let mut plugins = Vec::new();
        for id in ids {
            let at = |f: &str| format!("plugins.{id}.{f}");
            let Some(command) = self
                .get(&at("command"))
                .and_then(|l| l.value.as_str())
                .filter(|c| !c.trim().is_empty())
            else {
                return Err(format!("plugins.{id}: no command"));
            };
            let races = list(&at("races"))
                .iter()
                .map(|r| {
                    // `mikata`: light-coding's one body, whose race inside is the Summoner's.
                    let r = if r == "mikata" {
                        "summoner"
                    } else {
                        r.as_str()
                    };
                    Race::from_name(r)
                        .ok_or_else(|| format!("plugins.{id}.races: `{r}` is no race"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let named = list(&at("grants"));
            let grants: Vec<String> = if named.is_empty() {
                vec![GRANTS[0].to_owned()]
            } else {
                GRANTS
                    .iter()
                    .filter(|g| named.iter().any(|n| n == *g))
                    .map(|g| (*g).to_owned())
                    .collect()
            };
            plugins.push(Plugin {
                id: id.to_owned(),
                command: command.to_owned(),
                args: list(&at("args")),
                env: list(&at("env")),
                grants,
                races,
                enabled: self
                    .get(&at("enabled"))
                    .and_then(|l| l.value.as_bool())
                    .unwrap_or(true),
            });
        }
        Ok(Plugins {
            plugins,
            hook_ms: self.int("plugins.timeouts.hook_ms").unwrap_or(HOOK_MS),
        })
    }
}

/// Why a `units.*` or `plugins.*` key from a world's file is not taken, when it is not.
pub(crate) fn refused_from_world(
    key: &str,
    value: &toml::Value,
    layer: Layer,
) -> Option<&'static str> {
    if !matches!(layer, Layer::Tracked | Layer::Local) {
        return None;
    }
    if key.starts_with("plugins.") {
        return Some(
            "a plugin is a program started with its own reach, and its grants say what it may do to a session, so it is honoured only from your global file",
        );
    }
    (key.starts_with("units.") && value.as_bool() == Some(true)).then_some(
        "a world's file may only switch a unit off; switching one on is honoured only from your global file",
    )
}

/// A `units.*` or `plugins.*` value checked when the configuration loads: a lawful unit id, a
/// lawful plugin id, known grants, a deadline within bounds.
/// Whether `key` is one [`check`] reads: `units.*`, `plugins.*` or `profiles.*`.
pub(crate) fn owns(key: &str) -> bool {
    ["units.", "plugins.", "profiles."]
        .iter()
        .any(|p| key.starts_with(p))
}

pub(crate) fn check(key: &str, value: &toml::Value, at: &Origin) -> Result<(), String> {
    if key.starts_with("profiles.") {
        return match value.as_str() {
            Some(text) => Composition::parse(text)
                .map(|_| ())
                .map_err(|e| format!("{at}: {key}: {e}")),
            None => Ok(()),
        };
    }
    if let Some(id) = key.strip_prefix("units.") {
        if !lawful_unit_id(id) {
            return Err(format!(
                "{at}: units.{id}: a unit's id is a lower-case letter, then lower-case letters, digits, `_` and `-`, at most 32; a plugin is switched in [plugins.<id>], not here"
            ));
        }
        return Ok(());
    }
    if key == "plugins.timeouts.hook_ms" {
        return if value
            .as_integer()
            .and_then(|n| u64::try_from(n).ok())
            .is_some_and(|n| (1..=HOOK_MS_CEILING).contains(&n))
        {
            Ok(())
        } else {
            Err(format!(
                "{at}: plugins.timeouts.hook_ms is a count of milliseconds from 1 to {HOOK_MS_CEILING}"
            ))
        };
    }
    let Some(rest) = key.strip_prefix("plugins.") else {
        return Ok(());
    };
    let (id, field) = rest.split_once('.').unwrap_or((rest, ""));
    if !lawful_plugin_id(id) {
        return Err(format!(
            "{at}: plugins.{id}: a plugin's id is lower-case letters, digits and dashes, at most 24, and not `timeouts`"
        ));
    }
    if field == "grants" {
        for g in crate::strings(value) {
            if !GRANTS.contains(&g) {
                return Err(format!(
                    "{at}: plugins.{id}.grants: no grant `{g}`; the grants are {}",
                    GRANTS.join(", ")
                ));
            }
        }
    }
    Ok(())
}

/// One change a run's command line asks of the composition: a unit switched on or off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitChange {
    /// The unit's id: a native unit's (`memory`), or a plugin's (`plugin:no-force-push`).
    pub id: String,
    /// Whether it is switched on.
    pub on: bool,
}

impl UnitChange {
    /// The plugin's id, when the change names a plugin.
    #[must_use]
    pub fn plugin(&self) -> Option<&str> {
        self.id.strip_prefix(PLUGIN_PREFIX)
    }
}

impl fmt::Display for UnitChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", if self.on { '+' } else { '-' }, self.id)
    }
}

/// The composition a run's command line asks for (`--units "-memory,+plugin:no-force-push"`):
/// changes over the configured composition, in the order given, each unit named once. Empty, it
/// asks for nothing: the configured composition runs. It is the human's command line alone; no
/// configuration a world holds sets it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composition {
    changes: Vec<UnitChange>,
}

impl Composition {
    /// Read a `--units` value: items split by commas, each `+<id>` (on) or `-<id>` (off), where
    /// `<id>` is a native unit's id or `plugin:<plugin id>`. Blanks around an item are dropped.
    ///
    /// # Errors
    ///
    /// An empty item, an item with no sign, an unlawful id, or a unit named twice, each said for
    /// the human.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(Self::default());
        }
        let mut changes: Vec<UnitChange> = Vec::new();
        for item in text.split(',').map(str::trim) {
            let (on, id) = match item.split_at_checked(1) {
                Some(("+", id)) => (true, id.trim()),
                Some(("-", id)) => (false, id.trim()),
                _ if item.is_empty() => {
                    return Err("--units: an empty item between commas".into());
                }
                _ => {
                    return Err(format!(
                        "--units: `{item}`: each unit is +<id> to switch it on or -<id> to switch it off"
                    ));
                }
            };
            let lawful = match id.strip_prefix(PLUGIN_PREFIX) {
                Some(plugin) => lawful_plugin_id(plugin),
                None => lawful_unit_id(id),
            };
            if !lawful {
                return Err(format!(
                    "--units: `{item}`: `{id}` is no unit's id; a native unit's is lower-case letters, digits, `_` and `-` (memory, what_changed), a plugin's is plugin:<id> (plugin:no-force-push)"
                ));
            }
            if changes.iter().any(|c| c.id == id) {
                return Err(format!("--units: `{id}` is named twice"));
            }
            changes.push(UnitChange {
                id: id.to_owned(),
                on,
            });
        }
        Ok(Self { changes })
    }

    /// The changes, in the order given.
    #[must_use]
    pub fn changes(&self) -> &[UnitChange] {
        &self.changes
    }

    /// Whether it asks for nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

impl FromStr for Composition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Composition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, c) in self.changes.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}
