use crate::config::Config;
use crate::error::Result;
use crate::init::init_node;
use crate::settings::Settings;

use super::super::confirm::run_or_print;

pub(crate) fn run(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    run_or_print(dry_run, "init node", || {
        init_node(settings, config, assume_yes, force)
    })
}
