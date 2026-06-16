//! Neobrutalism-inspired SSR primitives for Dioxus pages.
//!
//! These components provide only reusable visual structure: bold borders,
//! hard shadows, high contrast fills, compact layout blocks, and stable class
//! names for server-rendered applications.

use dioxus::prelude::*;

use crate::util::class_name::compose_class;

automod::dir!(pub "src/neobrutal");

pub use shell::{
    NbContentSlot, NbFloatingPanelSlot, NbHeaderBar, NbIconButton, NbModelButton, NbNavLink,
    NbPluginGroup, NbProjectLayout, NbRightSlot, NbShell, NbSidebar, NbSidebarToggle,
    NbTitlebarControls, NbTitlebarNav, NbWorkspace, NbWorkspaceBody,
};

/// Full-page surface with a graph-paper background.
#[allow(non_snake_case)]
#[component]
pub fn NbPage(children: Element, #[props(default, into)] class: String) -> Element {
    let root_class = compose_class("nb-page", &class, &[]);

    rsx! {
        section { class: root_class, {children} }
    }
}

/// Top hero block for workbench pages.
#[allow(non_snake_case)]
#[component]
pub fn NbHero(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] compact: bool,
) -> Element {
    let hero_class = compose_class("nb-hero", &class, &[("nb-hero--compact", compact)]);

    rsx! {
        header { class: hero_class, {children} }
    }
}

/// Panel/card primitive with hard border and shadow.
#[allow(non_snake_case)]
#[component]
pub fn NbCard(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] accent: bool,
    #[props(default)] selected: bool,
) -> Element {
    let card_class = compose_class(
        "nb-card",
        &class,
        &[("nb-card--accent", accent), ("nb-card--selected", selected)],
    );

    rsx! {
        article { class: card_class, {children} }
    }
}

/// Link styled as a neobrutal button.
#[allow(non_snake_case)]
#[component]
pub fn NbLinkButton(
    href: String,
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] primary: bool,
) -> Element {
    let button_class = compose_class("nb-button", &class, &[("nb-button--primary", primary)]);

    rsx! {
        a { class: button_class, href: href, {children} }
    }
}

/// Submit or command button.
#[allow(non_snake_case)]
#[component]
pub fn NbButton(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] primary: bool,
    #[props(default = String::from("button"), into)] button_type: String,
) -> Element {
    let button_class = compose_class("nb-button", &class, &[("nb-button--primary", primary)]);

    rsx! {
        button { class: button_class, r#type: "{button_type}", {children} }
    }
}

/// Section title line used inside cards.
#[allow(non_snake_case)]
#[component]
pub fn NbBlockTitle(
    title: String,
    #[props(default, into)] subtitle: String,
    #[props(default, into)] class: String,
) -> Element {
    let title_class = compose_class("nb-block-title", &class, &[]);

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
pub fn NbEyebrow(children: Element, #[props(default, into)] class: String) -> Element {
    let eyebrow_class = compose_class("nb-eyebrow", &class, &[]);

    rsx! {
        p { class: eyebrow_class, {children} }
    }
}

/// Pill badge with optional accent fill.
#[allow(non_snake_case)]
#[component]
pub fn NbBadge(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] accent: bool,
) -> Element {
    let badge_class = compose_class("nb-badge", &class, &[("nb-badge--accent", accent)]);

    rsx! {
        span { class: badge_class, {children} }
    }
}

/// Responsive card grid.
#[allow(non_snake_case)]
#[component]
pub fn NbGrid(children: Element, #[props(default, into)] class: String) -> Element {
    let grid_class = compose_class("nb-grid", &class, &[]);

    rsx! {
        div { class: grid_class, {children} }
    }
}

/// Two-column workbench layout that collapses on narrow screens.
#[allow(non_snake_case)]
#[component]
pub fn NbSplit(children: Element, #[props(default, into)] class: String) -> Element {
    let split_class = compose_class("nb-split", &class, &[]);

    rsx! {
        div { class: split_class, {children} }
    }
}

/// Form field wrapper.
#[allow(non_snake_case)]
#[component]
pub fn NbField(
    label: String,
    children: Element,
    #[props(default, into)] hint: String,
    #[props(default, into)] class: String,
) -> Element {
    let field_class = compose_class("nb-field", &class, &[]);

    rsx! {
        label { class: field_class,
            span { class: "nb-field__label", "{label}" }
            {children}
            if !hint.is_empty() {
                span { class: "nb-field__hint", "{hint}" }
            }
        }
    }
}

/// Preformatted code block.
#[allow(non_snake_case)]
#[component]
pub fn NbCodeBlock(code: String, #[props(default, into)] class: String) -> Element {
    let code_class = compose_class("nb-code", &class, &[]);

    rsx! {
        pre { class: code_class,
            code { "{code}" }
        }
    }
}
