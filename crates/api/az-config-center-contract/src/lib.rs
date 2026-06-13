#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use az_derive_aliases::{apply, serde_code_default_ord_display_enum, serde_eq_default};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Shell 组件默认渲染输出路径。
pub const DEFAULT_SHELL_OUTPUT_PATH: &str = "~/.add_fn";
/// 桌面本地后端请求头中的会话令牌字段名。
pub const DESKTOP_SESSION_TOKEN_HEADER: &str = "x-aio-desktop-token";

/// 配置中心 API 的统一响应信封。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// 配置中心 API 的错误响应体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorBody {
    pub success: bool,
    pub message: String,
}

/// 配置中心健康状态响应数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusPayload {
    pub ok: bool,
    pub database: String,
}

/// 配置中心登录请求。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 配置中心登录成功后返回的会话信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginPayload {
    pub token: String,
    pub username: String,
}

/// 配置中心中的单条配置快照。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigItem {
    pub id: Uuid,
    pub namespace: String,
    pub config_key: String,
    pub config_value: String,
    pub value_type: String,
    pub description: String,
    pub enabled: bool,
    pub version: i32,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 配置列表查询参数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListQuery {
    pub namespace: Option<String>,
    pub keyword: Option<String>,
    pub include_disabled: Option<bool>,
}

/// 按命名空间和配置键读取配置的查询参数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetQuery {
    pub namespace: String,
    pub key: String,
}

/// 创建或更新配置的请求。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpsertRequest {
    pub namespace: String,
    pub key: String,
    pub value: String,
    #[serde(default = "default_config_value_type")]
    pub value_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_updated_by")]
    pub updated_by: String,
}

/// 修改配置启停状态的请求。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToggleRequest {
    pub namespace: String,
    pub key: String,
    pub enabled: bool,
    #[serde(default = "default_updated_by")]
    pub updated_by: String,
}

/// 删除配置的请求。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteRequest {
    pub namespace: String,
    pub key: String,
}

/// 删除配置后的影响行数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteResult {
    pub deleted: u64,
}

fn default_config_value_type() -> String {
    "text".to_owned()
}

fn default_enabled() -> bool {
    true
}

fn default_updated_by() -> String {
    String::new()
}

/// Shell 组件类型。
///
/// wire value 由 `strum(serialize = "...")` 固定为 `export`、`alias`、`function`、
/// `snippet`；`Display` 则返回渲染文件中的分组名。两者用途不同，不要互相替换。
#[apply(serde_code_default_ord_display_enum)]
pub enum ShellComponentKind {
    #[strum(serialize = "export")]
    #[display("exports")]
    Export,
    #[strum(serialize = "alias")]
    #[display("aliases")]
    Alias,
    #[strum(serialize = "function")]
    #[display("functions")]
    Function,
    #[default]
    #[strum(serialize = "snippet")]
    #[display("snippets")]
    Snippet,
}

/// Shell 组件注册表中的完整组件快照。
///
/// 该类型既用于 API 返回，也用于前端编辑态展示；具体 shell 输出由构建流程重新生成。
#[apply(serde_eq_default)]
pub struct ShellComponent {
    pub name: String,
    pub kind: ShellComponentKind,
    pub summary: String,
    pub enabled: bool,
    pub render_to_output: bool,
    pub export_value: Option<String>,
    pub alias_command: Option<String>,
    pub body: Option<String>,
    pub preview: String,
}

/// 创建或整体更新 Shell 组件的请求。
#[apply(serde_eq_default)]
pub struct ShellComponentUpsert {
    pub name: String,
    pub kind: ShellComponentKind,
    pub summary: String,
    pub enabled: bool,
    pub render_to_output: bool,
    pub export_value: Option<String>,
    pub alias_command: Option<String>,
    pub body: Option<String>,
}

/// 局部修改 Shell 组件启用状态和摘要的请求。
#[apply(serde_eq_default)]
pub struct ShellComponentPatch {
    pub name: String,
    pub summary: Option<String>,
    pub enabled: Option<bool>,
    pub render_to_output: Option<bool>,
}

/// 删除 Shell 组件的请求。
#[apply(serde_eq_default)]
pub struct ShellComponentRemove {
    pub name: String,
}

/// Shell 注册表构建配置。
#[apply(serde_eq_default)]
pub struct ShellComponentBuildConfig {
    pub output_path: String,
    pub resolved_output_path: String,
}

/// Shell 组件注册表快照。
#[apply(serde_eq_default)]
pub struct ShellComponentRegistry {
    pub config_path: String,
    pub build: ShellComponentBuildConfig,
    pub components: Vec<ShellComponent>,
}

/// 修改 Shell 组件构建配置的请求。
#[apply(serde_eq_default)]
pub struct ShellComponentConfigUpdate {
    pub output_path: Option<String>,
}

/// 触发 Shell 组件构建的请求。
///
/// `write = false` 表示只预览生成内容，不写入输出文件。
#[apply(serde_eq_default)]
pub struct ShellComponentBuildRequest {
    pub output_path: Option<String>,
    pub write: bool,
}

/// Shell 组件构建结果。
#[apply(serde_eq_default)]
pub struct ShellComponentBuildResult {
    pub config_path: String,
    pub output_path: String,
    pub written: bool,
    pub total_components: usize,
    pub included_components: usize,
    pub skipped_components: usize,
    pub included_names: Vec<String>,
    pub content: String,
}

/// 桌面后端健康状态和 Shell 注册表路径信息。
#[apply(serde_eq_default)]
pub struct DesktopBackendStatus {
    pub ok: bool,
    pub bind: String,
    pub desktop_mode: bool,
    pub shell_registry_path: String,
    pub output_path: String,
    pub resolved_output_path: String,
}
