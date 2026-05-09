use std::collections::BTreeMap;

pub use inventory;

#[derive(Clone, Copy, Debug)]
pub struct AdminDomainRegistration {
    pub id: &'static str,
    pub label: &'static str,
    pub order: u16,
    pub default_href: &'static str,
}

impl PartialEq for AdminDomainRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.order == other.order
            && self.default_href == other.default_href
    }
}

impl Eq for AdminDomainRegistration {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminNavigationKind {
    Branch,
    Page,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdminNavigationRegistration {
    pub kind: AdminNavigationKind,
    pub id: &'static str,
    pub domain_id: &'static str,
    pub parent_id: Option<&'static str>,
    pub label: &'static str,
    pub order: u16,
    pub href: &'static str,
    pub active_patterns: &'static [&'static str],
    pub permissions_any_of: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredAdminNode {
    pub id: &'static str,
    pub kind: AdminNavigationKind,
    pub label: &'static str,
    pub href: &'static str,
    pub active_patterns: &'static [&'static str],
    pub permissions_any_of: &'static [&'static str],
    pub children: Vec<RegisteredAdminNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredAdminSection {
    pub label: &'static str,
    pub menus: Vec<RegisteredAdminNode>,
}

inventory::collect!(AdminDomainRegistration);
inventory::collect!(AdminNavigationRegistration);

pub fn primary_domain() -> Option<AdminDomainRegistration> {
    registered_domains().into_iter().next()
}

pub fn domain_for_path(path: &str) -> Option<AdminDomainRegistration> {
    let node = active_node(path)?;
    domain_by_id(node.domain_id)
}

pub fn registered_domains() -> Vec<AdminDomainRegistration> {
    let mut domains: Vec<_> = inventory::iter::<AdminDomainRegistration>
        .into_iter()
        .copied()
        .collect();
    domains.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(right.label))
            .then(left.id.cmp(right.id))
    });
    domains.retain(|domain| has_nodes_for_domain(domain.id));
    domains
}

pub fn section_for_path(path: &str) -> Option<RegisteredAdminSection> {
    let domain = domain_for_path(path)?;
    Some(RegisteredAdminSection {
        label: domain.label,
        menus: navigation_tree_for_domain(domain.id),
    })
}

pub fn path_matches_patterns(path: &str, patterns: &[&str]) -> bool {
    let path = normalize_path(path);
    patterns
        .iter()
        .copied()
        .any(|pattern| path_matches_pattern(&path, pattern))
}

fn active_node(path: &str) -> Option<AdminNavigationRegistration> {
    all_nodes()
        .into_iter()
        .find(|node| path_matches_patterns(path, node.active_patterns))
}

fn domain_by_id(domain_id: &str) -> Option<AdminDomainRegistration> {
    inventory::iter::<AdminDomainRegistration>
        .into_iter()
        .copied()
        .find(|domain| domain.id == domain_id)
}

fn all_nodes() -> Vec<AdminNavigationRegistration> {
    let mut nodes: Vec<_> = inventory::iter::<AdminNavigationRegistration>
        .into_iter()
        .copied()
        .filter(|node| domain_by_id(node.domain_id).is_some())
        .collect();
    nodes.sort_by(|left, right| navigation_sort_key(*left).cmp(&navigation_sort_key(*right)));
    nodes
}

fn navigation_tree_for_domain(domain_id: &str) -> Vec<RegisteredAdminNode> {
    let nodes: Vec<_> = all_nodes()
        .into_iter()
        .filter(|node| node.domain_id == domain_id)
        .collect();
    let mut children_by_parent: BTreeMap<Option<&'static str>, Vec<AdminNavigationRegistration>> =
        BTreeMap::new();
    for node in nodes {
        children_by_parent
            .entry(node.parent_id)
            .or_default()
            .push(node);
    }
    build_children(None, &children_by_parent)
}

fn build_children(
    parent_id: Option<&'static str>,
    children_by_parent: &BTreeMap<Option<&'static str>, Vec<AdminNavigationRegistration>>,
) -> Vec<RegisteredAdminNode> {
    let Some(children) = children_by_parent.get(&parent_id) else {
        return Vec::new();
    };

    children
        .iter()
        .copied()
        .map(|node| RegisteredAdminNode {
            id: node.id,
            kind: node.kind,
            label: node.label,
            href: node.href,
            active_patterns: node.active_patterns,
            permissions_any_of: node.permissions_any_of,
            children: build_children(Some(node.id), children_by_parent),
        })
        .collect()
}

