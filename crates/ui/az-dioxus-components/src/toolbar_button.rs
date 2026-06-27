//! Button and link primitives using the `toolbar-button` class contract.

use dioxus::prelude::*;

use crate::class_name::compose_class;
use crate::component_style::component_style;

/// Visual intent for [`ToolbarButton`] and [`ToolbarButtonLink`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarButtonTone {
    /// Neutral toolbar action.
    #[default]
    Neutral,
    /// Primary action.
    Primary,
    /// Destructive action.
    Danger,
}

impl ToolbarButtonTone {
    fn modifier(self) -> (&'static str, bool) {
        match self {
            Self::Neutral => ("", false),
            Self::Primary => ("toolbar-button--primary", true),
            Self::Danger => ("toolbar-button--danger", true),
        }
    }
}

/// Renders a styled `<button>`.
#[allow(non_snake_case)]
#[component]
pub fn ToolbarButton(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] tone: ToolbarButtonTone,
    #[props(default = "button".to_string(), into)] button_type: String,
    #[props(default)] disabled: bool,
) -> Element {
    let class = button_class(&class, tone);

    rsx! {
        {component_style()}
        button { class: class, r#type: button_type, disabled: disabled, {children} }
    }
}

/// Renders a styled link that visually matches [`ToolbarButton`].
#[allow(non_snake_case)]
#[component]
pub fn ToolbarButtonLink(
    children: Element,
    #[props(into)] href: String,
    #[props(default, into)] class: String,
    #[props(default)] tone: ToolbarButtonTone,
) -> Element {
    let class = button_class(&class, tone);

    rsx! {
        {component_style()}
        a { class: class, href: href, {children} }
    }
}

fn button_class(extra: &str, tone: ToolbarButtonTone) -> String {
    let modifier = tone.modifier();
    compose_class("toolbar-button", extra, &[modifier])
}
