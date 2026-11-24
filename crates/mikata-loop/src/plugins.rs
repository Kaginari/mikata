//! Third-party plugins as units: the bridge from a plugin the human declared to a
//! [`mikata_kernel::Unit`] the run can load ([`crate::units::Catalog`]). The host — the process,
//! the handshake, the deadline, the strikes — is `mikata_tools::plugin`.

/// The unit for `plugin`, started, with what the human grants it.
///
/// # Errors
///
/// The plugin cannot be started, or does not speak Mikata's hooks.
pub fn unit(
    plugin: &mikata_config::Plugin,
    _hook_ms: u64,
) -> Result<crate::units::Granted, String> {
    Err(format!(
        "the plugin `{}` cannot be started: this build hosts no plugins yet",
        plugin.id
    ))
}
