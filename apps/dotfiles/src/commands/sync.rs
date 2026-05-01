use crate::config::ConfigStore;
use crate::dotfile_links::{LinkApplyMode, apply_configured_links};
use crate::error::Result;
use crate::git_sync::GitSync;
use crate::settings::Settings;

pub(crate) fn run(
    settings: &Settings,
    store: &ConfigStore,
    force_links: bool,
    replace_mismatch: bool,
) -> Result<()> {
    let config = store.load_or_init(settings)?;
    GitSync::new(settings, &config).pull(replace_mismatch)?;
    let config = store.load_or_init(settings)?;
    apply_configured_links(settings, &config, LinkApplyMode::from_force(force_links))
}
