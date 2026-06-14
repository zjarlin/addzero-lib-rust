use anyhow::Context;
use az_aio_plugin_api::register_native_plugin;
use az_aio_plugin_api::api::{
    ContributionSet, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};

use crate::{
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    page::AssetHubPage,
    routes::{AssetHubApiState, asset_hub_router},
    store::build_asset_hub_module,
};

#[derive(Default)]
pub struct AssetHubPlugin;

impl NativeAzAioPlugin for AssetHubPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let _module = build_asset_hub_module();
        let state = block_on_state(context.database_url.clone())?;
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

register_native_plugin!(AssetHubPlugin);

pub fn ensure_linked() {}

fn block_on_state(database_url: Option<String>) -> anyhow::Result<AssetHubApiState> {
    if database_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        return Ok(AssetHubApiState::degraded(database_url));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("create asset-hub toasty runtime")?;
    runtime
        .block_on(AssetHubApiState::new(database_url.clone()))
        .or_else(|_| Ok(AssetHubApiState::degraded(database_url)))
}

#[cfg(test)]
mod tests {
    use az_aio_plugin_api::api::PluginKind;

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
