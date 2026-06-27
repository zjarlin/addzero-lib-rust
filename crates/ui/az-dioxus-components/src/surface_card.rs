//! 遵循 `surface-card` class 契约的卡片基础组件。

use dioxus::prelude::*;

use crate::class_name::compose_class;
use crate::component_style::component_style;

/// 渲染遵循 `surface-card` class 契约的轻量卡片外壳。
#[allow(non_snake_case)]
#[component]
pub fn SurfaceCard(children: Element, #[props(default, into)] class: String) -> Element {
    let card_class = compose_class("surface-card", &class, &[]);

    rsx! {
        {component_style()}
        article { class: card_class,
            div { class: "surface-card__body", {children} }
        }
    }
}
