use az_aio_platform::plugin::api::{
    BackendApiContribution, ClientPageContribution, ContributionSet, NavItemContribution,
    PageContribution, PluginActivation, PluginDescriptor, PluginKind, ToolbarActionContribution,
    UiContribution, UiContributionSlot,
};

pub const PLUGIN_ID: &str = "software-center";
pub const ROUTE: &str = "/software";
pub const RENDERER_ID: &str = "software-center.page";

pub fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: PLUGIN_ID.to_string(),
        name: "软件中心".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description:
            "Installer scan, organize/archive, and catalog-linked package detail surfaces."
                .to_string(),
        activation: PluginActivation::Eager,
        priority: 880,
        dependencies: Vec::new(),
        capabilities: vec![
            "dioxus-ui-contract-page".to_string(),
            "axum-api".to_string(),
            "toasty-persistence".to_string(),
            "installer-scan".to_string(),
        ],
        permissions: vec![
            "read Downloads and Desktop".to_string(),
            "write installer archive".to_string(),
            "postgres-read-write".to_string(),
        ],
        kind: PluginKind::Native,
    }
}

pub fn contributions() -> ContributionSet {
    ContributionSet {
        nav_items: vec![NavItemContribution {
            id: "software-center.nav".to_string(),
            label: "软件".to_string(),
            icon: "⬢".to_string(),
            route: ROUTE.to_string(),
            order: 60,
        }],
        pages: vec![PageContribution {
            route: ROUTE.to_string(),
            title: "软件中心".to_string(),
            subtitle: "安装包扫描 · 归档目标 · 目录关联".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            placeholder_mark: "⬢".to_string(),
            order: 60,
        }],
        client_pages: vec![ClientPageContribution {
            route: ROUTE.to_string(),
            title: "软件中心".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            slot: UiContributionSlot::Content,
            order: 60,
        }],
        ui_contributions: vec![UiContribution {
            id: "software-center.ui.content".to_string(),
            slot: UiContributionSlot::Content,
            label: "Software Center Content".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            route: Some(ROUTE.to_string()),
            order: 10,
        }],
        backend_apis: vec![
            backend_api(
                "software-center.api.status",
                "GET",
                "/api/software-center/status",
                "Software Center Status",
                "Reports runtime, database URL availability, and table prefix.",
                10,
            ),
            backend_api(
                "software-center.api.installers",
                "GET",
                "/api/software-center/installers",
                "Installer Scan",
                "Scans Downloads and Desktop for installer packages.",
                20,
            ),
            backend_api(
                "software-center.api.organize",
                "POST",
                "/api/software-center/organize",
                "Organize Installers",
                "Archives detected installers into Software Center storage.",
                30,
            ),
            backend_api(
                "software-center.api.packages",
                "GET",
                "/api/software-center/packages",
                "Software Packages",
                "Lists persisted software package records.",
                40,
            ),
            backend_api(
                "software-center.api.package-upsert",
                "POST",
                "/api/software-center/package",
                "Save Software Package",
                "Creates or updates a software package record.",
                50,
            ),
        ],
        toolbar_actions: vec![
            toolbar_action("software-center.refresh", "Refresh", "RefreshCw", false, 10),
            toolbar_action("software-center.scan", "Scan", "ScanSearch", true, 20),
            toolbar_action("software-center.organize", "Organize", "Archive", false, 30),
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
