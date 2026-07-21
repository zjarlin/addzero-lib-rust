//! 系统后台聚合 provider。
//!
//! `AdminProvider` 是应用侧唯一的系统后台内容提供者，负责把系统 catalog
//! 投射成 shell 可消费的导航、页面、toolbar 与 API 契约。

use crate::{
    plugin::api::{
        AdminCliContribution, AdminFieldContract, AdminFieldKind, AdminMenuTree,
        AdminOperationContract, AdminPluginContribution, AdminPluginProvider,
        AdminResourceContract, BackendApiContribution, ContributionSet, NativePluginContext,
        NativePluginRuntime, NativeUiRenderer, NavItemContribution, PageContribution,
        PluginActivation, PluginDescriptor, PluginKind, ToolbarActionContribution, UiContribution,
        UiContributionSlot,
    },
    system::{
        api::{system_admin_router, SystemAdminApiState},
        catalog::{
            SYSTEM_DOMAIN_ID, SYSTEM_DOMAIN_LABEL, SYSTEM_RENDERER_ID, SYSTEM_SIDEBAR_RENDERER_ID,
            SystemFieldKind, SystemPageView, starter_backed_system_pages, system_dashboard_view,
            system_page_views,
        },
        dictionary_ui::{DICTIONARY_RENDERER_ID, dictionary_workbench_page},
        navigation::{AdminSectionSnapshot, system_admin_sections},
        store::SystemAdminStore,
    },
};
use rudi::Singleton;
use std::sync::Arc;

use crate::plugin::api::DynAdminPluginProvider;

#[derive(Clone, Debug, Default)]
pub struct AdminProvider;

impl AdminProvider {
    pub fn system_sections(&self) -> Vec<AdminSectionSnapshot> {
        system_admin_sections()
    }

    pub fn system_page_views(&self) -> Vec<SystemPageView> {
        system_page_views()
    }

    pub fn contributions(&self) -> ContributionSet {
        let mut nav_items = Vec::new();
        let mut pages = Vec::new();
        let mut ui_contributions = Vec::new();
        let mut backend_apis = Vec::new();
        let mut toolbar_actions = Vec::new();

        for page in starter_backed_system_pages() {
            nav_items.push(NavItemContribution {
                id: format!("{SYSTEM_DOMAIN_ID}.{}.nav", page.id),
                label: page.label.to_string(),
                icon: page.icon.to_string(),
                route: page.route.to_string(),
                order: 1_000 + page.order,
            });
        }

        for page in system_dashboard_view().pages {
            let renderer_id = if page.id == "dictionary" {
                DICTIONARY_RENDERER_ID
            } else {
                SYSTEM_RENDERER_ID
            };
            pages.push(PageContribution {
                route: page.route.clone(),
                title: format!("{SYSTEM_DOMAIN_LABEL} · {}", page.label),
                subtitle: page.description.clone(),
                renderer_id: renderer_id.to_string(),
                placeholder_mark: page.icon.clone(),
                order: 1_000 + page.order,
            });
            ui_contributions.push(UiContribution {
                id: format!("{SYSTEM_DOMAIN_ID}.{}.ui.content", page.id),
                slot: UiContributionSlot::Content,
                label: format!("{}内容区", page.label),
                renderer_id: renderer_id.to_string(),
                route: Some(page.route.clone()),
                order: 1_000 + page.order,
            });
            backend_apis.extend(page.operations.iter().map(|operation| {
                BackendApiContribution {
                    id: operation.id.to_string(),
                    method: operation.method.to_string(),
                    path: operation.path.to_string(),
                    label: operation.label.to_string(),
                    description: format!("{} / {}", page.label, operation.cli),
                    order: 1_000 + page.order,
                }
            }));
            toolbar_actions.extend(page.operations.iter().map(|operation| {
                ToolbarActionContribution {
                    id: operation.id.to_string(),
                    route: Some(page.route.clone()),
                    label: operation.label.to_string(),
                    icon: if operation.primary {
                        "Plus".to_string()
                    } else {
                        "RefreshCw".to_string()
                    },
                    primary: operation.primary,
                    order: 1_000 + page.order,
                }
            }));
        }

        backend_apis.extend([
            backend_api(
                "system.api.status",
                "GET",
                "/api/system/status",
                "系统后台状态",
                "返回系统后台接入页、参考页、PG 表与 API 面摘要。",
                900,
            ),
            backend_api(
                "system.api.dashboard",
                "GET",
                "/api/system/dashboard",
                "系统后台总览",
                "返回系统后台完整页面契约和 PG 边界。",
                901,
            ),
            backend_api(
                "system.api.navigation",
                "GET",
                "/api/system/navigation",
                "系统后台导航",
                "返回双轴上下文导航树快照。",
                902,
            ),
            backend_api(
                "system.api.store.pages",
                "GET",
                "/api/system/store/pages",
                "系统后台页面快照",
                "返回已同步到 PostgreSQL 的系统后台页面契约快照。",
                903,
            ),
            backend_api(
                "system.api.store.operations",
                "GET",
                "/api/system/store/operations",
                "系统后台操作审计",
                "返回通过统一操作入口写入 PostgreSQL 的执行记录。",
                904,
            ),
            backend_api(
                "system.api.store.records",
                "GET",
                "/api/system/store/records",
                "系统后台数据快照",
                "按 o/s 分页返回已同步到 PostgreSQL 的系统页面数据记录。",
                905,
            ),
        ]);
        ui_contributions.push(UiContribution {
            id: "system.ui.sidebar".to_string(),
            slot: UiContributionSlot::AppSidebar,
            label: "系统后台侧轴导航".to_string(),
            renderer_id: SYSTEM_SIDEBAR_RENDERER_ID.to_string(),
            route: Some(crate::system::catalog::SYSTEM_DEFAULT_ROUTE.to_string()),
            order: 900,
        });

        ContributionSet {
            nav_items,
            pages,
            ui_contributions,
            backend_apis,
            toolbar_actions,
            ..Default::default()
        }
    }

