use std::sync::Arc;

use anyhow::anyhow;
use az_aio_platform::plugin::contract::{
    AdminCliContribution, AdminFieldContract, AdminFieldKind, AdminMenuNode, AdminMenuNodeKind,
    AdminMenuSection, AdminMenuTree, AdminOperationContract, AdminResourceContract,
    BackendApiContribution, ContributionSet, DynAdminPluginProvider, NativePluginContext,
    NativePluginProvider, NativePluginRuntime, NativeUiRenderer, NavItemContribution,
    PageContribution, PluginActivation, PluginDescriptor, PluginKind, UiContribution,
    UiContributionSlot,
};
use az_engine::{
    FIELDS_PATH_TEMPLATE, HOOKS_PATH_TEMPLATE, MODELS_PATH, OP_FIELDS_CREATE, OP_FIELDS_LIST,
    OP_HOOKS_CREATE, OP_HOOKS_LIST, OP_MODELS_CREATE, OP_MODELS_LIST, OP_RECORDS_CREATE,
    OP_RECORDS_LIST, RECORDS_PATH_TEMPLATE,
};
use rudi::Singleton;

use crate::{
    routes::{LowcodeApiState, engine_router},
    state::{install_store, store_from_shared_db},
    ui::page::LowcodePage,
};

const PLUGIN_ID: &str = "lowcode";
const RENDERER_ID: &str = "lowcode.page";
const SIDEBAR_RENDERER_ID: &str = "lowcode.sidebar";
const ROUTE: &str = "/lowcode";
#[cfg(test)]
const API_PREFIX: &str = "/api/engine";

#[derive(Default)]
pub struct LowcodePlugin;

impl NativePluginProvider for LowcodePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PLUGIN_ID.into(),
            name: "低代码".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "PostgreSQL 驱动的低代码元模型、钩子和动态记录引擎".into(),
            activation: PluginActivation::Eager,
            priority: 600,
            dependencies: vec![],
            capabilities: vec![
                "dioxus-ui-contract-page".into(),
                "engine-meta-model".into(),
                "engine-record-pipeline".into(),
            ],
            permissions: vec!["PostgreSQL engine_* 表读写".into()],
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "engine.nav".into(),
                label: "低代码引擎".into(),
                icon: "▣".into(),
                route: ROUTE.into(),
                order: 50,
            }],
            pages: vec![PageContribution {
                route: ROUTE.into(),
                title: "低代码引擎".into(),
                subtitle: "模型、字段、钩子、记录统一管理".into(),
                renderer_id: RENDERER_ID.into(),
                placeholder_mark: "▣".into(),
                order: 50,
            }],
            ui_contributions: vec![
                UiContribution {
                    id: "engine.ui.content".into(),
                    slot: UiContributionSlot::Content,
                    label: "低代码内容区".into(),
                    renderer_id: RENDERER_ID.into(),
                    route: Some(ROUTE.into()),
                    order: 50,
                },
                UiContribution {
                    id: "engine.ui.sidebar".into(),
                    slot: UiContributionSlot::AppSidebar,
                    label: "低代码侧边栏".into(),
                    renderer_id: SIDEBAR_RENDERER_ID.into(),
                    route: Some(ROUTE.into()),
                    order: 50,
                },
            ],
            backend_apis: backend_api_contributions(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
            ..ContributionSet::default()
        })
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        if context
            .database_url
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(anyhow!("lowcode 插件启动需要 PostgreSQL DATABASE_URL"));
        }
        let shared_db = context
            .shared_db
            .ok_or_else(|| anyhow!("lowcode 插件启动需要共享 Db 单例"))?;
        let store = store_from_shared_db(shared_db);
        install_store(store.clone());

        Ok(NativePluginRuntime {
            renderers: vec![NativeUiRenderer {
                renderer_id: RENDERER_ID.into(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.into()),
                render: LowcodePage,
            }],
            router: engine_router(LowcodeApiState { store }),
            startup: None,
        })
    }

    fn admin_menu(&self, _contributions: &ContributionSet) -> AdminMenuTree {
        AdminMenuTree {
            sections: vec![AdminMenuSection {
                domain_id: "lowcode".to_string(),
                label: "低代码".to_string(),
                default_href: ROUTE.to_string(),
                order: 600,
                menus: vec![AdminMenuNode {
                    id: "engine.root".to_string(),
                    kind: AdminMenuNodeKind::Branch,
                    label: "低代码引擎".to_string(),
                    href: ROUTE.to_string(),
                    icon: "▣".to_string(),
                    order: 10,
                    active_patterns: vec![ROUTE.to_string()],
                    permissions_any_of: vec!["engine:read".to_string()],
                    children: vec![
                        admin_menu_node("engine.fields", "字段", "/lowcode?tab=fields", "▤", 10),
                        admin_menu_node("engine.hooks", "钩子", "/lowcode?tab=hooks", "⚑", 20),
                        admin_menu_node("engine.records", "记录", "/lowcode?tab=records", "▦", 30),
                    ],
                }],
            }],
        }
    }

    fn admin_resources(&self) -> Vec<AdminResourceContract> {
        vec![
            engine_model_resource(),
            engine_field_resource(),
            engine_hook_resource(),
            engine_record_resource(),
        ]
    }

    fn admin_cli(&self) -> Vec<AdminCliContribution> {
        vec![
            cli(
                "engine.cli.models",
                "列出模型",
                "az engine model list",
                "engine.models",
                OP_MODELS_LIST,
            ),
            cli(
                "engine.cli.fields",
                "列出字段",
                "az engine field list",
                "engine.fields",
                OP_FIELDS_LIST,
            ),
            cli(
                "engine.cli.hooks",
                "列出钩子",
                "az engine hook list",
                "engine.hooks",
                OP_HOOKS_LIST,
            ),
            cli(
                "engine.cli.records",
                "列出记录",
                "az engine record list",
                "engine.records",
                OP_RECORDS_LIST,
            ),
        ]
    }
}

