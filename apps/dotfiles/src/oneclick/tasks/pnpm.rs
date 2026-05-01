use crate::error::{Result, message};
use crate::platform::{command_exists, prompt_yes_no};

use super::super::confirm::confirm_and_run;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    if command_exists("pnpm") {
        println!("pnpm 已安装");
        return confirm_and_run(assume_yes, dry_run, "pnpm setup", "pnpm setup");
    }

    if !command_exists("npm") {
        return Err(message("未检测到 npm，无法安装 pnpm"));
    }
    if !prompt_yes_no(
        "pnpm 未安装，是否执行 npm install -g pnpm?",
        true,
        assume_yes,
    )? {
        println!("已跳过 pnpm 初始化");
        return Ok(());
    }
    confirm_and_run(
        assume_yes,
        dry_run,
        "安装并配置 pnpm",
        "npm install -g pnpm && pnpm setup",
    )
}
