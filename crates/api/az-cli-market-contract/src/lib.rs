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

use base64::{Engine as _, engine::general_purpose::STANDARD};
use macro_rules_attribute::apply;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// 将高频契约派生收成浅层 item 包装宏。
macro_rules! derive_error {
    ($item:item) => {
        #[derive(Clone, Debug, Error, PartialEq, Eq)]
        $item
    };
}

macro_rules! derive_snake_case_serde_enum_default {
    ($item:item) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        $item
    };
}

macro_rules! derive_serde_struct_default {
    ($item:item) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        $item
    };
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliMarketStatus {
    #[default]
    Draft,
    Reviewing,
    Published,
    Archived,
}

impl CliMarketStatus {
    pub const ALL: [Self; 4] = [
        Self::Draft,
        Self::Reviewing,
        Self::Published,
        Self::Archived,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Reviewing => "reviewing",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliMarketSourceType {
    #[default]
    Manual,
    ImportJson,
    ImportExcel,
    SyncExternal,
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliEntryKind {
    #[default]
    Cli,
    Wrapper,
    Installer,
    Bundle,
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliLocale {
    #[default]
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
}

impl CliLocale {
    pub const ALL: [Self; 2] = [Self::ZhCn, Self::EnUs];

    pub fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::EnUs => "en-US",
        }
    }
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliPlatform {
    Macos,
    Windows,
    Linux,
    #[default]
    CrossPlatform,
}

impl CliPlatform {
    pub const ALL: [Self; 4] = [Self::Macos, Self::Windows, Self::Linux, Self::CrossPlatform];

    pub fn code(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::CrossPlatform => "cross_platform",
        }
    }
}

#[apply(derive_snake_case_serde_enum_default)]
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
    pub const ALL: [Self; 9] = [
        Self::Brew,
        Self::Bun,
        Self::Npm,
        Self::Cargo,
        Self::Pipx,
        Self::Winget,
        Self::Scoop,
        Self::Curl,
        Self::Custom,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Brew => "brew",
            Self::Bun => "bun",
            Self::Npm => "npm",
            Self::Cargo => "cargo",
            Self::Pipx => "pipx",
            Self::Winget => "winget",
            Self::Scoop => "scoop",
            Self::Curl => "curl",
            Self::Custom => "custom",
        }
    }
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliImportFormat {
    #[default]
    Json,
    Xlsx,
}

#[apply(derive_snake_case_serde_enum_default)]
pub enum CliImportMode {
    #[default]
    Native,
    RegistryCompat,
}

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
pub struct CliInstallMethod {
    pub id: Option<String>,
    pub platform: CliPlatform,
    pub installer_kind: CliInstallerKind,
    pub package_id: String,
    pub command_template: String,
    pub validation_note: String,
    pub priority: i32,
}

#[apply(derive_serde_struct_default)]
pub struct CliDocRef {
    pub id: Option<String>,
    pub locale: CliLocale,
    pub title: String,
    pub url: String,
    pub version: String,
    pub source_label: String,
    pub summary: String,
}

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
pub struct CliMarketSummary {
    pub total_entries: usize,
    pub published_entries: usize,
    pub import_jobs: usize,
    pub categories: usize,
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketCatalog {
    pub schema_version: String,
    pub summary: CliMarketSummary,
    pub entries: Vec<CliMarketEntry>,
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketImportRequest {
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub file_name: String,
    pub payload_base64: String,
    pub submitted_by: String,
}

impl CliMarketImportRequest {
    pub fn decode_payload(&self) -> Result<Vec<u8>, CliMarketContractError> {
        STANDARD
            .decode(self.payload_base64.as_bytes())
            .map_err(|err| CliMarketContractError::Message(format!("无法解析导入文件：{err}")))
    }
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketImportRowReport {
    pub row_index: usize,
    pub slug: String,
    pub success: bool,
    pub error: Option<String>,
    pub market_id: Option<String>,
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketImportReport {
    pub job_id: String,
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub total_rows: usize,
    pub success_rows: usize,
    pub failed_rows: usize,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
pub struct CliMarketImportJobDetail {
    pub job: CliMarketImportJob,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketExportRequest {
    pub only_published: bool,
    pub locale: Option<CliLocale>,
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketExportArtifact {
    pub file_name: String,
    pub content_type: String,
    pub bytes_base64: String,
}

#[apply(derive_serde_struct_default)]
pub struct CliMarketInstallRequest {
    pub method_id: Option<String>,
}

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
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

#[apply(derive_serde_struct_default)]
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

#[apply(derive_error)]
pub enum CliMarketContractError {
    #[error("{0}")]
    Message(String),
}

impl CliMarketExportArtifact {
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

    pub fn decode(&self) -> Result<Vec<u8>, CliMarketContractError> {
        STANDARD
            .decode(self.bytes_base64.as_bytes())
            .map_err(|err| CliMarketContractError::Message(format!("无法解析导出文件：{err}")))
    }
}
