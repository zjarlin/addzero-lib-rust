use std::sync::Arc;

use az_aio_platform::plugin::api::{
    ContributionSet, DynNativeAzAioPlugin, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
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

impl NativeAzAioPlugin for AlgorithmCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
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
pub fn algorithm_center_plugin() -> DynNativeAzAioPlugin {
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
    }
}
