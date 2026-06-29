//! 系统后台导航快照。
//!
//! `registered_admin_sections` 保留编译期 starter 注册桥接能力；
//! `system_admin_sections` 则把完整系统后台 catalog 转成 admin shell 使用的显式树模型。

use az_admin_plugin_registry::api::{
    AdminNavigationKind, RegisteredAdminNode, registered_domains, section_for_path,
};

use crate::{
    plugin::api::{AdminMenuNode, AdminMenuNodeKind, AdminMenuSection},
    system::catalog::{
        SYSTEM_DEFAULT_ROUTE, SYSTEM_DOMAIN_ID, SYSTEM_DOMAIN_LABEL, SystemPage, system_pages,
    },
};

pub type AdminSectionSnapshot = AdminMenuSection;
pub type AdminNodeSnapshot = AdminMenuNode;
pub type AdminNodeKind = AdminMenuNodeKind;

pub fn registered_admin_sections() -> Vec<AdminSectionSnapshot> {
    az_system_starters::api::link_all();
    registered_domains()
        .into_iter()
        .filter_map(|domain| {
            section_for_path(domain.default_href).map(|section| AdminMenuSection {
                domain_id: domain.id.to_string(),
                label: section.label.to_string(),
                default_href: section.default_href.to_string(),
                order: i32::from(domain.order),
                menus: section.menus.into_iter().map(to_node_snapshot).collect(),
            })
        })
        .collect()
}

pub fn system_admin_sections() -> Vec<AdminSectionSnapshot> {
    vec![AdminMenuSection {
        domain_id: SYSTEM_DOMAIN_ID.to_string(),
        label: SYSTEM_DOMAIN_LABEL.to_string(),
        default_href: SYSTEM_DEFAULT_ROUTE.to_string(),
        order: 900,
        menus: SYSTEM_CONTEXT_BRANCHES.iter().map(to_branch_snapshot).collect(),
    }]
}

struct SystemContextBranch {
    id: &'static str,
    label: &'static str,
    page_ids: &'static [&'static str],
}

const SYSTEM_CONTEXT_BRANCHES: &[SystemContextBranch] = &[
    SystemContextBranch {
        id: "account-axis",
        label: "我的账户",
        page_ids: &["api_keys"],
    },
    SystemContextBranch {
        id: "permission-axis",
        label: "权限管理",
        page_ids: &["identity", "role", "menu"],
    },
    SystemContextBranch {
        id: "organization-tenant-axis",
        label: "组织租户",
        page_ids: &["organization", "tenant"],
    },
    SystemContextBranch {
        id: "auth-axis",
        label: "认证接入",
        page_ids: &["auth", "oauth2", "social"],
    },
    SystemContextBranch {
        id: "system-config-axis",
        label: "系统配置",
        page_ids: &["dictionary", "area"],
    },
    SystemContextBranch {
        id: "log-message-axis",
        label: "日志消息",
        page_ids: &["audit", "messaging"],
    },
];

fn to_branch_snapshot(branch: &SystemContextBranch) -> AdminMenuNode {
    let children = branch
        .page_ids
        .iter()
        .filter_map(|id| system_pages().iter().copied().find(|page| page.id == *id))
        .map(to_page_snapshot)
        .collect::<Vec<_>>();
    let href = children
        .first()
        .map(|node| node.href.clone())
        .unwrap_or_else(|| SYSTEM_DEFAULT_ROUTE.to_string());

    AdminMenuNode {
        id: branch.id.to_string(),
        kind: AdminMenuNodeKind::Branch,
        label: branch.label.to_string(),
        href,
        icon: "▸".to_string(),
        order: children.first().map(|node| node.order).unwrap_or(0),
        active_patterns: children
            .iter()
            .flat_map(|node| node.active_patterns.iter().cloned())
            .collect(),
        permissions_any_of: children
            .iter()
            .flat_map(|node| node.permissions_any_of.iter().cloned())
            .collect(),
        children,
    }
}

fn to_page_snapshot(page: SystemPage) -> AdminMenuNode {
    AdminMenuNode {
        id: page.id.to_string(),
        kind: AdminMenuNodeKind::Page,
        label: page.label.to_string(),
        href: page.route.to_string(),
        icon: page.icon.to_string(),
        order: page.order,
        active_patterns: vec![page.route.to_string()],
        permissions_any_of: page
            .permissions_any_of
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        children: Vec::new(),
    }
}

fn to_node_snapshot(node: RegisteredAdminNode) -> AdminMenuNode {
    AdminMenuNode {
        id: node.id.to_string(),
        kind: to_node_kind(node.kind),
        label: node.label.to_string(),
        href: node.href.to_string(),
        icon: "▸".to_string(),
        order: 0,
        active_patterns: node
            .active_patterns
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        permissions_any_of: node
            .permissions_any_of
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        children: node.children.into_iter().map(to_node_snapshot).collect(),
    }
}

fn to_node_kind(kind: AdminNavigationKind) -> AdminMenuNodeKind {
    match kind {
        AdminNavigationKind::Branch => AdminMenuNodeKind::Branch,
        AdminNavigationKind::Page => AdminMenuNodeKind::Page,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_bridge_exposes_system_domain() {
        let sections = registered_admin_sections();
        let system = sections.iter().find(|section| section.domain_id == "system");

        assert_eq!(
            system.map(|section| section.default_href.as_str()),
            Some("/system/identity/users")
        );
        assert!(
            system
                .map(|section| section
                    .menus
                    .iter()
                    .any(|node| node.href == "/system/audit/events"))
                .unwrap_or(false)
        );
    }

    #[test]
    fn catalog_navigation_exposes_reference_pages_too() {
        let sections = system_admin_sections();
        let system = sections.iter().find(|section| section.domain_id == "system");

        assert_eq!(
            system.map(|section| section.default_href.as_str()),
            Some("/system/identity/users")
        );
        assert!(
            system
                .map(|section| section
                    .menus
                    .iter()
                    .any(|node| node_contains_href(node, "/system/oauth2/clients")))
                .unwrap_or(false)
        );
    }

    #[test]
    fn catalog_navigation_uses_bi_axial_tree_branches() {
        let sections = system_admin_sections();
        let branch_labels = sections[0]
            .menus
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            branch_labels,
            vec!["我的账户", "权限管理", "组织租户", "认证接入", "系统配置", "日志消息"]
        );

        // 关键断言：侧轴不是平铺页面，而是按系统上下文树组织。
        assert!(sections[0].menus.iter().all(|node| !node.children.is_empty()));
    }

    fn node_contains_href(node: &AdminNodeSnapshot, href: &str) -> bool {
        if node.href == href {
            return true;
        }
        node.children
            .iter()
            .any(|child| node_contains_href(child, href))
    }
}
