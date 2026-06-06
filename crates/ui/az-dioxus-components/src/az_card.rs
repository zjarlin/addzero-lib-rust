use dioxus::prelude::*;

use crate::util::class_name::compose_class;

/// 渲染遵循 `az-card` class 契约的轻量卡片外壳。
#[allow(non_snake_case)]
#[component]
pub fn AzCard(children: Element, #[props(default, into)] class: String) -> Element {
    let card_class = compose_class("az-card", &class, &[]);

    rsx! {
        article { class: card_class,
            div { class: "az-card__body", {children} }
        }
    }
}
