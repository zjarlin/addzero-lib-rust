use clap::Parser;

use aio::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Migrate => {
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime
                .block_on(aio::server::run_migrations())
                .expect("run migrations");
        }
        Command::Status => {
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime
                .block_on(aio::server::print_migration_status())
                .expect("print migration status");
        }
        Command::Serve => {
            println!("aio backend API server starting...");
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime
                .block_on(aio::server::run_api_server())
                .expect("run api server");
        }
    }
}
