use clap::{CommandFactory, Parser};

use aio::cli::{Cli, Command};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        Cli::command().print_help()?;
        println!();
        return Ok(());
    };
    let runtime = tokio::runtime::Runtime::new()?;
    match command {
        Command::Reg(args) => {
            runtime.block_on(aio::auth_cli::run_reg_command(args))?;
        }
        Command::Login(args) => {
            runtime.block_on(aio::auth_cli::run_login_command(args))?;
        }
        Command::Logout(args) => {
            runtime.block_on(aio::auth_cli::run_logout_command(args))?;
        }
        Command::Whoami(args) => {
            runtime.block_on(aio::auth_cli::run_whoami_command(args))?;
        }
        Command::Key(command) => {
            runtime.block_on(aio::auth_cli::run_key_command(command))?;
        }
        Command::Cli(command) => {
            runtime.block_on(aio::external_cli::run_aio_cli_command(command))?;
        }
        Command::Migrate => {
            runtime.block_on(aio::server::run_migrations())?;
        }
        Command::Status => {
            runtime.block_on(aio::server::print_migration_status())?;
        }
        Command::System(system_cli) => {
            runtime.block_on(aio::system_cli::run_system_cli(system_cli))?;
        }
        Command::Drive(drive_command) => {
            runtime.block_on(az_drive_app::cli::run_drive_command(drive_command))?;
        }
        Command::Serve(args) => {
            println!("aio backend API server starting...");
            runtime.block_on(aio::server::run_api_server(aio::server::ApiServerOptions {
                bind: args.bind,
                desktop_token: args.desktop_token,
            }))?;
        }
    };
    Ok(())
}
