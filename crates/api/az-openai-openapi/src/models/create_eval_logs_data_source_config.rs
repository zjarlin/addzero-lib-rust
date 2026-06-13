// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalLogsDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// A data source config which specifies the metadata property of your logs query. This is usually
/// metadata like `usecase=chatbot` or `prompt-version=v2`, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalLogsDataSourceConfig {
    /// The type of data source. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Metadata filters for the logs data source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiJsonObject>,
}
