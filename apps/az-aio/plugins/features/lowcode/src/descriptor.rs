use az_aio_plugin_api::{
    AzAioPlugin, BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, SettingsDefaultContribution,
    SettingsSectionContribution, UiContribution, UiContributionSlot,
};

const DATABASE_SETTING_KEY: &str = "lowcode.database_url";
const DATABASE_NAMESPACE: &str = "az-aio.dev";

#[derive(Default)]
pub struct LowcodePlugin;

impl AzAioPlugin for LowcodePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "lowcode".to_string(),
            name: "低代码".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供基于 Toasty 持久化和 Axum API 的低代码平台骨架。".to_string(),
            activation: PluginActivation::Eager,
            priority: 940,
            dependencies: Vec::new(),
            capabilities: vec![
                "lowcode-platform".to_string(),
                "toasty-persistence".to_string(),
                "backend-api".to_string(),
                "config-center".to_string(),
            ],
            permissions: vec![
                "config-center-read".to_string(),
                "postgres-read-write".to_string(),
            ],
            kind: PluginKind::WasmComponent,
        }
    }

    fn contributions(&self) -> Result<ContributionSet, az_aio_plugin_api::PluginError> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "lowcode.nav".to_string(),
                label: "低代码".to_string(),
                icon: "▣".to_string(),
                route: "/lowcode".to_string(),
                order: 35,
            }],
            pages: vec![PageContribution {
                route: "/lowcode".to_string(),
                title: "低代码".to_string(),
                subtitle: "管理低代码应用、页面和运行配置。".to_string(),
                renderer_id: "placeholder".to_string(),
                placeholder_mark: "▣".to_string(),
                order: 35,
            }],
            ui_contributions: vec![
                ui_contribution(
                    "lowcode.ui.content",
                    UiContributionSlot::Content,
                    "低代码内容区",
                    "lowcode.workbench",
                    Some("/lowcode"),
                    10,
                ),
                ui_contribution(
                    "lowcode.ui.settings",
                    UiContributionSlot::SettingsContent,
                    "低代码设置",
                    "lowcode.settings",
                    Some("/settings"),
                    50,
                ),
            ],
            backend_apis: lowcode_backend_apis(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: vec![SettingsSectionContribution {
                id: "lowcode.defaults".to_string(),
                label: "低代码默认值".to_string(),
                order: 50,
                defaults: vec![
                    SettingsDefaultContribution {
                        key: "lowcode.config_namespace".to_string(),
                        label: "配置命名空间".to_string(),
                        value: DATABASE_NAMESPACE.to_string(),
                        description: "低代码插件数据库连接读取的配置中心命名空间。".to_string(),
                        order: 10,
                    },
                    SettingsDefaultContribution {
                        key: DATABASE_SETTING_KEY.to_string(),
                        label: "数据库连接".to_string(),
                        value: String::new(),
                        description: "低代码插件使用的 PostgreSQL 连接串，应写入 az-aio.dev 命名空间。"
                            .to_string(),
                        order: 20,
                    },
                ],
            }],
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }
}

fn lowcode_backend_apis() -> Vec<BackendApiContribution> {
    vec![
        backend_api(
            "lowcode.api.status",
            "GET",
            "/api/lowcode/status",
            "低代码状态",
            "返回低代码插件数据库配置来源和 Toasty 连接状态。",
            10,
        ),
        backend_api(
            "lowcode.api.apps",
            "GET",
            "/api/lowcode/apps",
            "低代码应用列表",
            "列出低代码平台中的应用定义。",
            20,
        ),
        backend_api(
            "lowcode.api.app-upsert",
            "POST",
            "/api/lowcode/app",
            "保存低代码应用",
            "创建或更新一个低代码应用定义。",
            30,
        ),
        backend_api(
            "lowcode.api.pages",
            "GET",
            "/api/lowcode/pages",
            "低代码页面列表",
            "按 appId 查询低代码页面定义。",
            40,
        ),
        backend_api(
            "lowcode.api.page-upsert",
            "POST",
            "/api/lowcode/page",
            "保存低代码页面",
            "创建或更新一个低代码页面定义。",
            50,
        ),
        backend_api(
            "lowcode.api.page-delete",
            "POST",
            "/api/lowcode/page/delete",
            "删除低代码页面",
            "按 pageId 删除低代码页面定义。",
            60,
        ),
    ]
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
