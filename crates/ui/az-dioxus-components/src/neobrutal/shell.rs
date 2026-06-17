//! Neobrutal shell primitives for SSR workbench applications.

#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::class_name::compose_class;

/// Full application shell grid with optional collapsed sidebar state.
#[component]
pub fn Shell(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] collapsed: bool,
) -> Element {
    let shell_class = compose_class("shell", &class, &[("shell--collapsed", collapsed)]);

    rsx! {
        main { class: shell_class, {children} }
    }
}

/// Fixed titlebar control strip shown above the sidebar.
#[component]
pub fn TitlebarControls(children: Element, #[props(default, into)] class: String) -> Element {
    let controls_class = compose_class("titlebar-controls", &class, &[]);

    rsx! {
        div { class: controls_class, {children} }
    }
}

/// Sidebar collapse toggle button.
#[component]
pub fn SidebarToggle(
    #[props(default = String::from("sidebar-toggle"), into)] id: String,
    #[props(default = String::from("折叠侧边栏"), into)] aria_label: String,
    #[props(default)] expanded: bool,
    #[props(default, into)] class: String,
) -> Element {
    let toggle_class = compose_class("sidebar-toggle", &class, &[]);
    let aria_expanded = expanded.to_string();

    rsx! {
        button {
            class: toggle_class,
            id: id,
            r#type: "button",
            "aria-label": aria_label,
            "aria-expanded": aria_expanded,
            span { class: "sidebar-toggle__glyph" }
        }
    }
}

/// Titlebar navigation glyph.
#[component]
pub fn TitlebarNav(
    #[props(into)] label: String,
    #[props(default)] disabled: bool,
    #[props(default, into)] class: String,
) -> Element {
    let nav_class = compose_class(
        "titlebar-nav",
        &class,
        &[("titlebar-nav--disabled", disabled)],
    );

    rsx! {
        span { class: nav_class, "aria-hidden": "true", "{label}" }
    }
}

/// Left workbench sidebar container.
#[component]
pub fn Sidebar(children: Element, #[props(default, into)] class: String) -> Element {
    let sidebar_class = compose_class("sidebar workbench-slot workbench-slot--side", &class, &[]);

    rsx! {
        aside { class: sidebar_class, {children} }
    }
}

/// Sidebar plugin disclosure group.
#[component]
pub fn PluginGroup(
    children: Element,
    #[props(default = String::from("插件"), into)] label: String,
    #[props(default = String::from("◎"), into)] icon: String,
    #[props(default = true)] open: bool,
    #[props(default, into)] class: String,
) -> Element {
    let group_class = compose_class("plugin-group", &class, &[]);

    rsx! {
        details { class: group_class, open: open,
            summary { class: "nav-button plugin-group__summary",
                span { class: "nav-button__icon", "{icon}" }
                span { class: "nav-button__label", "{label}" }
                span { class: "plugin-group__chevron", "⌄" }
            }
            div { class: "plugin-group__panel",
                {children}
            }
        }
    }
}

/// Sidebar navigation link with optional plugin and active states.
#[component]
pub fn NavLink(
    #[props(into)] href: String,
    #[props(into)] icon: String,
    #[props(into)] label: String,
    #[props(default, into)] detail: String,
    #[props(default)] active: bool,
    #[props(default)] plugin: bool,
    #[props(default, into)] class: String,
) -> Element {
    let link_class = compose_class(
        "nav-button",
        &class,
        &[
            ("nav-button--active", active),
            ("nav-button--plugin", plugin),
        ],
    );

    rsx! {
        a { class: link_class, href: href,
            span { class: "nav-button__icon", "{icon}" }
            span { class: "nav-button__label", "{label}" }
            if !detail.is_empty() {
                span { class: "nav-button__detail", "{detail}" }
            }
        }
    }
}

/// Main workbench viewport container.
#[component]
pub fn Workspace(children: Element, #[props(default, into)] class: String) -> Element {
    let workspace_class =
        compose_class("workspace workbench-slot workbench-slot--main", &class, &[]);

    rsx! {
        section { class: workspace_class, {children} }
    }
}

/// Workspace header bar with right-aligned action area.
#[component]
pub fn HeaderBar(children: Element, #[props(default, into)] class: String) -> Element {
    let header_class = compose_class("header-bar", &class, &[]);

    rsx! {
        header { class: header_class,
            div { class: "header-bar__actions",
                {children}
            }
        }
    }
}

/// Model selector style command button.
#[component]
pub fn ModelButton(
    #[props(default = String::from("AI"), into)] mark: String,
    #[props(default = String::from("Model"), into)] label: String,
    #[props(default = String::from("⌄"), into)] chevron: String,
    #[props(default, into)] class: String,
) -> Element {
    let button_class = compose_class("model-button", &class, &[]);

    rsx! {
        button { class: button_class, r#type: "button",
            span { class: "model-button__mark", "{mark}" }
            span { "{label}" }
            span { class: "model-button__chevron", "{chevron}" }
        }
    }
}

/// Compact icon anchor used by shell actions.
#[component]
pub fn IconButton(
    children: Element,
    #[props(default = String::from("#"), into)] href: String,
    #[props(default, into)] id: String,
    #[props(default, into)] aria_label: String,
    #[props(default, into)] class: String,
) -> Element {
    let button_class = compose_class("icon-button", &class, &[]);

    rsx! {
        a {
            class: button_class,
            href: href,
            id: id,
            "aria-label": aria_label,
            {children}
        }
    }
}

/// Workspace body grid that can switch to plugin-specific layouts.
#[component]
pub fn WorkspaceBody(
    children: Element,
    #[props(default)] lowcode: bool,
    #[props(default)] catalog: bool,
    #[props(default, into)] class: String,
) -> Element {
    let body_class = compose_class(
        "workspace__body",
        &class,
        &[
            ("workspace__body--lowcode", lowcode),
            ("workspace__body--catalog", catalog),
        ],
    );

    rsx! {
        div { class: body_class, {children} }
    }
}

/// Central content slot for welcome or plugin-rendered pages.
#[component]
pub fn ContentSlot(
    children: Element,
    #[props(default)] plugin: bool,
    #[props(default, into)] class: String,
) -> Element {
    let slot_class = compose_class(
        "content-center-slot",
        &class,
        &[
            ("content-center-slot--plugin", plugin),
            ("content-center-slot--welcome", !plugin),
        ],
    );

    rsx! {
        section { class: slot_class, {children} }
    }
}

/// Absolute two-column project layout mounted inside the workspace body.
#[component]
pub fn ProjectLayout(children: Element, #[props(default, into)] class: String) -> Element {
    let layout_class = compose_class("project-layout", &class, &[]);

    rsx! {
        div { class: layout_class, {children} }
    }
}

/// Right-side auxiliary slot.
#[component]
pub fn RightSlot(children: Element, #[props(default, into)] class: String) -> Element {
    let slot_class = compose_class("right-slot", &class, &[]);

    rsx! {
        aside { class: slot_class, {children} }
    }
}

/// Floating overlay slot.
#[component]
pub fn FloatingPanelSlot(children: Element, #[props(default, into)] class: String) -> Element {
    let slot_class = compose_class("floating-panel-slot", &class, &[]);

    rsx! {
        div { class: slot_class, {children} }
    }
}
