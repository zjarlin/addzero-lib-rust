use crate::error::Result;
use crate::init::init_homebrew;

use super::super::confirm::confirm_and_run_action;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    confirm_and_run_action(assume_yes, dry_run, "初始化 Homebrew", init_homebrew)
}