fn has_nodes_for_domain(domain_id: &str) -> bool {
    inventory::iter::<AdminNavigationRegistration>
        .into_iter()
        .any(|node| node.domain_id == domain_id)
}

fn navigation_sort_key(
    node: AdminNavigationRegistration,
) -> (
    u16,
    &'static str,
    &'static str,
    Option<&'static str>,
    u16,
    &'static str,
    &'static str,
) {
    let (domain_order, domain_label, domain_key) = domain_sort_key(node.domain_id);
    (
        domain_order,
        domain_label,
        domain_key,
        node.parent_id,
        node.order,
        node.label,
        node.id,
    )
}

fn domain_sort_key(domain_id: &'static str) -> (u16, &'static str, &'static str) {
    if let Some(domain) = domain_by_id(domain_id) {
        (domain.order, domain.label, domain.id)
    } else {
        (u16::MAX, "", domain_id)
    }
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    let path = split_segments(path);
    let pattern = split_segments(pattern);
    if path.len() != pattern.len() {
        return false;
    }

    path.iter()
        .zip(pattern.iter())
        .all(|(segment, matcher)| matcher.starts_with(':') || segment == matcher)
}

fn split_segments(path: &str) -> Vec<String> {
    let normalized = normalize_path(path);
    if normalized == "/" {
        return Vec::new();
    }

    normalized
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_path(path: &str) -> String {
    let without_hash = path.split('#').next().unwrap_or(path);
    let without_query = without_hash.split('?').next().unwrap_or(without_hash);
    let trimmed = without_query.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_string()
    } else {
        format!("/{}", trimmed.trim_matches('/'))
    }
}

#[macro_export]
macro_rules! register_admin_domain {
    (
        id: $id:expr,
        label: $label:expr,
        order: $order:expr,
        default_href: $default_href:expr $(,)?
    ) => {
        $crate::inventory::submit! {
            $crate::AdminDomainRegistration {
                id: $id,
                label: $label,
                order: $order,
                default_href: $default_href,
            }
        }
    };
}

#[macro_export]
macro_rules! register_admin_branch {
    (
        id: $id:expr,
        domain: $domain_id:expr,
        parent: $parent_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr,
        permissions_any_of: $permissions_any_of:expr $(,)?
    ) => {
        $crate::inventory::submit! {
            $crate::AdminNavigationRegistration {
                kind: $crate::AdminNavigationKind::Branch,
                id: $id,
                domain_id: $domain_id,
                parent_id: $parent_id,
                label: $label,
                order: $order,
                href: $href,
                active_patterns: $active_patterns,
                permissions_any_of: $permissions_any_of,
            }
        }
    };
}

