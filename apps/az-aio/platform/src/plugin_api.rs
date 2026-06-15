#![forbid(unsafe_code)]

use anyhow::Context;
use serde::{Deserialize, Serialize};

pub type DynNativeAzAioPlugin = std::sync::Arc<dyn NativeAzAioPlugin>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginActivation {
    Eager,
    Lazy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    Native,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginState {
    Discovered,
    Loaded,
    Active,
    Disabled,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub optional: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub activation: PluginActivation,
    pub priority: i32,
    pub dependencies: Vec<PluginDependency>,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub kind: PluginKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContributionSet {
    #[serde(default)]
    pub nav_items: Vec<NavItemContribution>,
    #[serde(default)]
    pub pages: Vec<PageContribution>,
    #[serde(default)]
    pub ui_contributions: Vec<UiContribution>,
    #[serde(default)]
    pub backend_apis: Vec<BackendApiContribution>,
    #[serde(default)]
    pub toolbar_actions: Vec<ToolbarActionContribution>,
    #[serde(default)]
    pub catalog_providers: Vec<CatalogProviderContribution>,
    #[serde(default)]
    pub settings_sections: Vec<SettingsSectionContribution>,
    #[serde(default)]
    pub shell_entries: Vec<ShellEntryContribution>,
    #[serde(default)]
    pub generated_files: Vec<GeneratedFileContribution>,
}

impl ContributionSet {
    pub fn merge(&mut self, other: Self) {
        self.nav_items.extend(other.nav_items);
        self.pages.extend(other.pages);
        self.ui_contributions.extend(other.ui_contributions);
        self.backend_apis.extend(other.backend_apis);
        self.toolbar_actions.extend(other.toolbar_actions);
        self.catalog_providers.extend(other.catalog_providers);
        self.settings_sections.extend(other.settings_sections);
        self.shell_entries.extend(other.shell_entries);
        self.generated_files.extend(other.generated_files);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NavItemContribution {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub route: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PageContribution {
    pub route: String,
    pub title: String,
    pub subtitle: String,
    pub renderer_id: String,
    pub placeholder_mark: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiContribution {
    pub id: String,
    pub slot: UiContributionSlot,
    pub label: String,
    pub renderer_id: String,
    pub route: Option<String>,
    pub order: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiContributionSlot {
    AppSidebar,
    AppTopbar,
    Content,
    SettingsContent,
    ProjectSidebar,
    ProjectContent,
    SandboxPanel,
}

impl UiContributionSlot {
    pub fn label(self) -> &'static str {
        match self {
            Self::AppSidebar => "应用侧边栏",
            Self::AppTopbar => "应用顶栏",
            Self::Content => "内容区",
            Self::SettingsContent => "设置内容区",
            Self::ProjectSidebar => "项目侧边栏",
            Self::ProjectContent => "项目内容区",
            Self::SandboxPanel => "沙箱调试面板",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackendApiContribution {
    pub id: String,
    pub method: String,
    pub path: String,
    pub label: String,
    pub description: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolbarActionContribution {
    pub id: String,
    pub route: Option<String>,
    pub label: String,
    pub icon: String,
    pub primary: bool,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CatalogProviderContribution {
    pub id: String,
    pub label: String,
    pub order: i32,
    pub items: Vec<CatalogItemContribution>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CatalogItemContribution {
    pub id: String,
    pub name: String,
    pub description: String,
    pub section: String,
    pub icon: String,
    pub accent_class: String,
    pub kind: CatalogItemKind,
    pub source: CatalogSource,
    pub installed: bool,
    #[serde(default)]
    pub tags: Vec<CatalogTagContribution>,
    pub permissions: Vec<String>,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CatalogTagContribution {
    pub id: String,
    pub label: String,
    pub group: CatalogTagGroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogTagGroup {
    Developer,
    Design,
}

impl CatalogTagGroup {
    pub fn label(self) -> &'static str {
        match self {
            Self::Developer => "开发人员",
            Self::Design => "设计",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogItemKind {
    Plugin,
    Skill,
}

impl CatalogItemKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Plugin => "插件",
            Self::Skill => "技能",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogSource {
    Bundled,
    Community,
    Local,
    System,
    User,
}

impl CatalogSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bundled => "预置",
            Self::Community => "社区",
            Self::Local => "本地",
            Self::System => "系统",
            Self::User => "用户",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsSectionContribution {
    pub id: String,
    pub label: String,
    pub order: i32,
    #[serde(default)]
    pub defaults: Vec<SettingsDefaultContribution>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsDefaultContribution {
    pub key: String,
    pub label: String,
    pub value: String,
    pub description: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShellEntryContribution {
    pub id: String,
    pub kind: ShellEntryKind,
    pub name: String,
    pub section: String,
    pub source_path: String,
    pub line_start: usize,
    pub preview: String,
    pub deprecated_source: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShellEntryKind {
    Alias,
    Export,
    Function,
    ScriptSnippet,
}

impl ShellEntryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Alias => "别名",
            Self::Export => "环境变量",
            Self::Function => "函数",
            Self::ScriptSnippet => "脚本片段",
        }
    }

    pub fn is_cli(self) -> bool {
        matches!(self, Self::Alias | Self::Function | Self::ScriptSnippet)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratedFileContribution {
    pub id: String,
    pub path: String,
    pub source_root: String,
    pub section_delimiter: String,
    pub deprecated_source_root: bool,
    pub entry_count: usize,
    pub backup_path: Option<String>,
    pub status: GeneratedFileStatus,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedFileStatus {
    Generated,
    Failed,
}

pub type NativeRenderFn = fn(NativeRenderContext) -> dioxus::prelude::Element;

#[derive(Clone, Debug)]
pub struct NativePluginContext {
    pub api_base_url: String,
    pub database_url: Option<String>,
    pub config_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

impl Default for NativePluginContext {
    fn default() -> Self {
        Self {
            api_base_url: "http://127.0.0.1:0".to_string(),
            database_url: None,
            config_dir: std::path::PathBuf::from("."),
            data_dir: std::path::PathBuf::from("."),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeRenderContext {
    pub active_route: String,
    pub api_base_url: String,
}

#[derive(Clone, Debug)]
pub struct NativeUiRenderer {
    pub renderer_id: String,
    pub slot: UiContributionSlot,
    pub route: Option<String>,
    pub render: NativeRenderFn,
}

impl PartialEq for NativeUiRenderer {
    fn eq(&self, other: &Self) -> bool {
        self.renderer_id == other.renderer_id
            && self.slot == other.slot
            && self.route == other.route
    }
}

#[derive(Clone)]
pub struct NativePluginRuntime {
    pub renderers: Vec<NativeUiRenderer>,
    pub router: axum::Router,
    pub startup: Option<fn(NativePluginContext) -> anyhow::Result<()>>,
}

impl Default for NativePluginRuntime {
    fn default() -> Self {
        Self {
            renderers: Vec::new(),
            router: axum::Router::new(),
            startup: None,
        }
    }
}

pub trait NativeAzAioPlugin: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;

    fn contributions(&self) -> anyhow::Result<ContributionSet>;

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime>;
}

pub fn descriptor_to_json(descriptor: &PluginDescriptor) -> anyhow::Result<String> {
    serde_json::to_string(descriptor).context("plugin descriptor serialization failed")
}

pub fn contributions_to_json(contributions: &ContributionSet) -> anyhow::Result<String> {
    serde_json::to_string(contributions).context("plugin contributions serialization failed")
}

pub fn descriptor_from_json(value: &str) -> anyhow::Result<PluginDescriptor> {
    serde_json::from_str(value).context("plugin descriptor parse failed")
}

pub fn contributions_from_json(value: &str) -> anyhow::Result<ContributionSet> {
    serde_json::from_str(value).context("plugin contributions parse failed")
}
