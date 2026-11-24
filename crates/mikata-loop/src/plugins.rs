//! Third-party plugins as units: the bridge from a plugin the human declared to a
//! [`mikata_kernel::Unit`] the run can load ([`crate::units::Catalog`]). The host — the process,
//! the handshake, the deadline, the strikes — is [`mikata_tools::plugin`].

use std::time::Duration;

use mikata_kernel::{Activation, Event, Manifest, Reply, Unit, UnitError};
use mikata_races::Race;
use mikata_tools::plugin::Plugin;

use crate::beats::Beat;
use crate::units::{Cx, Granted};

/// Where the run starts its plugins: the world, the body's race and sandbox.
#[derive(Debug, Clone)]
pub struct Host {
    /// The world the run works in.
    pub world: std::path::PathBuf,
    /// The body's race: a plugin joins only the bodies its `races` names.
    pub race: Race,
    /// The body's sandbox, which the plugin runs in.
    pub sandbox: mikata_tools::Sandbox,
    /// The human's word that a command may run with no kernel guard, as the body has it.
    pub unguarded: bool,
}

/// A started plugin, as a unit. Dropping it — an unload, a quarantine, the run's end — sends
/// `mikata/unload` and ends its process.
pub struct PluginUnit(Plugin);

impl Unit<Cx> for PluginUnit {
    fn manifest(&self) -> &Manifest {
        self.0.manifest()
    }

    fn activate(&mut self, _act: &mut Activation<'_, Cx>) -> Result<(), UnitError> {
        Ok(())
    }

    fn on(&mut self, cx: &mut Cx, event: &Event) -> Reply {
        let (reply, strike) = self.0.on(event);
        if let Some(strike) = strike {
            cx.beats.push(Beat::Note {
                text: format!(
                    "the unit `{}` earned strike {}: {}",
                    self.0.manifest().id,
                    strike.count,
                    strike.why
                ),
            });
        }
        reply
    }

    fn failed(&self) -> Option<String> {
        self.0.failed()
    }
}

impl Drop for PluginUnit {
    fn drop(&mut self) {
        self.0.unload();
    }
}

/// The unit for `plugin`, started on `host`, with what the human grants it.
///
/// # Errors
///
/// The plugin does not join this body, cannot be started, or does not speak Mikata's hooks.
pub fn unit(
    plugin: &mikata_config::Plugin,
    hook_ms: u64,
    host: Option<&Host>,
) -> Result<Granted, String> {
    let Some(host) = host else {
        return Err(format!(
            "the plugin `{}` has no place to run: the run has not opened",
            plugin.id
        ));
    };
    if !plugin.races.contains(&host.race) {
        return Err(format!(
            "the plugin `{}` does not join a {} body: its `races` does not name it",
            plugin.id,
            host.race.name()
        ));
    }
    let declared = plugin.declared();
    let cx = mikata_tools::Context::new(host.race, &host.world, &["."])
        .with_sandbox(host.sandbox.clone())
        .with_unguarded(host.unguarded);
    let scratch = mikata_tools::mcp::state_dir(&host.world)
        .join("plugins")
        .join(&plugin.id);
    std::fs::create_dir_all(&scratch)
        .map_err(|e| format!("the plugin `{}`: its scratch: {e}", plugin.id))?;
    let started = Plugin::start(&declared, &cx, &scratch, Duration::from_millis(hook_ms))?;
    let granted = started.manifest().grants.clone();
    Ok((Box::new(PluginUnit(started)), granted))
}
