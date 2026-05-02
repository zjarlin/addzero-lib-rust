use addzero_admin_plugin_registry as registry;
pub use addzero_admin_plugin_registry::AdminDomainRegistration;
use dioxus_components::{AdminMenu, AdminSection};

use crate::app::Route;
use crate::state::PermissionState;

pub fn primary_domain() -> Option<AdminDomainRegistration> {
    registry::primary_domain()
}

pub fn domain_for_route(route: &Route) -> Option<AdminDomainRegistration> {
    registry::domain_for_path(route.to_string().as_str())
}

pub fn registered_domains() -> Vec<AdminDomainRegistration> {
    registry::registered_domains()
}

pub fn section_for_route(
    route: &Route,
    permissions: PermissionState,
) -> Option<AdminSection<Route>> {
    let section = registry::section_for_path(route.to_string().as_str())?;
    let menus = section
        .menus
        .into_iter()
        .filter_map(|menu| build_menu(menu, permissions))
        .collect();

    Some(AdminSection {
        label: section.label.to_string(),
        menus,
    })
}

fn build_menu(
    node: addzero_admin_plugin_registry::RegisteredAdminNode,
    permissions: PermissionState,
) -> Option<AdminMenu<Route>> {
    if !node_visible(node.permissions_any_of, permissions) {
        return None;
    }

    let children = node
        .children
        .into_iter()
        .filter_map(|child| build_menu(child, permissions))
        .collect::<Vec<_>>();
    let patterns = node.active_patterns;
    let is_active = move |route: &Route| {
        registry::path_matches_patterns(route.to_string().as_str(), patterns)
    };

    if children.is_empty() {
        let to = node.href.parse::<Route>().ok()?;
        Some(AdminMenu::leaf(node.label, to, is_active))
    } else {
        let to = node.href.parse::<Route>().ok();
        Some(AdminMenu::branch(node.label, to, children, is_active))
    }
}

fn node_visible(permissions_any_of: &[&str], permissions: PermissionState) -> bool {
    permissions_any_of.is_empty()
        || permissions_any_of
            .iter()
            .copied()
            .any(|code| permissions.has(code))
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::Signal;

    use super::{domain_for_route, registered_domains, section_for_route};
    use crate::app::Route;
    use crate::state::PermissionState;

    fn permissions_with(codes: Option<Vec<&str>>) -> PermissionState {
        let permissions = PermissionState {
            codes: Signal::new(None),
        };
        let value = codes.map(|codes| Some(codes.into_iter().map(str::to_string).collect()));
        permissions.codes.set(Some(value));
        permissions
    }

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
            domain_for_route(&Route::Files).expect("knowledge domain").id,
            "knowledge"
        );
        assert_eq!(
            domain_for_route(&Route::SystemUsers).expect("system domain").id,
            "system"
        );
        assert_eq!(
            domain_for_route(&Route::Audit).expect("audit domain").id,
            "audit"
        );
    }

    #[test]
    fn section_for_route_should_build_cli_market_tree() {
        let permissions = permissions_with(None);
        let section = section_for_route(&Route::KnowledgeCliMarketImports, permissions)
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
        let knowledge_download = permissions_with(Some(vec!["knowledge:dl"]));
        let section = section_for_route(&Route::KnowledgePackages, knowledge_download)
            .expect("knowledge section");
        let labels: Vec<_> = section
            .menus
            .iter()
            .map(|menu| menu.label.as_str())
            .collect();

        assert_eq!(labels, vec!["下载与安装"]);
    }
}
