use clap::Parser;

use msc_aio::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Desktop) {
        Command::Desktop | Command::Status => {
            println!("msc-aio backend API server starting...");
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime
                .block_on(msc_aio::server::run_api_server())
                .expect("run api server");
        }
    }
}
