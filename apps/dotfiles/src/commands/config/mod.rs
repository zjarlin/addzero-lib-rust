mod cat;
mod set_cloud_url;
mod set_package_manager;
mod set_sync_dir;

use crate::cli::ConfigCommand;
use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Cat => cat::run(settings, store),
        ConfigCommand::SetCloudUrl { url } => set_cloud_url::run(settings, store, url),
        ConfigCommand::SetSyncDir { dir } => set_sync_dir::run(settings, store, dir),
        ConfigCommand::SetPackageManager { name } => {
            set_package_manager::run(settings, store, name)
        }
    }
}
