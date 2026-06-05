#![forbid(unsafe_code)]

use codex_plugin_api::{
    CodexPlugin, ContributionSet, NavItemContribution, PageContribution, PageRenderer,
    PluginActivation, PluginDescriptor, PluginKind, SettingsSectionContribution,
};

#[derive(Default)]
pub struct CoreNavPlugin;

impl CodexPlugin for CoreNavPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "builtin/core-nav".to_string(),
            name: "核心导航".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供 Codex 桌面端左侧导航和占位页面描述。".to_string(),
            activation: PluginActivation::Eager,
            priority: 1_000,
            dependencies: Vec::new(),
            capabilities: vec![
                "nav-items".to_string(),
                "pages".to_string(),
                "settings-sections".to_string(),
            ],
            permissions: Vec::new(),
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> Result<ContributionSet, codex_plugin_api::PluginError> {
        Ok(ContributionSet {
            nav_items: vec![
                nav_item("core-nav.chat", "新对话", "✎", "/chat", 10),
                nav_item("core-nav.search", "搜索", "⌕", "/search", 20),
                nav_item("core-nav.plugins", "插件", "⌘", "/plugins", 30),
                nav_item("core-nav.automations", "自动化", "◷", "/automations", 40),
            ],
            pages: vec![
                placeholder_page("/chat", "暂未开放", "对话功能会由后续插件接管。", "✎", 10),
                placeholder_page("/search", "暂未开放", "搜索功能会由后续插件接管。", "⌕", 20),
                PageContribution {
                    route: "/plugins".to_string(),
                    title: "插件与技能".to_string(),
                    subtitle: "查看本地插件、外部组件和技能。".to_string(),
                    renderer: PageRenderer::Catalog,
                    placeholder_mark: "⌘".to_string(),
                    order: 30,
                },
                placeholder_page(
                    "/automations",
                    "暂未开放",
                    "自动化功能会由后续插件接管。",
                    "◷",
                    40,
                ),
            ],
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: vec![SettingsSectionContribution {
                id: "core-nav.settings.general".to_string(),
                label: "通用".to_string(),
                order: 10,
            }],
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
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
        renderer: PageRenderer::Placeholder,
        placeholder_mark: mark.to_string(),
        order,
    }
}
