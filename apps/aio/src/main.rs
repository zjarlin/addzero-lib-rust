use clap::Parser;

use aio::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve | Command::Status => {
            println!("aio backend API server starting...");
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime
                .block_on(aio::server::run_api_server())
                .expect("run api server");
        }
    }
}
