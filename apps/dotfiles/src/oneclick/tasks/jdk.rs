use crate::config::{Config, PlatformType};
use crate::error::Result;
use crate::platform::{command_exists, prompt_yes_no, run_shell};
use crate::settings::Settings;

use super::pkg::package_manager;

pub(crate) fn run(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    dry_run: bool,
) -> Result<()> {
    if command_exists("java") {
        let output = run_shell("java -version")?;
        let version = if output.stderr.trim().is_empty() {
            output.stdout.trim()
        } else {
            output.stderr.trim()
        };
        println!("JDK 已安装: {version}");
        return Ok(());
    }

    if !prompt_yes_no(
        "JDK 未安装，是否通过当前包管理器安装 JDK 17?",
        true,
        assume_yes,
    )? {
        println!("已跳过 JDK 初始化");
        return Ok(());
    }

    let package = match settings.platform {
        PlatformType::Windows => "Microsoft.OpenJDK.17",
        PlatformType::Macos => "openjdk@17",
        PlatformType::Linux | PlatformType::Unknown => "openjdk-17-jdk",
    };
    let manager = package_manager(settings, config)?;
    if dry_run {
        println!("[dry-run] {} install {}", manager.name(), package);
        Ok(())
    } else {
        manager.install(package)
    }
}
