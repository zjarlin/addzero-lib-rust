#![forbid(unsafe_code)]

use az_aio_plugin_api::api::{
    ContributionSet, NativeAzAioPlugin, NativePluginContext, NativePluginRuntime, PluginActivation,
    PluginDescriptor, PluginKind, UiContribution, UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

#[derive(Default)]
pub struct GitNotesPlugin;

impl NativeAzAioPlugin for GitNotesPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "git/notes".to_string(),
            name: "Git 笔记".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "预留 Git 笔记插件；笔记功能尚未实现。".to_string(),
            activation: PluginActivation::Lazy,
            priority: 610,
            dependencies: Vec::new(),
            capabilities: vec!["notes-placeholder".to_string()],
            permissions: Vec::new(),
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            ui_contributions: vec![UiContribution {
                id: "git.notes.ui.sandbox".to_string(),
                slot: UiContributionSlot::SandboxPanel,
                label: "Git 笔记沙箱占位".to_string(),
                renderer_id: "git.notes.placeholder".to_string(),
                route: Some("/plugins".to_string()),
                order: 60,
            }],
            backend_apis: Vec::new(),
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

register_native_plugin!(GitNotesPlugin);

pub fn ensure_linked() {}
