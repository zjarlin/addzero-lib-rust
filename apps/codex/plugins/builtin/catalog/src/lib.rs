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
            name: "Plugin Catalog".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Catalog descriptors for plugins, skills, and future wasm components."
                .to_string(),
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
                        "Computer Use",
                        "Control Mac apps from Codex",
                        "Featured",
                        "◈",
                        "plugin-icon--aurora",
                        CatalogSource::Bundled,
                        false,
                        &[
                            "Screen inspection",
                            "Keyboard and mouse control",
                            "App window focus",
                        ],
                    ),
                    plugin_item(
                        "catalog.recommended.browser",
                        "Browser",
                        "Control local web targets with Codex",
                        "Featured",
                        "◉",
                        "plugin-icon--chrome",
                        CatalogSource::Bundled,
                        false,
                        &["Browser navigation", "Page inspection", "Form interaction"],
                    ),
                    plugin_item(
                        "catalog.local.terminal-runner",
                        "Terminal Runner",
                        "Run scoped shell commands in the active workspace",
                        "Development",
                        "⌁",
                        "plugin-icon--terminal",
                        CatalogSource::Local,
                        true,
                        &[
                            "Run shell commands",
                            "Read command output",
                            "Use active workspace",
                        ],
                    ),
                    plugin_item(
                        "catalog.community.git-review",
                        "Git Review",
                        "Inspect diffs and prepare review comments",
                        "Development",
                        "◇",
                        "plugin-icon--git",
                        CatalogSource::Community,
                        false,
                        &[
                            "Read git diff",
                            "Inspect changed files",
                            "Draft review comments",
                        ],
                    ),
                    plugin_item(
                        "catalog.local.automation-monitor",
                        "Automation Monitor",
                        "Track reminders, monitors, and follow-up runs",
                        "Local",
                        "◷",
                        "plugin-icon--automation",
                        CatalogSource::Local,
                        true,
                        &["Read automations", "Track run status", "Surface follow-ups"],
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
