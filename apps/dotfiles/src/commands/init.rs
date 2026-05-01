use crate::cli::InitCommand;
use crate::config::ConfigStore;
use crate::dotfile_links::{LinkApplyMode, apply_configured_links};
use crate::error::Result;
use crate::git_sync::GitSync;
use crate::init::{init_git, init_macos, init_node, init_packages};
use crate::settings::Settings;
use crate::status::StatusStore;

pub(crate) fn run(settings: &Settings, store: &ConfigStore, command: InitCommand) -> Result<()> {
    settings.ensure_dirs()?;
    let mut config = store.load_or_init(settings)?;
    let status_store = StatusStore::new(settings.status_file());
    status_store.ensure_dir()?;

    init_git(settings, &config, command.yes, command.force)?;
    init_node(settings, &config, command.yes, command.force)?;

    if command.install_packages {
        let packages = config
            .current_platform_config(settings.platform)
            .default_packages
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        init_packages(settings, &config, &packages, command.yes)?;
    }

    if command.sync {
        GitSync::new(settings, &config).pull(command.replace_mismatch)?;
        config = store.load_or_init(settings)?;
        apply_configured_links(
            settings,
            &config,
            LinkApplyMode::from_force(command.force_links),
        )?;
    }

    if command.macos_optimize {
        init_macos(settings, command.yes)?;
    }

    Ok(())
}
