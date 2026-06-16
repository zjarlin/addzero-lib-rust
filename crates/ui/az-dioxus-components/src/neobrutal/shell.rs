//! Neobrutal shell primitives for SSR workbench applications.

#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::util::class_name::compose_class;

/// Full application shell grid with optional collapsed sidebar state.
#[component]
pub fn NbShell(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] collapsed: bool,
) -> Element {
    let shell_class = compose_class(
        "az-aio-shell nb-shell",
        &class,
        &[
            ("az-aio-shell--collapsed", collapsed),
            ("nb-shell--collapsed", collapsed),
        ],
    );

    rsx! {
        main { class: shell_class, {children} }
    }
}

/// Fixed titlebar control strip shown above the sidebar.
#[component]
pub fn NbTitlebarControls(children: Element, #[props(default, into)] class: String) -> Element {
    let controls_class = compose_class("titlebar-controls nb-titlebar-controls", &class, &[]);

    rsx! {
        div { class: controls_class, {children} }
    }
}

/// Sidebar collapse toggle button.
#[component]
pub fn NbSidebarToggle(
    #[props(default = String::from("sidebar-toggle"), into)] id: String,
    #[props(default = String::from("折叠侧边栏"), into)] aria_label: String,
    #[props(default)] expanded: bool,
    #[props(default, into)] class: String,
) -> Element {
    let toggle_class = compose_class("sidebar-toggle nb-sidebar-toggle", &class, &[]);
    let aria_expanded = expanded.to_string();

    rsx! {
        button {
            class: toggle_class,
            id: id,
            r#type: "button",
            "aria-label": aria_label,
            "aria-expanded": aria_expanded,
            span { class: "sidebar-toggle__glyph nb-sidebar-toggle__glyph" }
        }
    }
}

/// Titlebar navigation glyph.
#[component]
pub fn NbTitlebarNav(
    #[props(into)] label: String,
    #[props(default)] disabled: bool,
    #[props(default, into)] class: String,
) -> Element {
    let nav_class = compose_class(
        "titlebar-nav nb-titlebar-nav",
        &class,
        &[
            ("titlebar-nav--disabled", disabled),
            ("nb-titlebar-nav--disabled", disabled),
        ],
    );

    rsx! {
        span { class: nav_class, "aria-hidden": "true", "{label}" }
    }
}

/// Left workbench sidebar container.
#[component]
pub fn NbSidebar(children: Element, #[props(default, into)] class: String) -> Element {
    let sidebar_class = compose_class(
        "sidebar workbench-slot workbench-slot--side nb-sidebar",
        &class,
        &[],
    );

    rsx! {
        aside { class: sidebar_class, {children} }
    }
}

/// Sidebar plugin disclosure group.
#[component]
pub fn NbPluginGroup(
    children: Element,
    #[props(default = String::from("插件"), into)] label: String,
    #[props(default = String::from("◎"), into)] icon: String,
    #[props(default = true)] open: bool,
    #[props(default, into)] class: String,
) -> Element {
    let group_class = compose_class("plugin-group nb-plugin-group", &class, &[]);

    rsx! {
        details { class: group_class, open: open,
            summary { class: "nav-button plugin-group__summary nb-plugin-group__summary",
                span { class: "nav-button__icon nb-nav-button__icon", "{icon}" }
                span { class: "nav-button__label nb-nav-button__label", "{label}" }
                span { class: "plugin-group__chevron nb-plugin-group__chevron", "⌄" }
            }
            div { class: "plugin-group__panel nb-plugin-group__panel",
                {children}
            }
        }
    }
}

/// Sidebar navigation link with optional plugin and active states.
#[component]
pub fn NbNavLink(
    #[props(into)] href: String,
    #[props(into)] icon: String,
    #[props(into)] label: String,
    #[props(default, into)] detail: String,
    #[props(default)] active: bool,
    #[props(default)] plugin: bool,
    #[props(default, into)] class: String,
) -> Element {
    let link_class = compose_class(
        "nav-button nb-nav-button",
        &class,
        &[
            ("nav-button--active", active),
            ("nb-nav-button--active", active),
            ("nav-button--plugin", plugin),
            ("nb-nav-button--plugin", plugin),
        ],
    );

    rsx! {
        a { class: link_class, href: href,
            span { class: "nav-button__icon nb-nav-button__icon", "{icon}" }
            span { class: "nav-button__label nb-nav-button__label", "{label}" }
            if !detail.is_empty() {
                span { class: "nav-button__detail nb-nav-button__detail", "{detail}" }
            }
        }
    }
}

