//! Button and link primitives using the `az-button` class contract.

use dioxus::prelude::*;

use crate::util::class_name::compose_class;

/// Visual intent for [`AzButton`] and [`AzButtonLink`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AzButtonTone {
    /// Neutral toolbar action.
    #[default]
    Neutral,
    /// Primary action.
    Primary,
    /// Destructive action.
    Danger,
}

impl AzButtonTone {
    fn modifier(self) -> (&'static str, bool) {
        match self {
            Self::Neutral => ("", false),
            Self::Primary => ("az-button--primary toolbar-button--primary", true),
            Self::Danger => ("az-button--danger toolbar-button--danger", true),
        }
    }
}

/// Renders a styled `<button>`.
#[allow(non_snake_case)]
#[component]
pub fn AzButton(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] tone: AzButtonTone,
    #[props(default = "button".to_string(), into)] button_type: String,
    #[props(default)] disabled: bool,
) -> Element {
    let class = button_class(&class, tone);

    rsx! {
        button { class: class, r#type: button_type, disabled: disabled, {children} }
    }
}

/// Renders a styled link that visually matches [`AzButton`].
#[allow(non_snake_case)]
#[component]
pub fn AzButtonLink(
    children: Element,
    #[props(into)] href: String,
    #[props(default, into)] class: String,
    #[props(default)] tone: AzButtonTone,
) -> Element {
    let class = button_class(&class, tone);

    rsx! {
        a { class: class, href: href, {children} }
    }
}

fn button_class(extra: &str, tone: AzButtonTone) -> String {
    let modifier = tone.modifier();
    compose_class("az-button toolbar-button", extra, &[modifier])
}
