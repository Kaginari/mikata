//! The composition over the client protocol: `units/list` names the units a session runs and its
//! digest, `units/change` asks for one change, applied at the unit's next safe point.
//!
//! The server reads and checks the request; the loop owns the composition and takes the change
//! at the unit's next point, through the desk it shares ([`mikata_loop::units::Desk`]).

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

/// The composition as the server reaches it for the client: the run's desk
/// ([`mikata_loop::units::Desk`]) once a session is open.
#[derive(Debug, Default)]
pub struct Units {
    /// The run's desk; none until a session is open.
    pub desk: Option<mikata_loop::units::UnitDesk>,
}

impl Units {
    /// What `units/list` answers, or why it cannot.
    ///
    /// # Errors
    ///
    /// No session is open: `(code, message)`.
    pub fn list(&self) -> Result<Value, (i64, String)> {
        let desk = self.desk.as_ref().ok_or_else(not_wired)?;
        let desk = desk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let units: Vec<Value> = desk
            .listed
            .iter()
            .map(|(id, version, state)| json!({"id": id, "version": version, "state": state}))
            .collect();
        Ok(json!({
            "units": units,
            "digest": desk.digest,
            "pending": desk.asked.len(),
            "refused": desk.refused,
        }))
    }

    /// What `units/change` answers: the change queued for the run's next point, or why not. A
    /// swap is the old unit off and the new one on, at the same point.
    ///
    /// # Errors
    ///
    /// Bad params (`-32602`), or no session open.
    pub fn change(&mut self, params: &Value) -> Result<Value, (i64, String)> {
        let change = Change::read(params).map_err(|e| (-32602, e))?;
        let desk = self.desk.as_ref().ok_or_else(not_wired)?;
        let mut desk = desk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match change {
            Change::On(id) => desk.asked.push_back((id, true)),
            Change::Off(id) => desk.asked.push_back((id, false)),
            Change::Swap(old, new) => {
                desk.asked.push_back((old, false));
                desk.asked.push_back((new, true));
            }
        }
        Ok(json!({"queued": true, "pending": desk.asked.len()}))
    }
}

fn not_wired() -> (i64, String) {
    (NOT_WIRED, "no session is open: initialize first".into())
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
    fn with_no_session_it_says_so_after_the_params_are_checked() {
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
    }

    #[test]
    fn a_wired_composition_lists_and_queues() {
        let desk = mikata_loop::units::UnitDesk::default();
        {
            let mut d = desk.lock().unwrap();
            d.listed = vec![("memory".into(), "1".into(), "active".into())];
            d.digest = "d".into();
        }
        let mut units = Units {
            desk: Some(desk.clone()),
        };
        assert_eq!(
            units
                .change(&json!({"op": "swap", "id": "plugin:a", "with": "plugin:b"}))
                .unwrap(),
            json!({"queued": true, "pending": 2})
        );
        assert_eq!(
            units.list().unwrap(),
            json!({"units": [{"id": "memory", "version": "1", "state": "active"}], "digest": "d", "pending": 2, "refused": []})
        );
        assert_eq!(
            desk.lock()
                .unwrap()
                .asked
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            [
                ("plugin:a".to_owned(), false),
                ("plugin:b".to_owned(), true)
            ]
        );
    }
}
