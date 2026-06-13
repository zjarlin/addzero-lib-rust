// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalCompletionsRunDataSourceInputMessages3,
    CreateEvalCompletionsRunDataSourceSamplingParams,
    CreateEvalCompletionsRunDataSourceSource,
};

/// A CompletionsRunDataSource object describing a model sampling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalCompletionsRunDataSource {
    /// The type of run data source. Always `completions`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Used when sampling from a model. Dictates the structure of the messages passed into the model. Can
    /// either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with
    /// variable references to the `item` namespace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_messages: Option<CreateEvalCompletionsRunDataSourceInputMessages3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: Option<CreateEvalCompletionsRunDataSourceSamplingParams>,
    /// The name of the model to use for generating completions (e.g. "o3-mini").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Determines what populates the `item` namespace in this run's data source.
    pub source: CreateEvalCompletionsRunDataSourceSource,
}
