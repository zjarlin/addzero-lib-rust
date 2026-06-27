use std::sync::Arc;

use az_aio_platform::plugin::api::{
    AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
    DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::routes::algorithm_center_router,
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    ui::page::AlgorithmCenterPage,
};

#[derive(Default)]
pub struct AlgorithmCenterPlugin;

impl NativePluginProvider for AlgorithmCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: "intelligent-gateway".to_string(),
                label: "智能网关".to_string(),
                default_href: ROUTE.to_string(),
                order: 300,
                menus: vec![AdminMenuNode {
                    id: "algorithm-center.nav".to_string(),
                    kind: AdminMenuNodeKind::Page,
                    label: "算法中心".to_string(),
                    href: ROUTE.to_string(),
                    icon: "◈".to_string(),
                    order: 20,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["read-algorithm-catalog".to_string()],
                    children: Vec::new(),
                }],
            }],
        }
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: AlgorithmCenterPage,
            }],
            router: algorithm_center_router(),
            startup: None,
        })
    }
}

#[Singleton(name = "algorithm-center")]
pub fn algorithm_center_plugin() -> DynAdminPluginProvider {
    Arc::new(AlgorithmCenterPlugin)
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::api::PluginKind;

    use super::*;

    #[test]
    fn descriptor_exposes_native_runtime_contract() {
        let plugin = AlgorithmCenterPlugin;
        let descriptor = plugin.descriptor();
        let contributions = plugin.contributions().unwrap();
        assert_eq!(descriptor.id, "algorithm-center");
        assert_eq!(descriptor.kind, PluginKind::Native);
        assert!(
            contributions
                .pages
                .iter()
                .any(|page| page.route == "/algorithms")
        );
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.path == "/api/algorithm-center/status")
        );
        assert_eq!(
            plugin
                .admin_menu(&contributions)
                .sections
                .first()
                .map(|section| section.label.as_str()),
            Some("智能网关")
        );
    }
}
