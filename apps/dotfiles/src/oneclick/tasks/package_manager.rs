use crate::config::Config;
use crate::error::Result;
use crate::init::init_packages;
use crate::settings::Settings;

use super::super::confirm::run_or_print;

pub(crate) fn run(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    dry_run: bool,
) -> Result<()> {
    let packages = config
        .current_platform_config(settings.platform)
        .default_packages
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    run_or_print(dry_run, "install configured packages", || {
        init_packages(settings, config, &packages, assume_yes)
    })
}
