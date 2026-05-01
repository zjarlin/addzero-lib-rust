use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, dir: String) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    config.sync_dir = dir;
    store.save(&config)
}
