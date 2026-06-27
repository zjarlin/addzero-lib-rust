use dioxus::prelude::*;

pub(crate) fn inline_style(id: &'static str, css: &'static str) -> Element {
    rsx! {
        style {
            "data-az-style": id,
            dangerous_inner_html: css,
        }
    }
}
