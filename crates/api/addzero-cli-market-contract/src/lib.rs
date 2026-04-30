use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliMarketSourceType {
    #[default]
    Manual,
    ImportJson,
    ImportExcel,
    SyncExternal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliEntryKind {
    #[default]
    Cli,
    Wrapper,
    Installer,
    Bundle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliImportFormat {
    #[default]
    Json,
    Xlsx,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliImportMode {
    #[default]
    Native,
    RegistryCompat,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliInstallMethod {
    pub id: Option<String>,
    pub platform: CliPlatform,
    pub installer_kind: CliInstallerKind,
    pub package_id: String,
    pub command_template: String,
    pub validation_note: String,
    pub priority: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliDocRef {
    pub id: Option<String>,
    pub locale: CliLocale,
    pub title: String,
    pub url: String,
    pub version: String,
    pub source_label: String,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketSummary {
    pub total_entries: usize,
    pub published_entries: usize,
    pub import_jobs: usize,
    pub categories: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketCatalog {
    pub schema_version: String,
    pub summary: CliMarketSummary,
    pub entries: Vec<CliMarketEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketImportRowReport {
    pub row_index: usize,
    pub slug: String,
    pub success: bool,
    pub error: Option<String>,
    pub market_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketImportReport {
    pub job_id: String,
    pub format: CliImportFormat,
    pub mode: CliImportMode,
    pub total_rows: usize,
    pub success_rows: usize,
    pub failed_rows: usize,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketImportJobDetail {
    pub job: CliMarketImportJob,
    pub rows: Vec<CliMarketImportRowReport>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketExportRequest {
    pub only_published: bool,
    pub locale: Option<CliLocale>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketExportArtifact {
    pub file_name: String,
    pub content_type: String,
    pub bytes_base64: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketInstallRequest {
    pub method_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Error, PartialEq, Eq)]
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
