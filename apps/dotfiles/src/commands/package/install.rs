use crate::config::ConfigStore;
use crate::error::Result;
use crate::init::init_packages;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, packages: Vec<String>) -> Result<()> {
    let config = store.load_or_init(settings)?;
    let packages = if packages.is_empty() {
        config
            .current_platform_config(settings.platform)
            .default_packages
            .iter()
            .cloned()
            .collect()
    } else {
        packages
    };
    init_packages(settings, &config, &packages, true)
}
