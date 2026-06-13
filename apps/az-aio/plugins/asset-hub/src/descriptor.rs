use az_aio_plugin_api::{
    BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, ToolbarActionContribution, UiContribution,
    UiContributionSlot,
};

pub const PLUGIN_ID: &str = "asset-hub";
pub const ROUTE: &str = "/assets";
pub const RENDERER_ID: &str = "asset-hub.page";

pub fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: PLUGIN_ID.to_string(),
        name: "Asset Hub".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Asset feed, skill scan, compose assets, and subtype metadata.".to_string(),
        activation: PluginActivation::Eager,
        priority: 920,
        dependencies: Vec::new(),
        capabilities: vec![
            "dioxus-renderer".to_string(),
            "axum-api".to_string(),
            "toasty-persistence".to_string(),
            "skill-scan".to_string(),
        ],
        permissions: vec![
            "read ~/.agents/skills".to_string(),
            "postgres-read-write".to_string(),
        ],
        kind: PluginKind::Native,
    }
}

pub fn contributions() -> ContributionSet {
    ContributionSet {
        nav_items: vec![NavItemContribution {
            id: "asset-hub.nav".to_string(),
            label: "Assets".to_string(),
            icon: "◆".to_string(),
            route: ROUTE.to_string(),
            order: 20,
        }],
        pages: vec![PageContribution {
            route: ROUTE.to_string(),
            title: "Asset Hub".to_string(),
            subtitle: "Asset feed, skill scan, and subtype-backed metadata.".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            placeholder_mark: "◆".to_string(),
            order: 20,
        }],
        ui_contributions: vec![UiContribution {
            id: "asset-hub.ui.content".to_string(),
            slot: UiContributionSlot::Content,
            label: "Asset Hub Content".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            route: Some(ROUTE.to_string()),
            order: 10,
        }],
        backend_apis: vec![
            backend_api(
                "asset-hub.api.status",
                "GET",
                "/api/asset-hub/status",
                "Asset Hub Status",
                "Reports runtime, database URL availability, and table prefix.",
                10,
            ),
            backend_api(
                "asset-hub.api.skills",
                "GET",
                "/api/asset-hub/skills",
                "Scanned Skills",
                "Scans skill directories and returns skill assets.",
                20,
            ),
            backend_api(
                "asset-hub.api.assets",
                "GET",
                "/api/asset-hub/assets",
                "Asset List",
                "Lists persisted Asset Hub records.",
                30,
            ),
            backend_api(
                "asset-hub.api.asset-upsert",
                "POST",
                "/api/asset-hub/asset",
                "Save Asset",
                "Creates or updates one Asset Hub record.",
                40,
            ),
        ],
        toolbar_actions: vec![
            toolbar_action("asset-hub.refresh", "Refresh", "RefreshCw", false, 10),
            toolbar_action("asset-hub.scan-skills", "Scan Skills", "ScanSearch", true, 20),
        ],
        catalog_providers: Vec::new(),
        settings_sections: Vec::new(),
        shell_entries: Vec::new(),
        generated_files: Vec::new(),
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