    pub fn renderers(&self) -> Vec<NativeUiRenderer> {
        vec![NativeUiRenderer {
            renderer_id: DICTIONARY_RENDERER_ID.to_string(),
            slot: UiContributionSlot::Content,
            route: Some("/system/dictionary/note-types".to_string()),
            render: dictionary_workbench_page,
        }]
    }

    pub fn contribution_bundle(&self) -> AdminPluginContribution {
        let native = self.contributions();
        let menu = AdminMenuTree {
            sections: self.system_sections(),
        };
        let resources = system_page_views()
            .into_iter()
            .map(system_resource_contract)
            .collect::<Vec<_>>();
        let cli = resources
            .iter()
            .flat_map(|resource| {
                resource.operations.iter().map(|operation| AdminCliContribution {
                    id: format!("{}.cli.{}", resource.id, operation.id),
                    label: operation.label.clone(),
                    command: operation.cli.clone(),
                    resource_id: Some(resource.id.clone()),
                    operation_id: Some(operation.id.clone()),
                })
            })
            .collect();

        AdminPluginContribution {
            menu,
            resources,
            cli,
            native,
        }
    }
}

impl AdminPluginProvider for AdminProvider {
    fn admin_descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: SYSTEM_DOMAIN_ID.to_string(),
            name: SYSTEM_DOMAIN_LABEL.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "系统核心后台插件，提供用户、角色、部门、菜单、字典、审计和租户管理。".to_string(),
            activation: PluginActivation::Eager,
            priority: 900,
            dependencies: Vec::new(),
            capabilities: vec![
                "admin-resource-contract".to_string(),
                "dioxus-ui-contract-page".to_string(),
                "axum-api".to_string(),
                "postgresql-required".to_string(),
            ],
            permissions: vec!["system-admin".to_string()],
            kind: PluginKind::Native,
        }
    }

    fn admin_contribution(&self) -> anyhow::Result<AdminPluginContribution> {
        Ok(self.contribution_bundle())
    }

    fn admin_runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        let state = system_admin_api_state(context.database_url, context.shared_db);
        Ok(NativePluginRuntime {
            renderers: self.renderers(),
            router: system_admin_router(state),
            startup: None,
        })
    }
}

