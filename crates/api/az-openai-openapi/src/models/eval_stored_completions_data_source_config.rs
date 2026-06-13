// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalStoredCompletionsDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

use crate::models::{
    Metadata,
};

/// Deprecated in favor of LogsDataSourceConfig.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalStoredCompletionsDataSourceConfig {
    /// The type of data source. Always `stored_completions`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The json schema for the run data source items. Learn how to build JSON schemas [here](https://json-
    /// schema.org/).
    pub schema: OpenAiJsonObject,
}
