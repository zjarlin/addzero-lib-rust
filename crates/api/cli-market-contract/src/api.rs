//! CLI 市场数据契约层——定义 CLI 生态中工具条目、安装方式、导入导出等全部共享类型。
//!
//! 本 crate 纯做类型定义，不包含任何 IO 或网络逻辑；API 服务端与 CLI 客户端
//! 共同依赖此 crate，从而保证协议层面的一致性。
//!
//! ## 核心类型
//!
//! - [`CliMarketEntry`] — 单条 CLI 工具市场条目（含国际化描述、安装方式、文档引用）
//! - [`CliMarketEntryUpsert`] — 创建 / 更新时使用的 DTO（`id` 可选）
//! - [`CliMarketCatalog`] — 带摘要统计的完整目录快照
//! - [`CliMarketImportRequest`] / [`CliMarketImportReport`] — 导入请求与逐行报告
//! - [`CliMarketExportRequest`] / [`CliMarketExportArtifact`] — 导出请求与 base64 附件
//! - [`CliMarketInstallRequest`] / [`CliMarketInstallResult`] — 安装执行与结果
//!
//! ## 枚举分类
//!
//! | 枚举                | 说明                 |
//! |---------------------|----------------------|
//! | `CliMarketStatus`   | 条目生命周期状态     |
//! | `CliEntryKind`      | 条目类型（CLI/Wrapper/Installer/Bundle） |
//! | `CliLocale`         | 语言区域             |
//! | `CliPlatform`       | 目标平台             |
//! | `CliInstallerKind`  | 包管理器 / 安装方式  |
//! | `CliImportFormat`   | 导入文件格式         |
//!
//! ## wire 约定
//!
//! 所有 `serde_code_default_enum` 枚举都通过 `code()` / `from_code()` 暴露稳定字符串，
//! 并用同一套字符串完成 serde 序列化。修改这些字符串等同于修改 API wire contract，
//! 需要同时更新服务端、CLI 客户端、导入导出测试和已有持久化数据。

use anyhow::Context;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// CLI 市场条目的生命周期状态。
///
/// 默认值 `Draft` 表示条目尚未进入公开目录；公开 API 中传输的是稳定 code 字符串。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliMarketStatus {
    #[default]
    Draft,
    Reviewing,
    Published,
    Archived,
}

impl CliMarketStatus {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 市场条目的来源类型。
///
/// 该字段用于区分人工录入、文件导入和外部同步来源，不代表条目的可信等级。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliMarketSourceType {
    #[default]
    Manual,
    ImportJson,
    ImportExcel,
    SyncExternal,
}

impl CliMarketSourceType {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// CLI 市场条目的功能形态。
///
/// `Cli` 表示直接可执行工具，`Wrapper` / `Installer` / `Bundle`
/// 用于描述包装脚本、安装器或组合分发包。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliEntryKind {
    #[default]
    Cli,
    Wrapper,
    Installer,
    Bundle,
}

impl CliEntryKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 条目文本内容的语言区域。
///
/// 这里的 wire value 使用 BCP 47 风格大小写（例如 `zh-CN`），不要改成 snake_case。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliLocale {
    #[default]
    #[serde(rename = "zh-CN")]
    #[strum(serialize = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    #[strum(serialize = "en-US")]
    EnUs,
}

impl CliLocale {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 安装方法适用的平台。
///
/// `CrossPlatform` 表示安装命令可跨平台复用，序列化值固定为 `cross_platform`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliPlatform {
    Macos,
    Windows,
    Linux,
    #[default]
    CrossPlatform,
}

impl CliPlatform {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 安装命令所属的工具链或包管理器。
///
/// `Custom` 作为兜底值，用于导入暂时无法归类的安装脚本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliInstallerKind {
    Brew,
    Bun,
    Npm,
    Cargo,
    Pipx,
    Winget,
    Scoop,
    Curl,
    #[default]
    Custom,
}

impl CliInstallerKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// CLI 市场导入文件格式。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliImportFormat {
    #[default]
    Json,
    Xlsx,
}

impl CliImportFormat {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// CLI 市场导入兼容模式。
///
/// `RegistryCompat` 用于兼容旧注册表 JSON 行格式，正式内部模型仍以 native DTO 为准。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CliImportMode {
    #[default]
    Native,
    RegistryCompat,
}

impl CliImportMode {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 某个语言区域下的展示文案和安装说明。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliLocaleText {
    pub locale: CliLocale,
    pub display_name: String,
    pub summary: String,
    pub description_md: String,
    pub install_guide_md: String,
    pub docs_summary: String,
    pub requires_text: String,
    pub install_command: String,
}

/// 单个可执行安装方法。
///
/// `command_template` 是最终执行命令模板，服务端生成安装任务前应完成平台和安全边界校验。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliInstallMethod {
    pub id: Option<String>,
    pub platform: CliPlatform,
    pub installer_kind: CliInstallerKind,
    pub package_id: String,
    pub command_template: String,
    pub validation_note: String,
    pub priority: i32,
}

