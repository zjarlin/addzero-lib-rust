use crate::config::ConfigStore;
use crate::error::Result;
use crate::package_manager::PackageManager;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, keyword: String) -> Result<()> {
    let config = store.load_or_init(settings)?;
    let manager = PackageManager::from_config(
        settings.platform,
        config.current_platform_config(settings.platform),
    )?;

    for package in manager.search(&keyword)? {
        println!("{package}");
    }
    Ok(())
}
