use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_init(settings)?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
