#![forbid(unsafe_code)]

automod::dir!("src");

use app::App;
use dioxus::desktop::{Config, WindowBuilder};

#[cfg(target_os = "macos")]
use dioxus::desktop::tao::{dpi::LogicalPosition, platform::macos::WindowBuilderExtMacOS};

fn main() {
    az_aio_plugin_bundled::ensure_linked();

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(app_window_builder()))
        .launch(App);
}

fn app_window_builder() -> WindowBuilder {
    let window = WindowBuilder::new()
        .with_title("AZ AIO")
        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1440.0, 900.0))
        .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(960.0, 640.0));

    #[cfg(target_os = "macos")]
    let window = window
        .with_title_hidden(true)
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true)
        .with_traffic_light_inset(LogicalPosition::new(18.0, 18.0));

    window
}
