use crate::config::Config;
use crate::error::Result;
use crate::package_manager::PackageManager;
use crate::settings::Settings;

use super::super::confirm::confirm_and_run_action;

pub(crate) fn run(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    dry_run: bool,
) -> Result<()> {
    let manager = package_manager(settings, config)?;
    confirm_and_run_action(
        assume_yes,
        dry_run,
        &format!("安装 {}", manager.name()),
        || manager.install_self(),
    )
}

pub(crate) fn package_manager(settings: &Settings, config: &Config) -> Result<PackageManager> {
    PackageManager::from_config(
        settings.platform,
        config.current_platform_config(settings.platform),
    )
}
