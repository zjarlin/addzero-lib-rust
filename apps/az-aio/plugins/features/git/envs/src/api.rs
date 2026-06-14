#![forbid(unsafe_code)]

use az_aio_plugin_api::api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, PluginActivation, PluginDescriptor, PluginKind, UiContribution,
    UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

#[derive(Default)]
pub struct GitEnvsPlugin;

impl NativeAzAioPlugin for GitEnvsPlugin {
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
            kind: PluginKind::Native,
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

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(GitEnvsPlugin);

pub fn ensure_linked() {}
