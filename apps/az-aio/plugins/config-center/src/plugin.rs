use std::sync::Arc;

use anyhow::Context;
use az_aio_platform::plugin_api::{
    ContributionSet, DynNativeAzAioPlugin, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{ConfigCenterApiState, config_center_router},
        store::build_config_center_context,
    },
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    ui::page::ConfigCenterPage,
};

#[derive(Default)]
pub struct ConfigCenterPlugin;

impl NativeAzAioPlugin for ConfigCenterPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let _context = build_config_center_context();
        let state = block_on_state(context.database_url.clone())?;
        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.to_string(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.to_string()),
                render: ConfigCenterPage,
            }],
            router: config_center_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "config-center")]
pub fn config_center_plugin() -> DynNativeAzAioPlugin {
    Arc::new(ConfigCenterPlugin)
}

fn block_on_state(database_url: Option<String>) -> anyhow::Result<ConfigCenterApiState> {
    if database_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        return Ok(ConfigCenterApiState::degraded(database_url));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("create config-center toasty runtime")?;
    runtime
        .block_on(ConfigCenterApiState::new(database_url.clone()))
        .or_else(|_| Ok(ConfigCenterApiState::degraded(database_url)))
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin_api::PluginKind;

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
    }
}
