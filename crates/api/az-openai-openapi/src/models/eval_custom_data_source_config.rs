// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalCustomDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// A CustomDataSourceConfig which specifies the schema of your `item` and optionally `sample`
/// namespaces. The response schema defines the shape of the data that will be: - Used to define your
/// testing criteria and - What data is required when creating a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCustomDataSourceConfig {
    /// The type of data source. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The json schema for the run data source items. Learn how to build JSON schemas [here](https://json-
    /// schema.org/).
    pub schema: OpenAiJsonObject,
}
