#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use az_aio_plugin_api::{
    AzAioPlugin, ContributionSet, PluginActivation, PluginDescriptor, PluginKind, UiContribution,
    UiContributionSlot,
};

#[derive(Default)]
pub struct GitNotesPlugin;

impl AzAioPlugin for GitNotesPlugin {
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
            kind: PluginKind::WasmComponent,
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
}

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::{AzAioPlugin, contributions_to_json, descriptor_to_json};

    use super::GitNotesPlugin;

    wit_bindgen::generate!({
        path: "../../../wit",
        world: "az-aio-plugin",
    });

    struct GitNotesWasm;

    impl Guest for GitNotesWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&GitNotesPlugin.descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = GitNotesPlugin
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

    export!(GitNotesWasm);
}
