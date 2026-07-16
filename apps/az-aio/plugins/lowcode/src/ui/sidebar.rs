#![cfg(any())]
#![allow(non_snake_case)]

//! lowcode 插件侧轴导航。

use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

/// 渲染 engine 四块工作台入口。
pub fn LowcodeSidebar(context: NativeRenderContext) -> Element {
    let items = [
        ("字段", "/?route=/lowcode&tab=fields", "▤"),
        ("钩子", "/?route=/lowcode&tab=hooks", "⚑"),
        ("记录", "/?route=/lowcode&tab=records", "▦"),
    ];

    rsx! {
        nav { class: "adui-menu adui-menu-inline",
            for (label, href, icon) in items {
                a {
                    class: if sidebar_active(&context.active_route, href) { "adui-menu-item adui-menu-item-selected" } else { "adui-menu-item" },
                    href,
                    span { class: "adui-menu-item-icon", "{icon}" }
                    span { class: "adui-menu-item-label", "{label}" }
                    span { class: "adui-tag", "engine" }
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
