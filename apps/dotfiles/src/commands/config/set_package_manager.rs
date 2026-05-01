use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, name: String) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    config
        .current_platform_config_mut(settings.platform)
        .package_manager = Some(name);
    store.save(&config)
}
