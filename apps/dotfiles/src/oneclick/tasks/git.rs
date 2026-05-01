use crate::config::Config;
use crate::error::Result;
use crate::init::init_git;
use crate::settings::Settings;

use super::super::confirm::run_or_print;

pub(crate) fn run(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    run_or_print(dry_run, "init git", || {
        init_git(settings, config, assume_yes, force)
    })
}
