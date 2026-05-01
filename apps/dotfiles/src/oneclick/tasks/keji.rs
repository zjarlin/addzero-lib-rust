use crate::config::PlatformType;
use crate::error::Result;

use super::super::confirm::confirm_and_run;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    if !PlatformType::current().is_unix() {
        println!("当前不是 Unix 平台，跳过 keji 面板");
        return Ok(());
    }
    confirm_and_run(
        assume_yes,
        dry_run,
        "运行 keji 面板安装脚本",
        "bash <(curl -sL kejilion.sh)",
    )
}
