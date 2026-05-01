use clap::Parser;

use msc_aio::cli::{Cli, Command};

const APP_ICON: &[u8] = include_bytes!("../assets/app-icon.png");

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Desktop) {
        Command::Desktop => {
            let cfg = desktop_config();
            dioxus::LaunchBuilder::desktop()
                .with_cfg(cfg)
                .launch(msc_aio::app::App);
        }
        Command::Status => {
            println!(
                "msc-aio 已切换为 native-first 入口：默认 desktop，CLI 由 clap 承载，web 不再是默认路径。"
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn desktop_config() -> dioxus_desktop::Config {
    let window = dioxus_desktop::WindowBuilder::new().with_title("msc-aio");
    let config = dioxus_desktop::Config::new().with_window(window);

    match dioxus_desktop::icon_from_memory(APP_ICON) {
        Ok(icon) => config.with_icon(icon),
        Err(err) => {
            eprintln!("failed to load app icon: {err}");
            config
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(msc_aio::app::App);
}
