use az_aio_platform::plugin::contract::{
    BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, ToolbarActionContribution, UiContribution,
    UiContributionSlot,
};

pub const PLUGIN_ID: &str = "edge-gateway";
pub const ROUTE: &str = "/gateway";
pub const RENDERER_ID: &str = "edge-gateway.page";

pub fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: PLUGIN_ID.to_string(),
        name: "边缘网关".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Gateway flow editor, plan generation, runtime execution, and helper references."
            .to_string(),
        activation: PluginActivation::Eager,
        priority: 890,
        dependencies: Vec::new(),
        capabilities: vec![
            "dioxus-ui-contract-page".to_string(),
            "axum-api".to_string(),
            "toasty-persistence".to_string(),
            "gateway-runtime".to_string(),
            "callable-assets".to_string(),
            "bearer-token-auth".to_string(),
        ],
        permissions: vec![
            "outbound-http".to_string(),
            "postgres-read-write".to_string(),
        ],
        kind: PluginKind::Native,
    }
}

pub fn contributions() -> ContributionSet {
    ContributionSet {
        nav_items: vec![NavItemContribution {
            id: "edge-gateway.nav".to_string(),
            label: "网关".to_string(),
            icon: "↗".to_string(),
            route: ROUTE.to_string(),
            order: 50,
        }],
        pages: vec![PageContribution {
            route: ROUTE.to_string(),
            title: "边缘网关".to_string(),
            subtitle: "网关流模板 · 运行时执行 · 结果面板".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            placeholder_mark: "↗".to_string(),
            order: 50,
        }],
        ui_contributions: vec![UiContribution {
            id: "edge-gateway.ui.content".to_string(),
            slot: UiContributionSlot::Content,
            label: "Edge Gateway Content".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            route: Some(ROUTE.to_string()),
            order: 10,
        }],
        backend_apis: vec![
            backend_api(
                "edge-gateway.api.status",
                "GET",
                "/api/edge-gateway/status",
                "Edge Gateway Status",
                "Reports runtime, database URL availability, and table prefix.",
                10,
            ),
            backend_api(
                "edge-gateway.api.example",
                "GET",
                "/api/edge-gateway/example",
                "Gateway Example Plan",
                "Returns a reference gateway plan.",
                20,
            ),
            backend_api(
                "edge-gateway.api.run",
                "POST",
                "/api/edge-gateway/run",
                "Run Gateway Plan",
                "Executes a gateway plan with the local runtime.",
                30,
            ),
            backend_api(
                "edge-gateway.api.assets",
                "GET",
                "/api/edge-gateway/assets",
                "Callable Edge Assets",
                "Lists token-gated callable edge assets exposed by the gateway.",
                40,
            ),
            backend_api(
                "edge-gateway.api.weather-current",
                "POST",
                "/api/edge-gateway/assets/weather/current",
                "Weather Current Asset",
                "Queries current weather through a bearer-token-gated edge asset.",
                45,
            ),
            backend_api(
                "edge-gateway.api.asset-usage",
                "GET",
                "/api/edge-gateway/assets/usage",
                "Edge Asset Usage",
                "Lists persisted usage records for callable edge assets, with in-memory degraded fallback.",
                46,
            ),
            backend_api(
                "edge-gateway.api.routes",
                "GET",
                "/api/edge-gateway/routes",
                "Managed API Routes",
                "Lists Toasty-backed GET/POST route and script definitions.",
                47,
            ),
            backend_api(
                "edge-gateway.api.route-upsert",
                "POST",
                "/api/edge-gateway/route",
                "Save Managed API Route",
                "Creates or updates a Toasty-backed route definition and script draft.",
                48,
            ),
            backend_api(
                "edge-gateway.api.flows",
                "GET",
                "/api/edge-gateway/flows",
                "Gateway Flows",
                "Lists persisted gateway flow metadata.",
                50,
            ),
            backend_api(
                "edge-gateway.api.flow-upsert",
                "POST",
                "/api/edge-gateway/flow",
                "Save Gateway Flow",
                "Creates or updates gateway flow metadata.",
                60,
            ),
        ],
        toolbar_actions: vec![
            toolbar_action("edge-gateway.refresh", "Refresh", "RefreshCw", false, 10),
            toolbar_action("edge-gateway.run-example", "Run Example", "Play", true, 20),
        ],
        catalog_providers: Vec::new(),
        settings_sections: Vec::new(),
        shell_entries: Vec::new(),
        generated_files: Vec::new(),
        ..ContributionSet::default()
    }
}

fn backend_api(
    id: &str,
    method: &str,
    path: &str,
    label: &str,
    description: &str,
    order: i32,
) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        order,
    }
}

fn toolbar_action(
    id: &str,
    label: &str,
    icon: &str,
    primary: bool,
    order: i32,
) -> ToolbarActionContribution {
    ToolbarActionContribution {
        id: id.to_string(),
        route: Some(ROUTE.to_string()),
        label: label.to_string(),
        icon: icon.to_string(),
        primary,
        order,
    }
}
