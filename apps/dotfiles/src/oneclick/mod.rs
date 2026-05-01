mod confirm;
mod downloads;
mod model;
mod shell;
mod tasks;

use crate::cli::OneClickTask;
use crate::config::Config;
use crate::error::Result;
use crate::settings::Settings;

use self::model::ONE_CLICK_SPECS;

pub fn print_one_click_tasks() {
    println!("AI 可调用的一键任务:");
    for task in ONE_CLICK_SPECS {
        println!("  {:<20} {}", task.name, task.description);
    }
}

pub fn run_one_click_task(
    settings: &Settings,
    config: &Config,
    task: OneClickTask,
    assume_yes: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    match task {
        OneClickTask::List => {
            print_one_click_tasks();
            Ok(())
        }
        OneClickTask::All => run_all(settings, config, assume_yes, dry_run, force),
        OneClickTask::EnvScripts => tasks::env_scripts(settings, assume_yes, dry_run),
        OneClickTask::Git => tasks::git(settings, config, assume_yes, dry_run, force),
        OneClickTask::Node => tasks::node(settings, config, assume_yes, dry_run, force),
        OneClickTask::Jdk => tasks::jdk(settings, config, assume_yes, dry_run),
        OneClickTask::Pnpm => tasks::pnpm(assume_yes, dry_run),
        OneClickTask::Pkg => tasks::pkg(settings, config, assume_yes, dry_run),
        OneClickTask::PkgManager => tasks::package_manager(settings, config, assume_yes, dry_run),
        OneClickTask::Graalvm => tasks::graalvm(settings, assume_yes, dry_run),
        OneClickTask::FinalShell => tasks::final_shell(assume_yes, dry_run),
        OneClickTask::Idea => tasks::idea(settings, assume_yes, dry_run),
        OneClickTask::ZuluJdk => tasks::zulu_jdk(settings, assume_yes, dry_run),
        OneClickTask::Powershell => tasks::powershell(assume_yes, dry_run),
        OneClickTask::PowershellEnv => tasks::powershell_env(assume_yes, dry_run),
        OneClickTask::Quark => tasks::quark(settings, assume_yes, dry_run),
        OneClickTask::Docker => tasks::docker(assume_yes, dry_run),
        OneClickTask::Lazyvim => tasks::lazyvim(assume_yes, dry_run),
        OneClickTask::Homebrew => tasks::homebrew(assume_yes, dry_run),
        OneClickTask::Ohmyzsh => tasks::ohmyzsh(assume_yes, dry_run),
        OneClickTask::Macos => tasks::macos(settings, assume_yes, dry_run),
        OneClickTask::EnableAllSources => tasks::enable_all_sources(assume_yes, dry_run),
        OneClickTask::Keji => tasks::keji(assume_yes, dry_run),
    }
}

fn run_all(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    for task in [
        OneClickTask::EnvScripts,
        OneClickTask::Git,
        OneClickTask::Node,
        OneClickTask::Jdk,
        OneClickTask::Pnpm,
        OneClickTask::Pkg,
        OneClickTask::PkgManager,
        OneClickTask::Graalvm,
        OneClickTask::FinalShell,
        OneClickTask::Idea,
        OneClickTask::ZuluJdk,
        OneClickTask::PowershellEnv,
        OneClickTask::Powershell,
        OneClickTask::Quark,
    ] {
        run_one_click_task(settings, config, task, assume_yes, dry_run, force)?;
    }
    Ok(())
}
