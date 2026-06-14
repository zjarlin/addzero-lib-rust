use az_aio_platform::plugin_api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, NativeUiRenderer, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, UiContribution, UiContributionSlot,
};
use az_aio_platform::register_native_plugin;

use crate::{
    page::LowcodePage,
    routes::{LowcodeApiState, lowcode_router},
    record::RecordStore,
    store::LowcodeStore,
};

const PLUGIN_ID: &str = "lowcode";
const RENDERER_ID: &str = "lowcode.page";
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
            }],
            backend_apis: vec![BackendApiContribution {
                id: "lowcode.api.meta".into(),
                method: "GET".into(),
                path: "/api/lowcode/models".into(),
                label: "获取数据模型列表".into(),
                description: "返回所有 MetaModel 及其字段统计".into(),
                order: 10,
            }],
            ..Default::default()
        })
    }

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        // Initialize global store (seed demo data once)
        let _ = LowcodeStore::global();
        RecordStore::global().seed_demo();

        // API store: try DB, fall back to global singleton
        let api_store = LowcodeStore::degraded(context.database_url);
        api_store.seed_demo();

        let renderers = vec![NativeUiRenderer {
            renderer_id: RENDERER_ID.into(),
            slot: UiContributionSlot::Content,
            route: Some(ROUTE.into()),
            render: LowcodePage,
        }];

        let router = lowcode_router(LowcodeApiState {
            store: api_store,
        });

        Ok(NativePluginRuntime {
            renderers,
            router,
            startup: None,
        })
    }
}

register_native_plugin!(LowcodePlugin);
