//! Rust 代码生成的共享操作契约。

use serde::{Deserialize, Serialize};

pub const STATUS_PATH: &str = "/api/codegen/status";
pub const RUST_FILES_PATH: &str = "/api/codegen/rust-files";
pub const UI_ACTION_PATH: &str = "/api/codegen/ui-action";
pub const OP_RUST_FILE_GENERATE: &str = "codegen.rust-files.generate";

/// 在当前客户机生成一个 Rust 文件的请求。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRustFileRequest {
    pub target_directory: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub overwrite: bool,
    pub definition: RustTypeDefinition,
}

/// 当前支持的 Rust 类型定义。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum RustTypeDefinition {
    Enum {
        type_name: String,
        variants: Vec<RustEnumVariant>,
    },
    Struct {
        type_name: String,
        fields: Vec<RustStructField>,
    },
}

impl RustTypeDefinition {
    /// 返回定义中的 Rust 类型名。
    pub fn type_name(&self) -> &str {
        match self {
            Self::Enum { type_name, .. } | Self::Struct { type_name, .. } => type_name,
        }
    }
}

/// Rust 枚举变体定义。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustEnumVariant {
    pub name: String,
    #[serde(default)]
    pub discriminant: Option<i64>,
}

/// Rust 结构体字段定义。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustStructField {
    pub name: String,
    pub rust_type: String,
}

/// 文件实际执行所在的节点类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodegenExecutionTarget {
    CurrentClient,
}

/// 客户机成功生成的 Rust 文件。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedRustFile {
    pub execution_target: CodegenExecutionTarget,
    pub file_path: String,
    pub byte_length: usize,
    pub source: String,
}

/// 代码生成节点状态。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodegenStatus {
    pub execution_target: CodegenExecutionTarget,
    pub default_target_directory: String,
    pub supported_kinds: Vec<String>,
}
