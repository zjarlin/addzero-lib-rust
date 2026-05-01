use crate::cli::{OneClickCommand, OneClickTask};
use crate::config::{Config, ConfigStore};
use crate::error::Result;
use crate::oneclick::{print_one_click_tasks, run_one_click_task};
use crate::settings::Settings;

pub(crate) fn run(
    settings: &Settings,
    store: &ConfigStore,
    command: OneClickCommand,
) -> Result<()> {
    if matches!(command.task, OneClickTask::List) {
        print_one_click_tasks();
        return Ok(());
    }

    if command.dry_run && !settings.config_file.exists() {
        let config = Config::default_for_settings(settings);
        return run_one_click_task(
            settings,
            &config,
            command.task,
            command.yes,
            command.dry_run,
            command.force,
        );
    }

    settings.ensure_dirs()?;
    let config = store.load_or_init(settings)?;
    run_one_click_task(
        settings,
        &config,
        command.task,
        command.yes,
        command.dry_run,
        command.force,
    )
}
