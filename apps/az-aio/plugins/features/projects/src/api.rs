#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

use az_aio_plugin_api::api::{
    AzAioPlugin, BackendApiContribution, ContributionSet, NavItemContribution, PageContribution,
    PluginActivation, PluginDescriptor, PluginKind, UiContribution, UiContributionSlot,
};

#[derive(Default)]
pub struct ProjectsPlugin;

impl AzAioPlugin for ProjectsPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "projects".to_string(),
            name: "项目".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供项目侧边栏、项目详情内容区和项目 API。".to_string(),
            activation: PluginActivation::Eager,
            priority: 960,
            dependencies: Vec::new(),
            capabilities: vec![
                "project-sidebar".to_string(),
                "project-content".to_string(),
                "backend-api".to_string(),
            ],
            permissions: Vec::new(),
            kind: PluginKind::WasmComponent,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: vec![NavItemContribution {
                id: "projects.nav".to_string(),
                label: "项目".to_string(),
                icon: "▦".to_string(),
                route: "/projects".to_string(),
                order: 25,
            }],
            pages: vec![PageContribution {
                route: "/projects".to_string(),
                title: "项目".to_string(),
                subtitle: "管理本地工作区、同步目录和项目上下文。".to_string(),
                renderer_id: "placeholder".to_string(),
                placeholder_mark: "▦".to_string(),
                order: 25,
            }],
            ui_contributions: vec![
                ui_contribution(
                    "projects.ui.sidebar",
                    UiContributionSlot::ProjectSidebar,
                    "项目侧边栏",
                    "projects.sidebar",
                    Some("/projects"),
                    10,
                ),
                ui_contribution(
                    "projects.ui.content",
                    UiContributionSlot::ProjectContent,
                    "项目详情内容区",
                    "projects.detail",
                    Some("/projects"),
                    20,
                ),
            ],
            backend_apis: vec![backend_api(
                "projects.api.list",
                "GET",
                "/api/projects",
                "项目列表",
                "列出默认同步根目录下的项目绑定。",
                10,
            )],
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
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

    use super::ProjectsPlugin;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "az-aio-plugin",
    });

    struct ProjectsWasm;

    impl Guest for ProjectsWasm {
        fn describe() -> Result<String, String> {
            descriptor_to_json(&ProjectsPlugin.descriptor()).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = ProjectsPlugin
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

    export!(ProjectsWasm);
}
