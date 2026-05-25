#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use az_derive_aliases::{apply, serde_code_default_ord_display_enum, serde_eq_default};

pub const DEFAULT_SHELL_OUTPUT_PATH: &str = "~/.add_fn";
pub const DESKTOP_SESSION_TOKEN_HEADER: &str = "x-aio-desktop-token";

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

#[apply(serde_eq_default)]
pub struct ShellComponentPatch {
    pub name: String,
    pub summary: Option<String>,
    pub enabled: Option<bool>,
    pub render_to_output: Option<bool>,
}

#[apply(serde_eq_default)]
pub struct ShellComponentRemove {
    pub name: String,
}

#[apply(serde_eq_default)]
pub struct ShellComponentBuildConfig {
    pub output_path: String,
    pub resolved_output_path: String,
}

#[apply(serde_eq_default)]
pub struct ShellComponentRegistry {
    pub config_path: String,
    pub build: ShellComponentBuildConfig,
    pub components: Vec<ShellComponent>,
}

#[apply(serde_eq_default)]
pub struct ShellComponentConfigUpdate {
    pub output_path: Option<String>,
}

#[apply(serde_eq_default)]
pub struct ShellComponentBuildRequest {
    pub output_path: Option<String>,
    pub write: bool,
}

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
