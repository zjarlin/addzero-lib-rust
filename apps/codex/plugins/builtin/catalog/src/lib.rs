#![forbid(unsafe_code)]

use codex_plugin_api::{
    CatalogItemContribution, CatalogItemKind, CatalogProviderContribution, CatalogSource,
    CodexPlugin, ContributionSet, PluginActivation, PluginDependency, PluginDescriptor, PluginKind,
    ToolbarActionContribution,
};

#[derive(Default)]
pub struct CatalogPlugin;

impl CodexPlugin for CatalogPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "builtin/catalog".to_string(),
            name: "插件目录".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "提供插件、技能和外部组件的目录描述。".to_string(),
            activation: PluginActivation::Eager,
            priority: 900,
            dependencies: vec![PluginDependency {
                id: "builtin/core-nav".to_string(),
                optional: false,
            }],
            capabilities: vec![
                "catalog-provider".to_string(),
                "toolbar-actions".to_string(),
            ],
            permissions: Vec::new(),
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> Result<ContributionSet, codex_plugin_api::PluginError> {
        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            toolbar_actions: vec![
                toolbar_action("catalog.refresh", "刷新", "⟳", true, 10),
                toolbar_action("catalog.manage", "管理", "⚙", false, 20),
                toolbar_action("catalog.create", "创建", "＋", false, 30),
            ],
            catalog_providers: vec![CatalogProviderContribution {
                id: "catalog.recommended-plugins".to_string(),
                label: "推荐插件".to_string(),
                order: 10,
                items: vec![
                    plugin_item(
                        "catalog.recommended.computer-use",
                        "电脑控制",
                        "从 Codex 控制本机应用窗口",
                        "推荐",
                        "◈",
                        "plugin-icon--aurora",
                        CatalogSource::Bundled,
                        false,
                        &["读取屏幕结构", "控制键盘和鼠标", "聚焦应用窗口"],
                    ),
                    plugin_item(
                        "catalog.recommended.browser",
                        "浏览器控制",
                        "从 Codex 控制本地网页目标",
                        "推荐",
                        "◉",
                        "plugin-icon--chrome",
                        CatalogSource::Bundled,
                        false,
                        &["浏览器导航", "页面检查", "表单交互"],
                    ),
                    plugin_item(
                        "catalog.local.terminal-runner",
                        "终端执行器",
                        "在当前工作区执行受限命令",
                        "开发",
                        "⌁",
                        "plugin-icon--terminal",
                        CatalogSource::Local,
                        true,
                        &["执行命令", "读取命令输出", "使用当前工作区"],
                    ),
                    plugin_item(
                        "catalog.community.git-review",
                        "代码评审",
                        "检查变更并生成评审意见",
                        "开发",
                        "◇",
                        "plugin-icon--git",
                        CatalogSource::Community,
                        false,
                        &["读取 Git 差异", "检查变更文件", "草拟评审意见"],
                    ),
                    plugin_item(
                        "catalog.local.automation-monitor",
                        "自动化监控",
                        "跟踪提醒、监控和后续执行",
                        "本地",
                        "◷",
                        "plugin-icon--automation",
                        CatalogSource::Local,
                        true,
                        &["读取自动化任务", "跟踪运行状态", "显示后续事项"],
                    ),
                ],
            }],
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }
}

fn toolbar_action(
    id: &str,
    label: &str,
    icon: &str,
    primary: bool,
    order: i32,
) -> ToolbarActionContribution {
    ToolbarActionContribution {
        id: id.to_string(),
        route: Some("/plugins".to_string()),
        label: label.to_string(),
        icon: icon.to_string(),
        primary,
        order,
    }
}

fn plugin_item(
    id: &str,
    name: &str,
    description: &str,
    section: &str,
    icon: &str,
    accent_class: &str,
    source: CatalogSource,
    installed: bool,
    permissions: &[&str],
) -> CatalogItemContribution {
    CatalogItemContribution {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        section: section.to_string(),
        icon: icon.to_string(),
        accent_class: accent_class.to_string(),
        kind: CatalogItemKind::Plugin,
        source,
        installed,
        tags: Vec::new(),
        permissions: permissions
            .iter()
            .map(|permission| (*permission).to_string())
            .collect(),
        path: None,
    }
}
