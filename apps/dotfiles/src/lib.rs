pub mod cli;
pub mod commands;
pub mod config;
pub mod dotfile_links;
pub mod error;
pub mod git_sync;
pub mod init;
pub mod oneclick;
pub mod package_manager;
pub mod platform;
pub mod settings;
pub mod status;

use crate::error::Result;

pub fn run() -> Result<()> {
    commands::run()
}

pub fn run_with_cli(cli: cli::Cli) -> Result<()> {
    commands::run_with_cli(cli)
}
