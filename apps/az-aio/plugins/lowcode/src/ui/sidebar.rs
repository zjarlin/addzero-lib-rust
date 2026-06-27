#![allow(non_snake_case)]

//! lowcode 插件侧轴导航。

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::prelude::NavLink;
use dioxus::prelude::*;

/// 渲染 engine 四块工作台入口。
pub fn LowcodeSidebar(context: NativeRenderContext) -> Element {
    let items = [
        ("字段", "/?route=/lowcode&tab=fields", "▤"),
        ("钩子", "/?route=/lowcode&tab=hooks", "⚑"),
        ("记录", "/?route=/lowcode&tab=records", "▦"),
    ];

    rsx! {
        nav { class: "sidebar-tree sidebar-tree--primary lowcode-sidebar",
            for (label, href, icon) in items {
                NavLink {
                    href: href.to_string(),
                    icon: icon.to_string(),
                    label: label.to_string(),
                    detail: "engine".to_string(),
                    active: sidebar_active(&context.active_route, href),
                    plugin: true,
                    class: "nav-button--nested",
                }
            }
        }
    }
}

fn sidebar_active(route: &str, href: &str) -> bool {
    let Some(tab) = href.split("tab=").nth(1) else {
        return route == "/lowcode";
    };
    route.contains(&format!("tab={tab}")) || (tab == "fields" && route == "/lowcode")
}
