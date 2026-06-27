#![allow(non_snake_case)]

use az_aio_platform::plugin::api::{
    AdminMenuNode, AdminMenuSection, AdminMenuTree, NativeRenderContext,
};
use az_dioxus_components::neobrutal::{NavLink, Sidebar};
use dioxus::prelude::*;

use super::model::RenderSlot;

#[derive(PartialEq, Clone, Props)]
pub(super) struct ShellSidebarProps {
    pub(super) admin_menu_tree: AdminMenuTree,
    pub(super) route: String,
    pub(super) sidebar_renderer: Option<RenderSlot>,
    pub(super) render_context: NativeRenderContext,
}

pub(super) fn ShellSidebar(props: ShellSidebarProps) -> Element {
    rsx! {
        Sidebar {
            PluginNavGroup {
                admin_menu_tree: props.admin_menu_tree,
                route: props.route,
                sidebar_renderer: props.sidebar_renderer,
                render_context: props.render_context,
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct PluginGroupProps {
    admin_menu_tree: AdminMenuTree,
    route: String,
    sidebar_renderer: Option<RenderSlot>,
    render_context: NativeRenderContext,
}

fn PluginNavGroup(props: PluginGroupProps) -> Element {
    rsx! {
        section { class: "sidebar__section sidebar__section--actions",
            SidebarMenuSearch {}
            div { class: "sidebar-menu-shell",
                if !props.admin_menu_tree.sections.is_empty() {
                    AdminMenuNav {
                        sections: props.admin_menu_tree.sections,
                        route: props.route,
                    }
                } else {
                    EmptyPluginNav {}
                }
            }
        }
    }
}

fn SidebarMenuSearch() -> Element {
    rsx! {
        label { class: "sidebar-menu-search",
            span { class: "sidebar-menu-search__icon", "⌕" }
            input {
                id: "admin-menu-search",
                r#type: "search",
                placeholder: "搜索菜单",
                autocomplete: "off",
                "aria-label": "搜索后台菜单",
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct AdminMenuNavProps {
    sections: Vec<AdminMenuSection>,
    route: String,
}

fn AdminMenuNav(props: AdminMenuNavProps) -> Element {
    rsx! {
        div { class: "sidebar-tree sidebar-tree--primary",
            for section in &props.sections {
                div {
                    class: "sidebar-menu-domain",
                    "data-menu-domain": "true",
                    "data-menu-text": section_search_text(section),
                    p { class: "sidebar__heading", "{section.label}" }
                }
                nav { class: "sidebar-tree sidebar-tree--domain",
                    for node in &section.menus {
                        AdminMenuNodeLink {
                            node: node.clone(),
                            route: props.route.clone(),
                            depth: 0usize,
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct AdminMenuNodeLinkProps {
    node: AdminMenuNode,
    route: String,
    depth: usize,
}

fn AdminMenuNodeLink(props: AdminMenuNodeLinkProps) -> Element {
    let route = route_without_query(&props.route);
    let active = node_is_active(&props.node, route);
    let search_text = node_search_text(&props.node);
    let style = tree_style(props.depth);
    let children = props.node.children.clone();

    if !children.is_empty() {
        let detail = format!("{}项", children.len());
        let icon = if props.node.icon.is_empty() {
            "▸".to_string()
        } else {
            props.node.icon.clone()
        };

        return rsx! {
            details {
                class: "sidebar-tree-branch",
                open: active,
                style,
                "data-menu-node": "true",
                "data-menu-text": search_text,
                summary { class: sidebar_branch_class(active),
                    span { class: "nav-button__icon", "{icon}" }
                    span { class: "nav-button__label", "{props.node.label}" }
                    span { class: "nav-button__detail", "{detail}" }
                    span { class: "sidebar-tree-branch__chevron", "⌄" }
                }
                nav { class: "sidebar-tree sidebar-tree--nested",
                    for child in children {
                        AdminMenuNodeLink {
                            node: child,
                            route: props.route.clone(),
                            depth: props.depth + 1,
                        }
                    }
                }
            }
        };
    }

    rsx! {
        div {
            class: "sidebar-menu-node",
            style,
            "data-menu-node": "true",
            "data-menu-text": search_text,
            NavLink {
                href: format!("/?route={}", props.node.href),
                icon: props.node.icon.clone(),
                label: props.node.label.clone(),
                detail: String::new(),
                active,
                plugin: true,
                class: "nav-button--tree",
            }
        }
    }
}

fn sidebar_branch_class(active: bool) -> &'static str {
    if active {
        "nav-button nav-button--plugin nav-button--active nav-button--tree sidebar-tree-branch__summary"
    } else {
        "nav-button nav-button--plugin nav-button--tree sidebar-tree-branch__summary"
    }
}

fn tree_style(depth: usize) -> String {
    let indent = depth * 14;
    let branch_line = indent + 8;
    let parent_line = depth.saturating_sub(1) * 14 + 8;
    format!(
        "--tree-depth: {}; --tree-indent: {}px; --tree-line: {}px; --tree-parent-line: {}px;",
        depth, indent, branch_line, parent_line
    )
}

fn EmptyPluginNav() -> Element {
    rsx! {
        div { class: "empty-panel empty-panel--compact",
            div { class: "empty-panel__mark", "∅" }
            p { "未加载后台插件" }
        }
    }
}

fn route_without_query(route: &str) -> &str {
    route
        .split_once('?')
        .map(|(route, _)| route)
        .unwrap_or(route)
}

fn section_search_text(section: &AdminMenuSection) -> String {
    format!("{} {}", section.label, section.domain_id)
}

fn node_search_text(node: &AdminMenuNode) -> String {
    format!(
        "{} {} {} {}",
        node.label,
        node.href,
        node.id,
        node.permissions_any_of.join(" ")
    )
}

fn node_is_active(node: &AdminMenuNode, route: &str) -> bool {
    if node.href == route || node.active_patterns.iter().any(|pattern| pattern == route) {
        return true;
    }

    node.children
        .iter()
        .any(|child| node_is_active(child, route))
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use super::*;

    #[test]
    fn route_query_is_removed_for_active_menu_matching() {
        assert_eq!(route_without_query("/lowcode?mode=screens"), "/lowcode");
    }

    #[test]
    fn node_search_text_stays_scoped_to_current_node() {
        let node = AdminMenuNode {
            id: "root".to_string(),
            kind: az_aio_platform::plugin::api::AdminMenuNodeKind::Branch,
            label: "账号权限".to_string(),
            href: "/system/identity/users".to_string(),
            icon: "▸".to_string(),
            order: 10,
            active_patterns: Vec::new(),
            permissions_any_of: Vec::new(),
            children: vec![AdminMenuNode {
                id: "role".to_string(),
                kind: az_aio_platform::plugin::api::AdminMenuNodeKind::Page,
                label: "角色管理".to_string(),
                href: "/system/permission/roles".to_string(),
                icon: "●".to_string(),
                order: 20,
                active_patterns: Vec::new(),
                permissions_any_of: vec!["system:role".to_string()],
                children: Vec::new(),
            }],
        };

        let text = node_search_text(&node);

        assert!(text.contains("账号权限"));
        assert!(!text.contains("角色管理"));
        assert!(!text.contains("system:role"));
    }

    #[test]
    fn branch_nodes_render_nested_details_tree() {
        let node = AdminMenuNode {
            id: "identity".to_string(),
            kind: az_aio_platform::plugin::api::AdminMenuNodeKind::Branch,
            label: "身份与权限".to_string(),
            href: "/system/identity/users".to_string(),
            icon: "▸".to_string(),
            order: 10,
            active_patterns: Vec::new(),
            permissions_any_of: Vec::new(),
            children: vec![AdminMenuNode {
                id: "role".to_string(),
                kind: az_aio_platform::plugin::api::AdminMenuNodeKind::Page,
                label: "角色管理".to_string(),
                href: "/system/permission/roles".to_string(),
                icon: "●".to_string(),
                order: 20,
                active_patterns: Vec::new(),
                permissions_any_of: vec!["system:role".to_string()],
                children: Vec::new(),
            }],
        };

        let markup = dioxus_ssr::render_element(rsx! {
            AdminMenuNodeLink {
                node,
                route: "/system/permission/roles".to_string(),
                depth: 0usize,
            }
        });

        assert!(markup.contains("<details"));
        assert!(markup.contains("sidebar-tree-branch"));
        assert!(markup.contains("<summary"));
        assert!(markup.contains("sidebar-tree--nested"));
        assert!(markup.contains("--tree-indent: 0px"));
        assert!(markup.contains("角色管理"));
    }
}
