use dioxus::prelude::*;
use registry::ui::badge::{Badge, BadgeVariant};

use crate::{
    bootstrap::{
        AdminMenuNode, initial_route, load_from_document, menu_node_active, page_title, push_route,
    },
    screens::render as render_client_screen,
};

pub fn App() -> Element {
    let bootstrap = use_signal(load_from_document);
    let initial_route = initial_route(&bootstrap.read());
    let active_route = use_signal(move || initial_route);
    let route = active_route.read().clone();
    let snapshot = bootstrap.read().clone();
    let title = page_title(&snapshot, &route);
    let renderer_id = snapshot
        .client_pages
        .iter()
        .find(|page| page.route == route)
        .map(|page| page.renderer_id.as_str());
    let content = render_client_screen(renderer_id, &route, snapshot.api_base_url.clone());

    rsx! {
        document::Stylesheet { href: "/assets/app.css" }
        div { class: "aio-shell-frame bg-background text-foreground",
        aside { class: "aio-sidebar border-r bg-card",
            header { class: "aio-sidebar-header",
                div { class: "aio-sidebar-brand px-3 py-2",
                    p { class: "aio-sidebar-brand-title", "AIO" }
                    p { class: "aio-sidebar-brand-subtitle", "Dioxus client plugin workbench" }
                }
            }
            nav { class: "aio-sidebar-scroll space-y-4", role: "menu",
                for section in snapshot.admin_menu_tree.sections.iter() {
                    section { class: "space-y-1",
                        p { class: "px-3 text-xs font-semibold uppercase tracking-wide text-muted-foreground", "{section.label}" }
                        div { class: "space-y-1",
                            for node in section.menus.iter() {
                                {render_menu_node(node, &route, active_route)}
                            }
                        }
                    }
                }
            }
        }
        main { class: "aio-main min-w-0",
            header { class: "aio-topbar border-b bg-background/95 backdrop-blur",
                div { class: "aio-topbar-title min-w-0",
                    h1 { class: "truncate text-sm font-semibold", "{title}" }
                }
                div {}
                div { class: "aio-toolbar-actions",
                    Badge { variant: BadgeVariant::Outline, "{route}" }
                }
            }
            section { class: "aio-main-scroll bg-muted/30", {content} }
        }
        }
    }
}

fn render_menu_node(
    node: &AdminMenuNode,
    active_route: &str,
    mut active_route_signal: Signal<String>,
) -> Element {
    let class = if menu_node_active(node, active_route) {
        "aio-sidebar-menu-link flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground"
    } else {
        "aio-sidebar-menu-link flex items-center gap-2 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground"
    };
    let href = if node.href.is_empty() {
        "#".to_string()
    } else {
        format!("?route={}", node.href)
    };
    let route_for_click = node.href.clone();
    rsx! {
        div { role: "menuitem",
            a { class, href,
                onclick: move |event| {
                    if !route_for_click.is_empty() {
                        event.prevent_default();
                        active_route_signal.set(route_for_click.clone());
                        push_route(&route_for_click);
                    }
                },
                span { class: "aio-sidebar-menu-icon", "{node.icon}" }
                span { class: "aio-sidebar-menu-label", "{node.label}" }
            }
            if !node.children.is_empty() {
                div { class: "aio-sidebar-menu-children ml-2 space-y-1",
                    for child in node.children.iter() {
                        {render_menu_node(child, active_route, active_route_signal)}
                    }
                }
            }
        }
    }
}
