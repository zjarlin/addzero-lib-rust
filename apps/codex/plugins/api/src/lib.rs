#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    WasmComponent,
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
    pub renderer: PageRenderer,
    pub placeholder_mark: String,
    pub order: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageRenderer {
    Placeholder,
    Catalog,
    CliCatalog,
    EnvVars,
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
            Self::Wasm => "Wasm",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsSectionContribution {
    pub id: String,
    pub label: String,
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
            Self::Alias => "Alias",
            Self::Export => "Export",
            Self::Function => "Function",
            Self::ScriptSnippet => "Shell",
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

pub trait CodexPlugin: Send {
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

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PluginError {
    #[error("duplicate plugin id: {0}")]
    DuplicateId(String),
    #[error("missing dependency `{dependency}` for plugin `{plugin}`")]
    MissingDependency { plugin: String, dependency: String },
    #[error("dependency cycle contains plugin `{0}`")]
    DependencyCycle(String),
    #[error("plugin `{plugin}` failed during {phase}: {message}")]
    Lifecycle {
        plugin: String,
        phase: String,
        message: String,
    },
    #[error("wasm component error for `{plugin}`: {message}")]
    Wasm { plugin: String, message: String },
    #[error("io error for `{plugin}` at `{path}`: {message}")]
    Io {
        plugin: String,
        path: String,
        message: String,
    },
    #[error("descriptor decode error: {0}")]
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
