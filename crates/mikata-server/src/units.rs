//! The composition over the client protocol: `units/list` names the units a session runs and its
//! digest, `units/change` asks for one change, applied at the unit's next safe point.
//!
//! The server only reads and checks the request here. The loop owns the composition: until it
//! hands the server a door to it, both methods answer that the composition is not wired.

use std::collections::VecDeque;

use mikata_loop::config::{PLUGIN_PREFIX, lawful_plugin_id, lawful_unit_id};
use serde_json::{Value, json};

/// The JSON-RPC code of an answer while the loop has not wired the composition to the server.
pub const NOT_WIRED: i64 = -32004;

/// One change the human asked of the composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Switch a unit on: a native unit back on, a plugin loaded.
    On(String),
    /// Switch a unit off: a native unit off, a plugin unloaded.
    Off(String),
    /// Put the second unit in the place of the first.
    Swap(String, String),
}

impl Change {
    /// Read `units/change`'s params: `{"op": "on" | "off", "id": <unit>}` or
    /// `{"op": "swap", "id": <unit>, "with": <unit>}`, where a unit is a native unit's id
    /// (`memory`) or `plugin:<id>`.
    ///
    /// # Errors
    ///
    /// No `op`, an unknown one, a missing or unlawful id, or a swap of a unit with itself, said
    /// for the client.
    pub fn read(params: &Value) -> Result<Self, String> {
        let id = |key: &str| -> Result<String, String> {
            let Some(id) = params[key].as_str().map(str::trim) else {
                return Err(format!("units/change needs `{key}`"));
            };
            let lawful = match id.strip_prefix(PLUGIN_PREFIX) {
                Some(plugin) => lawful_plugin_id(plugin),
                None => lawful_unit_id(id),
            };
            if lawful {
                Ok(id.to_owned())
            } else {
                Err(format!(
                    "units/change: `{}` is no unit's id; a native unit's is lower-case letters, digits, `_` and `-`, a plugin's is plugin:<id>",
                    id.chars().take(80).collect::<String>()
                ))
            }
        };
        match params["op"].as_str() {
            Some("on") => Ok(Change::On(id("id")?)),
            Some("off") => Ok(Change::Off(id("id")?)),
            Some("swap") => {
                let (old, new) = (id("id")?, id("with")?);
                if old == new {
                    return Err(format!("units/change: `{old}` cannot take its own place"));
                }
                Ok(Change::Swap(old, new))
            }
            _ => Err("units/change needs `op`: `on`, `off` or `swap`".into()),
        }
    }
}

/// The composition as the server holds it for the client.
#[derive(Debug, Default)]
pub struct Units {
    /// Whether the loop wired it: until then both methods answer [`NOT_WIRED`].
    // TODO(lane D): the loop sets this when the session's registry is made.
    pub wired: bool,
    /// The units as the loop last listed them, `(id, version, state)`, in resolved order.
    // TODO(lane D): the loop writes this and `digest` after each reconcile.
    pub listed: Vec<(String, String, String)>,
    /// The composition's digest, as the loop last listed it.
    pub digest: String,
    /// The changes asked and not yet taken by the loop, in order.
    // TODO(lane D): the loop drains this at each safe point and journals each row.
    pub asked: VecDeque<Change>,
}

impl Units {
    /// What `units/list` answers, or why it cannot.
    ///
    /// # Errors
    ///
    /// The composition is not wired: `(code, message)`.
    pub fn list(&self) -> Result<Value, (i64, String)> {
        if !self.wired {
            return Err(not_wired());
        }
        let units: Vec<Value> = self
            .listed
            .iter()
            .map(|(id, version, state)| json!({"id": id, "version": version, "state": state}))
            .collect();
        Ok(json!({"units": units, "digest": self.digest, "pending": self.asked.len()}))
    }

    /// What `units/change` answers to `params`, the change queued for the loop, or why not.
    ///
    /// # Errors
    ///
    /// Params that are no change (`-32602`), or a composition not wired: `(code, message)`.
    pub fn change(&mut self, params: &Value) -> Result<Value, (i64, String)> {
        let change = Change::read(params).map_err(|e| (-32602, e))?;
        if !self.wired {
            return Err(not_wired());
        }
        self.asked.push_back(change);
        Ok(json!({"queued": true, "pending": self.asked.len()}))
    }
}

fn not_wired() -> (i64, String) {
    (
        NOT_WIRED,
        "the session's composition is not wired to the server yet".into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_is_read_from_its_params() {
        assert_eq!(
            Change::read(&json!({"op": "off", "id": "memory"})),
            Ok(Change::Off("memory".into()))
        );
        assert_eq!(
            Change::read(&json!({"op": "on", "id": "plugin:no-force-push"})),
            Ok(Change::On("plugin:no-force-push".into()))
        );
        assert_eq!(
            Change::read(&json!({"op": "swap", "id": "plugin:a", "with": "plugin:b"})),
            Ok(Change::Swap("plugin:a".into(), "plugin:b".into()))
        );
    }

    #[test]
    fn a_bad_change_says_why() {
        for (params, says) in [
            (json!({}), "needs `op`"),
            (json!({"op": "load", "id": "memory"}), "needs `op`"),
            (json!({"op": "on"}), "needs `id`"),
            (json!({"op": "on", "id": "Memory"}), "no unit's id"),
            (json!({"op": "on", "id": "plugin:timeouts"}), "no unit's id"),
            (json!({"op": "swap", "id": "plugin:a"}), "needs `with`"),
            (
                json!({"op": "swap", "id": "plugin:a", "with": "plugin:a"}),
                "its own place",
            ),
        ] {
            let err = Change::read(&params).unwrap_err();
            assert!(err.contains(says), "{params}: {err}");
        }
    }

    #[test]
    fn an_unwired_composition_says_so_after_the_params_are_checked() {
        let mut units = Units::default();
        assert_eq!(units.list().unwrap_err().0, NOT_WIRED);
        assert_eq!(units.change(&json!({"op": "on"})).unwrap_err().0, -32602);
        assert_eq!(
            units
                .change(&json!({"op": "off", "id": "memory"}))
                .unwrap_err()
                .0,
            NOT_WIRED
        );
        assert!(units.asked.is_empty());
    }

    #[test]
    fn a_wired_composition_lists_and_queues() {
        let mut units = Units {
            wired: true,
            listed: vec![("memory".into(), "1".into(), "active".into())],
            digest: "d".into(),
            ..Units::default()
        };
        assert_eq!(
            units.change(&json!({"op": "off", "id": "memory"})).unwrap(),
            json!({"queued": true, "pending": 1})
        );
        assert_eq!(
            units.list().unwrap(),
            json!({"units": [{"id": "memory", "version": "1", "state": "active"}], "digest": "d", "pending": 1})
        );
        assert_eq!(units.asked.front(), Some(&Change::Off("memory".into())));
    }
}
