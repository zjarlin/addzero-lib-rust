use az_aio_platform::plugin::contract::{
    BackendApiContribution, ClientPageContribution, ContributionSet, NavItemContribution,
    PageContribution, PluginActivation, PluginDescriptor, PluginKind, ToolbarActionContribution,
    UiContribution, UiContributionSlot,
};

pub const PLUGIN_ID: &str = "drive-center";
pub const ROUTE: &str = "/drive";
pub const RENDERER_ID: &str = "drive-center.page";

pub fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: PLUGIN_ID.to_string(),
        name: "网盘中心".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Drive hosting, tracked items, queue, conflicts, and root aliases."
            .to_string(),
        activation: PluginActivation::Eager,
        priority: 900,
        dependencies: Vec::new(),
        capabilities: vec![
            "dioxus-ui-contract-page".to_string(),
            "axum-api".to_string(),
            "toasty-persistence".to_string(),
            "drive-queue".to_string(),
        ],
        permissions: vec![
            "postgres-read-write".to_string(),
            "read hosted paths".to_string(),
        ],
        kind: PluginKind::Native,
    }
}

pub fn contributions() -> ContributionSet {
    ContributionSet {
        nav_items: vec![NavItemContribution {
            id: "drive-center.nav".to_string(),
            label: "网盘".to_string(),
            icon: "⇄".to_string(),
            route: ROUTE.to_string(),
            order: 40,
        }],
        pages: vec![PageContribution {
            route: ROUTE.to_string(),
            title: "网盘中心".to_string(),
            subtitle: "Drive hosting, tracked items, queue, conflicts, and root aliases."
                .to_string(),
            renderer_id: RENDERER_ID.to_string(),
            placeholder_mark: "⇄".to_string(),
            order: 40,
        }],
        client_pages: vec![ClientPageContribution {
            route: ROUTE.to_string(),
            title: "网盘中心".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            slot: UiContributionSlot::Content,
            order: 40,
        }],
        ui_contributions: vec![UiContribution {
            id: "drive-center.ui.content".to_string(),
            slot: UiContributionSlot::Content,
            label: "Drive Center Content".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            route: Some(ROUTE.to_string()),
            order: 10,
        }],
        backend_apis: vec![
            backend_api(
                "drive-center.api.status",
                "GET",
                "/api/drive-center/status",
                "Drive Center Status",
                "Reports runtime, database URL availability, and table prefix.",
                10,
            ),
            backend_api(
                "drive-center.api.tasks",
                "GET",
                "/api/drive-center/tasks",
                "Drive Task List",
                "Lists queued and tracked drive tasks.",
                20,
            ),
            backend_api(
                "drive-center.api.task-enqueue",
                "POST",
                "/api/drive-center/task",
                "Enqueue Drive Task",
                "Adds a drive sync/host/unhost task.",
                30,
            ),
        ],
        toolbar_actions: vec![
            toolbar_action("drive-center.refresh", "Refresh", "RefreshCw", false, 10),
            toolbar_action("drive-center.sync", "Sync", "RefreshCcw", true, 20),
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
