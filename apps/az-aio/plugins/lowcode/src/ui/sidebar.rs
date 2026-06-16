#![allow(non_snake_case)]

//! Lowcode-owned sidebar renderer.

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::neobrutal::NbNavLink;
use dioxus::prelude::*;

use crate::{contract::LowcodeMenuContribution, metadata::configurable_lowcode_menus};

/// Renders the lowcode menu tree from plugin metadata.
pub fn LowcodeSidebar(context: NativeRenderContext) -> Element {
    let menus = configurable_lowcode_menus();
    let mut roots = menus
        .iter()
        .filter(|menu| menu.visible && menu.parent_id.is_none())
        .cloned()
        .collect::<Vec<_>>();
    roots.sort_by(menu_order);

    rsx! {
        nav { class: "sidebar-tree sidebar-tree--primary lowcode-sidebar",
            for root in roots {
                details { class: "plugin-group nb-plugin-group", open: true,
                    summary { class: "nav-button nav-button--plugin nb-nav-button nb-nav-button--plugin plugin-group__summary nb-plugin-group__summary",
                        span { class: "nav-button__icon nb-nav-button__icon", "{root.icon}" }
                        span { class: "nav-button__label nb-nav-button__label", "{root.label}" }
                        span { class: "nav-button__detail nb-nav-button__detail", "{root.order}" }
                    }
                    nav { class: "sidebar-tree sidebar-tree--nested",
                        for child in menu_children(&menus, &root.id) {
                            NbNavLink {
                                href: menu_href(&child.route),
                                icon: child.icon.clone(),
                                label: child.label.clone(),
                                detail: child.order.to_string(),
                                active: menu_active(&context.active_route, &child.route),
                                plugin: true,
                                class: "nav-button--nested",
                            }
                        }
                    }
                }
            }
        }
    }
}

fn menu_children(menus: &[LowcodeMenuContribution], parent_id: &str) -> Vec<LowcodeMenuContribution> {
    let mut children = menus
        .iter()
        .filter(|menu| menu.visible && menu.parent_id.as_deref() == Some(parent_id))
        .cloned()
        .collect::<Vec<_>>();
    children.sort_by(menu_order);
    children
}

fn menu_order(left: &LowcodeMenuContribution, right: &LowcodeMenuContribution) -> std::cmp::Ordering {
    left.order.cmp(&right.order).then(left.id.cmp(&right.id))
}

fn menu_href(route: &str) -> String {
    if route.starts_with("/?") {
        route.to_string()
    } else if let Some((path, query)) = route.split_once('?') {
        format!("/?route={path}&{query}")
    } else {
        format!("/?route={route}")
    }
}

fn menu_active(current_route: &str, menu_route: &str) -> bool {
    if current_route == menu_route {
        return true;
    }
    !current_route.contains('?') && current_route == menu_route.trim_end_matches('/')
}
