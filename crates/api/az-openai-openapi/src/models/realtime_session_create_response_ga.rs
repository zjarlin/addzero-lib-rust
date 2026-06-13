// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateResponseGA` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Prompt,
    RealtimeReasoning,
    RealtimeSessionCreateResponseGAAudio,
    RealtimeSessionCreateResponseGAMaxOutputTokens,
    RealtimeSessionCreateResponseGATool,
    RealtimeSessionCreateResponseGAToolChoice,
    RealtimeSessionCreateResponseGATracing2,
    RealtimeTruncation,
};

/// A Realtime session configuration object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseGA {
    /// The type of session to create. Always `realtime` for the Realtime API.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Unique identifier for the session that looks like `sess_1234567890abcdef`.
    pub id: String,
    /// The object type. Always `realtime.session`.
    pub object: String,
    /// Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
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
    pub audio: Option<RealtimeSessionCreateResponseGAAudio>,
    /// Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include
    /// logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: Option<RealtimeSessionCreateResponseGATracing2>,
    /// Tools available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeSessionCreateResponseGATool>>,
    /// How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<RealtimeSessionCreateResponseGAToolChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<RealtimeReasoning>,
    /// Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an
    /// integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a
    /// given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<RealtimeSessionCreateResponseGAMaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: Option<RealtimeTruncation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
}