/// 条目关联的外部文档引用。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliDocRef {
    pub id: Option<String>,
    pub locale: CliLocale,
    pub title: String,
    pub url: String,
    pub version: String,
    pub source_label: String,
    pub summary: String,
}

/// CLI 市场条目的完整快照。
///
/// 这是服务端列表、详情和导出共用的正式 DTO；`raw` 只保留导入源补充信息，
/// 不应成为业务查询的主要字段。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketEntry {
    pub id: String,
    pub slug: String,
    pub status: CliMarketStatus,
    pub source_type: CliMarketSourceType,
    pub entry_kind: CliEntryKind,
    pub vendor_name: String,
    pub latest_version: String,
    pub homepage_url: String,
    pub repo_url: String,
    pub docs_url: String,
    pub entry_point: String,
    pub category_code: String,
    pub tags: Vec<String>,
    pub locales: Vec<CliLocaleText>,
    pub install_methods: Vec<CliInstallMethod>,
    pub doc_refs: Vec<CliDocRef>,
    pub raw: serde_json::Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 创建或更新 CLI 市场条目时使用的 DTO。
///
/// `id` 为 `None` 时由服务端创建新条目；有值时表示更新现有条目。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketEntryUpsert {
    pub id: Option<String>,
    pub slug: String,
    pub status: CliMarketStatus,
    pub source_type: CliMarketSourceType,
    pub entry_kind: CliEntryKind,
    pub vendor_name: String,
    pub latest_version: String,
    pub homepage_url: String,
    pub repo_url: String,
    pub docs_url: String,
    pub entry_point: String,
    pub category_code: String,
    pub tags: Vec<String>,
    pub locales: Vec<CliLocaleText>,
    pub install_methods: Vec<CliInstallMethod>,
    pub doc_refs: Vec<CliDocRef>,
    pub raw: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketSummary {
    pub total_entries: usize,
    pub published_entries: usize,
    pub import_jobs: usize,
    pub categories: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketCatalog {
    pub schema_version: String,
    pub summary: CliMarketSummary,
    pub entries: Vec<CliMarketEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketImportRequest {
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub file_name: String,
    pub payload_base64: String,
    pub submitted_by: String,
}

impl CliMarketImportRequest {
    /// 解码导入文件的 base64 负载。
    ///
    /// 该方法只处理传输编码，不校验文件格式内容。
    pub fn decode_payload(&self) -> anyhow::Result<Vec<u8>> {
        STANDARD
            .decode(self.payload_base64.as_bytes())
            .context("无法解析导入文件")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketImportRowReport {
    pub row_index: usize,
    pub slug: String,
    pub success: bool,
    pub error: Option<String>,
    pub market_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketImportReport {
    pub job_id: String,
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub total_rows: usize,
    pub success_rows: usize,
    pub failed_rows: usize,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketImportJob {
    pub id: String,
    pub file_name: String,
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub submitted_by: String,
    pub total_rows: usize,
    pub success_rows: usize,
    pub failed_rows: usize,
    pub status: String,
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketImportJobDetail {
    pub job: CliMarketImportJob,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketExportRequest {
    pub only_published: bool,
    pub locale: Option<CliLocale>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketExportArtifact {
    pub file_name: String,
    pub content_type: String,
    pub bytes_base64: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketInstallRequest {
    pub method_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketInstallResult {
    pub entry_id: String,
    pub slug: String,
    pub method_id: String,
    pub platform: CliPlatform,
    pub installer_kind: CliInstallerKind,
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliMarketInstallHistoryItem {
    pub id: String,
    pub entry_id: String,
    pub slug: String,
    pub method_id: Option<String>,
    pub platform: CliPlatform,
    pub installer_kind: CliInstallerKind,
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub started_at: String,
    pub finished_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CliRegistryCompatEntry {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub requires: String,
    pub install_cmd: String,
    pub entry_point: String,
    pub category: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliSimpleMetadata {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub requires: String,
    pub install_cmd: String,
    pub entry_point: String,
    pub category: String,
}

impl CliMarketExportArtifact {
    /// 将导出文件内容封装为 API 可传输的 base64 artifact。
    pub fn encode(
        file_name: impl Into<String>,
        content_type: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            file_name: file_name.into(),
            content_type: content_type.into(),
            bytes_base64: STANDARD.encode(bytes),
        }
    }

    /// 解码导出 artifact 中的 base64 文件内容。
    pub fn decode(&self) -> anyhow::Result<Vec<u8>> {
        STANDARD
            .decode(self.bytes_base64.as_bytes())
            .context("无法解析导出文件")
    }
}
