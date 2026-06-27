//! Neobrutalism-inspired SSR primitives for Dioxus pages.
//!
//! These components provide only reusable visual structure: bold borders,
//! hard shadows, high contrast fills, compact layout blocks, and stable class
//! names for server-rendered applications.

use dioxus::prelude::*;

use crate::class_name::compose_class;
use crate::style::inline_style;

pub mod shell;
mod style;

pub use shell::{
    ContentSlot, FloatingPanelSlot, HeaderBar, IconButton, ModelButton, NavLink, PluginGroup,
    ProjectLayout, RightSlot, Shell, Sidebar, SidebarToggle, TitlebarControls, TitlebarNav,
    Workspace, WorkspaceBody,
};
use style::{NEOBRUTAL_CSS, NEOBRUTAL_STYLE_ID};

pub(crate) fn neobrutal_style() -> Element {
    inline_style(NEOBRUTAL_STYLE_ID, NEOBRUTAL_CSS)
}

/// Full-page surface with a graph-paper background.
#[allow(non_snake_case)]
#[component]
pub fn Page(children: Element, #[props(default, into)] class: String) -> Element {
    let root_class = compose_class("page", &class, &[]);

    rsx! {
        {neobrutal_style()}
        section { class: root_class, {children} }
    }
}

/// Top hero block for workbench pages.
#[allow(non_snake_case)]
#[component]
pub fn Hero(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] compact: bool,
) -> Element {
    let hero_class = compose_class("hero", &class, &[("hero--compact", compact)]);

    rsx! {
        header { class: hero_class, {children} }
    }
}

/// Panel/card primitive with hard border and shadow.
#[allow(non_snake_case)]
#[component]
pub fn Card(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] accent: bool,
    #[props(default)] selected: bool,
) -> Element {
    let card_class = compose_class(
        "card",
        &class,
        &[("card--accent", accent), ("card--selected", selected)],
    );

    rsx! {
        article { class: card_class, {children} }
    }
}

/// Link styled as a neobrutal button.
#[allow(non_snake_case)]
#[component]
pub fn LinkButton(
    href: String,
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] primary: bool,
) -> Element {
    let button_class = compose_class("button", &class, &[("button--primary", primary)]);

    rsx! {
        a { class: button_class, href: href, {children} }
    }
}

/// Submit or command button.
#[allow(non_snake_case)]
#[component]
pub fn Button(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] primary: bool,
    #[props(default = String::from("button"), into)] button_type: String,
) -> Element {
    let button_class = compose_class("button", &class, &[("button--primary", primary)]);

    rsx! {
        button { class: button_class, r#type: "{button_type}", {children} }
    }
}

/// Section title line used inside cards.
#[allow(non_snake_case)]
#[component]
pub fn BlockTitle(
    title: String,
    #[props(default, into)] subtitle: String,
    #[props(default, into)] class: String,
) -> Element {
    let title_class = compose_class("block-title", &class, &[]);

    rsx! {
        div { class: title_class,
            h2 { "{title}" }
            if !subtitle.is_empty() {
                p { "{subtitle}" }
            }
        }
    }
}

/// Compact all-caps label.
#[allow(non_snake_case)]
#[component]
pub fn Eyebrow(children: Element, #[props(default, into)] class: String) -> Element {
    let eyebrow_class = compose_class("eyebrow", &class, &[]);

    rsx! {
        p { class: eyebrow_class, {children} }
    }
}

/// Pill badge with optional accent fill.
#[allow(non_snake_case)]
#[component]
pub fn Badge(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] accent: bool,
) -> Element {
    let badge_class = compose_class("badge", &class, &[("badge--accent", accent)]);

    rsx! {
        span { class: badge_class, {children} }
    }
}

/// Responsive card grid.
#[allow(non_snake_case)]
#[component]
pub fn Grid(children: Element, #[props(default, into)] class: String) -> Element {
    let grid_class = compose_class("grid", &class, &[]);

    rsx! {
        div { class: grid_class, {children} }
    }
}

/// Two-column workbench layout that collapses on narrow screens.
#[allow(non_snake_case)]
#[component]
pub fn Split(children: Element, #[props(default, into)] class: String) -> Element {
    let split_class = compose_class("split", &class, &[]);

    rsx! {
        div { class: split_class, {children} }
    }
}

/// Form field wrapper.
#[allow(non_snake_case)]
#[component]
pub fn Field(
    label: String,
    children: Element,
    #[props(default, into)] hint: String,
    #[props(default, into)] class: String,
) -> Element {
    let field_class = compose_class("field", &class, &[]);

    rsx! {
        label { class: field_class,
            span { class: "field__label", "{label}" }
            {children}
            if !hint.is_empty() {
                span { class: "field__hint", "{hint}" }
            }
        }
    }
}

/// Preformatted code block.
#[allow(non_snake_case)]
#[component]
pub fn CodeBlock(code: String, #[props(default, into)] class: String) -> Element {
    let code_class = compose_class("code-block", &class, &[]);

    rsx! {
        pre { class: code_class,
            code { "{code}" }
        }
    }
}
