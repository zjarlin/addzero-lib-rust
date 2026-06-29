use std::sync::Arc;

use az_aio_platform::plugin::api::{
    ContributionSet, DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{SoftwareCenterApiState, software_center_router},
        store::{SoftwareCenterStore, build_software_center_context_with_db},
    },
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    ui::{page::SoftwareCenterPage, state::install_state},
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

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let store = context.shared_db.clone().map(|shared_db| {
            let mut plugin_context = build_software_center_context_with_db(shared_db.clone());
            plugin_context.resolve::<SoftwareCenterStore>()
        });
        let state = SoftwareCenterApiState::from_store(context.database_url.clone(), store);
        install_state(state.clone());
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: SoftwareCenterPage,
            }],
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
    use az_aio_platform::plugin::api::PluginKind;

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
