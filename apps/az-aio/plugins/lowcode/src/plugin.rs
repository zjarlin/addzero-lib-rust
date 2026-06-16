use std::sync::Arc;

use az_aio_platform::plugin::api::{
    BackendApiContribution, ContributionSet, DynNativeAzAioPlugin, NativeAzAioPlugin,
    NativePluginContext, NativePluginRuntime, NativeUiRenderer, NavItemContribution,
    PageContribution, PluginActivation, PluginDescriptor, PluginKind, UiContribution,
    UiContributionSlot,
};
use rudi::Singleton;

use crate::{
    backend::{
        record::RecordStore,
        routes::{LowcodeApiState, lowcode_router},
        store::LowcodeStore,
    },
    ui::{page::LowcodePage, sidebar::LowcodeSidebar},
};

const PLUGIN_ID: &str = "lowcode";
const RENDERER_ID: &str = "lowcode.page";
const SIDEBAR_RENDERER_ID: &str = "lowcode.sidebar";
const ROUTE: &str = "/lowcode";

#[derive(Default)]
pub struct LowcodePlugin;

impl NativeAzAioPlugin for LowcodePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PLUGIN_ID.into(),
            name: "低代码工作台".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "元数据建模与低代码 AppScreen 管理".into(),
            activation: PluginActivation::Eager,
            priority: 600,
            dependencies: vec![],
            capabilities: vec!["lowcode-meta-model".into(), "lowcode-app-screen".into()],
            permissions: vec!["数据库读写 lowcode_* 表".into()],
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "lowcode.nav".into(),
                label: "低代码".into(),
                icon: "▣".into(),
                route: ROUTE.into(),
                order: 50,
            }],
            pages: vec![PageContribution {
                route: ROUTE.into(),
                title: "低代码工作台".into(),
                subtitle: "元数据建模 & AppScreen 低代码管理".into(),
                renderer_id: RENDERER_ID.into(),
                placeholder_mark: "▣".into(),
                order: 50,
            }],
            ui_contributions: vec![UiContribution {
                id: "lowcode.ui.content".into(),
                slot: UiContributionSlot::Content,
                label: "Lowcode 内容区".into(),
                renderer_id: RENDERER_ID.into(),
                route: Some(ROUTE.into()),
                order: 50,
            }, UiContribution {
                id: "lowcode.ui.sidebar".into(),
                slot: UiContributionSlot::AppSidebar,
                label: "Lowcode 侧边栏".into(),
                renderer_id: SIDEBAR_RENDERER_ID.into(),
                route: Some(ROUTE.into()),
                order: 50,
            }],
            backend_apis: vec![BackendApiContribution {
                id: "lowcode.api.meta".into(),
                method: "GET".into(),
                path: "/api/lowcode/models".into(),
                label: "获取数据模型列表".into(),
                description: "返回所有 MetaModel 及其字段统计".into(),
                order: 10,
            }],
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
            ..ContributionSet::default()
        })
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        // Initialize global store (seed demo data once)
        let _ = LowcodeStore::global();
        RecordStore::global().seed_demo();

        // API store: try DB, fall back to global singleton
        let api_store = LowcodeStore::degraded(context.database_url);
        api_store.seed_demo();

        // Wire RecordStore to DB if available
        if let Some(ref db) = api_store.db {
            RecordStore::init_db(db.clone());
        }

        let renderers = vec![
            NativeUiRenderer {
                renderer_id: RENDERER_ID.into(),
                slot: UiContributionSlot::Content,
                route: Some(ROUTE.into()),
                render: LowcodePage,
            },
            NativeUiRenderer {
                renderer_id: SIDEBAR_RENDERER_ID.into(),
                slot: UiContributionSlot::AppSidebar,
                route: Some(ROUTE.into()),
                render: LowcodeSidebar,
            },
        ];

        let router = lowcode_router(LowcodeApiState { store: api_store });

        Ok(NativePluginRuntime {
            renderers,
            router,
            startup: None,
        })
    }
}

#[Singleton(name = "lowcode")]
pub fn lowcode_plugin() -> DynNativeAzAioPlugin {
    Arc::new(LowcodePlugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contract::layout_descriptors, metadata::metadata_provider, ui::page::strategy::layout_strategies};

    #[test]
    fn contributions_include_content_and_sidebar_slots() {
        let contributions = LowcodePlugin.contributions().unwrap();

        assert!(
            contributions
                .ui_contributions
                .iter()
                .any(|ui| ui.id == "lowcode.ui.content" && ui.slot == UiContributionSlot::Content)
        );
        assert!(
            contributions
                .ui_contributions
                .iter()
                .any(|ui| ui.id == "lowcode.ui.sidebar" && ui.slot == UiContributionSlot::AppSidebar)
        );
    }

    #[test]
    fn metadata_provider_exposes_configurable_lowcode_menus_and_models() {
        let metadata = metadata_provider();
        let menus = metadata.menus().unwrap();
        let models = metadata.models().unwrap();

        assert!(menus.iter().any(|menu| menu.id == "lowcode.screens"));
        assert!(models.iter().any(|model| model.name == "Project"));
    }

    #[test]
    fn lowcode_layout_descriptors_include_builtin_strategies() {
        let layouts = layout_descriptors(&layout_strategies());

        assert!(
            layouts
                .iter()
                .any(|layout| layout.code == "MasterDetail")
        );
    }
}
