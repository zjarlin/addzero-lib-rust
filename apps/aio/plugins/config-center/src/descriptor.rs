use az_aio_platform::plugin::contract::{
    BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, SettingsDefaultContribution,
    SettingsSectionContribution, ToolbarActionContribution, UiContribution, UiContributionSlot,
};

pub const PLUGIN_ID: &str = "config-center";
pub const ROUTE: &str = "/config";
pub const RENDERER_ID: &str = "config-center.page";

pub fn descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: PLUGIN_ID.to_string(),
        name: "配置中心".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Dotfiles monitor, pairing identity, XDG paths, and provider configuration."
            .to_string(),
        activation: PluginActivation::Eager,
        priority: 910,
        dependencies: Vec::new(),
        capabilities: vec![
            "dioxus-ui-contract-page".to_string(),
            "axum-api".to_string(),
            "toasty-persistence".to_string(),
            "dotfiles-monitor".to_string(),
            "pairing".to_string(),
        ],
        permissions: vec![
            "read-write xdg config".to_string(),
            "read dotfiles root".to_string(),
            "postgres-read-write".to_string(),
        ],
        kind: PluginKind::Native,
    }
}

pub fn contributions() -> ContributionSet {
    ContributionSet {
        nav_items: vec![NavItemContribution {
            id: "config-center.nav".to_string(),
            label: "配置".to_string(),
            icon: "⚙".to_string(),
            route: ROUTE.to_string(),
            order: 30,
        }],
        pages: vec![PageContribution {
            route: ROUTE.to_string(),
            title: "配置中心".to_string(),
            subtitle: "Dotfiles 监控 · 配对身份 · 路径配置".to_string(),
            renderer_id: RENDERER_ID.to_string(),
            placeholder_mark: "⚙".to_string(),
            order: 30,
        }],
        ui_contributions: vec![
            ui_contribution(
                "config-center.ui.content",
                UiContributionSlot::Content,
                "Config Center Content",
                RENDERER_ID,
                Some(ROUTE),
                10,
            ),
            ui_contribution(
                "config-center.ui.settings",
                UiContributionSlot::SettingsContent,
                "Config Center Settings",
                "config-center.settings",
                Some("/settings"),
                20,
            ),
        ],
        backend_apis: vec![
            backend_api(
                "config-center.api.status",
                "GET",
                "/api/config-center/status",
                "Config Center Status",
                "Reports runtime, database URL availability, table prefix, and XDG paths.",
                10,
            ),
            backend_api(
                "config-center.api.dotfiles",
                "GET",
                "/api/config-center/dotfiles",
                "Dotfiles Status",
                "Scans dotfiles status and conflicts.",
                20,
            ),
            backend_api(
                "config-center.api.pairing",
                "GET",
                "/api/config-center/pairing",
                "Pairing Identity",
                "Returns current machine pairing identity.",
                30,
            ),
            backend_api(
                "config-center.api.entries",
                "GET",
                "/api/config-center/entries",
                "Config Entries",
                "Lists persisted config entries by namespace.",
                40,
            ),
            backend_api(
                "config-center.api.entry-upsert",
                "POST",
                "/api/config-center/entry",
                "Save Config Entry",
                "Creates or updates a config entry.",
                50,
            ),
        ],
        toolbar_actions: vec![toolbar_action(
            "config-center.refresh",
            "Refresh",
            "RefreshCw",
            true,
            10,
        )],
        catalog_providers: Vec::new(),
        settings_sections: vec![SettingsSectionContribution {
            id: "config-center.defaults".to_string(),
            label: "Config Center Defaults".to_string(),
            order: 20,
            defaults: vec![SettingsDefaultContribution {
                key: "config-center.database_url".to_string(),
                label: "Database URL".to_string(),
                value: String::new(),
                description: "PostgreSQL URL used by Config Center Toasty store.".to_string(),
                order: 10,
            }],
        }],
        shell_entries: Vec::new(),
        generated_files: Vec::new(),
        ..ContributionSet::default()
    }
}

fn ui_contribution(
    id: &str,
    slot: UiContributionSlot,
    label: &str,
    renderer_id: &str,
    route: Option<&str>,
    order: i32,
) -> UiContribution {
    UiContribution {
        id: id.to_string(),
        slot,
        label: label.to_string(),
        renderer_id: renderer_id.to_string(),
        route: route.map(str::to_string),
        order,
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
