// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalLogsDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

use crate::models::{
    Metadata,
};

/// A LogsDataSourceConfig which specifies the metadata property of your logs query. This is usually
/// metadata like `usecase=chatbot` or `prompt-version=v2`, etc. The schema returned by this data source
/// config is used to defined what variables are available in your evals. `item` and `sample` are both
/// defined when using this data source config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalLogsDataSourceConfig {
    /// The type of data source. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The json schema for the run data source items. Learn how to build JSON schemas [here](https://json-
    /// schema.org/).
    pub schema: OpenAiJsonObject,
}
