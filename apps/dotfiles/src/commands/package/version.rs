use crate::config::ConfigStore;
use crate::error::Result;
use crate::package_manager::PackageManager;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, package: String) -> Result<()> {
    let config = store.load_or_init(settings)?;
    let manager = PackageManager::from_config(
        settings.platform,
        config.current_platform_config(settings.platform),
    )?;

    match manager.version(&package)? {
        Some(version) => println!("{version}"),
        None => println!("{package} 未安装或无法读取版本"),
    }
    Ok(())
}
