#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DEFAULT_SHELL_OUTPUT_PATH: &str = "~/.add_fn";
pub const DESKTOP_SESSION_TOKEN_HEADER: &str = "x-aio-desktop-token";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellComponentKind {
    Export,
    Alias,
    Function,
    #[default]
    Snippet,
}

impl ShellComponentKind {
    pub const ALL: [Self; 4] = [Self::Export, Self::Alias, Self::Function, Self::Snippet];

    pub fn code(self) -> &'static str {
        match self {
            Self::Export => "export",
            Self::Alias => "alias",
            Self::Function => "function",
            Self::Snippet => "snippet",
        }
    }

    pub fn section_title(self) -> &'static str {
        match self {
            Self::Export => "exports",
            Self::Alias => "aliases",
            Self::Function => "functions",
            Self::Snippet => "snippets",
        }
    }

    pub fn sort_key(self) -> u8 {
        match self {
            Self::Export => 0,
            Self::Alias => 1,
            Self::Function => 2,
            Self::Snippet => 3,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellComponentPatch {
    pub name: String,
    pub summary: Option<String>,
    pub enabled: Option<bool>,
    pub render_to_output: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellComponentRemove {
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellComponentBuildConfig {
    pub output_path: String,
    pub resolved_output_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellComponentRegistry {
    pub config_path: String,
    pub build: ShellComponentBuildConfig,
    pub components: Vec<ShellComponent>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellComponentConfigUpdate {
    pub output_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellComponentBuildRequest {
    pub output_path: Option<String>,
    pub write: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopBackendStatus {
    pub ok: bool,
    pub bind: String,
    pub desktop_mode: bool,
    pub shell_registry_path: String,
    pub output_path: String,
    pub resolved_output_path: String,
}
