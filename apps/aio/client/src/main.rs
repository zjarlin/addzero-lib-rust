#![forbid(unsafe_code)]
#![allow(non_snake_case)]

mod app;
mod bootstrap;
mod http;
mod screens;

use app::App;

fn main() {
    dioxus::LaunchBuilder::web()
        .with_cfg(dioxus_web::Config::new().rootname("aio-client-root"))
        .launch(App);
}
