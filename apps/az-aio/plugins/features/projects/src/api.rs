#![forbid(unsafe_code)]

use az_aio_plugin_api::api::{
    BackendApiContribution, ContributionSet, NativeAzAioPlugin, NativePluginContext,
    NativePluginRuntime, NavItemContribution, PageContribution, PluginActivation, PluginDescriptor,
    PluginKind, UiContribution, UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

#[derive(Default)]
pub struct ProjectsPlugin;

impl NativeAzAioPlugin for ProjectsPlugin {
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
            kind: PluginKind::Native,
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

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(ProjectsPlugin);

pub fn ensure_linked() {}

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
