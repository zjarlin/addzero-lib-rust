use clap::Parser;

use msc_aio::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::ServeApi) {
        Command::ServeApi => {
            let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
            runtime
                .block_on(msc_aio::server::run_api_server())
                .expect("run api server");
        }
        Command::Status => {
            println!(
                "msc-aio 已切换为 backend-first 入口：管理界面由 Next.js + Tauri 承载，当前二进制仅保留 API/CLI。"
            );
        }
    }
}
