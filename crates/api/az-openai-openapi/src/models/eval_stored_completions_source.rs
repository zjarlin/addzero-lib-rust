// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalStoredCompletionsSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
};

/// A StoredCompletionsRunDataSource configuration describing a set of filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalStoredCompletionsSource {
    /// The type of source. Always `stored_completions`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_after: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_before: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}
