// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSession` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    Prompt,
    RealtimeFunctionTool,
    RealtimeSessionInputAudioNoiseReduction,
    RealtimeSessionInputAudioTranscription,
    RealtimeSessionMaxResponseOutputTokens,
    RealtimeSessionTracing2,
    RealtimeTurnDetection,
    VoiceIdsShared,
};

/// Realtime session object for the beta interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSession {
    /// Unique identifier for the session that looks like `sess_1234567890abcdef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The object type. Always `realtime.session`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<OpenAiJsonValue>,
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
    /// The voice the model uses to respond. Voice cannot be changed during the session once the model has
    /// responded with audio at least once. Current voice options are `alloy`, `ash`, `ballad`, `coral`,
    /// `echo`, `sage`, `shimmer`, and `verse`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceIdsShared>,
    /// The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`. For `pcm16`, input
    /// audio must be 16-bit PCM at a 24kHz sample rate, single channel (mono), and little-endian byte
    /// order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<String>,
    /// The format of output audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`. For `pcm16`, output
    /// audio is sampled at a rate of 24kHz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<RealtimeSessionInputAudioTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeTurnDetection>,
    /// Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise
    /// reduction filters audio added to the input audio buffer before it is sent to VAD and the model.
    /// Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model
    /// performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: Option<RealtimeSessionInputAudioNoiseReduction>,
    /// The speed of the model's spoken response. 1.0 is the default speed. 0.25 is the minimum speed. 1.5
    /// is the maximum speed. This value can only be changed in between model turns, not while a response is
    /// in progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: Option<RealtimeSessionTracing2>,
    /// Tools (functions) available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeFunctionTool>>,
    /// How the model chooses tools. Options are `auto`, `none`, `required`, or specify a function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    /// Sampling temperature for the model, limited to [0.6, 1.2]. For audio models a temperature of 0.8 is
    /// highly recommended for best performance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an
    /// integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a
    /// given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<RealtimeSessionMaxResponseOutputTokens>,
    /// Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}
