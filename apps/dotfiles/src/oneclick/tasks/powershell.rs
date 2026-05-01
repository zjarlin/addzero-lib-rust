use crate::config::PlatformType;
use crate::error::Result;

use super::super::confirm::confirm_and_run;
use super::windows_optimizations::WINDOWS_OPTIMIZATIONS;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    if PlatformType::current() != PlatformType::Windows {
        println!("当前不是 Windows，跳过 PowerShell 系统优化");
        return Ok(());
    }
    let command = WINDOWS_OPTIMIZATIONS.join("; ");
    confirm_and_run(
        assume_yes,
        dry_run,
        "执行 Windows PowerShell 系统优化",
        &command,
    )
}
