#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use az_aio_plugin_api::api::{
    AzAioPlugin, BackendApiContribution, ContributionSet, PluginActivation, PluginDescriptor,
    PluginKind, UiContribution, UiContributionSlot,
};

#[derive(Default)]
pub struct GitEnvsPlugin;

impl AzAioPlugin for GitEnvsPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "git/envs".to_string(),
            name: "Git 环境变量".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供 Git 工作流环境变量 UI 和 API 贡献点。".to_string(),
            activation: PluginActivation::Eager,
            priority: 620,
            dependencies: Vec::new(),
            capabilities: vec![
                "env-page".to_string(),
                "backend-api".to_string(),
                "sandbox-panel".to_string(),
            ],
            permissions: Vec::new(),
            kind: PluginKind::WasmComponent,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            ui_contributions: vec![UiContribution {
                id: "git.envs.ui.content".to_string(),
                slot: UiContributionSlot::Content,
                label: "环境变量内容区".to_string(),
                renderer_id: "git.envs.manager".to_string(),
                route: Some("/plugins".to_string()),
                order: 40,
            }],
            backend_apis: vec![BackendApiContribution {
                id: "git.envs.api.list".to_string(),
                method: "GET".to_string(),
                path: "/api/git/envs".to_string(),
                label: "环境变量列表".to_string(),
                description: "返回 Git 工作流相关环境变量贡献。".to_string(),
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

    use super::GitEnvsPlugin;

    wit_bindgen::generate!({
        path: "../../../wit",
        world: "az-aio-plugin",
    });

    struct GitEnvsWasm;

    impl Guest for GitEnvsWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&GitEnvsPlugin.descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = GitEnvsPlugin
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

    export!(GitEnvsWasm);
}
