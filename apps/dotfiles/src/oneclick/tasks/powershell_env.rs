use crate::config::PlatformType;
use crate::error::Result;

use super::super::confirm::confirm_and_run;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    if PlatformType::current() != PlatformType::Windows {
        println!("当前不是 Windows，跳过 PowerShell 环境初始化");
        return Ok(());
    }
    confirm_and_run(
        assume_yes,
        dry_run,
        "创建 PowerShell profile 并设置 RemoteSigned",
        "if (!(Test-Path $profile)) { New-Item -Path $profile -ItemType File -Force }; Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force",
    )
}
