use crate::error::Result;
use crate::init::init_docker;

use super::super::confirm::confirm_and_run_action;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    confirm_and_run_action(
        assume_yes,
        dry_run,
        "通过 linuxmirrors.cn 脚本初始化 Docker",
        init_docker,
    )
}
