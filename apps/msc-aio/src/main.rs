use clap::Parser;

use msc_aio::cli::{Cli, Command};

include!(concat!(env!("OUT_DIR"), "/app_icon.rs"));
const APP_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app_icon.rgba"));

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

    match dioxus_desktop::tao::window::Icon::from_rgba(
        APP_ICON_RGBA.to_vec(),
        APP_ICON_WIDTH,
        APP_ICON_HEIGHT,
    ) {
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