#[Singleton(name = "lowcode")]
pub fn lowcode_plugin() -> DynAdminPluginProvider {
    Arc::new(LowcodePlugin)
}

fn backend_api_contributions() -> Vec<BackendApiContribution> {
    vec![
        api(OP_MODELS_LIST, "GET", MODELS_PATH, "模型列表", 10),
        api(OP_MODELS_CREATE, "POST", MODELS_PATH, "创建模型", 20),
        api(OP_FIELDS_LIST, "GET", FIELDS_PATH_TEMPLATE, "字段列表", 30),
        api(
            OP_FIELDS_CREATE,
            "POST",
            FIELDS_PATH_TEMPLATE,
            "创建字段",
            40,
        ),
        api(OP_HOOKS_LIST, "GET", HOOKS_PATH_TEMPLATE, "钩子列表", 50),
        api(OP_HOOKS_CREATE, "POST", HOOKS_PATH_TEMPLATE, "创建钩子", 60),
        api(
            OP_RECORDS_LIST,
            "GET",
            RECORDS_PATH_TEMPLATE,
            "记录列表",
            70,
        ),
        api(
            OP_RECORDS_CREATE,
            "POST",
            RECORDS_PATH_TEMPLATE,
            "写入记录",
            80,
        ),
    ]
}

fn api(id: &str, method: &str, path: &str, label: &str, order: i32) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: format!("{label}: {path}"),
        order,
    }
}

fn admin_menu_node(id: &str, label: &str, href: &str, icon: &str, order: i32) -> AdminMenuNode {
    AdminMenuNode {
        id: id.to_string(),
        kind: AdminMenuNodeKind::Page,
        label: label.to_string(),
        href: href.to_string(),
        icon: icon.to_string(),
        order,
        active_patterns: vec![href.to_string()],
        permissions_any_of: vec!["engine:read".to_string()],
        children: Vec::new(),
    }
}

