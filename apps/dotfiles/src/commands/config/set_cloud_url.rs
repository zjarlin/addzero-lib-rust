use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, url: String) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    config.cloud_url = Some(url);
    store.save(&config)
}
