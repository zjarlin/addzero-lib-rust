use std::path::PathBuf;

use clap::Parser;

mod add_dotfiles_command;
mod command;
mod config_command;
mod init_command;
mod oneclick_command;
mod package_command;
mod package_subcommand;
mod remove_dotfiles_command;

pub use add_dotfiles_command::AddDotfilesCommand;
pub use command::Command;
pub use config_command::ConfigCommand;
pub use init_command::InitCommand;
pub use oneclick_command::{OneClickCommand, OneClickTask};
pub use package_command::PackageCommand;
pub use package_subcommand::PackageSubcommand;
pub use remove_dotfiles_command::RemoveDotfilesCommand;

#[derive(Debug, Parser)]
#[command(name = "dotfiles")]
#[command(about = "Rust port of the Addzero dotfiles CLI", long_about = None)]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn cat_config_legacy_command_parses() {
        let cli = Cli::parse_from(["dotfiles", "cat-config"]);

        assert!(matches!(cli.command, Some(Command::CatConfig)));
    }

    #[test]
    fn rm_dotfiles_alias_parses_as_remove_dotfiles() {
        let cli = Cli::parse_from(["dotfiles", "rm-dotfiles", ".zshrc", "--no-push"]);

        assert!(matches!(
            cli.command,
            Some(Command::RemoveDotfiles(RemoveDotfilesCommand {
                no_push: true,
                ..
            }))
        ));
    }

    #[test]
    fn task_alias_parses_oneclick_dry_run() {
        let cli = Cli::parse_from(["dotfiles", "task", "--dry-run", "docker"]);

        assert!(matches!(
            cli.command,
            Some(Command::Oneclick(OneClickCommand {
                dry_run: true,
                task: OneClickTask::Docker,
                ..
            }))
        ));
    }
}
