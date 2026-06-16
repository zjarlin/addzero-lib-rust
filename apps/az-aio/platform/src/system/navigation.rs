//! Bridge from compile-time admin registrations into owned platform data.

use az_admin_plugin_registry::api::{
    AdminNavigationKind, RegisteredAdminNode, registered_domains, section_for_path,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminSectionSnapshot {
    pub domain_id: String,
    pub label: String,
    pub default_href: String,
    pub menus: Vec<AdminNodeSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminNodeSnapshot {
    pub id: String,
    pub kind: AdminNodeKind,
    pub label: String,
    pub href: String,
    pub active_patterns: Vec<String>,
    pub permissions_any_of: Vec<String>,
    pub children: Vec<AdminNodeSnapshot>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdminNodeKind {
    Branch,
    Page,
}

pub fn registered_admin_sections() -> Vec<AdminSectionSnapshot> {
    az_system_starters::api::link_all();
    registered_domains()
        .into_iter()
        .filter_map(|domain| {
            section_for_path(domain.default_href).map(|section| AdminSectionSnapshot {
                domain_id: domain.id.to_string(),
                label: section.label.to_string(),
                default_href: section.default_href.to_string(),
                menus: section.menus.into_iter().map(to_node_snapshot).collect(),
            })
        })
        .collect()
}

fn to_node_snapshot(node: RegisteredAdminNode) -> AdminNodeSnapshot {
    AdminNodeSnapshot {
        id: node.id.to_string(),
        kind: to_node_kind(node.kind),
        label: node.label.to_string(),
        href: node.href.to_string(),
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

fn to_node_kind(kind: AdminNavigationKind) -> AdminNodeKind {
    match kind {
        AdminNavigationKind::Branch => AdminNodeKind::Branch,
        AdminNavigationKind::Page => AdminNodeKind::Page,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_bridge_exposes_system_domain() {
        let sections = registered_admin_sections();
        let system = sections
            .iter()
            .find(|section| section.domain_id == "system")
            .expect("system section should be linked by az-system-starters");

        assert_eq!(system.default_href, "/system/identity/users");
        assert!(
            system
                .menus
                .iter()
                .any(|node| node.href == "/system/audit/events")
        );
    }
}
