#![forbid(unsafe_code)]

mod api;
mod model;
mod pages;

use adui_dioxus::theme;
use dioxus::prelude::*;
use pages::{ModelEditor, ScreenView};

fn main() {
    console_log::init_with_level(log::Level::Info).ok();
    dioxus::launch(App);
}

#[derive(Clone, PartialEq)]
enum AppPage {
    Models,
    Screen(String),
}

#[component]
fn App() -> Element {
    let mut page = use_signal(|| AppPage::Models);

    rsx! {
        theme::ThemeProvider { theme: theme::Theme::light(),
            div { class: "az-lowcode-shell", style: "width:100%; height:100vh;",
                match page() {
                    AppPage::Models => rsx! {
                        ModelEditor { on_view_screen: move |sid: String| page.set(AppPage::Screen(sid)) }
                    },
                    AppPage::Screen(sid) => rsx! {
                        ScreenView {
                            key: "{sid}",
                            screen_id: sid,
                            on_back: move |()| page.set(AppPage::Models),
                        }
                    },
                }
            }
        }
    }
}
