// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalCustomDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// A CustomDataSourceConfig object that defines the schema for the data source used for the evaluation
/// runs. This schema is used to define the shape of the data that will be: - Used to define your
/// testing criteria and - What data is required when creating a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalCustomDataSourceConfig {
    /// The type of data source. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The json schema for each row in the data source.
    pub item_schema: OpenAiJsonObject,
    /// Whether the eval should expect you to populate the sample namespace (ie, by generating responses off
    /// of your data source)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_sample_schema: Option<bool>,
}
