#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use az_derive_aliases::{apply, serde_code_default_ord_display_enum, serde_eq_default};

/// Shell 组件默认渲染输出路径。
pub const DEFAULT_SHELL_OUTPUT_PATH: &str = "~/.add_fn";
/// 桌面本地后端请求头中的会话令牌字段名。
pub const DESKTOP_SESSION_TOKEN_HEADER: &str = "x-aio-desktop-token";

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

#[cfg(test)]
mod tests {
    use super::ShellComponentKind;

    #[test]
    fn shell_component_kind_keeps_wire_codes() {
        assert_eq!(ShellComponentKind::Alias.code(), "alias");
        assert_eq!(
            ShellComponentKind::from_code("function"),
            Some(ShellComponentKind::Function)
        );
        assert_eq!(ShellComponentKind::Export.to_string(), "exports");
        assert!(ShellComponentKind::Export < ShellComponentKind::Alias);
        assert_eq!(
            serde_json::to_string(&ShellComponentKind::Snippet)
                .expect("shell component kind should serialize"),
            "\"snippet\""
        );
    }
}
