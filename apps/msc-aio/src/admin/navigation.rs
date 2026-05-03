use addzero_admin_plugin_registry as registry;
pub use addzero_admin_plugin_registry::AdminDomainRegistration;
use dioxus::prelude::ReadableExt;
use dioxus_components::{AdminMenu, AdminSection};

use crate::app::Route;
use crate::state::PermissionState;

pub fn primary_domain() -> Option<AdminDomainRegistration> {
    ensure_registry_linked();
    registry::primary_domain()
}

pub fn domain_for_route(route: &Route) -> Option<AdminDomainRegistration> {
    ensure_registry_linked();
    registry::domain_for_path(route.to_string().as_str())
}

pub fn registered_domains() -> Vec<AdminDomainRegistration> {
    ensure_registry_linked();
    registry::registered_domains()
}

pub fn section_for_route(
    route: &Route,
    permissions: PermissionState,
) -> Option<AdminSection<Route>> {
    ensure_registry_linked();
    let permission_snapshot = permissions.codes.read().clone();
    section_for_route_with_snapshot(route, &permission_snapshot)
}

fn section_for_route_with_snapshot(
    route: &Route,
    permission_snapshot: &Option<Option<Vec<String>>>,
) -> Option<AdminSection<Route>> {
    let section = registry::section_for_path(route.to_string().as_str())?;
    let menus = section
        .menus
        .into_iter()
        .filter_map(|menu| build_menu(menu, permission_snapshot))
        .collect();

    Some(AdminSection {
        label: section.label.to_string(),
        menus,
    })
}

fn build_menu(
    node: addzero_admin_plugin_registry::RegisteredAdminNode,
    permission_snapshot: &Option<Option<Vec<String>>>,
) -> Option<AdminMenu<Route>> {
    if !node_visible(node.permissions_any_of, permission_snapshot) {
        return None;
    }

    let children = node
        .children
        .into_iter()
        .filter_map(|child| build_menu(child, permission_snapshot))
        .collect::<Vec<_>>();
    let patterns = node.active_patterns;
    let is_active =
        move |route: &Route| registry::path_matches_patterns(route.to_string().as_str(), patterns);

    if children.is_empty() {
        let to = node.href.parse::<Route>().ok()?;
        Some(AdminMenu::leaf(node.label, to, is_active))
    } else {
        let to = node.href.parse::<Route>().ok();
        Some(AdminMenu::branch(node.label, to, children, is_active))
    }
}

fn node_visible(
    permissions_any_of: &[&str],
    permission_snapshot: &Option<Option<Vec<String>>>,
) -> bool {
    if permissions_any_of.is_empty() {
        return true;
    }

    match permission_snapshot {
        None => false,
        Some(None) => true,
        Some(Some(codes)) => permissions_any_of
            .iter()
            .copied()
            .any(|code| codes.iter().any(|item| item == code)),
    }
}

fn ensure_registry_linked() {
    addzero_admin_domain_audit::ensure_linked();
}

#[cfg(test)]
mod tests {
    use super::{domain_for_route, registered_domains, section_for_route_with_snapshot};
    use crate::app::Route;

    #[test]
    fn registered_domains_should_follow_plugin_order() {
        let ids: Vec<_> = registered_domains()
            .into_iter()
            .map(|domain| domain.id)
            .collect();

        assert_eq!(
            ids,
            vec!["overview", "agents", "chat", "knowledge", "system", "audit"]
        );
    }

    #[test]
    fn domain_for_route_should_resolve_primary_axes() {
        assert_eq!(
            domain_for_route(&Route::Home).expect("overview domain").id,
            "overview"
        );
        assert_eq!(
            domain_for_route(&Route::Dashboard)
                .expect("overview domain")
                .id,
            "overview"
        );
        assert_eq!(
            domain_for_route(&Route::Agents).expect("agents domain").id,
            "agents"
        );
        assert_eq!(
            domain_for_route(&Route::AgentEditor {
                name: "demo".to_string(),
            })
            .expect("agents domain")
            .id,
            "agents"
        );
        assert_eq!(
            domain_for_route(&Route::Chat).expect("chat domain").id,
            "chat"
        );
        assert_eq!(
            domain_for_route(&Route::DownloadStation)
                .expect("knowledge domain")
                .id,
            "knowledge"
        );
        assert_eq!(
            domain_for_route(&Route::Files)
                .expect("knowledge domain")
                .id,
            "knowledge"
        );
        assert_eq!(
            domain_for_route(&Route::SystemUsers)
                .expect("system domain")
                .id,
            "system"
        );
        assert_eq!(
            domain_for_route(&Route::Audit).expect("audit domain").id,
            "audit"
        );
    }

    #[test]
    fn section_for_route_should_build_cli_market_tree() {
        let section =
            section_for_route_with_snapshot(&Route::KnowledgeCliMarketImports, &Some(None))
                .expect("knowledge section");
        let labels: Vec<_> = section
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect();
        let cli_market = section
            .menus
            .iter()
            .find(|menu| menu.label == "CLI 市场")
            .expect("cli market branch");
        let child_labels: Vec<_> = cli_market
            .children
            .iter()
            .map(|menu| menu.label.as_str())
            .collect();

        assert_eq!(section.label, "知识库");
        assert_eq!(labels, vec!["笔记", "下载与安装", "CLI 市场", "下载站"]);
        assert!((cli_market.is_active)(&Route::KnowledgeCliMarketDocs));
        assert_eq!(child_labels, vec!["注册表", "导入任务", "CLI 文档"]);
        assert!((cli_market.children[1].is_active)(
            &Route::KnowledgeCliMarketImports
        ));
    }

    #[test]
    fn section_for_route_should_filter_by_permission_metadata() {
        let section = section_for_route_with_snapshot(
            &Route::KnowledgePackages,
            &Some(Some(vec!["knowledge:pkg".to_string()])),
        )
        .expect("knowledge section");
        let labels: Vec<_> = section
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect();

        assert_eq!(labels, vec!["下载与安装"]);
    }
}
