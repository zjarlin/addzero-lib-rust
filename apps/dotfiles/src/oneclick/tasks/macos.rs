use crate::error::Result;
use crate::init::init_macos;
use crate::settings::Settings;

use super::super::confirm::run_or_print;

pub(crate) fn run(settings: &Settings, assume_yes: bool, dry_run: bool) -> Result<()> {
    run_or_print(dry_run, "macOS defaults optimizations", || {
        init_macos(settings, assume_yes)
    })
}
