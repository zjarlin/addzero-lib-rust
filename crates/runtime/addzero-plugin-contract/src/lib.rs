use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginKind {
    System,
    #[default]
    Business,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    Available,
    #[default]
    Installed,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMenuContribution {
    pub section: String,
    pub label: String,
    pub page_id: String,
    pub order: i32,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPage {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub schema: PageSchema,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PageSchema {
    Table(TableSchema),
    Form(FormSchema),
    Detail(DetailSchema),
    Board(BoardSchema),
    Markdown(MarkdownSchema),
    Graph(GraphSchema),
}

impl Default for PageSchema {
    fn default() -> Self {
        Self::Markdown(MarkdownSchema::default())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSchema {
    pub columns: Vec<String>,
    pub rows: Vec<TableRow>,
    pub empty_message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormSchema {
    pub fields: Vec<DisplayField>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetailSchema {
    pub summary: String,
    pub fields: Vec<DisplayField>,
    pub timeline: Vec<RecordItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardSchema {
    pub metrics: Vec<MetricCard>,
    pub groups: Vec<RecordGroup>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownSchema {
    pub body: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSchema {
    pub nodes: Vec<GraphNodeSchema>,
    pub edges: Vec<GraphEdgeSchema>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayField {
    pub label: String,
    pub value: String,
    pub readonly: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub detail: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordGroup {
    pub title: String,
    pub items: Vec<RecordItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordItem {
    pub title: String,
    pub detail: String,
    pub meta: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeSchema {
    pub id: String,
    pub label: String,
    pub category: String,
    pub description: String,
    pub details: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeSchema {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSnapshot {
    pub username: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCounts {
    pub system_plugins: usize,
    pub installed_business_plugins: usize,
    pub plugin_instances: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellSnapshot {
    pub actor: ActorSnapshot,
    pub nav_sections: Vec<NavigationSection>,
    pub counts: PluginCounts,
    pub dev_auth_mode: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationSection {
    pub label: String,
    pub items: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationItem {
    pub label: String,
    pub href: String,
    pub plugin_id: Option<String>,
    pub page_id: Option<String>,
    pub badge: Option<String>,
    pub kind: NavigationItemKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationItemKind {
    #[default]
    Fixed,
    SystemPage,
    BusinessInstance,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceSnapshot {
    pub entries: Vec<MarketplaceEntry>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInstanceConfig {
    pub label: String,
    pub permissions: Vec<String>,
    pub dictionary_namespace: Option<String>,
    pub allowed_origins: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageScope {
    #[default]
    Fixed,
    System,
    Instance,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeOverview {
    pub counts: PluginCounts,
    pub package_root: String,
    pub dev_auth_mode: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationOutcome {
    pub ok: bool,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPackageManifest {
    pub descriptor: PluginDescriptor,
    pub runtime: RuntimeBinding,
    pub default_instance_label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBinding {
    pub binary_path: String,
    pub checksum_path: String,
    pub assets_dir: Option<String>,
}