fn engine_model_resource() -> AdminResourceContract {
    AdminResourceContract {
        id: "engine.models".to_string(),
        label: "模型".to_string(),
        description: "engine 元模型定义，正式持久化于 PostgreSQL。".to_string(),
        route: ROUTE.to_string(),
        table_name: "engine_meta_models".to_string(),
        permissions_any_of: vec!["engine:model".to_string()],
        fields: vec![
            field("name", "名称", AdminFieldKind::Text, true, true),
            field("display_name", "显示名", AdminFieldKind::Text, true, true),
            field(
                "created_at_ms",
                "创建时间",
                AdminFieldKind::Time,
                false,
                false,
            ),
            field(
                "updated_at_ms",
                "更新时间",
                AdminFieldKind::Time,
                false,
                false,
            ),
        ],
        operations: vec![
            operation(
                OP_MODELS_LIST,
                "查询模型",
                "GET",
                MODELS_PATH,
                "az engine model list",
                false,
            ),
            operation(
                OP_MODELS_CREATE,
                "新建模型",
                "POST",
                MODELS_PATH,
                "az engine model create",
                true,
            ),
        ],
    }
}

fn engine_field_resource() -> AdminResourceContract {
    AdminResourceContract {
        id: "engine.fields".to_string(),
        label: "字段".to_string(),
        description: "engine MetaField，包含固定字段类型和 computed 依赖配置。".to_string(),
        route: "/lowcode?tab=fields".to_string(),
        table_name: "engine_meta_fields".to_string(),
        permissions_any_of: vec!["engine:field".to_string()],
        fields: vec![
            field("model_name", "模型", AdminFieldKind::Text, true, true),
            field("name", "名称", AdminFieldKind::Text, true, true),
            field("field_type", "类型", AdminFieldKind::Badge, true, true),
            field("is_required", "必填", AdminFieldKind::Boolean, false, true),
            field(
                "dependency_json",
                "依赖",
                AdminFieldKind::Json,
                false,
                false,
            ),
        ],
        operations: vec![
            operation(
                OP_FIELDS_LIST,
                "查询字段",
                "GET",
                FIELDS_PATH_TEMPLATE,
                "az engine field list",
                false,
            ),
            operation(
                OP_FIELDS_CREATE,
                "新建字段",
                "POST",
                FIELDS_PATH_TEMPLATE,
                "az engine field create",
                true,
            ),
        ],
    }
}

fn engine_hook_resource() -> AdminResourceContract {
    AdminResourceContract {
        id: "engine.hooks".to_string(),
        label: "钩子".to_string(),
        description: "engine Rhai 生命周期钩子，只暴露受控 payload 和命令队列。".to_string(),
        route: "/lowcode?tab=hooks".to_string(),
        table_name: "engine_hook_definitions".to_string(),
        permissions_any_of: vec!["engine:hook".to_string()],
        fields: vec![
            field("model_name", "模型", AdminFieldKind::Text, true, true),
            field("trigger_event", "事件", AdminFieldKind::Badge, true, true),
            field("script_content", "脚本", AdminFieldKind::Json, true, false),
            field("is_active", "启用", AdminFieldKind::Boolean, false, true),
        ],
        operations: vec![
            operation(
                OP_HOOKS_LIST,
                "查询钩子",
                "GET",
                HOOKS_PATH_TEMPLATE,
                "az engine hook list",
                false,
            ),
            operation(
                OP_HOOKS_CREATE,
                "新建钩子",
                "POST",
                HOOKS_PATH_TEMPLATE,
                "az engine hook create",
                true,
            ),
        ],
    }
}

