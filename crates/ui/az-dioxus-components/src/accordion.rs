//! Accordion primitives for dense workbench panels.

use dioxus::prelude::*;

use crate::class_name::compose_class;
use crate::component_style::component_style;

/// Renders a `<details>` accordion.
#[allow(non_snake_case)]
#[component]
pub fn Accordion(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] class: String,
    #[props(default, into)] summary_class: String,
    #[props(default, into)] body_class: String,
    #[props(default)] open: bool,
) -> Element {
    let root_class = compose_class("accordion lowcode-accordion", &class, &[]);
    let summary_class = compose_class(
        "accordion__summary lowcode-accordion__summary",
        &summary_class,
        &[],
    );
    let body_class = compose_class("accordion__body lowcode-accordion__body", &body_class, &[]);

    rsx! {
        {component_style()}
        details { class: root_class, open: open,
            summary { class: summary_class, "{title}" }
            div { class: body_class, {children} }
        }
    }
}
