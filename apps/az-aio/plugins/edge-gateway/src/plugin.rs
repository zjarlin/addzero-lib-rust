use std::sync::Arc;

use anyhow::Context;
use az_aio_platform::plugin::api::{
    AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
    DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
    NativeUiRenderer, PluginDescriptor, UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{EdgeGatewayApiState, edge_gateway_router},
        store::build_edge_gateway_context,
    },
    descriptor::{RENDERER_ID, ROUTE, contributions, descriptor},
    ui::{page::EdgeGatewayPage, state::install_state},
};

#[derive(Default)]
pub struct EdgeGatewayPlugin;

impl NativePluginProvider for EdgeGatewayPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        descriptor()
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(contributions())
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: "intelligent-gateway".to_string(),
                label: "智能网关".to_string(),
                default_href: ROUTE.to_string(),
                order: 300,
                menus: vec![AdminMenuNode {
                    id: "edge-gateway.nav".to_string(),
                    kind: AdminMenuNodeKind::Page,
                    label: "网关编排".to_string(),
                    href: ROUTE.to_string(),
                    icon: "↗".to_string(),
                    order: 10,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["outbound-http".to_string()],
                    children: Vec::new(),
                }],
            }],
        }
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let _context = build_edge_gateway_context();
        let state = block_on_state(context.database_url.clone())?;
        install_state(state.clone());
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

#[Singleton(name = "edge-gateway")]
pub fn edge_gateway_plugin() -> DynAdminPluginProvider {
    Arc::new(EdgeGatewayPlugin)
}

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
        .inspect_err(|error| eprintln!("edge-gateway Toasty startup degraded: {error:#}"))
        .or_else(|_| Ok(EdgeGatewayApiState::degraded(database_url)))
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::api::PluginKind;

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
        assert_eq!(
            plugin
                .admin_menu(&contributions)
                .sections
                .first()
                .map(|section| section.label.as_str()),
            Some("智能网关")
        );
    }
}
