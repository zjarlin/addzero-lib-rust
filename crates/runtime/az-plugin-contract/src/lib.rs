//! 插件系统的核心契约层，定义插件宿主与插件之间共享的所有数据模型。
//!
//! 本 crate 不包含任何业务逻辑，仅提供可序列化的类型定义，用于：
//! - 插件描述符（[`PluginDescriptor`]）：声明插件的元信息、能力、页面和菜单贡献
//! - UI 页面模式（[`PageSchema`]）：支持表格、表单、详情、看板、Markdown、图谱等多种页面形态
//! - 导航与 Shell 快照（[`ShellSnapshot`]）：为前端 Shell 提供当前用户、导航树和统计信息
//! - 插件市场（[`MarketplaceSnapshot`]）：展示可用插件及其安装状态
//! - 插件实例（[`PluginInstance`]）：已安装插件的运行时实例及其配置
//! - 运行时概览（[`RuntimeOverview`]）：内核的全局状态摘要
//!
//! 所有类型均派生了 `Clone`、`Debug`、`Serialize`、`Deserialize`，可直接用于 JSON 传输。

use az_derive_aliases::{apply, serde_eq, serde_eq_default};
use chrono::{DateTime, Utc};

#[apply(serde_eq_default)]
pub enum PluginKind {
    System,
    #[default]
    Business,
}

#[apply(serde_eq_default)]
pub enum PluginStatus {
    Available,
    #[default]
    Installed,
    Disabled,
}

#[apply(serde_eq)]
pub enum HostCapability {
    Auth,
    Rbac,
    Dictionary,
    Audit,
    Storage,
    Http,
    Db,
    Kv,
    Log,
}

#[apply(serde_eq_default)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    pub summary: String,
    pub tags: Vec<String>,
    pub icon: Option<String>,
    pub compatibility: Vec<String>,
    pub capabilities: Vec<HostCapability>,
    pub menus: Vec<PluginMenuContribution>,
    pub pages: Vec<PluginPage>,
    #[serde(default)]
    pub metadata: PluginMetadata,
    #[serde(default)]
    pub cli_commands: Vec<PluginCliCommand>,
}

#[apply(serde_eq_default)]
pub struct PluginMetadata {
    pub github_url: String,
    pub description: String,
    pub maintainer_type: String,
    pub maintainer_name: String,
    pub primary_language: String,
    pub category: String,
    pub install_command: String,
    pub agent_install_command: String,
}

#[apply(serde_eq_default)]
pub struct PluginCliCommand {
    pub command_name: String,
    pub file_name: String,
    pub object_bucket: String,
    pub object_key: String,
    pub object_sha256: String,
    pub object_size_bytes: u64,
    pub content_type: String,
    pub install_path: String,
    pub status: String,
}

#[apply(serde_eq_default)]
pub struct PluginMenuContribution {
    pub section: String,
    pub label: String,
    pub page_id: String,
    pub order: i32,
    pub icon: Option<String>,
}

#[apply(serde_eq_default)]
pub struct PluginPage {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub schema: PageSchema,
}

#[apply(serde_eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PageSchema {
    Table(TableSchema),
    Form(FormSchema),
    Detail(DetailSchema),
    Board(BoardSchema),
    Markdown(MarkdownSchema),
    Graph(GraphSchema),
    NotesFragments(NotesFragmentsSchema),
}

impl Default for PageSchema {
    fn default() -> Self {
        Self::Markdown(MarkdownSchema::default())
    }
}

#[apply(serde_eq_default)]
pub struct TableSchema {
    pub columns: Vec<String>,
    pub rows: Vec<TableRow>,
    pub empty_message: String,
}

#[apply(serde_eq_default)]
pub struct TableRow {
    pub cells: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct FormSchema {
    pub fields: Vec<DisplayField>,
}

#[apply(serde_eq_default)]
pub struct DetailSchema {
    pub summary: String,
    pub fields: Vec<DisplayField>,
    pub timeline: Vec<RecordItem>,
}

#[apply(serde_eq_default)]
pub struct BoardSchema {
    pub metrics: Vec<MetricCard>,
    pub groups: Vec<RecordGroup>,
}

#[apply(serde_eq_default)]
pub struct MarkdownSchema {
    pub body: String,
}

#[apply(serde_eq_default)]
pub struct NotesFragmentsSchema {
    pub list_path: String,
    pub save_path: String,
    pub delete_path: String,
    pub placeholder: String,
    pub empty_message: String,
}

#[apply(serde_eq_default)]
pub struct GraphSchema {
    pub nodes: Vec<GraphNodeSchema>,
    pub edges: Vec<GraphEdgeSchema>,
}

#[apply(serde_eq_default)]
pub struct DisplayField {
    pub label: String,
    pub value: String,
    pub readonly: bool,
}

#[apply(serde_eq_default)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub detail: String,
}

