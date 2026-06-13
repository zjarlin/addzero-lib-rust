use az_aio_plugin_api::{
    BackendApiContribution, ContributionSet, PluginSandboxDebugReport,
    SettingsDefaultContribution, SettingsSectionContribution, UiContribution,
    UiContributionSlot,
};

#[test]
fn sandbox_debug_report_extracts_ui_api_and_settings_defaults() {
    let contributions = ContributionSet {
        ui_contributions: vec![UiContribution {
            id: "projects.ui.sidebar".to_string(),
            slot: UiContributionSlot::ProjectSidebar,
            label: "项目侧边栏".to_string(),
            renderer_id: "projects.sidebar".to_string(),
            route: Some("/projects".to_string()),
            order: 10,
        }],
        backend_apis: vec![BackendApiContribution {
            id: "projects.api.list".to_string(),
            method: "GET".to_string(),
            path: "/api/projects".to_string(),
            label: "项目列表".to_string(),
            description: "列出已绑定项目。".to_string(),
            order: 10,
        }],
        settings_sections: vec![SettingsSectionContribution {
            id: "settings.project-defaults".to_string(),
            label: "项目默认目录".to_string(),
            order: 10,
            defaults: vec![SettingsDefaultContribution {
                key: "projects.default_sync_root".to_string(),
                label: "默认同步根目录".to_string(),
                value: "az-sync/workspace".to_string(),
                description: "项目插件扫描和绑定时使用的默认同步根目录。".to_string(),
                order: 10,
            }],
        }],
        ..ContributionSet::default()
    };

    let report = PluginSandboxDebugReport::from_contributions(&contributions);

    assert_eq!(report.ui_contributions[0].slot_label, "项目侧边栏");
    assert_eq!(report.backend_apis[0].request_hint, "GET /api/projects");
    assert_eq!(report.settings_defaults[0].value, "az-sync/workspace");
}
