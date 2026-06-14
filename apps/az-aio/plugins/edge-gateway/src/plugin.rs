use anyhow::Context;
use az_aio_platform::register_native_plugin;
use az_aio_platform::plugin_api::{
    ContributionSet, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};

use crate::{
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    page::EdgeGatewayPage,
    routes::{EdgeGatewayApiState, edge_gateway_router},
    store::build_edge_gateway_module,
};

#[derive(Default)]
pub struct EdgeGatewayPlugin;

impl NativeAzAioPlugin for EdgeGatewayPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let _module = build_edge_gateway_module();
        let state = block_on_state(context.database_url.clone())?;
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: EdgeGatewayPage,
            }],
            router: edge_gateway_router(state),
            startup: None,
        })
    }
}

register_native_plugin!(EdgeGatewayPlugin);

pub fn ensure_linked() {}

fn block_on_state(database_url: Option<String>) -> anyhow::Result<EdgeGatewayApiState> {
    if database_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        return Ok(EdgeGatewayApiState::degraded(database_url));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("create edge-gateway toasty runtime")?;
    runtime
        .block_on(EdgeGatewayApiState::new(database_url.clone()))
        .or_else(|_| Ok(EdgeGatewayApiState::degraded(database_url)))
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin_api::PluginKind;

    use super::*;

    #[test]
    fn descriptor_exposes_native_runtime_contract() {
        let plugin = EdgeGatewayPlugin;
        let descriptor = plugin.descriptor();
        let contributions = plugin.contributions().unwrap();
        assert_eq!(descriptor.id, "edge-gateway");
        assert_eq!(descriptor.kind, PluginKind::Native);
        assert!(contributions.pages.iter().any(|page| page.route == "/gateway"));
        assert!(
            contributions
                .backend_apis
                .iter()
                .any(|api| api.path == "/api/edge-gateway/status")
        );
    }
}
