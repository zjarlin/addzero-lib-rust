#![allow(non_snake_case)]

use dioxus::prelude::*;

use super::model::PageChrome;

#[derive(PartialEq, Clone, Props)]
pub(super) struct WelcomeStartProps {
    pub(super) page: PageChrome,
}

pub(super) fn WelcomeStart(props: WelcomeStartProps) -> Element {
    rsx! {
        div { class: "codex-start",
            div { class: "empty-panel empty-panel--compact",
                div { class: "empty-panel__mark", "{props.page.mark}" }
                h1 { "{props.page.title}" }
                p { "{props.page.subtitle}" }
            }
        }
    }
}
