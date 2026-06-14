use az_aio_platform::register_native_plugin;
use az_aio_platform::plugin_api::{
    ContributionSet, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};

use crate::{
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    page::AlgorithmCenterPage,
    routes::algorithm_center_router,
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

register_native_plugin!(AlgorithmCenterPlugin);

pub fn ensure_linked() {}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin_api::PluginKind;

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
