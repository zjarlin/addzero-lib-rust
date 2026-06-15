use std::sync::Arc;

use anyhow::Context;
use az_aio_platform::plugin_api::{
    ContributionSet, DynNativeAzAioPlugin, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{SoftwareCenterApiState, software_center_router},
        store::build_software_center_context,
    },
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    ui::page::SoftwareCenterPage,
};

#[derive(Default)]
pub struct SoftwareCenterPlugin;

impl NativeAzAioPlugin for SoftwareCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let _context = build_software_center_context();
        let state = block_on_state(context.database_url.clone())?;
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
pub fn software_center_plugin() -> DynNativeAzAioPlugin {
    Arc::new(SoftwareCenterPlugin)
}

fn block_on_state(database_url: Option<String>) -> anyhow::Result<SoftwareCenterApiState> {
    if database_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        return Ok(SoftwareCenterApiState::degraded(database_url));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("create software-center toasty runtime")?;
    runtime
        .block_on(SoftwareCenterApiState::new(database_url.clone()))
        .or_else(|_| Ok(SoftwareCenterApiState::degraded(database_url)))
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin_api::PluginKind;

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
