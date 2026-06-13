// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceSamplingParams` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionTool,
    CreateEvalCompletionsRunDataSourceSamplingParamsResponseFormat,
    ReasoningEffort,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceSamplingParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// A higher temperature increases randomness in the outputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// The maximum number of tokens in the generated output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i32>,
    /// An alternative to temperature for nucleus sampling; 1.0 includes all tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// A seed value to initialize the randomness, during sampling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    /// An object specifying the format that the model must output. Setting to `{ "type": "json_schema",
    /// "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied
    /// JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting
    /// to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model
    /// generates is valid JSON. Using `json_schema` is preferred for models that support it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<CreateEvalCompletionsRunDataSourceSamplingParamsResponseFormat>,
    /// A list of tools the model may call. Currently, only functions are supported as a tool. Use this to
    /// provide a list of functions the model may generate JSON inputs for. A max of 128 functions are
    /// supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatCompletionTool>>,
}
