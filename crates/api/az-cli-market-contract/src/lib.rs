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

use az_derive_aliases::{apply, error_eq, serde_code_default_enum, serde_eq_default};
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[apply(serde_code_default_enum)]
pub enum CliMarketStatus {
    #[default]
    Draft,
    Reviewing,
    Published,
    Archived,
}

#[apply(serde_code_default_enum)]
pub enum CliMarketSourceType {
    #[default]
    Manual,
    ImportJson,
    ImportExcel,
    SyncExternal,
}

#[apply(serde_code_default_enum)]
pub enum CliEntryKind {
    #[default]
    Cli,
    Wrapper,
    Installer,
    Bundle,
}

#[apply(serde_code_default_enum)]
pub enum CliLocale {
    #[default]
    #[serde(rename = "zh-CN")]
    #[strum(serialize = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    #[strum(serialize = "en-US")]
    EnUs,
}

#[apply(serde_code_default_enum)]
pub enum CliPlatform {
    Macos,
    Windows,
    Linux,
    #[default]
    CrossPlatform,
}

#[apply(serde_code_default_enum)]
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

#[apply(serde_code_default_enum)]
pub enum CliImportFormat {
    #[default]
    Json,
    Xlsx,
}

#[apply(serde_code_default_enum)]
pub enum CliImportMode {
    #[default]
    Native,
    RegistryCompat,
}

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
pub struct CliInstallMethod {
    pub id: Option<String>,
    pub platform: CliPlatform,
    pub installer_kind: CliInstallerKind,
    pub package_id: String,
    pub command_template: String,
    pub validation_note: String,
    pub priority: i32,
}

#[apply(serde_eq_default)]
pub struct CliDocRef {
    pub id: Option<String>,
    pub locale: CliLocale,
    pub title: String,
    pub url: String,
    pub version: String,
    pub source_label: String,
    pub summary: String,
}

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
pub struct CliMarketSummary {
    pub total_entries: usize,
    pub published_entries: usize,
    pub import_jobs: usize,
    pub categories: usize,
}

#[apply(serde_eq_default)]
pub struct CliMarketCatalog {
    pub schema_version: String,
    pub summary: CliMarketSummary,
    pub entries: Vec<CliMarketEntry>,
}

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
pub struct CliMarketImportRowReport {
    pub row_index: usize,
    pub slug: String,
    pub success: bool,
    pub error: Option<String>,
    pub market_id: Option<String>,
}

#[apply(serde_eq_default)]
pub struct CliMarketImportReport {
    pub job_id: String,
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub total_rows: usize,
    pub success_rows: usize,
    pub failed_rows: usize,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
pub struct CliMarketImportJobDetail {
    pub job: CliMarketImportJob,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[apply(serde_eq_default)]
pub struct CliMarketExportRequest {
    pub only_published: bool,
    pub locale: Option<CliLocale>,
}

#[apply(serde_eq_default)]
pub struct CliMarketExportArtifact {
    pub file_name: String,
    pub content_type: String,
    pub bytes_base64: String,
}

#[apply(serde_eq_default)]
pub struct CliMarketInstallRequest {
    pub method_id: Option<String>,
}

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
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

#[apply(serde_eq_default)]
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

#[apply(error_eq)]
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

#[cfg(test)]
mod tests {
    use super::{CliInstallerKind, CliLocale, CliPlatform};

    #[test]
    fn code_enums_keep_public_wire_values() {
        assert_eq!(CliLocale::ZhCn.code(), "zh-CN");
        assert_eq!(CliLocale::from_code("en-US"), Some(CliLocale::EnUs));
        assert_eq!(
            serde_json::to_string(&CliLocale::ZhCn).expect("locale should serialize"),
            "\"zh-CN\""
        );

        assert_eq!(CliPlatform::CrossPlatform.code(), "cross_platform");
        assert_eq!(
            CliInstallerKind::from_code("brew"),
            Some(CliInstallerKind::Brew)
        );
    }
}
