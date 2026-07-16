use std::sync::Arc;

use az_aio_platform::{
    plugin::api::{
        AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
        DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
        PluginDescriptor,
    },
    system::catalog::{SYSTEM_DOMAIN_ID, SYSTEM_DOMAIN_LABEL},
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{ConfigCenterApiState, config_center_router},
        store::{ConfigCenterStore, build_config_center_context_with_db},
    },
    descriptor::{ROUTE, contributions, descriptor},
    ui::state::install_state,
};

#[derive(Default)]
pub struct ConfigCenterPlugin;

impl NativePluginProvider for ConfigCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: SYSTEM_DOMAIN_ID.to_string(),
                label: SYSTEM_DOMAIN_LABEL.to_string(),
                default_href: String::new(),
                order: 900,
                menus: vec![AdminMenuNode {
                    id: "system-config-axis".to_string(),
                    kind: AdminMenuNodeKind::Branch,
                    label: "系统配置".to_string(),
                    href: ROUTE.to_string(),
                    icon: "▸".to_string(),
                    order: 30,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["config-center:read".to_string()],
                    children: vec![AdminMenuNode {
                        id: "config-center.nav".to_string(),
                        kind: AdminMenuNodeKind::Page,
                        label: "配置中心".to_string(),
                        href: ROUTE.to_string(),
                        icon: "⚙".to_string(),
                        order: 25,
                        active_patterns: vec![ROUTE.to_string()],
                        permissions_any_of: vec!["config-center:read".to_string()],
                        children: Vec::new(),
                    }],
                }],
            }],
        }
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let store = context.shared_db.clone().map(|shared_db| {
            let mut plugin_context = build_config_center_context_with_db(shared_db.clone());
            plugin_context.resolve::<ConfigCenterStore>()
        });
        let state = ConfigCenterApiState::from_store(context.database_url.clone(), store);
        install_state(state.clone());
        Ok(NativePluginRuntime {
            renderers: Vec::new(),
            router: config_center_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "config-center")]
pub fn config_center_plugin() -> DynAdminPluginProvider {
    Arc::new(ConfigCenterPlugin)
}


#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::api::PluginKind;

    use super::*;

    #[test]
    fn descriptor_exposes_native_runtime_contract() {
        let plugin = ConfigCenterPlugin;
        let descriptor = plugin.descriptor();
        let contributions = plugin.contributions().unwrap();
        assert_eq!(descriptor.id, "config-center");
        assert_eq!(descriptor.kind, PluginKind::Native);
        assert!(contributions.pages.iter().any(|page| page.route == "/config"));
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.path == "/api/config-center/status")
        );
        assert!(
            plugin
                .admin_menu(&contributions)
                .sections
                .iter()
                .any(|section| section
                    .menus
                    .iter()
                    .any(|node| node.children.iter().any(|child| child.href == "/config")))
        );
    }
}
