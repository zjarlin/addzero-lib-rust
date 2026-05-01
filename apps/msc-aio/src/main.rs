use clap::Parser;

use msc_aio::cli::{Cli, Command};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Desktop) {
        Command::Desktop => {
            dioxus::launch(msc_aio::app::App);
        }
        Command::Status => {
            println!("msc-aio 已切换为 native-first 入口：默认 desktop，CLI 由 clap 承载，web 不再是默认路径。");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(msc_aio::app::App);
}
