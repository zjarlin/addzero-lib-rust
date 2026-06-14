#![forbid(unsafe_code)]

use az_aio_plugin_api::api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, NavItemContribution, PageContribution, PluginActivation, PluginDescriptor,
    PluginKind, UiContribution, UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

#[derive(Default)]
pub struct SearchPlugin;

impl NativeAzAioPlugin for SearchPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "search".to_string(),
            name: "搜索".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供 AZ AIO 全局搜索入口和搜索内容区。".to_string(),
            activation: PluginActivation::Eager,
            priority: 970,
            dependencies: Vec::new(),
            capabilities: vec!["nav-items".to_string(), "search-content".to_string()],
            permissions: Vec::new(),
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "search.nav".to_string(),
                label: "搜索".to_string(),
                icon: "⌕".to_string(),
                route: "/search".to_string(),
                order: 20,
            }],
            pages: vec![PageContribution {
                route: "/search".to_string(),
                title: "搜索".to_string(),
                subtitle: "跨插件、项目和本地资源搜索。".to_string(),
                renderer_id: "placeholder".to_string(),
                placeholder_mark: "⌕".to_string(),
                order: 20,
            }],
            ui_contributions: vec![UiContribution {
                id: "search.ui.content".to_string(),
                slot: UiContributionSlot::Content,
                label: "搜索内容区".to_string(),
                renderer_id: "search.panel".to_string(),
                route: Some("/search".to_string()),
                order: 10,
            }],
            backend_apis: vec![BackendApiContribution {
                id: "search.api.query".to_string(),
                method: "GET".to_string(),
                path: "/api/search".to_string(),
                label: "搜索查询".to_string(),
                description: "按查询条件返回跨插件、项目和本地资源的搜索结果。".to_string(),
                order: 10,
            }],
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(SearchPlugin);

pub fn ensure_linked() {}
