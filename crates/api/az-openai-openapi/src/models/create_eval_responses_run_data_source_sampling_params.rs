// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceSamplingParams` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalResponsesRunDataSourceSamplingParamsText,
    ReasoningEffort,
    Tool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSourceSamplingParams {
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
    /// An array of tools the model may call while generating a response. You can specify which tool to use
    /// by setting the `tool_choice` parameter. The two categories of tools you can provide the model are: -
    /// **Built-in tools**: Tools that are provided by OpenAI that extend the model's capabilities, like
    /// [web search](/docs/guides/tools-web-search) or [file search](/docs/guides/tools-file-search). Learn
    /// more about [built-in tools](/docs/guides/tools). - **Function calls (custom tools)**: Functions that
    /// are defined by you, enabling the model to call your own code. Learn more about [function
    /// calling](/docs/guides/function-calling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Configuration options for a text response from the model. Can be plain text or structured JSON data.
    /// Learn more: - [Text inputs and outputs](/docs/guides/text) - [Structured
    /// Outputs](/docs/guides/structured-outputs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<CreateEvalResponsesRunDataSourceSamplingParamsText>,
}