fn engine_record_resource() -> AdminResourceContract {
    AdminResourceContract {
        id: "engine.records".to_string(),
        label: "记录".to_string(),
        description: "engine DataRecord，payload 使用 Toasty JSON 字段。".to_string(),
        route: "/lowcode?tab=records".to_string(),
        table_name: "engine_data_records".to_string(),
        permissions_any_of: vec!["engine:record".to_string()],
        fields: vec![
            field("model_name", "模型", AdminFieldKind::Text, true, true),
            field("payload", "Payload", AdminFieldKind::Json, true, false),
            field(
                "created_at_ms",
                "创建时间",
                AdminFieldKind::Time,
                false,
                false,
            ),
            field(
                "updated_at_ms",
                "更新时间",
                AdminFieldKind::Time,
                false,
                false,
            ),
        ],
        operations: vec![
            operation(
                OP_RECORDS_LIST,
                "查询记录",
                "GET",
                RECORDS_PATH_TEMPLATE,
                "az engine record list",
                false,
            ),
            operation(
                OP_RECORDS_CREATE,
                "写入记录",
                "POST",
                RECORDS_PATH_TEMPLATE,
                "az engine record create",
                true,
            ),
        ],
    }
}

fn field(
    name: &str,
    label: &str,
    kind: AdminFieldKind,
    required: bool,
    searchable: bool,
) -> AdminFieldContract {
    AdminFieldContract {
        name: name.to_string(),
        label: label.to_string(),
        kind,
        required,
        searchable,
        table_visible: true,
        form_visible: true,
    }
}

fn operation(
    id: &str,
    label: &str,
    method: &str,
    path: &str,
    cli: &str,
    primary: bool,
) -> AdminOperationContract {
    AdminOperationContract {
        id: id.to_string(),
        label: label.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        cli: cli.to_string(),
        primary,
        audit: true,
    }
}

fn cli(
    id: &str,
    label: &str,
    command: &str,
    resource_id: &str,
    operation_id: &str,
) -> AdminCliContribution {
    AdminCliContribution {
        id: id.to_string(),
        label: label.to_string(),
        command: command.to_string(),
        resource_id: Some(resource_id.to_string()),
        operation_id: Some(operation_id.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributions_include_content_sidebar_and_new_api_prefix() {
        let contributions = match LowcodePlugin.contributions() {
            Ok(value) => value,
            Err(error) => panic!("贡献声明应创建成功: {error}"),
        };

        // 插件只贡献低代码引擎的内容区、侧栏和 REST API。
        assert!(
            contributions.ui_contributions.iter().any(|ui| {
                ui.id == "engine.ui.content" && ui.slot == UiContributionSlot::Content
            })
        );
        assert!(contributions.ui_contributions.iter().any(|ui| {
            ui.id == "engine.ui.sidebar" && ui.slot == UiContributionSlot::AppSidebar
        }));
        assert!(
            contributions
                .backend_apis
                .iter()
                .all(|api| api.path.starts_with(API_PREFIX))
        );
    }

    #[test]
    fn runtime_without_database_url_fails() {
        let error = match LowcodePlugin.runtime(NativePluginContext::default()) {
            Ok(_) => String::new(),
            Err(error) => error.to_string(),
        };

        // 缺少 DATABASE_URL 时不能降级到内存实现。
        assert!(error.contains("DATABASE_URL"));
    }

    #[test]
    fn admin_menu_does_not_expose_old_screen_entries() {
        let tree = LowcodePlugin.admin_menu(&ContributionSet::default());
        let serialized = match serde_json::to_string(&tree) {
            Ok(value) => value,
            Err(error) => panic!("菜单应可序列化: {error}"),
        };

        // Admin 菜单只暴露低代码引擎四块，不再出现旧的页面类入口。
        let old_renderer = ["App", "Screen"].join("");
        let old_config = ["页面", "配置"].join("");
        let old_preview = ["发布", "预览"].join("");
        assert!(!serialized.contains(&old_renderer));
        assert!(!serialized.contains(&old_config));
        assert!(!serialized.contains(&old_preview));
    }

    #[test]
    fn resources_reference_engine_tables_only() {
        let resources = LowcodePlugin.admin_resources();

        // 资源契约不再引用旧的低代码表。
        let old_prefix = ["biz", "lowcode"].join("_");
        assert!(
            resources
                .iter()
                .all(|resource| resource.table_name.starts_with("engine_"))
        );
        assert!(
            resources
                .iter()
                .all(|resource| !resource.table_name.starts_with(&old_prefix))
        );
    }
}
