#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
pub use inventory;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginActivation {
    Eager,
    Lazy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    WasmComponent,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginBundleManifest {
    pub schema_version: u16,
    pub platform: String,
    pub bundle_id: String,
    pub package: String,
    pub descriptor: PluginDescriptor,
    pub contributions: ContributionSet,
    pub artifacts: Vec<PluginBundleArtifact>,
    #[serde(default)]
    pub sandbox_debug: PluginSandboxDebugReport,
    pub sandbox: PluginBundleSandbox,
}

impl PluginBundleManifest {
    pub const SCHEMA_VERSION: u16 = 1;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginBundleArtifact {
    pub kind: PluginBundleArtifactKind,
    pub name: String,
    pub source: String,
    pub path: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginBundleArtifactKind {
    WasmComponent,
    FrontendBundle,
    BackendBundle,
    BackendBinary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginFrontendBundle {
    pub schema_version: u16,
    pub plugin_id: String,
    pub nav_items: Vec<NavItemContribution>,
    pub pages: Vec<PageContribution>,
    pub ui_contributions: Vec<UiContribution>,
    pub toolbar_actions: Vec<ToolbarActionContribution>,
    pub catalog_providers: Vec<CatalogProviderContribution>,
    pub settings_sections: Vec<SettingsSectionContribution>,
}

impl PluginFrontendBundle {
    pub const SCHEMA_VERSION: u16 = 1;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginBackendBundle {
    pub schema_version: u16,
    pub plugin_id: String,
    pub backend_apis: Vec<BackendApiContribution>,
    pub shell_entries: Vec<ShellEntryContribution>,
    pub generated_files: Vec<GeneratedFileContribution>,
}

impl PluginBackendBundle {
    pub const SCHEMA_VERSION: u16 = 1;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginBundleSandbox {
    pub command: Vec<String>,
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginSandboxDebugReport {
    #[serde(default)]
    pub ui_contributions: Vec<PluginSandboxUiContributionDebug>,
    #[serde(default)]
    pub backend_apis: Vec<PluginSandboxBackendApiDebug>,
    #[serde(default)]
    pub settings_defaults: Vec<PluginSandboxSettingsDefaultDebug>,
}

impl PluginSandboxDebugReport {
    pub fn from_contributions(contributions: &ContributionSet) -> Self {
        Self {
            ui_contributions: contributions
                .ui_contributions
                .iter()
                .map(|contribution| PluginSandboxUiContributionDebug {
                    id: contribution.id.clone(),
                    slot: contribution.slot,
                    slot_label: contribution.slot.label().to_string(),
                    label: contribution.label.clone(),
                    renderer_id: contribution.renderer_id.clone(),
                    route: contribution.route.clone(),
                    order: contribution.order,
                })
                .collect(),
            backend_apis: contributions
                .backend_apis
                .iter()
                .map(|api| PluginSandboxBackendApiDebug {
                    id: api.id.clone(),
                    method: api.method.clone(),
                    path: api.path.clone(),
                    label: api.label.clone(),
                    description: api.description.clone(),
                    request_hint: format!("{} {}", api.method, api.path),
                    order: api.order,
                })
                .collect(),
            settings_defaults: contributions
                .settings_sections
                .iter()
                .flat_map(|section| {
                    section
                        .defaults
                        .iter()
                        .map(|default| PluginSandboxSettingsDefaultDebug {
                            section_id: section.id.clone(),
                            section_label: section.label.clone(),
                            key: default.key.clone(),
                            label: default.label.clone(),
                            value: default.value.clone(),
                            description: default.description.clone(),
                            order: default.order,
                        })
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginSandboxUiContributionDebug {
    pub id: String,
    pub slot: UiContributionSlot,
    pub slot_label: String,
    pub label: String,
    pub renderer_id: String,
    pub route: Option<String>,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginSandboxBackendApiDebug {
    pub id: String,
    pub method: String,
    pub path: String,
    pub label: String,
    pub description: String,
    pub request_hint: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginSandboxSettingsDefaultDebug {
    pub section_id: String,
    pub section_label: String,
    pub key: String,
    pub label: String,
    pub value: String,
    pub description: String,
    pub order: i32,
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
    Wasm,
}

impl CatalogSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bundled => "预置",
            Self::Community => "社区",
            Self::Local => "本地",
            Self::System => "系统",
            Self::User => "用户",
            Self::Wasm => "Wasm 组件",
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

pub trait AzAioPlugin: Send {
    fn descriptor(&self) -> PluginDescriptor;

    fn contributions(&self) -> Result<ContributionSet, PluginError> {
        Ok(ContributionSet::default())
    }

    fn on_load(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_enable(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_disable(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type NativeRenderFn = fn(NativeRenderContext) -> dioxus::prelude::Element;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct NativePluginContext {
    pub api_base_url: String,
    pub database_url: Option<String>,
    pub config_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativePluginContext {
    fn default() -> Self {
        Self {
            api_base_url: "http://127.0.0.1:0".to_string(),
            database_url: std::env::var("AZ_AIO_DATABASE_URL").ok(),
            config_dir: std::path::PathBuf::from("."),
            data_dir: std::path::PathBuf::from("."),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeRenderContext {
    pub active_route: String,
    pub api_base_url: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct NativeUiRenderer {
    pub renderer_id: String,
    pub slot: UiContributionSlot,
    pub route: Option<String>,
    pub render: NativeRenderFn,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct NativePluginRuntime {
    pub renderers: Vec<NativeUiRenderer>,
    pub router: axum::Router,
    pub startup: Option<fn(NativePluginContext) -> anyhow::Result<()>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativePluginRuntime {
    fn default() -> Self {
        Self {
            renderers: Vec::new(),
            router: axum::Router::new(),
            startup: None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait NativeAzAioPlugin: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;

    fn contributions(&self) -> Result<ContributionSet, PluginError>;

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime>;
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy)]
pub struct NativePluginRegistration {
    pub constructor: fn() -> Box<dyn NativeAzAioPlugin>,
}

#[cfg(not(target_arch = "wasm32"))]
inventory::collect!(NativePluginRegistration);

#[cfg(not(target_arch = "wasm32"))]
pub fn default_native_plugin_constructor<P>() -> Box<dyn NativeAzAioPlugin>
where
    P: NativeAzAioPlugin + Default + 'static,
{
    Box::new(P::default())
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! register_native_plugin {
    ($plugin_ty:ty $(,)?) => {
        $crate::inventory::submit! {
            $crate::NativePluginRegistration {
                constructor: $crate::default_native_plugin_constructor::<$plugin_ty>,
            }
        }
    };
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PluginError {
    #[error("插件 ID 重复：{0}")]
    DuplicateId(String),
    #[error("插件 `{plugin}` 缺少依赖 `{dependency}`")]
    MissingDependency { plugin: String, dependency: String },
    #[error("插件 `{plugin}` 依赖 `{dependency}` 未成功加载")]
    DependencyFailed { plugin: String, dependency: String },
    #[error("依赖环包含插件 `{0}`")]
    DependencyCycle(String),
    #[error("插件 `{plugin}` 在 {phase} 阶段失败：{message}")]
    Lifecycle {
        plugin: String,
        phase: String,
        message: String,
    },
    #[error("Wasm 组件 `{plugin}` 运行失败：{message}")]
    Wasm { plugin: String, message: String },
    #[error("插件 `{plugin}` 访问 `{path}` 失败：{message}")]
    Io {
        plugin: String,
        path: String,
        message: String,
    },
    #[error("描述符解析失败：{0}")]
    Descriptor(String),
}

pub fn descriptor_to_json(descriptor: &PluginDescriptor) -> Result<String, PluginError> {
    serde_json::to_string(descriptor).map_err(|error| PluginError::Descriptor(error.to_string()))
}

pub fn contributions_to_json(contributions: &ContributionSet) -> Result<String, PluginError> {
    serde_json::to_string(contributions).map_err(|error| PluginError::Descriptor(error.to_string()))
}

pub fn descriptor_from_json(value: &str) -> Result<PluginDescriptor, PluginError> {
    serde_json::from_str(value).map_err(|error| PluginError::Descriptor(error.to_string()))
}

pub fn contributions_from_json(value: &str) -> Result<ContributionSet, PluginError> {
    serde_json::from_str(value).map_err(|error| PluginError::Descriptor(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        BackendApiContribution, ContributionSet, PluginSandboxDebugReport,
        SettingsDefaultContribution, SettingsSectionContribution, UiContribution,
        UiContributionSlot,
    };

    #[test]
    fn sandbox_debug_report_extracts_ui_api_and_settings_defaults() {
        let contributions = ContributionSet {
            ui_contributions: vec![UiContribution {
                id: "projects.ui.sidebar".to_string(),
                slot: UiContributionSlot::ProjectSidebar,
                label: "项目侧边栏".to_string(),
                renderer_id: "projects.sidebar".to_string(),
                route: Some("/projects".to_string()),
                order: 10,
            }],
            backend_apis: vec![BackendApiContribution {
                id: "projects.api.list".to_string(),
                method: "GET".to_string(),
                path: "/api/projects".to_string(),
                label: "项目列表".to_string(),
                description: "列出已绑定项目。".to_string(),
                order: 10,
            }],
            settings_sections: vec![SettingsSectionContribution {
                id: "settings.project-defaults".to_string(),
                label: "项目默认目录".to_string(),
                order: 10,
                defaults: vec![SettingsDefaultContribution {
                    key: "projects.default_sync_root".to_string(),
                    label: "默认同步根目录".to_string(),
                    value: "az-sync/workspace".to_string(),
                    description: "项目插件扫描和绑定时使用的默认同步根目录。".to_string(),
                    order: 10,
                }],
            }],
            ..ContributionSet::default()
        };

        let report = PluginSandboxDebugReport::from_contributions(&contributions);

        assert_eq!(report.ui_contributions[0].slot_label, "项目侧边栏");
        assert_eq!(report.backend_apis[0].request_hint, "GET /api/projects");
        assert_eq!(report.settings_defaults[0].value, "az-sync/workspace");
    }
}
