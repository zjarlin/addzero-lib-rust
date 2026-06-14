#![forbid(unsafe_code)]

automod::dir!(pub "src");

use app::App;
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};

#[cfg(target_os = "macos")]
use dioxus::desktop::tao::{dpi::LogicalPosition, platform::macos::WindowBuilderExtMacOS};

fn main() {
    az_aio_plugin_bundled::api::ensure_linked();

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(app_window_builder()))
        .launch(App);
}

fn app_window_builder() -> WindowBuilder {
    let window = WindowBuilder::new()
        .with_title("AZ AIO")
        .with_window_icon(Some(app_icon()))
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

/// 生成应用图标（32×32 RGBA）。
///
/// 深色圆角方块 + 中心浅色竖线，简约技术风格。
fn app_icon() -> Icon {
    const SIZE: u32 = 32;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    let half = (SIZE / 2) as i32;
    let radius = (SIZE / 2 - 2) as f64;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as i32 - half;
            let dy = y as i32 - half;
            let dist = ((dx * dx + dy * dy) as f64).sqrt();

            if dist > radius {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let edge = 1.0 - (dist / radius).clamp(0.0, 1.0);
                let base: u8 = (20.0 + edge * 15.0) as u8;
                let r: u8 = base;
                let g: u8 = base;
                let b: u8 = (35.0 + edge * 30.0) as u8;

                let center_stripe = dx >= -2 && dx <= 2;
                let highlight: u8 = if center_stripe {
                    (80u16 + (edge * 40.0) as u16).min(255) as u8
                } else {
                    0
                };

                rgba.push(r.saturating_add(highlight / 3));
                rgba.push(g.saturating_add(highlight / 2));
                rgba.push(b.saturating_add(highlight));
                rgba.push(255);
            }
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).expect("icon rgba data should be valid")
}