#[apply(serde_eq_default)]
pub struct RecordGroup {
    pub title: String,
    pub items: Vec<RecordItem>,
}

#[apply(serde_eq_default)]
pub struct RecordItem {
    pub title: String,
    pub detail: String,
    pub meta: String,
}

#[apply(serde_eq_default)]
pub struct GraphNodeSchema {
    pub id: String,
    pub label: String,
    pub category: String,
    pub description: String,
    pub details: String,
}

#[apply(serde_eq_default)]
pub struct GraphEdgeSchema {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub label: Option<String>,
}

#[apply(serde_eq_default)]
pub struct ActorSnapshot {
    pub username: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct PluginCounts {
    pub system_plugins: usize,
    pub installed_business_plugins: usize,
    pub plugin_instances: usize,
}

#[apply(serde_eq_default)]
pub struct ShellSnapshot {
    pub actor: ActorSnapshot,
    pub nav_sections: Vec<NavigationSection>,
    pub counts: PluginCounts,
    pub dev_auth_mode: String,
}

#[apply(serde_eq_default)]
pub struct NavigationSection {
    pub label: String,
    pub items: Vec<NavigationItem>,
}

#[apply(serde_eq_default)]
pub struct NavigationItem {
    pub label: String,
    pub href: String,
    pub plugin_id: Option<String>,
    pub page_id: Option<String>,
    pub badge: Option<String>,
    pub kind: NavigationItemKind,
}

#[apply(serde_eq_default)]
pub enum NavigationItemKind {
    #[default]
    Fixed,
    SystemPage,
    BusinessInstance,
}

#[apply(serde_eq_default)]
pub struct MarketplaceSnapshot {
    pub entries: Vec<MarketplaceEntry>,
    pub tags: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct MarketplaceEntry {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    pub summary: String,
    pub tags: Vec<String>,
    pub icon: Option<String>,
    pub compatibility: Vec<String>,
    pub capabilities: Vec<HostCapability>,
    pub status: PluginStatus,
    pub instances: usize,
}

#[apply(serde_eq_default)]
pub struct PluginInstanceConfig {
    pub label: String,
    pub permissions: Vec<String>,
    pub dictionary_namespace: Option<String>,
    pub allowed_origins: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct PluginInstance {
    pub plugin_id: String,
    pub plugin_name: String,
    pub slug: String,
    pub label: String,
    pub status: PluginStatus,
    pub page_ids: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub config: PluginInstanceConfig,
}

#[apply(serde_eq_default)]
pub struct ResolvedPage {
    pub scope: PageScope,
    pub plugin_id: String,
    pub plugin_name: String,
    pub page_id: String,
    pub title: String,
    pub subtitle: String,
    pub breadcrumbs: Vec<String>,
    pub schema: PageSchema,
}

#[apply(serde_eq_default)]
pub enum PageScope {
    #[default]
    Fixed,
    System,
    Instance,
}

#[apply(serde_eq_default)]
pub struct RuntimeOverview {
    pub counts: PluginCounts,
    pub package_root: String,
    pub dev_auth_mode: String,
}

#[apply(serde_eq_default)]
pub struct OperationOutcome {
    pub ok: bool,
    pub message: String,
}

#[apply(serde_eq_default)]
pub struct PluginPackageManifest {
    pub descriptor: PluginDescriptor,
    pub runtime: RuntimeBinding,
    pub default_instance_label: Option<String>,
}

#[apply(serde_eq_default)]
pub struct RuntimeBinding {
    pub binary_path: String,
    pub checksum_path: String,
    pub assets_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{MarkdownSchema, PageSchema, PluginDescriptor, PluginKind, PluginStatus};

    #[test]
    fn plugin_contract_aliases_keep_wire_shape_and_defaults() {
        assert_eq!(PluginKind::default(), PluginKind::Business);
        assert_eq!(PluginStatus::default(), PluginStatus::Installed);
        assert_eq!(
            serde_json::to_string(&PageSchema::Markdown(MarkdownSchema {
                body: "hello".to_owned(),
            }))
            .expect("page schema should serialize"),
            r#"{"kind":"markdown","body":"hello"}"#
        );

        let descriptor = PluginDescriptor::default();
        assert!(descriptor.metadata.install_command.is_empty());
        assert!(descriptor.cli_commands.is_empty());
    }
}
