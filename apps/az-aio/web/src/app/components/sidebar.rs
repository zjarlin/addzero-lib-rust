#![allow(non_snake_case)]

use az_aio_platform::plugin::api::{NativeRenderContext, NavItemContribution};
use az_dioxus_components::neobrutal::{NavLink, PluginGroup as PluginGroupDisclosure, Sidebar};
use dioxus::prelude::*;

use super::model::RenderSlot;

#[derive(PartialEq, Clone, Props)]
pub(super) struct ShellSidebarProps {
    pub(super) nav_items: Vec<NavItemContribution>,
    pub(super) route: String,
    pub(super) sidebar_renderer: Option<RenderSlot>,
    pub(super) render_context: NativeRenderContext,
}

pub(super) fn ShellSidebar(props: ShellSidebarProps) -> Element {
    rsx! {
        Sidebar {
            PrimaryActions {}
            PluginNavGroup {
                nav_items: props.nav_items,
                route: props.route,
                sidebar_renderer: props.sidebar_renderer,
                render_context: props.render_context,
            }
            ProjectSection {}
            SidebarFooter {}
        }
    }
}

fn PrimaryActions() -> Element {
    rsx! {
        section { class: "sidebar__section sidebar__section--actions",
            NavLink { href: "/", icon: "✎", label: "新对话" }
            NavLink { href: "/?route=/assets", icon: "⌕", label: "搜索" }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct PluginGroupProps {
    nav_items: Vec<NavItemContribution>,
    route: String,
    sidebar_renderer: Option<RenderSlot>,
    render_context: NativeRenderContext,
}

fn PluginNavGroup(props: PluginGroupProps) -> Element {
    rsx! {
        section { class: "sidebar__section sidebar__section--actions",
            PluginGroupDisclosure {
                if let Some(render) = props.sidebar_renderer {
                    {render.render(props.render_context.clone())}
                } else {
                    PluginNavList {
                        nav_items: props.nav_items,
                        route: props.route,
                    }
                }
            }
            NavLink { href: "/?route=/gateway", icon: "◷", label: "自动化" }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct PluginNavListProps {
    nav_items: Vec<NavItemContribution>,
    route: String,
}

fn PluginNavList(props: PluginNavListProps) -> Element {
    rsx! {
        nav { class: "sidebar-tree sidebar-tree--primary",
            for item in &props.nav_items {
                NavLink {
                    href: "/?route={item.route}",
                    icon: item.icon.clone(),
                    label: item.label.clone(),
                    detail: item.order.to_string(),
                    active: item.route == props.route,
                    plugin: true,
                }
            }
        }
    }
}

fn ProjectSection() -> Element {
    rsx! {
        section { class: "sidebar__section sidebar__section--contents",
            p { class: "sidebar__heading", "项目" }
            div { class: "project-list",
                a { class: "project-row", href: "/",
                    span { class: "project-row__icon", "▱" }
                    span { class: "project-row__label", "AZ AIO 脚手架" }
                }
                span { class: "thread-row", "插件 ui/backend 拆分" }
                span { class: "thread-row", "Rudi 组合入口" }
                span { class: "thread-row", "Codex 风格骨架" }
            }
        }
    }
}

fn SidebarFooter() -> Element {
    rsx! {
        section { class: "sidebar__footer",
            a { class: "settings-button", href: "/?route=/config",
                span { class: "settings-button__icon", "⚙" }
                span { class: "settings-button__label", "设置" }
            }
        }
    }
}
