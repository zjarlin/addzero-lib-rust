use dioxus::prelude::*;

use crate::class_name::compose_class;

/// Renders a lightweight card shell with the `az-card` class contract.
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
