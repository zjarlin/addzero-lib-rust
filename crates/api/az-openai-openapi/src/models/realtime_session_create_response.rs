// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    RealtimeFunctionTool,
    RealtimeSessionCreateResponseAudio,
    RealtimeSessionCreateResponseMaxOutputTokens,
    RealtimeSessionCreateResponseTracing2,
    RealtimeSessionCreateResponseTurnDetection,
};

/// A Realtime session configuration object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponse {
    /// Unique identifier for the session that looks like `sess_1234567890abcdef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The object type. Always `realtime.session`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// Additional fields to include in server outputs. - `item.input_audio_transcription.logprobs`: Include
    /// logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<OpenAiJsonValue>,
    /// The default system instructions (i.e. system message) prepended to model calls. This field allows
    /// the client to guide the model on desired responses. The model can be instructed on response content
    /// and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses")
    /// and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently").
    /// The instructions are not guaranteed to be followed by the model, but they provide guidance to the
    /// model on the desired behavior. Note that the server sets default instructions which will be used if
    /// this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Configuration for input and output audio for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeSessionCreateResponseAudio>,
    /// Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a
    /// session, the configuration cannot be modified. `auto` will create a trace for the session with
    /// default values for the workflow name, group id, and metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: Option<RealtimeSessionCreateResponseTracing2>,
    /// Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model
    /// will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeSessionCreateResponseTurnDetection>,
    /// Tools (functions) available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeFunctionTool>>,
    /// How the model chooses tools. Options are `auto`, `none`, `required`, or specify a function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    /// Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an
    /// integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a
    /// given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<RealtimeSessionCreateResponseMaxOutputTokens>,
}
