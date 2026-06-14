#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use az_aio_plugin_api::api::{
    AzAioPlugin, BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, UiContribution, UiContributionSlot,
};

#[derive(Default)]
pub struct SearchPlugin;

impl AzAioPlugin for SearchPlugin {
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
            kind: PluginKind::WasmComponent,
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
}

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::api::{AzAioPlugin, contributions_to_json, descriptor_to_json};

    use super::SearchPlugin;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "az-aio-plugin",
    });

    struct SearchWasm;

    impl Guest for SearchWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&SearchPlugin.descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = SearchPlugin
                .contributions()
                .map_err(|error| error.to_string())?;
            contributions_to_json(&contributions).map_err(|error| error.to_string())
        }

        fn on_load() -> Result<(), String> {
            Ok(())
        }

        fn on_enable() -> Result<(), String> {
            Ok(())
        }

        fn on_disable() -> Result<(), String> {
            Ok(())
        }

        fn on_unload() -> Result<(), String> {
            Ok(())
        }
    }

    export!(SearchWasm);
}
