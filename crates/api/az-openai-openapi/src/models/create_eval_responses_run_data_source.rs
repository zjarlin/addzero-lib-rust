// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalResponsesRunDataSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceInputMessages3,
    CreateEvalResponsesRunDataSourceSamplingParams,
    CreateEvalResponsesRunDataSourceSource,
};

/// A ResponsesRunDataSource object describing a model sampling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSource {
    /// The type of run data source. Always `responses`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Used when sampling from a model. Dictates the structure of the messages passed into the model. Can
    /// either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with
    /// variable references to the `item` namespace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_messages: Option<CreateEvalResponsesRunDataSourceInputMessages3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: Option<CreateEvalResponsesRunDataSourceSamplingParams>,
    /// The name of the model to use for generating completions (e.g. "o3-mini").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Determines what populates the `item` namespace in this run's data source.
    pub source: CreateEvalResponsesRunDataSourceSource,
}
