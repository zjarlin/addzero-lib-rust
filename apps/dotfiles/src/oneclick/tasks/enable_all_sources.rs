use crate::error::Result;
use crate::init::enable_all_sources;

use super::super::confirm::confirm_and_run_action;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    confirm_and_run_action(
        assume_yes,
        dry_run,
        "macOS 开启所有来源",
        enable_all_sources,
    )
}