#[macro_export]
macro_rules! register_admin_page {
    (
        id: $id:expr,
        domain: $domain_id:expr,
        parent: $parent_id:expr,
        label: $label:expr,
        order: $order:expr,
        href: $href:expr,
        active_patterns: $active_patterns:expr,
        permissions_any_of: $permissions_any_of:expr $(,)?
    ) => {
        $crate::inventory::submit! {
            $crate::AdminNavigationRegistration {
                kind: $crate::AdminNavigationKind::Page,
                id: $id,
                domain_id: $domain_id,
                parent_id: $parent_id,
                label: $label,
                order: $order,
                href: $href,
                active_patterns: $active_patterns,
                permissions_any_of: $permissions_any_of,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{
        AdminNavigationKind, normalize_path, path_matches_patterns, registered_domains,
        section_for_path,
    };

    const OVERVIEW_DOMAIN_ID: &str = "test-overview";
    const SYSTEM_DOMAIN_ID: &str = "test-system";
    const KNOWLEDGE_DOMAIN_ID: &str = "test-knowledge";
    const CLI_MARKET_NODE_ID: &str = "test-cli-market";

    crate::register_admin_domain! {
        id: KNOWLEDGE_DOMAIN_ID,
        label: "Knowledge",
        order: 20,
        default_href: "/knowledge/notes",
    }

    crate::register_admin_domain! {
        id: OVERVIEW_DOMAIN_ID,
        label: "Overview",
        order: 10,
        default_href: "/",
    }

    crate::register_admin_domain! {
        id: SYSTEM_DOMAIN_ID,
        label: "System",
        order: 30,
        default_href: "/system/users",
    }

    crate::register_admin_page! {
        id: "test-home",
        domain: OVERVIEW_DOMAIN_ID,
        parent: None,
        label: "Home",
        order: 10,
        href: "/",
        active_patterns: &["/", "/dashboard"],
        permissions_any_of: &[],
    }

    crate::register_admin_page! {
        id: "test-notes",
        domain: KNOWLEDGE_DOMAIN_ID,
        parent: None,
        label: "Notes",
        order: 10,
        href: "/knowledge/notes",
        active_patterns: &["/knowledge/notes"],
        permissions_any_of: &["knowledge:note"],
    }

    crate::register_admin_branch! {
        id: CLI_MARKET_NODE_ID,
        domain: KNOWLEDGE_DOMAIN_ID,
        parent: None,
        label: "CLI Market",
        order: 20,
        href: "/knowledge/cli-market",
        active_patterns: &[
            "/knowledge/cli-market",
            "/knowledge/cli-market/imports",
            "/knowledge/cli-market/docs",
        ],
        permissions_any_of: &["knowledge:cli"],
    }

    crate::register_admin_page! {
        id: "test-cli-market-imports",
        domain: KNOWLEDGE_DOMAIN_ID,
        parent: Some(CLI_MARKET_NODE_ID),
        label: "Imports",
        order: 10,
        href: "/knowledge/cli-market/imports",
        active_patterns: &["/knowledge/cli-market/imports"],
        permissions_any_of: &["knowledge:cli"],
    }

    crate::register_admin_page! {
        id: "test-cli-market-docs",
        domain: KNOWLEDGE_DOMAIN_ID,
        parent: Some(CLI_MARKET_NODE_ID),
        label: "Docs",
        order: 20,
        href: "/knowledge/cli-market/docs",
        active_patterns: &["/knowledge/cli-market/docs"],
        permissions_any_of: &["knowledge:cli"],
    }

    crate::register_admin_page! {
        id: "test-system-users",
        domain: SYSTEM_DOMAIN_ID,
        parent: None,
        label: "Users",
        order: 10,
        href: "/system/users",
        active_patterns: &["/system/users/:id", "/system/users"],
        permissions_any_of: &["system:user"],
    }

    #[test]
    fn path_matching_should_support_dynamic_segments() {
        assert!(path_matches_patterns("/agents/demo", &["/agents/:name"]));
        assert!(path_matches_patterns(
            "/api/admin/system/users/42",
            &["/api/admin/system/users/:id"]
        ));
        assert!(!path_matches_patterns(
            "/agents/demo/edit",
            &["/agents/:name"]
        ));
    }

    #[test]
    fn normalize_path_should_strip_query_and_trailing_slash() {
        assert_eq!(normalize_path("/files/?tab=recent"), "/files");
        assert_eq!(normalize_path(" / "), "/");
    }

    #[test]
    fn registered_domains_should_follow_order_and_drop_empty_domains() {
        let ids: Vec<_> = registered_domains()
            .into_iter()
            .map(|domain| domain.id)
            .collect();

        assert_eq!(
            ids,
            vec![OVERVIEW_DOMAIN_ID, KNOWLEDGE_DOMAIN_ID, SYSTEM_DOMAIN_ID]
        );
    }

    #[test]
    fn section_for_path_should_build_tree_and_preserve_permissions() {
        let section = section_for_path("/knowledge/cli-market/imports")
            .expect("cli market section should exist");
        let labels: Vec<_> = section.menus.iter().map(|menu| menu.label).collect();
        let cli_market = section
            .menus
            .iter()
            .find(|menu| menu.id == CLI_MARKET_NODE_ID)
            .expect("cli market branch");
        let child_labels: Vec<_> = cli_market.children.iter().map(|menu| menu.label).collect();

        assert_eq!(section.label, "Knowledge");
        assert_eq!(labels, vec!["Notes", "CLI Market"]);
        assert_eq!(cli_market.kind, AdminNavigationKind::Branch);
        assert_eq!(cli_market.permissions_any_of, &["knowledge:cli"]);
        assert_eq!(child_labels, vec!["Imports", "Docs"]);
        assert_eq!(cli_market.children[0].kind, AdminNavigationKind::Page);
        assert_eq!(
            cli_market.children[0].permissions_any_of,
            &["knowledge:cli"]
        );
    }
}
