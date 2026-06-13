// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalStoredCompletionsDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Deprecated in favor of LogsDataSourceConfig.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalStoredCompletionsDataSourceConfig {
    /// The type of data source. Always `stored_completions`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Metadata filters for the stored completions data source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiJsonObject>,
}
