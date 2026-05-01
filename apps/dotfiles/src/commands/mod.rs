mod add_dotfiles;
mod add_pkg;
mod cat_status;
mod config;
mod enable_all_sources;
mod git_config;
mod init;
mod init_docker;
mod init_homebrew;
mod init_lazyvim;
mod init_ohmyzsh;
mod kill_port;
mod oneclick;
mod package;
mod push;
mod remove_dotfiles;
mod repl;
mod rm_pkg;
mod show_port;
mod status;
mod sync;

use clap::Parser;

use crate::cli::{Cli, Command, ConfigCommand};
use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    run_with_cli(cli)
}

pub(crate) fn run_with_cli(cli: Cli) -> Result<()> {
    let settings = Settings::load(cli.config)?;
    let store = ConfigStore::new(settings.config_file.clone());
    let command = cli.command.unwrap_or(Command::Repl);

    match command {
        Command::Repl => repl::run(settings),
        Command::CatConfig => config::run(&settings, &store, ConfigCommand::Cat),
        Command::Config { command } => config::run(&settings, &store, command),
        Command::Sync {
            force_links,
            replace_mismatch,
        } => sync::run(&settings, &store, force_links, replace_mismatch),
        Command::Push { message } => push::run(&settings, &store, message),
        Command::AddDotfiles(command) => add_dotfiles::run(&settings, &store, command),
        Command::RemoveDotfiles(command) => remove_dotfiles::run(&settings, &store, command),
        Command::AddPkg(command) => add_pkg::run(&settings, &store, command),
        Command::RmPkg(command) => rm_pkg::run(&settings, &store, command),
        Command::Package { command } => package::run(&settings, &store, command),
        Command::Status => status::run(&settings, &store),
        Command::CatStatus => cat_status::run(&settings),
        Command::ShowPort { port } => show_port::run(port),
        Command::KillPort { port } => kill_port::run(port),
        Command::GitConfig { username, email } => git_config::run(username, email),
        Command::Init(command) => init::run(&settings, &store, command),
        Command::Oneclick(command) => oneclick::run(&settings, &store, command),
        Command::InitDocker => init_docker::run(),
        Command::InitLazyvim => init_lazyvim::run(),
        Command::InitHomebrew => init_homebrew::run(),
        Command::InitOhmyzsh => init_ohmyzsh::run(),
        Command::EnableAllSources => enable_all_sources::run(),
    }
}