/// Main workbench viewport container.
#[component]
pub fn NbWorkspace(children: Element, #[props(default, into)] class: String) -> Element {
    let workspace_class = compose_class(
        "workspace workbench-slot workbench-slot--main nb-workspace",
        &class,
        &[],
    );

    rsx! {
        section { class: workspace_class, {children} }
    }
}

/// Workspace header bar with right-aligned action area.
#[component]
pub fn NbHeaderBar(children: Element, #[props(default, into)] class: String) -> Element {
    let header_class = compose_class("header-bar nb-header-bar", &class, &[]);

    rsx! {
        header { class: header_class,
            div { class: "header-bar__actions nb-header-bar__actions",
                {children}
            }
        }
    }
}

/// Model selector style command button.
#[component]
pub fn NbModelButton(
    #[props(default = String::from("AZ"), into)] mark: String,
    #[props(default = String::from("AZ AIO"), into)] label: String,
    #[props(default = String::from("⌄"), into)] chevron: String,
    #[props(default, into)] class: String,
) -> Element {
    let button_class = compose_class("model-button nb-model-button", &class, &[]);

    rsx! {
        button { class: button_class, r#type: "button",
            span { class: "model-button__mark nb-model-button__mark", "{mark}" }
            span { "{label}" }
            span { class: "model-button__chevron nb-model-button__chevron", "{chevron}" }
        }
    }
}

/// Compact icon anchor used by shell actions.
#[component]
pub fn NbIconButton(
    children: Element,
    #[props(default = String::from("#"), into)] href: String,
    #[props(default, into)] id: String,
    #[props(default, into)] aria_label: String,
    #[props(default, into)] class: String,
) -> Element {
    let button_class = compose_class("icon-button nb-icon-button", &class, &[]);

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
pub fn NbWorkspaceBody(
    children: Element,
    #[props(default)] lowcode: bool,
    #[props(default)] catalog: bool,
    #[props(default, into)] class: String,
) -> Element {
    let body_class = compose_class(
        "workspace__body nb-workspace-body",
        &class,
        &[
            ("workspace__body--lowcode", lowcode),
            ("nb-workspace-body--lowcode", lowcode),
            ("workspace__body--catalog", catalog),
            ("nb-workspace-body--catalog", catalog),
        ],
    );

    rsx! {
        div { class: body_class, {children} }
    }
}

/// Central content slot for welcome or plugin-rendered pages.
#[component]
pub fn NbContentSlot(
    children: Element,
    #[props(default)] plugin: bool,
    #[props(default, into)] class: String,
) -> Element {
    let slot_class = compose_class(
        "content-center-slot nb-content-slot",
        &class,
        &[
            ("content-center-slot--plugin", plugin),
            ("nb-content-slot--plugin", plugin),
            ("content-center-slot--welcome", !plugin),
            ("nb-content-slot--welcome", !plugin),
        ],
    );

    rsx! {
        section { class: slot_class, {children} }
    }
}

/// Absolute two-column project layout mounted inside the workspace body.
#[component]
pub fn NbProjectLayout(children: Element, #[props(default, into)] class: String) -> Element {
    let layout_class = compose_class("project-layout nb-project-layout", &class, &[]);

    rsx! {
        div { class: layout_class, {children} }
    }
}

/// Right-side auxiliary slot.
#[component]
pub fn NbRightSlot(children: Element, #[props(default, into)] class: String) -> Element {
    let slot_class = compose_class("right-slot nb-right-slot", &class, &[]);

    rsx! {
        aside { class: slot_class, {children} }
    }
}

/// Floating overlay slot.
#[component]
pub fn NbFloatingPanelSlot(children: Element, #[props(default, into)] class: String) -> Element {
    let slot_class = compose_class("floating-panel-slot nb-floating-panel-slot", &class, &[]);

    rsx! {
        div { class: slot_class, {children} }
    }
}
