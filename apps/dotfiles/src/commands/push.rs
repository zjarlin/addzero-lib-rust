use crate::config::ConfigStore;
use crate::error::Result;
use crate::git_sync::GitSync;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, message: String) -> Result<()> {
    let config = store.load_or_init(settings)?;
    GitSync::new(settings, &config).commit_and_push(&message)
}
