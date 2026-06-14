#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use az_aio_plugin_api::api::{
    AzAioPlugin, ContributionSet, NavItemContribution, PageContribution, PluginActivation,
    PluginDescriptor, PluginKind, SettingsSectionContribution, UiContribution, UiContributionSlot,
};

#[derive(Default)]
pub struct CoreNavPlugin;

impl AzAioPlugin for CoreNavPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "navigation".to_string(),
            name: "导航".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供 AZ AIO 桌面端左侧导航和基础路由描述。".to_string(),
            activation: PluginActivation::Eager,
            priority: 1_000,
            dependencies: Vec::new(),
            capabilities: vec![
                "nav-items".to_string(),
                "pages".to_string(),
                "settings-sections".to_string(),
            ],
            permissions: Vec::new(),
            kind: PluginKind::WasmComponent,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![
                nav_item("core-nav.chat", "新对话", "✎", "/chat", 10),
                nav_item("core-nav.plugins", "插件", "⌘", "/plugins", 30),
                nav_item(
                    "core-nav.az-platform",
                    "az-platform",
                    "◇",
                    "/az-platform",
                    35,
                ),
                nav_item("core-nav.automations", "自动化", "◷", "/automations", 40),
            ],
            pages: vec![
                placeholder_page("/chat", "暂未开放", "对话功能会由后续插件接管。", "✎", 10),
                PageContribution {
                    route: "/plugins".to_string(),
                    title: "插件与技能".to_string(),
                    subtitle: "查看本地插件包、组件和技能。".to_string(),
                    renderer_id: "catalog".to_string(),
                    placeholder_mark: "⌘".to_string(),
                    order: 30,
                },
                PageContribution {
                    route: "/az-platform".to_string(),
                    title: "az-platform".to_string(),
                    subtitle: "插件贡献点和后端接口沙箱。".to_string(),
                    renderer_id: "az-platform-sandbox".to_string(),
                    placeholder_mark: "◇".to_string(),
                    order: 35,
                },
                placeholder_page(
                    "/automations",
                    "暂未开放",
                    "自动化功能会由后续插件接管。",
                    "◷",
                    40,
                ),
            ],
            ui_contributions: vec![
                ui_contribution(
                    "core-nav.ui.app-sidebar",
                    UiContributionSlot::AppSidebar,
                    "主导航",
                    "core-nav.sidebar",
                    None,
                    10,
                ),
                ui_contribution(
                    "core-nav.ui.content",
                    UiContributionSlot::Content,
                    "路由内容区",
                    "core-nav.content-router",
                    None,
                    20,
                ),
            ],
            backend_apis: Vec::new(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: vec![SettingsSectionContribution {
                id: "core-nav.settings.general".to_string(),
                label: "通用".to_string(),
                order: 10,
                defaults: Vec::new(),
            }],
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }
}

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::api::{AzAioPlugin, contributions_to_json, descriptor_to_json};

    use super::CoreNavPlugin;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "az-aio-plugin",
    });

    struct NavigationWasm;

    impl Guest for NavigationWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&CoreNavPlugin.descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = CoreNavPlugin
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

    export!(NavigationWasm);
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

fn nav_item(id: &str, label: &str, icon: &str, route: &str, order: i32) -> NavItemContribution {
    NavItemContribution {
        id: id.to_string(),
        label: label.to_string(),
        icon: icon.to_string(),
        route: route.to_string(),
        order,
    }
}

fn placeholder_page(
    route: &str,
    title: &str,
    subtitle: &str,
    mark: &str,
    order: i32,
) -> PageContribution {
    PageContribution {
        route: route.to_string(),
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        renderer_id: "placeholder".to_string(),
        placeholder_mark: mark.to_string(),
        order,
    }
}
