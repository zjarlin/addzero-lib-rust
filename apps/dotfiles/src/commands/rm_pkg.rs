use crate::cli::PackageCommand;
use crate::config::ConfigStore;
use crate::error::Result;
use crate::git_sync::GitSync;
use crate::settings::Settings;

use super::status::print_current_status;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, command: PackageCommand) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    let platform_config = config.current_platform_config_mut(settings.platform);
    for package in command.packages {
        if platform_config.default_packages.remove(package.trim()) {
            println!("已删除软件包: {}", package.trim());
        }
    }
    store.save(&config)?;
    if !command.no_push {
        GitSync::new(settings, &config).commit_and_push("Update packages")?;
    }
    print_current_status(settings, &config);
    Ok(())
}
