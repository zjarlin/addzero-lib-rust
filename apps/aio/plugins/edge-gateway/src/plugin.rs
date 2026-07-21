use std::sync::Arc;

use az_aio_platform::plugin::contract::{
    AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminMenuTree, ContributionSet,
    DynAdminPluginProvider, NativePluginProvider, NativePluginContext, NativePluginRuntime,
    PluginDescriptor,
};
use rudi::Singleton;

use crate::{
    backend::{
        routes::{EdgeGatewayApiState, edge_gateway_router},
        store::{EdgeGatewayStore, build_edge_gateway_context_with_db},
    },
    descriptor::{ROUTE, contributions, descriptor},
    ui::state::install_state,
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
        let store = edge_gateway_store(context.shared_db.clone());
        if let Some(store) = &store {
            seed_edge_gateway_store(store.clone());
        }
        let state = EdgeGatewayApiState::from_store(context.database_url.clone(), store);
        install_state(state.clone());
        Ok(NativePluginRuntime {
            renderers: Vec::new(),
            router: edge_gateway_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "edge-gateway")]
pub fn edge_gateway_plugin() -> DynAdminPluginProvider {
    Arc::new(EdgeGatewayPlugin)
}


fn edge_gateway_store(shared_db: Option<az_aio_platform::core::db::Db>) -> Option<EdgeGatewayStore> {
    shared_db.map(|shared_db| {
        let mut plugin_context = build_edge_gateway_context_with_db(shared_db);
        plugin_context.resolve::<EdgeGatewayStore>()
    })
}

fn seed_edge_gateway_store(store: EdgeGatewayStore) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("edge-gateway Toasty startup degraded: {error:#}");
            return;
        }
    };
    if let Err(error) = runtime.block_on(async {
        store.ensure_demo_weather_token().await?;
        store.ensure_builtin_weather_route().await
    }) {
        eprintln!("edge-gateway Toasty startup degraded: {error:#}");
    }
}


#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::contract::PluginKind;

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