#[Singleton(name = "system-admin")]
pub fn system_admin_provider() -> DynAdminPluginProvider {
    Arc::new(AdminProvider)
}

pub fn system_contributions() -> ContributionSet {
    AdminProvider.contributions()
}

fn system_admin_api_state(
    database_url: Option<String>,
    shared_db: Option<crate::core::db::Db>,
) -> SystemAdminApiState {
    if database_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        return SystemAdminApiState::degraded(database_url);
    }
    let store = shared_db.map(SystemAdminStore::from_shared);
    if let Some(store) = &store {
        sync_system_catalog(store.clone());
    }
    SystemAdminApiState::from_store(database_url, store)
}

fn sync_system_catalog(store: SystemAdminStore) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("system-admin Toasty startup degraded: {error:#}");
            return;
        }
    };
    if let Err(error) = runtime.block_on(store.sync_catalog_snapshot()) {
        eprintln!("system-admin Toasty startup degraded: {error:#}");
    }
}

fn system_resource_contract(page: SystemPageView) -> AdminResourceContract {
    let fields = page
        .columns
        .iter()
        .map(|column| AdminFieldContract {
            name: column.key.to_string(),
            label: column.label.to_string(),
            kind: admin_field_kind(column.kind),
            required: false,
            searchable: matches!(column.kind, SystemFieldKind::Text | SystemFieldKind::Route),
            table_visible: true,
            form_visible: matches!(column.kind, SystemFieldKind::Text | SystemFieldKind::Route),
        })
        .collect();
    let operations = page
        .operations
        .iter()
        .map(|operation| AdminOperationContract {
            id: operation.id.to_string(),
            label: operation.label.to_string(),
            method: operation.method.to_string(),
            path: operation.path.to_string(),
            cli: operation.cli.to_string(),
            primary: operation.primary,
            audit: true,
        })
        .collect();

    AdminResourceContract {
        id: page.id,
        label: page.label,
        description: page.description,
        route: page.route,
        table_name: page
            .pg_tables
            .first()
            .cloned()
            .unwrap_or_else(|| "sys_unmapped".to_string()),
        permissions_any_of: page.permissions_any_of,
        fields,
        operations,
    }
}

fn admin_field_kind(kind: SystemFieldKind) -> AdminFieldKind {
    match kind {
        SystemFieldKind::Text | SystemFieldKind::Route => AdminFieldKind::Text,
        SystemFieldKind::Badge => AdminFieldKind::Badge,
        SystemFieldKind::Count => AdminFieldKind::Number,
        SystemFieldKind::Time => AdminFieldKind::Time,
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

#[cfg(test)]
mod tests {
    use crate::system::catalog::SYSTEM_RENDERER_ID;

    use super::*;

    #[test]
    fn provider_exports_starter_backed_routes_as_primary_nav() {
        let contributions = AdminProvider.contributions();
        let routes = contributions
            .nav_items
            .iter()
            .map(|item| item.route.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            routes,
            vec![
                "/system/account/api-keys",
                "/system/identity/users",
                "/system/permission/roles",
                "/system/organization/departments",
                "/system/dictionary/note-types",
                "/system/menu/mounting",
                "/system/audit/events",
            ]
        );
    }

    #[test]
    fn provider_pages_include_reference_routes_with_same_renderer() {
        let contributions = AdminProvider.contributions();
        let renderer_id = contributions
            .pages
            .iter()
            .find(|page| page.route == "/system/oauth2/clients")
            .map(|page| page.renderer_id.as_str());

        assert_eq!(renderer_id, Some(SYSTEM_RENDERER_ID));
    }

    #[test]
    fn provider_registers_dictionary_native_renderer_only() {
        let renderers = AdminProvider.renderers();

        // 字典页需要真实 CRUD 工作台，其他系统页继续走资源契约渲染。
        assert_eq!(renderers.len(), 1);
        assert_eq!(renderers[0].renderer_id, DICTIONARY_RENDERER_ID);
    }
}
