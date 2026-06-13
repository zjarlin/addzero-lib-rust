// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponsesClientEventResponseCreate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContextManagementParam,
    ConversationParam,
    IncludeEnum,
    InputParam,
    Metadata,
    ModelIdsResponses,
    Prompt,
    Reasoning,
    ResponseStreamOptions,
    ResponseTextParam,
    ServiceTier,
    ToolChoiceParam,
    ToolsArray,
};

/// Client event for creating a response over a persistent WebSocket connection. This payload uses the
/// same top-level fields as `POST /v1/responses`. Notes: - `stream` is implicit over WebSocket and
/// should not be sent. - `background` is not supported over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsesClientEventResponseCreate {
    /// The type of the client event. Always `response.create`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key`
    /// instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost
    /// cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse.
    /// [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// A stable identifier used to help detect users of your application that may be violating OpenAI's
    /// usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length
    /// of 64 characters. We recommend hashing their username or email address, in order to avoid sending us
    /// any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces
    /// the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models
    /// with different capabilities, performance characteristics, and price points. Refer to the [model
    /// guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelIdsResponses>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsArray>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<InputParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<IncludeEnum>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ResponseStreamOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_management: Option<Vec<ContextManagementParam>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
}
