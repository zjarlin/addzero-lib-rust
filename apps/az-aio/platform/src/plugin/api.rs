#![forbid(unsafe_code)]

use anyhow::Context;
use serde::{Deserialize, Serialize};

pub type DynAdminPluginProvider = std::sync::Arc<dyn AdminPluginProvider>;

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
    pub client_pages: Vec<ClientPageContribution>,
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminPluginContribution {
    pub menu: AdminMenuTree,
    #[serde(default)]
    pub resources: Vec<AdminResourceContract>,
    #[serde(default)]
    pub cli: Vec<AdminCliContribution>,
    pub native: ContributionSet,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminMenuTree {
    #[serde(default)]
    pub sections: Vec<AdminMenuSection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminMenuSection {
    pub domain_id: String,
    pub label: String,
    pub default_href: String,
    pub order: i32,
    #[serde(default)]
    pub menus: Vec<AdminMenuNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminMenuNode {
    pub id: String,
    pub kind: AdminMenuNodeKind,
    pub label: String,
    pub href: String,
    pub icon: String,
    pub order: i32,
    #[serde(default)]
    pub active_patterns: Vec<String>,
    #[serde(default)]
    pub permissions_any_of: Vec<String>,
    #[serde(default)]
    pub children: Vec<AdminMenuNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdminMenuNodeKind {
    Branch,
    Page,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminResourceContract {
    pub id: String,
    pub label: String,
    pub description: String,
    pub route: String,
    pub table_name: String,
    #[serde(default)]
    pub permissions_any_of: Vec<String>,
    #[serde(default)]
    pub fields: Vec<AdminFieldContract>,
    #[serde(default)]
    pub operations: Vec<AdminOperationContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminFieldContract {
    pub name: String,
    pub label: String,
    pub kind: AdminFieldKind,
    pub required: bool,
    pub searchable: bool,
    pub table_visible: bool,
    pub form_visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdminFieldKind {
    Text,
    Number,
    Boolean,
    Badge,
    Time,
    Json,
    Relation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminOperationContract {
    pub id: String,
    pub label: String,
    pub method: String,
    pub path: String,
    pub cli: String,
    pub primary: bool,
    pub audit: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminCliContribution {
    pub id: String,
    pub label: String,
    pub command: String,
    pub resource_id: Option<String>,
    pub operation_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminRendererContribution {
    pub renderer_id: String,
    pub slot: UiContributionSlot,
    pub route: Option<String>,
}

/// Browser bootstrap contract consumed by the Dioxus web shell.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientBootstrapPayload {
    pub admin_menu_tree: AdminMenuTree,
    pub pages: Vec<PageContribution>,
    pub client_pages: Vec<ClientPageContribution>,
    pub plugins: Vec<ClientPluginRecord>,
    pub default_route: String,
    pub api_base_url: String,
}

/// Enabled plugin metadata exposed to the browser bootstrap payload.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPluginRecord {
    pub descriptor: PluginDescriptor,
    pub state: PluginState,
}

impl ContributionSet {
    pub fn merge(&mut self, other: Self) {
        self.nav_items.extend(other.nav_items);
        self.pages.extend(other.pages);
        self.client_pages.extend(other.client_pages);
        self.ui_contributions.extend(other.ui_contributions);
        self.backend_apis.extend(other.backend_apis);
        self.toolbar_actions.extend(other.toolbar_actions);
        self.catalog_providers.extend(other.catalog_providers);
        self.settings_sections.extend(other.settings_sections);
        self.shell_entries.extend(other.shell_entries);
        self.generated_files.extend(other.generated_files);
    }
}

impl AdminPluginContribution {
    pub fn from_native(native: ContributionSet) -> Self {
        Self {
            menu: AdminMenuTree::default(),
            resources: Vec::new(),
            cli: Vec::new(),
            native,
        }
    }
}

pub fn merge_menu_tree(target: &mut AdminMenuTree, source: AdminMenuTree) {
    for section in source.sections {
        match target
            .sections
            .iter_mut()
            .find(|item| item.domain_id == section.domain_id)
        {
            Some(existing) => merge_menu_section(existing, section),
            None => target.sections.push(section),
        }
    }
    sort_menu_sections(&mut target.sections);
}

fn merge_menu_section(target: &mut AdminMenuSection, source: AdminMenuSection) {
    if target.default_href.is_empty() {
        target.default_href = source.default_href;
    }
    for node in source.menus {
        merge_menu_node(&mut target.menus, node);
    }
    sort_menu_nodes(&mut target.menus);
}

fn merge_menu_node(nodes: &mut Vec<AdminMenuNode>, source: AdminMenuNode) {
    match nodes.iter_mut().find(|node| node.id == source.id) {
        Some(existing) => {
            for child in source.children {
                merge_menu_node(&mut existing.children, child);
            }
            sort_menu_nodes(&mut existing.children);
        }
        None => nodes.push(source),
    }
}

pub fn sort_menu_sections(sections: &mut [AdminMenuSection]) {
    sections.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.domain_id.cmp(&right.domain_id))
    });
    for section in sections {
        sort_menu_nodes(&mut section.menus);
    }
}

pub fn sort_menu_nodes(nodes: &mut [AdminMenuNode]) {
    nodes.sort_by(|left, right| left.order.cmp(&right.order).then(left.id.cmp(&right.id)));
    for node in nodes {
        sort_menu_nodes(&mut node.children);
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

/// Browser-side route contribution for a compiled-in admin plugin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPageContribution {
    pub route: String,
    pub title: String,
    pub renderer_id: String,
    pub slot: UiContributionSlot,
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
    pub shared_db: Option<crate::core::db::Db>,
    pub config_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

impl Default for NativePluginContext {
    fn default() -> Self {
        Self {
            api_base_url: "http://127.0.0.1:0".to_string(),
            database_url: None,
            shared_db: None,
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

pub trait NativePluginProvider: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;

    fn contributions(&self) -> anyhow::Result<ContributionSet>;

    fn runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime>;

    fn admin_menu(&self, contributions: &ContributionSet) -> AdminMenuTree {
        AdminPluginContribution::from_native(contributions.clone()).menu
    }

    fn admin_resources(&self) -> Vec<AdminResourceContract> {
        Vec::new()
    }

    fn admin_cli(&self) -> Vec<AdminCliContribution> {
        Vec::new()
    }
}

pub trait AdminPluginProvider: Send + Sync {
    fn admin_descriptor(&self) -> PluginDescriptor;

    fn admin_contribution(&self) -> anyhow::Result<AdminPluginContribution>;

    fn admin_runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime>;
}

impl<T> AdminPluginProvider for T
where
    T: NativePluginProvider,
{
    fn admin_descriptor(&self) -> PluginDescriptor {
        self.descriptor()
    }

    fn admin_contribution(&self) -> anyhow::Result<AdminPluginContribution> {
        let native = self.contributions()?;
        let menu = self.admin_menu(&native);
        Ok(AdminPluginContribution {
            menu,
            resources: self.admin_resources(),
            cli: self.admin_cli(),
            native,
        })
    }

    fn admin_runtime(&self, context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        self.runtime(context)
    }
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
