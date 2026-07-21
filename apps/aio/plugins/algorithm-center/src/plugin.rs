use std::sync::Arc;

use az_aio_platform::plugin::contract::{
    AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
    DynAdminPluginProvider, NativePluginContext, NativePluginProvider, NativePluginRuntime,
    PluginDescriptor,
};
use az_algorithm::di::{create_algorithm_context, resolve_algorithm_catalog};
use rudi::Singleton;

use crate::{
    backend::routes::{AlgorithmCenterApiState, algorithm_center_router},
    descriptor::{ROUTE, contributions, descriptor},
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
        let mut algorithm_context = create_algorithm_context();
        let catalog = resolve_algorithm_catalog(&mut algorithm_context)?;
        Ok(NativePluginRuntime {
            renderers: Vec::new(),
            router: algorithm_center_router(AlgorithmCenterApiState { catalog }),
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
    use az_aio_platform::plugin::contract::PluginKind;

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
