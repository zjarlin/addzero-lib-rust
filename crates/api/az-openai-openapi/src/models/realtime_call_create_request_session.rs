// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeCallCreateRequestSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Prompt,
    RealtimeCallCreateRequestSessionAudio,
    RealtimeCallCreateRequestSessionMaxOutputTokens,
    RealtimeCallCreateRequestSessionTool,
    RealtimeCallCreateRequestSessionToolChoice,
    RealtimeCallCreateRequestSessionTracing2,
    RealtimeReasoning,
    RealtimeTruncation,
};

/// Optional session configuration to apply before the realtime session is created. Use the same
/// parameters you would send in a [`create client secret`](/docs/api-reference/realtime-
/// sessions/create-realtime-client-secret) request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCallCreateRequestSession {
    /// The type of session to create. Always `realtime` for the Realtime API.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The set of modalities the model can respond with. It defaults to `["audio"]`, indicating that the
    /// model will respond with audio plus a transcript. `["text"]` can be used to make the model respond
    /// with text only. It is not possible to request both `text` and `audio` at the same time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<Vec<String>>,
    /// The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The default system instructions (i.e. system message) prepended to model calls. This field allows
    /// the client to guide the model on desired responses. The model can be instructed on response content
    /// and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses")
    /// and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently").
    /// The instructions are not guaranteed to be followed by the model, but they provide guidance to the
    /// model on the desired behavior. Note that the server sets default instructions which will be used if
    /// this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Configuration for input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeCallCreateRequestSessionAudio>,
    /// Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include
    /// logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// Realtime API can write session traces to the [Traces
    /// Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once
    /// tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace
    /// for the session with default values for the workflow name, group id, and metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: Option<RealtimeCallCreateRequestSessionTracing2>,
    /// Tools available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeCallCreateRequestSessionTool>>,
    /// How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<RealtimeCallCreateRequestSessionToolChoice>,
    /// Whether the model may call multiple tools in parallel. Only supported by reasoning Realtime models
    /// such as `gpt-realtime-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<RealtimeReasoning>,
    /// Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an
    /// integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a
    /// given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<RealtimeCallCreateRequestSessionMaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: Option<RealtimeTruncation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
}
