use std::sync::Arc;

use az_aio_platform::{
    admin_scenes::{KNOWLEDGE_BASE_DOMAIN_ID, KNOWLEDGE_BASE_DOMAIN_LABEL},
    plugin::contract::{
        AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
        DynAdminPluginProvider, NativePluginContext, NativePluginProvider, NativePluginRuntime,
        PluginDescriptor,
    },
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{SoftwareCenterApiState, software_center_router},
        store::{SoftwareCenterStore, build_software_center_context_with_db},
    },
    descriptor::{PLUGIN_ID, ROUTE, contributions, descriptor},
    ui::state::install_state,
};

#[derive(Default)]
pub struct SoftwareCenterPlugin;

impl NativePluginProvider for SoftwareCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: KNOWLEDGE_BASE_DOMAIN_ID.to_string(),
                label: KNOWLEDGE_BASE_DOMAIN_LABEL.to_string(),
                default_href: ROUTE.to_string(),
                order: 200,
                menus: vec![AdminMenuNode {
                    id: format!("{}.nav", PLUGIN_ID),
                    kind: AdminMenuNodeKind::Page,
                    label: "软件中心".to_string(),
                    href: ROUTE.to_string(),
                    icon: "⬢".to_string(),
                    order: 30,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: Vec::new(),
                    children: Vec::new(),
                }],
            }],
        }
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let store = context.shared_db.clone().map(|shared_db| {
            let mut plugin_context = build_software_center_context_with_db(shared_db.clone());
            plugin_context.resolve::<SoftwareCenterStore>()
        });
        let state = SoftwareCenterApiState::from_store(context.database_url.clone(), store);
        install_state(state.clone());
        Ok(NativePluginRuntime {
            renderers: Vec::new(),
            router: software_center_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "software-center")]
pub fn software_center_plugin() -> DynAdminPluginProvider {
    Arc::new(SoftwareCenterPlugin)
}


#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::contract::PluginKind;

    use super::*;

    #[test]
    fn descriptor_exposes_native_runtime_contract() {
        let plugin = SoftwareCenterPlugin;
        let descriptor = plugin.descriptor();
        let contributions = plugin.contributions().unwrap();
        assert_eq!(descriptor.id, "software-center");
        assert_eq!(descriptor.kind, PluginKind::Native);
        assert!(contributions.pages.iter().any(|page| page.route == "/software"));
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.path == "/api/software-center/status")
        );
    }
}
