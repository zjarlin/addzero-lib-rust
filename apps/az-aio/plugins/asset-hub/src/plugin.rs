use std::sync::Arc;

use az_aio_platform::plugin::api::{
    ContributionSet, DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{AssetHubApiState, asset_hub_router},
        store::{AssetHubStore, build_asset_hub_context_with_db},
    },
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    ui::{page::AssetHubPage, state::install_state},
};

#[derive(Default)]
pub struct AssetHubPlugin;

impl NativePluginProvider for AssetHubPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let store = context.shared_db.clone().map(|shared_db| {
            let mut plugin_context = build_asset_hub_context_with_db(shared_db.clone());
            plugin_context.resolve::<AssetHubStore>()
        });
        let state = AssetHubApiState::from_store(context.database_url.clone(), store);
        install_state(state.clone());
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: AssetHubPage,
            }],
            router: asset_hub_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "asset-hub")]
pub fn asset_hub_plugin() -> DynAdminPluginProvider {
    Arc::new(AssetHubPlugin)
}


#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::api::PluginKind;

    use super::*;

    #[test]
    fn descriptor_exposes_native_runtime_contract() {
        let plugin = AssetHubPlugin;
        let descriptor = plugin.descriptor();
        let contributions = plugin.contributions().unwrap();
        assert_eq!(descriptor.id, "asset-hub");
        assert_eq!(descriptor.kind, PluginKind::Native);
        assert!(contributions.pages.iter().any(|page| page.route == "/assets"));
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.path == "/api/asset-hub/status")
        );
    }
}
