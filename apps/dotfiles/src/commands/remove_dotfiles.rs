use crate::cli::RemoveDotfilesCommand;
use crate::config::ConfigStore;
use crate::dotfile_links::remove_dotfiles;
use crate::error::Result;
use crate::git_sync::GitSync;
use crate::settings::Settings;

use super::status::print_current_status;

pub(crate) fn run(
    settings: &Settings,
    store: &ConfigStore,
    command: RemoveDotfilesCommand,
) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    let changed = remove_dotfiles(settings, &mut config, &command.paths, command.abs)?;
    if changed {
        store.save(&config)?;
        if !command.no_push {
            GitSync::new(settings, &config).commit_and_push("Update dotfiles")?;
        }
    }
    print_current_status(settings, &config);
    Ok(())
}
