#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use az_aio_plugin_api::api::{
    AzAioPlugin, BackendApiContribution, ContributionSet, PageContribution, PluginActivation,
    PluginDescriptor, PluginKind, SettingsDefaultContribution, SettingsSectionContribution,
    UiContribution, UiContributionSlot,
};

const PROJECT_DEFAULT_SYNC_ROOT: &str = "az-sync/workspace";

#[derive(Default)]
pub struct SettingsPlugin;

impl AzAioPlugin for SettingsPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "settings".to_string(),
            name: "设置".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "管理 AZ AIO 设置和项目默认目录。".to_string(),
            activation: PluginActivation::Eager,
            priority: 980,
            dependencies: Vec::new(),
            capabilities: vec![
                "settings-content".to_string(),
                "project-defaults".to_string(),
                "backend-api".to_string(),
            ],
            permissions: Vec::new(),
            kind: PluginKind::WasmComponent,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: vec![PageContribution {
                route: "/settings".to_string(),
                title: "设置".to_string(),
                subtitle: "管理 AZ AIO 设置和项目默认目录。".to_string(),
                renderer_id: "settings.page".to_string(),
                placeholder_mark: "⚙".to_string(),
                order: 90,
            }],
            ui_contributions: vec![ui_contribution(
                "settings.ui.project-defaults",
                UiContributionSlot::SettingsContent,
                "项目默认目录",
                "settings.project-defaults",
                Some("/settings"),
                10,
            )],
            backend_apis: vec![backend_api(
                "settings.api.project-defaults",
                "GET",
                "/api/settings/project-defaults",
                "项目默认设置",
                "读取默认同步根目录和项目绑定默认值。",
                10,
            )],
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: vec![SettingsSectionContribution {
                id: "settings.project-defaults".to_string(),
                label: "项目默认目录".to_string(),
                order: 10,
                defaults: vec![SettingsDefaultContribution {
                    key: "projects.default_sync_root".to_string(),
                    label: "默认同步根目录".to_string(),
                    value: PROJECT_DEFAULT_SYNC_ROOT.to_string(),
                    description: "项目插件扫描和绑定时使用的默认同步根目录。".to_string(),
                    order: 10,
                }],
            }],
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }
}

fn ui_contribution(
    id: &str,
    slot: UiContributionSlot,
    label: &str,
    renderer_id: &str,
    route: Option<&str>,
    order: i32,
) -> UiContribution {
    UiContribution {
        id: id.to_string(),
        slot,
        label: label.to_string(),
        renderer_id: renderer_id.to_string(),
        route: route.map(str::to_string),
        order,
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

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::api::{AzAioPlugin, contributions_to_json, descriptor_to_json};

    use super::SettingsPlugin;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "az-aio-plugin",
    });

    struct SettingsWasm;

    impl Guest for SettingsWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&SettingsPlugin.descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = SettingsPlugin
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

    export!(SettingsWasm);
}
